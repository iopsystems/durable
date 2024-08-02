use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use futures_concurrency::future::Join;
use futures_util::FutureExt;
use rand::Rng;
use serde_json::value::RawValue;
use sqlx::postgres::PgNotification;
use sqlx::types::Json;
use tokio::sync::{broadcast, Notify};
use tokio::task::JoinSet;
use tokio::time::Instant;
use tracing::Instrument;

use crate::bindings::exports::wasi::cli::run::GuestPre;
use crate::error::TaskStatus;
use crate::event::{self, Event, EventSource, Notification};
use crate::flag::{ShutdownFlag, ShutdownGuard};
use crate::plugin::{DurablePlugin, Plugin};
use crate::task::{Task, TaskState};
use crate::util::{IntoPgInterval, Mailbox};
use crate::Config;

const LOG_ERROR_INDEX: i32 = i32::MAX - 1;
const LOG_PANIC_INDEX: i32 = i32::MAX;

pub(crate) struct SharedState {
    pub shutdown: ShutdownFlag,
    pub pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub notifications: broadcast::Sender<Notification>,
    pub config: Config,
    pub plugins: Vec<Box<dyn Plugin>>,

    leader: Mailbox<i64>,
    suspend: Notify,
}

pub(crate) struct TaskData {
    pub id: i64,
    pub name: String,
    pub wasm: Option<Vec<u8>>,
    pub data: Json<Box<RawValue>>,
}

pub struct WorkerBuilder {
    config: Config,
    pool: sqlx::PgPool,
    event_source: Option<Box<dyn EventSource>>,
    client: Option<reqwest::Client>,
    engine: Option<wasmtime::Engine>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl WorkerBuilder {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            config: Config::default(),
            pool,
            event_source: None,
            client: None,
            engine: None,
            plugins: vec![Box::new(DurablePlugin)],
        }
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn engine(mut self, engine: wasmtime::Engine) -> Self {
        self.engine = Some(engine);
        self
    }

    pub async fn build(self) -> anyhow::Result<Worker> {
        let shared = Arc::new(SharedState {
            shutdown: ShutdownFlag::new(),
            pool: self.pool,
            client: self.client.unwrap_or_default(),
            notifications: broadcast::channel(128).0,
            config: self.config,
            plugins: self.plugins,
            leader: Mailbox::new(-1),
            suspend: Notify::new(),
        });

        let engine = self.engine.unwrap_or_default();
        let event_source = match self.event_source {
            Some(source) => source,
            None => Box::new(PgEventSource::new(&shared.pool).await?),
        };

        Ok(Worker {
            shared,
            engine,
            event_source,

            // A worker id of -1 should never overlap with an existing worker id.
            worker_id: -1,
            tasks: JoinSet::new(),
        })
    }
}

pub struct WorkerHandle {
    shared: Arc<SharedState>,
}

impl WorkerHandle {
    pub fn shutdown(&self) {
        self.shared.shutdown.raise();
    }
}

pub struct Worker {
    shared: Arc<SharedState>,
    engine: wasmtime::Engine,
    event_source: Box<dyn EventSource>,

    worker_id: i64,
    tasks: JoinSet<()>,
}

impl Worker {
    pub fn handle(&self) -> WorkerHandle {
        WorkerHandle {
            shared: self.shared.clone(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.worker_id = sqlx::query!(
            "
            INSERT INTO durable.worker(heartbeat_at)
            VALUES (CURRENT_TIMESTAMP)
            RETURNING id
            "
        )
        .fetch_one(&self.shared.pool)
        .await?
        .id;

        tracing::info!("durable worker id is {}", self.worker_id);

        self.shared.shutdown.reset();

        let worker_id = self.worker_id;
        let heartbeat = Self::heartbeat(self.shared.clone(), self.worker_id);
        let validate = Self::validate_workers(self.shared.clone(), self.worker_id);
        let leader = Self::leader(self.shared.clone(), self.worker_id);
        let process = self.process_events();

        // We want to run these all in the same tokio task so that if it has problems
        // then the heartbeat will fail.
        //
        // Spawned tasks are put into their own joinset because running everything in a
        // single task is not reasonable.
        let (heartbeat, validate, leader, process) = (heartbeat, validate, leader, process)
            .join()
            .instrument(tracing::info_span!("worker", worker_id))
            .await;

        let result = sqlx::query!("DELETE FROM durable.worker WHERE id = $1", self.worker_id)
            .execute(&self.shared.pool)
            .await;

        process?;
        validate?;
        heartbeat?;
        leader?;
        result?;

        Ok(())
    }

    /// This task is responsible for keeping the heartbeat up to date.
    async fn heartbeat(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
        let _guard = ShutdownGuard::new(&shared.shutdown);
        let mut shutdown = std::pin::pin!(shared.shutdown.wait());
        let mut next = Instant::now();

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                _ = tokio::time::sleep_until(next) => ()
            }

            let record = sqlx::query!(
                "UPDATE durable.worker
                  SET heartbeat_at = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING id",
                worker_id
            )
            .fetch_optional(&shared.pool)
            .await?;

            // Our record is gone from the database. This means that some other worker
            // determined that we were inactive.
            //
            // We should shutdown and then (optionally) restart with a new worker id.
            if record.is_none() {
                shared.shutdown.raise();
                anyhow::bail!("worker entry was deleted from the database");
            }

            let mut interval = shared.config.heartbeat_interval;
            let jitter = rand::thread_rng().gen_range(0..(interval / 4).as_nanos());
            interval -= Duration::from_nanos(jitter as u64);

            next += interval;
        }

        Ok(())
    }

    /// This task is responsible for periodically validating that all workers in
    /// the table are still live.
    async fn validate_workers(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
        let _guard = ShutdownGuard::new(&shared.shutdown);
        let mut shutdown = std::pin::pin!(shared.shutdown.wait());
        let mut next = Instant::now();

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                _ = tokio::time::sleep_until(next) => ()
            }

            let mut tx = shared.pool.begin().await?;
            let timeout = shared.config.heartbeat_timeout.into_pg_interval();

            let result = sqlx::query!(
                "DELETE FROM durable.worker
                WHERE CURRENT_TIMESTAMP - heartbeat_at > $1
                  AND NOT id = $2",
                timeout,
                worker_id
            )
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() > 0 {
                tracing::trace!(
                    target: "durable_runtime::validate_workers",
                    "expired {} inactive workers",
                    result.rows_affected()
                );
            }

            let record = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM durable.worker"#)
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;

            // To avoid weird cases with large clusters the maximum interval is 1 day.
            let mut interval = ((shared.config.heartbeat_timeout / 2)
                * (record.count as u32).max(1))
            .min(Duration::from_secs(24 * 3600));
            let jitter = rand::thread_rng().gen_range(0..(interval / 2).as_nanos());
            interval -= Duration::from_nanos(jitter as u64);

            tracing::trace!(
                target: "durable_runtime::validate_workers",
                "sleeping for {}s",
                interval.as_secs_f32()
            );

            next += interval;
        }

        Ok(())
    }

    async fn leader(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
        let _guard = ShutdownGuard::new(&shared.shutdown);
        let mut shutdown = std::pin::pin!(shared.shutdown.wait());

        // Start with the instant at the current time so we do an immediate
        let mut instant = Instant::now();
        let mut leader_id = shared.leader.get();
        let mut leader_stream = std::pin::pin!(shared.leader.stream());

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                id = leader_stream.as_mut().next() => leader_id = id,
                _ = shared.suspend.notified() => (),
                _ = tokio::time::sleep_until(instant) => ()
            }

            if leader_id != worker_id {
                instant += Duration::from_secs(3600);
                continue;
            }

            let mut conn = shared.pool.acquire().await?;
            sqlx::query!(
                "
                UPDATE durable.task
                  SET state = 'active',
                      wakeup_at = NULL,
                      running_on = (
                        SELECT id
                         FROM durable.worker
                        ORDER BY random()
                        LIMIT 1
                      )
                WHERE state = 'suspended'
                  AND wakeup_at <= NOW()
                "
            )
            .execute(&mut *conn)
            .await?;

            let wakeup_at = sqlx::query!(
                r#"
                SELECT wakeup_at as "wakeup_at!"
                 FROM durable.task
                WHERE state = 'suspended'
                  AND wakeup_at IS NOT NULL
                ORDER BY wakeup_at ASC
                LIMIT 1
                "#
            )
            .fetch_optional(&mut *conn)
            .await?
            .map(|record| record.wakeup_at);

            let now = Utc::now();
            let delay = match wakeup_at {
                Some(wakeup_at) => now
                    .signed_duration_since(wakeup_at)
                    .to_std()
                    .unwrap_or(Duration::ZERO),
                None => Duration::from_secs(60),
            };

            instant += delay;
        }

        Ok(())
    }

    async fn process_events(&mut self) -> anyhow::Result<()> {
        let shutdown = self.shared.shutdown.clone();
        let _guard = ShutdownGuard::new(&shutdown);
        let mut shutdown = std::pin::pin!(shutdown.wait());

        self.spawn_new_tasks().await?;
        self.load_leader_id().await?;

        'outer: loop {
            let event = tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                event = self.event_source.next() => event?
            };

            // Clean up any tasks that have completed already.
            while let Some(_) = self.tasks.try_join_next() {}

            match event {
                Event::Notification(notif) => {
                    let _ = self.shared.notifications.send(notif);
                }
                // Check if the task is scheduled to another worker. Don't do anything in that case.
                Event::Task(event::Task {
                    running_on: Some(id),
                    ..
                }) if id != self.worker_id => (),
                Event::Task(_) => self.spawn_new_tasks().await?,
                Event::TaskSuspend(_) => {
                    self.shared.suspend.notify_waiters();
                }

                Event::Worker(event::Worker { worker_id }) => {
                    let leader_id = self.shared.leader.get();
                    match leader_id {
                        id if id == worker_id => (),
                        id if id == -1 => (),
                        _ => continue,
                    }

                    self.load_leader_id().await?;
                }

                // We don't know what we missed so do everything.
                Event::Lagged => {
                    self.spawn_new_tasks().await?;
                    self.load_leader_id().await?;
                    self.shared.suspend.notify_waiters();
                }
            }
        }

        Ok(())
    }

    async fn load_leader_id(&mut self) -> anyhow::Result<()> {
        let record = sqlx::query!(
            "
            SELECT id
             FROM durable.worker
            ORDER BY started_at ASC
            LIMIT 1
            "
        )
        .fetch_optional(&self.shared.pool)
        .await?;

        let new_leader = match record {
            Some(record) => record.id,
            None => -1,
        };

        self.shared.leader.store(new_leader);

        Ok(())
    }

    /// Spawn all new tasks that are scheduled on this server and also those
    /// that aren't scheduled on any server.
    async fn spawn_new_tasks(&mut self) -> anyhow::Result<()> {
        let mut tx = self.shared.pool.begin().await?;

        let tasks = sqlx::query_as!(
            TaskData,
            r#"
            WITH selected AS (
                SELECT id
                 FROM durable.task
                WHERE state = 'active'
                  AND (running_on = $1 OR running_on IS NULL)
                FOR UPDATE SKIP LOCKED
            )
            UPDATE durable.task
              SET running_on = $1
             FROM selected, durable.wasm
            WHERE selected.id = task.id
              AND task.wasm = wasm.id
            RETURNING
                task.id     as id,
                task.name   as name,
                wasm.wasm   as wasm,
                task.data   as "data!: Json<Box<RawValue>>"
            "#,
            self.worker_id
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        for task in tasks {
            let shared = self.shared.clone();
            let engine = self.engine.clone();
            let worker_id = self.worker_id;

            self.tasks.spawn(async move {
                let task_id = task.id;
                if let Err(e) = Self::run_task(shared, engine, task, worker_id)
                    .instrument(tracing::info_span!("task", task_id))
                    .await
                {
                    tracing::error!(task_id, "worker task exited with an error: {e}");
                }
            });
        }

        Ok(())
    }

    async fn run_task(
        shared: Arc<SharedState>,
        engine: wasmtime::Engine,
        task: TaskData,
        worker_id: i64,
    ) -> anyhow::Result<()> {
        let task_id = task.id;

        let status =
            match AssertUnwindSafe(Self::run_task_impl(shared.clone(), engine, task, worker_id))
                .catch_unwind()
                .await
            {
                Ok(Ok(status)) => status,
                Ok(Err(error)) => {
                    let message = format!("{error:?}\n");

                    let result = sqlx::query!(
                        "INSERT INTO durable.log(task_id, index, message)
                        VALUES ($1, $2, $3)",
                        task_id,
                        LOG_ERROR_INDEX,
                        message
                    )
                    .execute(&shared.pool)
                    .await;

                    if let Err(e) = result {
                        tracing::error!("failed to save error logs to the database: {e}");
                    }

                    TaskStatus::ExitFailure
                }
                Err(payload) => {
                    let message: &str = if let Some(message) = payload.downcast_ref::<String>() {
                        &message
                    } else if let Some(message) = payload.downcast_ref::<&str>() {
                        message
                    } else {
                        "Box<dyn Any>"
                    };

                    tracing::error!("task {task_id} panicked: {message}");

                    let result = sqlx::query!(
                        "INSERT INTO durable.log(task_id, index, message)
                         VALUES ($1, $2, $3)",
                        task_id,
                        LOG_PANIC_INDEX,
                        format!("task panicked: {message}\n")
                    )
                    .execute(&shared.pool)
                    .await;

                    if let Err(e) = result {
                        tracing::error!("failed to save error logs to the database: {e}");
                    }

                    TaskStatus::ExitFailure
                }
            };

        match status {
            TaskStatus::NotScheduledOnWorker => {
                tracing::debug!("task {task_id} was taken by another worker");

                // Don't do anything since we no longer own this task.
            }
            TaskStatus::Suspend => {
                // The task should have set itself to the suspended state before
                // returning this error code. Nothing we need to do here.
            }
            TaskStatus::ExitSuccess => {
                sqlx::query!(
                    "UPDATE durable.task
                    SET state = 'complete',
                        completed_at = CURRENT_TIMESTAMP,
                        running_on = NULL,
                        wasm = NULL
                    WHERE id = $1
                    ",
                    task_id
                )
                .execute(&shared.pool)
                .await?;
            }
            TaskStatus::ExitFailure => {
                sqlx::query!(
                    "UPDATE durable.task
                    SET state = 'failed',
                        completed_at = CURRENT_TIMESTAMP,
                        running_on = NULL,
                        wasm = NULL
                    WHERE id = $1",
                    task_id
                )
                .execute(&shared.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn run_task_impl(
        shared: Arc<SharedState>,
        engine: wasmtime::Engine,
        mut task: TaskData,
        worker_id: i64,
    ) -> anyhow::Result<TaskStatus> {
        use wasmtime::component::*;

        tracing::info!("launching task `{}`", task.name);
        let wasm = match task.wasm.take() {
            Some(wasm) => wasm,
            None => {
                tracing::warn!("task {} was active but had wasm field set to NULL", task.id);

                let mut tx = shared.pool.begin().await?;

                sqlx::query!(
                    "UPDATE durable.task
                      SET state = 'failed',
                          running_on = NULL,
                          completed_at = CURRENT_TIMESTAMP
                    WHERE id = $1",
                    task.id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                return Ok(TaskStatus::ExitFailure);
            }
        };

        let component = tokio::task::spawn_blocking({
            let engine = engine.clone();
            move || Component::new(&engine, &wasm)
        })
        .await;

        let component = match component {
            Ok(result) => result?,
            Err(_) => {
                anyhow::bail!("component compilation panicked")
            }
        };

        let task_id = task.id;
        let mut task = Task {
            state: TaskState::new(shared.clone(), task, worker_id),
            plugins: Default::default(),
        };

        let mut linker = Linker::new(&engine);
        for plugin in shared.plugins.iter() {
            plugin
                .setup(&mut linker, &mut task)
                .with_context(|| format!("failed to set up plugin `{}`", plugin.name()))?;
        }

        // linker
        //     .define_unknown_imports_as_traps(&component)
        //     .context("failed to define unknown imports as traps")?;

        let mut store = wasmtime::Store::new(&engine, task);

        let guest = GuestPre::new(&component) //
            .context("failed to pre-load the wasm:cli/run export")?;
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .context("failed to instantiate the wasm component")?;

        let guest = guest
            .load(&mut store, &instance)
            .context("failed to load the wasi:cli/run export")?;

        let mut error = None;
        let status = match guest.call_run(&mut store).await {
            Ok(Ok(())) => TaskStatus::ExitSuccess,
            Ok(Err(())) => TaskStatus::ExitFailure,
            Err(e) => {
                if let Some(exit) = as_task_exit(&e) {
                    exit
                } else {
                    error = Some(e);
                    TaskStatus::ExitFailure
                }
            }
        };

        if !status.is_final() {
            return Ok(status);
        }

        if let Some(txn) = store.data_mut().state.transaction_mut() {
            let index = txn.index();
            let logs = std::mem::take(&mut txn.logs);

            if !logs.is_empty() {
                tracing::debug!(
                    target: "durable::task_log",
                    "task {}: {}",
                    task_id,
                    logs.trim_end()
                );

                if let Err(e) = sqlx::query!(
                    "INSERT INTO durable.log(task_id, index, message)
                     VALUES ($1, $2, $3)",
                    task_id,
                    index,
                    logs
                )
                .execute(&shared.pool)
                .await
                {
                    tracing::error!("failed to save remaining logs to the database: {e}");
                }
            }
        }

        if let Some(error) = error {
            let message = format!("{error:?}\n");

            tracing::warn!("task failed to execute with an error: {message}");

            let result = sqlx::query!(
                "INSERT INTO durable.log(task_id, index, message)
                 VALUES ($1, $2, $3)",
                task_id,
                LOG_ERROR_INDEX,
                message
            )
            .execute(&shared.pool)
            .await;

            if let Err(e) = result {
                tracing::error!("failed to save error logs to the database: {e}");
            }
        }

        Ok(status)
    }
}

fn as_task_exit(error: &anyhow::Error) -> Option<TaskStatus> {
    error
        .chain()
        .filter_map(|e| e.downcast_ref::<TaskStatus>())
        .copied()
        .next()
}

pub struct PgEventSource {
    listener: sqlx::postgres::PgListener,
}

impl PgEventSource {
    pub async fn new(pool: &sqlx::PgPool) -> sqlx::Result<Self> {
        let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await?;

        listener
            .listen_all([
                "durable:task",
                "durable:task-suspend",
                "durable:notification",
                "durable:worker",
            ])
            .await?;

        Ok(Self { listener })
    }
}

#[async_trait]
impl EventSource for PgEventSource {
    async fn next(&mut self) -> anyhow::Result<Event> {
        fn parse_event<T, F>(name: &str, event: &PgNotification, func: F) -> Event
        where
            F: FnOnce(T) -> Event,
            T: serde::de::DeserializeOwned,
        {
            match serde_json::from_str(event.payload()) {
                Ok(payload) => func(payload),
                Err(e) => {
                    tracing::warn!("listener received an invalid `{name}` notification: {e}");
                    Event::Lagged
                }
            }
        }

        loop {
            break match self.listener.try_recv().await {
                Ok(Some(event)) => {
                    tracing::trace!("received event {}: {}", event.channel(), event.payload());

                    match event.channel() {
                        "durable:task" => Ok(parse_event("durable:task", &event, Event::Task)),
                        "durable:task-suspend" => Ok(parse_event(
                            "durable:task-suspend",
                            &event,
                            Event::TaskSuspend,
                        )),
                        "durable:notification" => Ok(parse_event(
                            "durable:notification",
                            &event,
                            Event::Notification,
                        )),
                        "durable:worker" => {
                            Ok(parse_event("durable:worker", &event, Event::Worker))
                        }
                        _ => continue,
                    }
                }
                Ok(None) => Ok(Event::Lagged),
                Err(e) => {
                    tracing::warn!("listener received an error: {e}");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Ok(Event::Lagged)
                }
            };
        }
    }
}
