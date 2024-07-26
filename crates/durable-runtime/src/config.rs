use std::time::Duration;

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

    /// The maximum permitted timeout when a workflow makes HTTP requests.
    ///
    /// Timeouts longer than this maximum will be clamped and if no timeout is
    /// provided then this is the timeout that will be used.
    pub max_http_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            heartbeat_timeout: Duration::from_secs(120),
            max_http_timeout: Duration::from_secs(60),
        }
    }
}
