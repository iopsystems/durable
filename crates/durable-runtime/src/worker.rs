use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use futures_concurrency::future::Join;
use futures_util::FutureExt;
use rand::Rng;
use serde_json::value::RawValue;
use sqlx::postgres::types::PgInterval;
use sqlx::types::Json;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::time::Instant;
use tracing::Instrument;

use crate::error::AbortError;
use crate::event::{Event, EventSource, NotificationInserted, TaskInserted};
use crate::flag::{ShutdownFlag, ShutdownGuard};
use crate::plugin::durable::DurablePlugin;
use crate::plugin::{Plugin, TaskState};
use crate::{Config, Task};

pub(crate) struct SharedState {
    pub shutdown: ShutdownFlag,
    pub pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub notifications: broadcast::Sender<NotificationInserted>,
    pub config: Config,
    plugins: Vec<Box<dyn Plugin>>,
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
            plugins: vec![
                Box::new(DurablePlugin),
            ],
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
            "INSERT INTO worker(last_heartbeat) VALUES (CURRENT_TIMESTAMP) RETURNING id"
        )
        .fetch_one(&self.shared.pool)
        .await?
        .id;

        let heartbeat = Self::heartbeat(self.shared.clone(), self.worker_id);
        let validate = Self::validate_workers(self.shared.clone());
        let process = self.process_events();

        // We want to run these all in the same tokio task so that if it has problems
        // then the heartbeat will fail.
        //
        // Spawned tasks are put into their own joinset because running everything in a
        // single task is not reasonable.
        let (heartbeat, validate, process) = (heartbeat, validate, process).join().await;

        let result = sqlx::query!("DELETE FROM worker WHERE id = $1", self.worker_id)
            .execute(&self.shared.pool)
            .await;

        process?;
        validate?;
        heartbeat?;
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
                "UPDATE worker SET last_heartbeat = CURRENT_TIMESTAMP WHERE id = $1 RETURNING id",
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
    async fn validate_workers(shared: Arc<SharedState>) -> anyhow::Result<()> {
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
            let timeout = PgInterval {
                months: 0,
                days: 0,
                microseconds: shared
                    .config
                    .heartbeat_interval
                    .as_micros()
                    .try_into()
                    .unwrap_or(i64::MAX),
            };

            sqlx::query!(
                "DELETE FROM worker WHERE CURRENT_TIMESTAMP - last_heartbeat > $1",
                timeout
            )
            .execute(&mut *tx)
            .await?;

            let record = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM worker"#)
                .fetch_one(&mut *tx)
                .await?;

            // To avoid weird cases with large clusters the maximum interval is 1 day.
            let mut interval = ((shared.config.heartbeat_timeout / 2)
                * (record.count as u32).max(1))
            .min(Duration::from_secs(24 * 3600));
            let jitter = rand::thread_rng().gen_range(0..(interval / 2).as_nanos());
            interval -= Duration::from_nanos(jitter as u64);

            next += interval;
        }

        Ok(())
    }

    async fn process_events(&mut self) -> anyhow::Result<()> {
        let shutdown = self.shared.shutdown.clone();
        let _guard = ShutdownGuard::new(&shutdown);
        let mut shutdown = std::pin::pin!(shutdown.wait());

        'outer: loop {
            let event = tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                event = self.event_source.next() => event?
            };

            // Clean up any tasks that have completed already.
            while let Some(_) = self.tasks.try_join_next() {}

            match event {
                Event::NotificationInserted(notif) => {
                    let _ = self.shared.notifications.send(notif);
                    continue;
                }
                // Check if the task is scheduled to another worker. Don't do anything in that case.
                Event::TaskInserted(TaskInserted {
                    running_on: Some(id),
                    ..
                }) if id != self.worker_id => continue,
                _ => (),
            }

            self.spawn_new_tasks().await?;
        }

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
                 FROM task
                WHERE state = 'active'
                  AND (running_on = $1 OR running_on IS NULL)
                FOR UPDATE SKIP LOCKED
            )
            UPDATE task
              SET running_on = $1
             FROM selected
            WHERE selected.id = task.id
            RETURNING
                task.id     as id,
                task.name   as name,
                task.wasm   as wasm,
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

        match AssertUnwindSafe(Self::run_task_impl(shared.clone(), engine, task, worker_id))
            .catch_unwind()
            .await
        {
            Ok(Ok(())) => {
                sqlx::query!(
                    "UPDATE task
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
            Ok(Err(e)) if e.is::<AbortError>() => {
                tracing::info!("task {task_id} was taken by another worker");

                // Don't do anything since we no longer own the task.
            }
            Ok(Err(e)) => {
                tracing::warn!("task {task_id} failed to execute with an error: {e:?}");

                let mut tx = shared.pool.begin().await?;

                sqlx::query!(
                    "INSERT INTO event(task_id, index, label, value)
                    VALUES ($1, -1, 'durable:error-message', $2)",
                    task_id,
                    Json(format!("{e:?}")) as Json<String>
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!(
                    "UPDATE task
                    SET state = 'failed',
                        completed_at = CURRENT_TIMESTAMP,
                        running_on = NULL,
                        wasm = NULL
                    WHERE id = $1",
                    task_id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
            Err(e) => {
                let message: &str = if let Some(message) = e.downcast_ref::<String>() {
                    &message
                } else if let Some(message) = e.downcast_ref::<&str>() {
                    message
                } else {
                    "<opaque panic payload>"
                };

                tracing::error!("task {task_id} panicked: {message}");

                let mut tx = shared.pool.begin().await?;

                sqlx::query!(
                    "INSERT INTO event(task_id, index, label, value)
                    VALUES ($1, -1, 'durable:error-message', $2)",
                    task_id,
                    Json(message) as Json<&str>
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!(
                    "UPDATE task
                    SET state = 'failed',
                        completed_at = CURRENT_TIMESTAMP,
                        running_on = NULL,
                        wasm = NULL
                    WHERE id = $1",
                    task_id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
        }

        Ok(())
    }

    async fn run_task_impl(
        shared: Arc<SharedState>,
        engine: wasmtime::Engine,
        mut task: TaskData,
        worker_id: i64,
    ) -> anyhow::Result<()> {
        use wasmtime::component::*;

        tracing::info!("launching task `{}`", task.name);
        let wasm = match task.wasm.take() {
            Some(wasm) => wasm,
            None => {
                tracing::warn!("task {} was active but had wasm field set to NULL", task.id);

                let mut tx = shared.pool.begin().await?;

                sqlx::query!(
                    "UPDATE task
                      SET state = 'failed',
                          running_on = NULL,
                          completed_at = CURRENT_TIMESTAMP
                    WHERE id = $1",
                    task.id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                return Ok(());
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

        linker.define_unknown_imports_as_traps(&component)?;

        let mut store = wasmtime::Store::new(&engine, task);
        let instance = linker
            .instantiate(&mut store, &component)
            .context("failed to pre-instantiate the wasm component")?;

        let start = match instance.get_func(&mut store, "_start") {
            Some(start) => start,
            None => anyhow::bail!("task wasm module did not contain a `_start` function"),
        };

        start.call_async(&mut store, &[], &mut []).await
    }
}

pub struct PgEventSource {
    listener: sqlx::postgres::PgListener,
}

impl PgEventSource {
    pub async fn new(pool: &sqlx::PgPool) -> sqlx::Result<Self> {
        let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await?;

        listener
            .listen_all(["durable:task-inserted", "durable:notification-inserted"])
            .await?;

        Ok(Self { listener })
    }
}

#[async_trait]
impl EventSource for PgEventSource {
    async fn next(&mut self) -> anyhow::Result<Event> {
        loop {
            break match self.listener.try_recv().await {
                Ok(Some(event)) => match event.channel() {
                    "durable:task-inserted" => {
                        let payload = match serde_json::from_str(event.payload()) {
                            Ok(payload) => payload,
                            Err(e) => {
                                tracing::warn!(
                                    "listener received an invalid `durable:task-inserted` \
                                     notification: {e}"
                                );
                                return Ok(Event::Lagged);
                            }
                        };

                        Ok(Event::TaskInserted(payload))
                    }
                    "durable:notification-inserted" => {
                        let payload = match serde_json::from_str(event.payload()) {
                            Ok(payload) => payload,
                            Err(e) => {
                                tracing::warn!(
                                    "listener received an invalid `durable:notification-inserted` \
                                     notification: {e}"
                                );
                                return Ok(Event::Lagged);
                            }
                        };

                        Ok(Event::NotificationInserted(payload))
                    }
                    _ => continue,
                },
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
