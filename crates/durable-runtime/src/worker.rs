use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use cache_compute::Cached;
use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
use futures_concurrency::future::Join;
use futures_util::FutureExt;
use metrics::{Counter, Gauge, Histogram};
use rand::Rng;
use serde_json::value::RawValue;
use sqlx::postgres::PgNotification;
use sqlx::types::Json;
use tokio::sync::{broadcast, mpsc, Mutex, Notify, Semaphore};
use tokio::task::JoinSet;
use tokio::time::{Instant, MissedTickBehavior};
use tracing::Instrument;
use wasmtime::component::Component;

use crate::error::{ClonableAnyhowError, TaskStatus};
use crate::event::{self, Event, EventSource, Notification};
use crate::flag::{ShutdownFlag, ShutdownGuard};
use crate::plugin::{DurablePlugin, Plugin};
use crate::task::{RecordedEvent, Task, TaskState};
use crate::util::{IntoPgInterval, Mailbox, MetricSpan};
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
    cache: Mutex<uluru::LRUCache<ProgramCache, 32>>,

    /// Limit how many task compilations are allowed to be ongoing at the same
    /// time. These are expensive and can easily end up eating all the memory
    /// and compute time needed by a worker if it gets hammered.
    compile_sema: Semaphore,

    pub(crate) metrics: SharedMetrics,
}

pub(crate) struct SharedMetrics {
    task_spawn: Counter,
    task_suspend: Counter,
    task_complete: Counter,
    task_failed: Counter,
    task_taken: Counter,
    wasm_compile_latency: Histogram,
}

impl SharedMetrics {
    pub fn new() -> Self {
        Self {
            task_spawn: metrics::counter!("durable.task_spawn"),
            task_suspend: metrics::counter!("durable.task_suspend"),
            task_complete: metrics::counter!("durable.task_complete"),
            task_failed: metrics::counter!("durable.task_failed"),
            task_taken: metrics::counter!("durable.task_tasken"),

            wasm_compile_latency: metrics::histogram!("durable.wasm_compile_latency"),
        }
    }
}

pub(crate) struct TaskData {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub wasm: i64,
    pub data: Json<Box<RawValue>>,
}

pub struct WorkerBuilder {
    config: Config,
    pool: sqlx::PgPool,
    event_source: Option<Box<dyn EventSource>>,
    client: Option<reqwest::Client>,
    wasmtime_config: Option<wasmtime::Config>,
    plugins: Vec<Box<dyn Plugin>>,
    migrate: bool,
    validate: bool,
}

impl WorkerBuilder {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            config: Config::default(),
            pool,
            event_source: None,
            client: None,
            wasmtime_config: None,
            plugins: vec![Box::new(DurablePlugin)],
            migrate: false,
            validate: true,
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

    pub fn wasmtime_config(mut self, config: wasmtime::Config) -> Self {
        self.wasmtime_config = Some(config);
        self
    }

    /// Add a new API plugin to the runtime.
    pub fn plugin(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Whether the database should be automatically migrated on runner startup
    /// if the schema version in the database differs from what we expect.
    ///
    /// If true, this will attempt to migrate the database during the
    /// [`WorkerBuilder::build`] call. Otherwise, this will error if the
    /// database version does not match the worker version.
    ///
    /// This is a low-effort way to ensure that the database is always as
    /// expected when running with a single worker. It is not recommended to use
    /// in a larger cluster.
    ///
    /// Note that automatic migrations will never revert previous migrations,
    /// this means that if you downgrade the runner then it will fail to start
    /// until a manual database revert is performed.
    ///
    /// This is false by default.
    pub fn migrate(mut self, migrate: bool) -> Self {
        self.migrate = migrate;
        self
    }

    /// Validate that the database matches what this worker needs.
    pub fn validate_database(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    pub async fn build(self) -> anyhow::Result<Worker> {
        let migrator = crate::migrate::Migrator::new();
        let mut conn = self.pool.acquire().await?;
        if self.migrate {
            let options = crate::migrate::Options {
                target: migrator.latest(),
                transaction_mode: durable_migrate::TransactionMode::Single,
                ..Default::default()
            };

            migrator
                .migrate(&mut conn, &options)
                .await
                .context("failed to migrate the database")?;
        } else if self.validate {
            let version = migrator
                .read_database_version(&mut conn)
                .await?
                .unwrap_or(0);
            let latest = migrator.latest_version();

            if version != latest {
                anyhow::bail!(
                    "database version does not match that required by this durable worker \
                     (expected {latest}, got {version} instead)"
                )
            }
        }
        drop(conn);

        let shared = Arc::new(SharedState {
            shutdown: ShutdownFlag::new(),
            client: self.client.unwrap_or_default(),
            notifications: broadcast::channel(128).0,
            leader: Mailbox::new(-1),
            suspend: Notify::new(),
            cache: Mutex::new(uluru::LRUCache::new()),
            compile_sema: Semaphore::new(self.config.max_concurrent_compilations),
            pool: self.pool,
            config: self.config,
            plugins: self.plugins,
            metrics: SharedMetrics::new(),
        });

        let mut config = self.wasmtime_config.unwrap_or_else(|| {
            let mut config = wasmtime::Config::new();
            config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
            config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            config.debug_info(true);
            config
        });

        config.async_support(true);

        let engine = wasmtime::Engine::new(&config)?;
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
            blocked: false,

            active_tasks: metrics::gauge!("durable.active_tasks"),
        })
    }
}

#[derive(Clone)]
pub struct WorkerHandle {
    shared: Arc<SharedState>,
}

impl WorkerHandle {
    /// Tell the [`Worker`] to shut down.
    ///
    /// Future calls to [`Worker::run`] will continue to shut down immediately
    /// until the flag is cleared by calling [`reset`].
    ///
    /// [`reset`]: WorkerHandle::reset
    pub fn shutdown(&self) {
        self.shared.shutdown.raise();
    }

    /// Reset the handle and allow future calls to [`Worker::run`] to process
    /// workflow tasks.
    pub fn reset(&self) {
        self.shared.shutdown.reset();
    }
}

struct ProgramCache {
    id: i64,
    value: Arc<Cached<Component, ClonableAnyhowError>>,
}

pub struct Worker {
    shared: Arc<SharedState>,
    engine: wasmtime::Engine,
    event_source: Box<dyn EventSource>,

    worker_id: i64,
    tasks: JoinSet<()>,
    blocked: bool,

    /// A metric tracking how many tasks are currently active on this worker.
    active_tasks: Gauge,
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

        self.load_leader_id().await?;

        let worker_id = self.worker_id;
        let heartbeat = Self::heartbeat(self.shared.clone(), self.worker_id)
            .instrument(tracing::info_span!("heartbeat"));
        let validate = Self::validate_workers(self.shared.clone(), self.worker_id)
            .instrument(tracing::info_span!("validate_workers"));
        let leader = Self::leader(self.shared.clone(), self.worker_id)
            .instrument(tracing::info_span!("leader"));
        let cleanup = Self::task_cleanup(self.shared.clone(), worker_id)
            .instrument(tracing::info_span!("task_cleanup"));
        let process = self
            .process_events()
            .instrument(tracing::info_span!("process"));

        // We want to run these all in the same tokio task so that if it has problems
        // then the heartbeat will fail.
        //
        // Spawned tasks are put into their own joinset because running everything in a
        // single task is not reasonable.
        let (heartbeat, validate, leader, process, cleanup) =
            (heartbeat, validate, leader, process, cleanup)
                .join()
                .instrument(tracing::info_span!("worker", worker_id))
                .await;

        tracing::info!("deleting worker database entry");
        let result = sqlx::query!("DELETE FROM durable.worker WHERE id = $1", self.worker_id)
            .execute(&self.shared.pool)
            .await
            .context("failed to delete the worker entry from the database");

        self.tasks.abort_all();

        process?;
        validate?;
        heartbeat?;
        leader?;
        cleanup?;
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
            let jitter = rand::rng().random_range(0..(interval / 4).as_nanos());
            interval -= Duration::from_nanos(jitter as u64);

            next += interval;
        }

        Ok(())
    }

    /// This task is responsible for periodically validating that all workers in
    /// the table are still live.
    async fn validate_workers(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
        // When detecting whether workers are live we want two main things:
        // - Dead workers should be removed promptly, as soon as they fail to update
        //   their heartbeat within the requested interval.
        // - We would like to avoid having the database melt down when there are a few
        //   workers.
        //
        // Explicitly out of consideration is working well when there are 1000+ workers.
        //
        // How we do things here is that we order workers by id, each worker looks at
        // the one just in front of it and schedules a liveness check for just after
        // that worker would expire. The worker with the oldest ID is then responsible
        // for checking the one with the newest ID.

        let _guard = ShutdownGuard::new(&shared.shutdown);
        let mut shutdown = std::pin::pin!(shared.shutdown.wait());
        let mut next = Instant::now();

        let mut following: Option<i64> = None;

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                _ = tokio::time::sleep_until(next) => ()
            }

            let mut tx = shared.pool.begin().await?;
            let timeout = shared.config.heartbeat_timeout.into_pg_interval();

            let mut result = if let Some(following) = following.take() {
                sqlx::query!(
                    "
                    DELETE FROM durable.worker
                    WHERE id = $1
                      AND CURRENT_TIMESTAMP - heartbeat_at > $2
                    ",
                    following,
                    timeout
                )
                .execute(&mut *tx)
                .await?
            } else {
                Default::default()
            };

            if result.rows_affected() > 0 {
                result.extend(std::iter::once(
                    sqlx::query!(
                        "
                        DELETE FROM durable.worker
                        WHERE CURRENT_TIMESTAMP - heartbeat_at > $2
                        AND NOT id = $1
                        ",
                        worker_id,
                        timeout
                    )
                    .execute(&mut *tx)
                    .await?,
                ));

                tracing::debug!(
                    target: "durable_runtime::validate_workers",
                    "deleted {} expired workers",
                    result.rows_affected()
                );
            }

            // Select either the next worker in sequence, or the newest id in the sequence.
            let record = sqlx::query!(
                r#"
                WITH
                    prev AS (
                        SELECT id, heartbeat_at
                        FROM durable.worker
                        WHERE id < $1
                        ORDER BY id DESC
                        LIMIT 1
                    ),
                    next AS (
                        SELECT id, heartbeat_at
                        FROM durable.worker
                        WHERE NOT id = $1
                        ORDER BY id DESC
                        LIMIT 1
                    ),
                    combined AS (
                        SELECT * FROM prev
                        UNION ALL
                        SELECT * FROM next
                    )
                SELECT
                    id as "id!",
                    heartbeat_at as "heartbeat_at!"
                FROM combined
                ORDER BY id ASC
                LIMIT 1
                "#,
                worker_id
            )
            .fetch_optional(&mut *tx)
            .await?;

            tx.commit().await?;

            let interval;
            if let Some(record) = record {
                tracing::trace!(
                    target: "durable_runtime::validate_workers",
                    "following worker {}",
                    record.id
                );

                following = Some(record.id);

                let expires =
                    record.heartbeat_at + shared.config.heartbeat_timeout + Duration::from_secs(1);
                interval = (expires - Utc::now()).to_std().unwrap_or_default();
            } else {
                interval = shared.config.heartbeat_interval;
            }

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

        tracing::info!("cluster leader is {}", leader_id);

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                id = leader_stream.as_mut().next() => {
                    tracing::info!("cluster leader is now {id}");
                    leader_id = id
                },
                _ = shared.suspend.notified(), if leader_id == worker_id => (),
                _ = tokio::time::sleep_until(instant) => ()
            }

            if leader_id != worker_id {
                instant += Duration::from_secs(3600);
                continue;
            }

            let mut conn = shared.pool.acquire().await?;

            // Note that we include the task id in the subquery ORDER BY clause so that
            // postgresql is forced to evaluate it for each row.
            //
            // If we don't do that all the rows here get the same random number.
            let result = sqlx::query!(
                "
                UPDATE durable.task
                  SET state = 'ready',
                      wakeup_at = NULL,
                      running_on = (
                        SELECT id
                         FROM durable.worker
                        ORDER BY random() + task.id
                        LIMIT 1
                      )
                WHERE state = 'suspended'
                  AND wakeup_at <= (NOW() - $1::interval)
                ",
                shared.config.suspend_margin.into_pg_interval()
            )
            .execute(&mut *conn)
            .await?;

            let count = result.rows_affected();
            if count > 0 {
                tracing::info!("woke up {count} tasks");
            }

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

    /// This task is responsible for deleting entries for old tasks.
    async fn task_cleanup(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
        let cleanup_age = match shared.config.cleanup_age {
            Some(cleanup_age) => cleanup_age,
            None => {
                shared.shutdown.wait().await;
                return Ok(());
            }
        };

        let _guard = ShutdownGuard::new(&shared.shutdown);
        let mut shutdown = std::pin::pin!(shared.shutdown.wait());

        let mut leader_id = shared.leader.get();
        let mut leader_stream = std::pin::pin!(shared.leader.stream());

        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        'outer: loop {
            tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                _ = interval.tick(), if leader_id == worker_id => (),
                new_leader = leader_stream.as_mut().next() => {
                    leader_id = new_leader;
                    continue 'outer;
                }
            }

            let limit = shared.config.cleanup_batch_limit as i64;
            let interval = cleanup_age.into_pg_interval();
            let mut conn = match shared.pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::warn!("failed to acquire a connection to perform task cleanup: {e}");
                    continue;
                }
            };

            // We do cleanup
            loop {
                let result = sqlx::query!(
                    r#"
                    DELETE FROM durable.task
                    WHERE task.ctid = ANY(ARRAY(
                        SELECT ctid
                        FROM durable.task
                        WHERE completed_at < NOW() - $1::interval
                        LIMIT $2
                        FOR UPDATE
                    ))
                    "#,
                    interval,
                    limit
                )
                .execute(&mut *conn)
                .await;

                match result {
                    Ok(result) if result.rows_affected() < limit as u64 => break,
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!("failed to clean up old durable tasks: {e}");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_events(&mut self) -> anyhow::Result<()> {
        let shutdown = self.shared.shutdown.clone();
        let _guard = ShutdownGuard::new(&shutdown);
        let mut shutdown = std::pin::pin!(shutdown.wait());
        let (tx, mut rx) = tokio::sync::mpsc::channel::<i64>(1024);

        self.spawn_new_tasks(&tx).await?;
        self.load_leader_id().await?;

        'outer: loop {
            let event = tokio::select! {
                biased;

                _ = shutdown.as_mut() => break 'outer,
                _ = self.tasks.join_next(), if !self.tasks.is_empty() => LoopEvent::TaskComplete,
                id = rx.recv() => LoopEvent::TaskFailed(id.expect("failed task channel closed unexpectedly")),
                event = self.event_source.next() => LoopEvent::Event(event?),
            };

            // Clean up any tasks that have completed already.
            while self.tasks.try_join_next().is_some() {}

            let event = match event {
                LoopEvent::Event(event) => event,
                LoopEvent::TaskComplete => {
                    if self.blocked {
                        self.spawn_new_tasks(&tx).await?;
                    }

                    continue;
                }
                LoopEvent::TaskFailed(id) => {
                    let mut failed = vec![id];

                    let mut count = 0;
                    while let Ok(id) = rx.try_recv() {
                        failed.push(id);

                        count += 1;
                        if count > 1024 {
                            break;
                        }
                    }

                    sqlx::query!(
                        "
                        UPDATE durable.task
                          SET state = 'ready',
                              running_on = NULL
                        WHERE id = ANY($1::bigint[])
                          AND running_on = $2
                        ",
                        &failed,
                        self.worker_id
                    )
                    .execute(&self.shared.pool)
                    .await?;

                    continue;
                }
            };

            match event {
                Event::Notification(notif) => {
                    let _ = self.shared.notifications.send(notif);
                }
                // Check if the task is scheduled to another worker. Don't do anything in that case.
                Event::Task(event::Task {
                    running_on: Some(id),
                    ..
                }) if id != self.worker_id => (),
                Event::Task(_) => self.spawn_new_tasks(&tx).await?,
                Event::TaskSuspend(_) => {
                    self.shared.suspend.notify_waiters();
                }

                Event::Worker(event::Worker { worker_id }) => {
                    let leader_id = self.shared.leader.get();
                    match leader_id {
                        id if id == worker_id => (),
                        -1 => (),
                        _ => continue,
                    }

                    self.load_leader_id().await?;
                }

                // We don't know what we missed so do everything.
                Event::Lagged => {
                    self.spawn_new_tasks(&tx).await?;
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
            ORDER BY started_at ASC, id ASC
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
    async fn spawn_new_tasks(&mut self, failure: &mpsc::Sender<i64>) -> anyhow::Result<()> {
        let max_tasks = self.shared.config.max_tasks;
        let allowed = max_tasks.saturating_sub(self.tasks.len());
        if allowed == 0 {
            return Ok(());
        }

        let mut tx = self.shared.pool.begin().await?;
        let tasks = sqlx::query_as!(
            TaskData,
            r#"
            WITH selected AS (
                SELECT id
                 FROM durable.task
                WHERE (state IN ('ready', 'active') AND running_on IS NULL)
                   OR (state = 'ready' AND running_on = $1)
                ORDER BY id ASC
                FOR NO KEY UPDATE SKIP LOCKED
                LIMIT $2
            )
            UPDATE durable.task
              SET running_on = $1,
                  state = 'active'
             FROM selected
            WHERE selected.id = task.id
            RETURNING
                task.id         as id,
                task.name       as name,
                task.created_at as created_at,
                task.wasm       as "wasm!",
                task.data       as "data!: Json<Box<RawValue>>"
            "#,
            self.worker_id,
            allowed as i64
        )
        .fetch_all(&mut *tx)
        .await?;

        if tasks.len() + self.tasks.len() >= max_tasks {
            sqlx::query!(
                "
                UPDATE durable.task
                  SET running_on = NULL
                WHERE state = 'ready'
                  AND running_on = $1
                ",
                self.worker_id
            )
            .execute(&mut *tx)
            .await?;

            self.blocked = true;
        }

        tx.commit().await?;

        if !tasks.is_empty() {
            tracing::debug!("launching {} tasks", tasks.len());
        }

        for task in tasks {
            let shared = self.shared.clone();
            let engine = self.engine.clone();
            let worker_id = self.worker_id;
            let failures = failure.clone();

            let task_id = task.id;
            tracing::trace!(
                target: "durable_runtime::worker::spawn_new_tasks",
                id = task_id,
                "launching task {}", task.name
            );

            let active_tasks = self.active_tasks.clone();
            let future = async move {
                let _guard = MetricSpan::enter(active_tasks);
                let task_id = task.id;
                if let Err(e) = Self::run_task(shared, engine, task, worker_id)
                    .instrument(tracing::info_span!("task", task_id))
                    .await
                {
                    tracing::error!(task_id, "worker task exited with an error: {e}");

                    // An error here means we are already shutting down and normal shutdown recovery
                    // should take care of any remaining tasks.
                    let _ = failures.send(task_id).await;
                }
            };

            cfg_if! {
                if #[cfg(all(tokio_unstable, feature = "tokio-console"))] {
                    self.tasks
                        .build_task()
                        .name(&format!("task {}", task_id))
                        .spawn(future)
                        .context("failed to spawn task on the joinset")?;
                } else {
                    self.tasks.spawn(future);
                }
            }
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

        shared.metrics.task_spawn.increment(1);

        // We are using the loop here to do some early breaks.
        #[allow(clippy::never_loop)]
        let status = loop {
            let future = Self::run_task_impl(shared.clone(), engine, task, worker_id);
            break match AssertUnwindSafe(future).catch_unwind().await {
                Ok(Ok(status)) => status,
                Ok(Err(error)) => {
                    match find_sqlx_error(&error) {
                        // These errors are external to the runtime and should usually be resolvable
                        // if the workflow is retried somewhere else at a later point in time.
                        //
                        // This should also help to reduce the number of workflow aborts that are
                        // "not the fault" of the workflow itself. These will (eventually) instead
                        // get turned into worker errors, which can be handled at a higher level.
                        Some(
                            sqlx::Error::PoolTimedOut
                            | sqlx::Error::WorkerCrashed
                            | sqlx::Error::Io(_),
                        ) => {
                            // Attempt to reset the task state so it can be picked up again.
                            //
                            // If this fails then the task failure gets reported to the main event
                            // loop which can ensure it gets retried.
                            sqlx::query!(
                                "
                                UPDATE durable.task
                                SET state = 'ready',
                                    running_on = NULL
                                WHERE id = $1
                                  AND running_on = $2
                                ",
                                task_id,
                                worker_id
                            )
                            .execute(&shared.pool)
                            .await?;

                            break TaskStatus::Suspend;
                        }
                        Some(sqlx::Error::PoolClosed) => {
                            // Nothing we can do, since we can't make database queries.
                            break TaskStatus::Suspend;
                        }
                        _ => (),
                    }

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
                        message
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
        };

        match status {
            TaskStatus::NotScheduledOnWorker => {
                tracing::debug!("task {task_id} was taken by another worker");

                // Don't do anything since we no longer own this task.
                shared.metrics.task_taken.increment(1);
            }
            TaskStatus::Suspend => {
                // The task should have set itself to the suspended state before
                // returning this error code. Nothing we need to do here.
                shared.metrics.task_suspend.increment(1);
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

                shared.metrics.task_complete.increment(1);
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

                shared.metrics.task_failed.increment(1);
            }
        }

        tracing::trace!("task exited with status {status:?}");

        Ok(())
    }

    async fn run_task_impl(
        shared: Arc<SharedState>,
        engine: wasmtime::Engine,
        task: TaskData,
        worker_id: i64,
    ) -> anyhow::Result<TaskStatus> {
        use wasmtime::component::*;

        use crate::bindings::Imports;

        // tracing::info!(
        //     target: "durable_runtime::worker::task_launch",
        //     "launching task `{}`", task.name);
        let component = {
            let mut cache = shared.cache.lock().await;

            match cache.find(|entry| entry.id == task.wasm) {
                Some(entry) => entry.value.clone(),
                None => {
                    let cached = Arc::new(Cached::new());

                    cache.insert(ProgramCache {
                        id: task.wasm,
                        value: cached.clone(),
                    });

                    cached
                }
            }
        };

        // Compile the component, but perform request coalescing so that it only happens
        // once. Compiling one is an expensive operation, so if
        let component = component
            .get_or_compute(|| async {
                let record = sqlx::query!("SELECT wasm FROM durable.wasm WHERE id = $1", task.wasm)
                    .fetch_one(&shared.pool)
                    .await
                    .map_err(anyhow::Error::from)?;

                // If an error occurs then we just allow ourselves to proceed anyway.
                let _permit = shared.compile_sema.acquire().await;

                let wasm = record.wasm;
                let start = Instant::now();
                let component = tokio::task::spawn_blocking({
                    let engine = engine.clone();
                    move || Component::new(&engine, &wasm)
                })
                .await
                .context("component compilation panicked")??;

                let elapsed = start.elapsed();
                tracing::debug!(
                    target: "durable_runtime::worker::task_compile",
                    id = task.wasm,
                    "compiling new module took {}",
                    humantime::Duration::from(elapsed)
                );

                shared.metrics.wasm_compile_latency.record(elapsed);

                Ok(component)
            })
            .await?;

        let events = sqlx::query_as!(
            RecordedEvent,
            r#"
            SELECT
                index,
                label,
                value as "value!: Json<Box<RawValue>>"
             FROM durable.event
            WHERE task_id = $1
            ORDER BY index ASC
            LIMIT 1000
            "#,
            task.id
        )
        .fetch_all(&shared.pool)
        .await
        .unwrap_or_default();

        let task_id = task.id;
        let mut task = Task {
            state: TaskState::new(shared.clone(), task, worker_id, events),
            plugins: Default::default(),
            resources: crate::Resources::default(),
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

        let instance = Imports::instantiate_async(&mut store, &component, &linker)
            .await
            .context("failed to instantiate the wasm component")?;
        let guest = instance.wasi_cli_run();

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

        // Some errors are recoverable. We handle those at up one method so that retries
        // can propagate upwards if recovery fails.
        let error = match error {
            Some(error) if is_recoverable_error(&error) => return Err(error),
            error => error,
        };

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
                 VALUES ($1, $2, $3)
                 ON CONFLICT ON CONSTRAINT log_pkey DO UPDATE
                 SET message = $3
                 ",
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
        let mut listener = sqlx::postgres::PgListener::connect_with(pool).await?;

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
                Ok(None) => {
                    while let Err(e) = sqlx::query("SELECT 1").execute(&mut self.listener).await {
                        tracing::warn!("listener received an error: {e}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    Ok(Event::Lagged)
                }
                Err(mut e) => {
                    loop {
                        tracing::warn!("listener received an error: {e}");

                        match sqlx::query("SELECT 1").execute(&mut self.listener).await {
                            Ok(_) => break,
                            Err(err) => e = err,
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    Ok(Event::Lagged)
                }
            };
        }
    }
}

enum LoopEvent {
    Event(Event),
    TaskComplete,
    TaskFailed(i64),
}

fn find_sqlx_error(error: &anyhow::Error) -> Option<&sqlx::Error> {
    error.chain().filter_map(|e| e.downcast_ref()).next()
}

fn is_recoverable_error(error: &anyhow::Error) -> bool {
    if let Some(error) = find_sqlx_error(error) {
        return matches!(
            error,
            sqlx::Error::PoolTimedOut
                | sqlx::Error::WorkerCrashed
                | sqlx::Error::Io(_)
                | sqlx::Error::PoolClosed
        );
    }

    false
}
