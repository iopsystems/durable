use std::time::Duration;

use derive_setters::Setters;

/// Config options controlling the behaviour of this worker.
#[derive(Clone, Debug, Setters)]
pub struct Config {
    /// The period with which the worker will update its heartbeat timestamp in
    /// the database.
    ///
    /// The actual update periods will be jittered downwards by up to 1/4 of the
    /// period to avoid thundering herds on the database server.
    pub heartbeat_interval: Duration,

    /// The timeout after which a worker is considered to have disappeared if it
    /// doesn't update its heartbeat timestamp.
    ///
    /// It is recommended to set this to at least 2x the heartbeat interval, if
    /// not more.
    ///
    /// All the workers will attempt to collectively scale their dead host
    /// checks so that on average there are 2 checks per heartbeat period.
    /// Note that a normal shutdown of a worker will proactively delete its
    /// worker entry in the database and doesn't need to be expired via a
    /// heartbeat.
    pub heartbeat_timeout: Duration,

    /// The duration that the entry for a binary will be kept around after it
    /// was last used before the a worker attempts to remove it.
    ///
    /// This must be greater than 2 hours otherwise it is likely that programs
    /// will be removed out from underneath clients.
    ///
    /// The default duration is 24 hours.
    pub wasm_entry_ttl: Duration,

    /// The maximum permitted timeout when a workflow makes HTTP requests.
    ///
    /// Timeouts longer than this maximum will be clamped and if no timeout is
    /// provided then this is the timeout that will be used.
    pub max_http_timeout: Duration,

    /// The maximum permitted number of events that can be emitted by a workflow
    /// before it will be automatically terminated.
    ///
    /// This is meant as a safety measure against workflows that would use too
    /// many resources.
    pub max_workflow_events: u32,

    /// The maximum number of bytes that are permitted to be logged in a single
    /// transaction.
    ///
    /// Bytes written after this cap is reached will still succeed in the guest
    /// but will be silently dropped without being saved.
    ///
    /// The default limit here is 128KB.
    pub max_log_bytes_per_transaction: usize,

    /// The maximum permitted size, in bytes, of any buffers that are directly
    /// controlled by the workflow program.
    ///
    /// Some WASI methods allow the workflow program to instruct the runtime to
    /// directly construct buffers of a given size. This can lead to DOS
    /// vulnerabilities if the runtime attempts to construct an extremely large
    /// buffer. This function serves to limit that to something more reasonable.
    ///
    /// By default this is set to 8MB.
    pub max_returned_buffer_len: usize,

    /// The duration that a task will wait on a timer or notification without
    /// suspending itself.
    ///
    /// For timers, this means that if the deadline is further away then the
    /// timeout (+margin for wakeup) then the task will be suspended. For
    /// notifications, the task will wait for this timeout and then suspend
    /// itself.
    ///
    /// By default, this timeout is 1 minute.
    pub suspend_timeout: Duration,

    /// The duration that a task will be woken up before its suspend timeout
    /// completes.
    ///
    /// This is meant to give a task time to replay up until its original
    /// location so that when the timer actually expires the task is already at
    /// that location and ready to go.
    pub suspend_margin: Duration,

    /// The maximum number of tasks that are allowed to be running on this node
    /// at once.
    ///
    /// Each task takes up some resources, so there is already a limit to how
    /// many can be run at once. Reaching the available resource limit will
    /// result in tasks failing since they could not allocate the resources they
    /// need. Instead of having that happen, you can explicitly limit the number
    /// of tasks that will run on the worker here.
    ///
    /// Note that with default memory settings there is a hard limit at 64k
    /// active tasks on a single worker (assuming x86_64). This is because, by
    /// default, each wasm memory uses 6GB of virtual memory, even if the memory
    /// itself is much smaller. Depending on the sqlx database pool settings,
    /// you may also find that there are not enough database connections to
    /// service all the tasks.
    ///
    /// The default limit is 2000 tasks.
    pub max_tasks: usize,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            heartbeat_timeout: Duration::from_secs(120),
            wasm_entry_ttl: Duration::from_secs(24 * 3600),
            max_http_timeout: Duration::from_secs(60),
            max_workflow_events: i32::MAX as u32,
            max_log_bytes_per_transaction: 1024 * 128,
            max_returned_buffer_len: 1024 * 1024 * 8,
            suspend_timeout: Duration::from_secs(60),
            suspend_margin: Duration::from_secs(10),
            max_tasks: 1000,
        }
    }
}
