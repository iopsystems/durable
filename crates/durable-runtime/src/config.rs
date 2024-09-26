use std::time::Duration;

use derive_setters::Setters;

use crate::util::EmptyMapDeserializer;

/// Config options controlling the behaviour of this worker.
#[derive(Clone, Debug, Setters, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Config {
    /// The period with which the worker will update its heartbeat timestamp in
    /// the database.
    ///
    /// The actual update periods will be jittered downwards by up to 1/4 of the
    /// period to avoid thundering herds on the database server.
    #[serde(default = "default_seconds::<30>")]
    #[serde(with = "duration_seconds")]
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
    #[serde(default = "default_seconds::<120>")]
    #[serde(with = "duration_seconds")]
    pub heartbeat_timeout: Duration,

    /// The duration that the entry for a binary will be kept around after it
    /// was last used before the a worker attempts to remove it.
    ///
    /// This must be greater than 2 hours otherwise it is likely that programs
    /// will be removed out from underneath clients.
    ///
    /// The default duration is 24 hours.
    #[serde(default = "default_seconds::<{ 24 * 3600 }>")]
    #[serde(with = "duration_seconds")]
    pub wasm_entry_ttl: Duration,

    /// The maximum permitted timeout when a workflow makes HTTP requests.
    ///
    /// Timeouts longer than this maximum will be clamped and if no timeout is
    /// provided then this is the timeout that will be used.
    #[serde(default = "default_seconds::<60>")]
    #[serde(with = "duration_seconds")]
    pub max_http_timeout: Duration,

    /// The maximum permitted number of events that can be emitted by a workflow
    /// before it will be automatically terminated.
    ///
    /// This is meant as a safety measure against workflows that would use too
    /// many resources.
    #[serde(default = "default_u32::<{ i32::MAX as u32 }>")]
    pub max_workflow_events: u32,

    /// The maximum number of bytes that are permitted to be logged in a single
    /// transaction.
    ///
    /// Bytes written after this cap is reached will still succeed in the guest
    /// but will be silently dropped without being saved.
    ///
    /// The default limit here is 128KB.
    #[serde(default = "default_usize::<{ 1024 * 128 }>")]
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
    #[serde(default = "default_usize::<{ 8 * 1024 * 1024 }>")]
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
    #[serde(default = "default_seconds::<60>")]
    #[serde(with = "duration_seconds")]
    pub suspend_timeout: Duration,

    /// The duration that a task will be woken up before its suspend timeout
    /// completes.
    ///
    /// This is meant to give a task time to replay up until its original
    /// location so that when the timer actually expires the task is already at
    /// that location and ready to go.
    #[serde(default = "default_seconds::<10>")]
    #[serde(with = "duration_seconds")]
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
    #[serde(default = "default_usize::<2000>")]
    pub max_tasks: usize,

    /// The maximum number of WASM binaries that can be compiled concurrently.
    ///
    /// Compiling WASM down to machine code is moderately expensive (e.g. a
    /// decent sized module can take 300ms to compile) so if a worker has to
    /// build it can use up all the available cores and memory on a machine.
    ///
    /// Note that, once compiled, the resulting machine code is cached so if the
    /// same WASM binary is encountered multiple times then the compiled code
    /// will be reused.
    ///
    /// The default limit is 4 concurrent compilation tasks.
    #[serde(default = "default_usize::<4>")]
    pub max_concurrent_compilations: usize,

    /// Print task logs directly to stdout while running.
    ///
    /// This is mainly meant as a debugging option for use in tests.
    #[serde(default)]
    pub debug_emit_task_logs: bool,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        use serde::Deserialize;

        #[allow(unreachable_patterns)]
        match Self::deserialize(EmptyMapDeserializer) {
            Ok(config) => config,
            Err(e) => match e {},
        }
    }
}

#[test]
#[cfg(test)]
fn config_default_does_not_panic() {
    let _ = Config::default();
}

const fn default_seconds<const SECONDS: u64>() -> Duration {
    Duration::from_secs(SECONDS)
}

const fn default_u32<const N: u32>() -> u32 {
    N
}

const fn default_usize<const N: usize>() -> usize {
    N
}

mod duration_seconds {
    use std::time::Duration;

    use serde::Serialize;

    pub(crate) fn serialize<S>(duration: &Duration, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if ser.is_human_readable() && duration.subsec_nanos() == 0 {
            duration.as_secs().serialize(ser)
        } else {
            duration.as_secs_f64().serialize(ser)
        }
    }

    pub(crate) fn deserialize<'de, D>(de: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Duration;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a duration in seconds")
            }

            fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Duration::from_secs(v))
            }

            fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                if v < 0 {
                    return Err(Error::custom("durations cannot be negative"));
                }

                self.visit_u64(v as u64)
            }

            fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
                if v < 0.0 {
                    return Err(Error::custom("durations cannot be negative"));
                }

                match Duration::try_from_secs_f32(v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(Error::custom("duration was too large to be represented")),
                }
            }

            fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                if v < 0.0 {
                    return Err(Error::custom("durations cannot be negative"));
                }

                match Duration::try_from_secs_f64(v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(Error::custom("duration was too large to be represented")),
                }
            }
        }

        de.deserialize_f64(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_defaults() {
        let toml = r#"
heartbeat_interval = 30
heartbeat_timeout = 120
wasm_entry_ttl = 86400
max_http_timeout = 60
max_workflow_events = 2147483647
max_log_bytes_per_transaction = 131072
max_returned_buffer_len = 8388608
suspend_timeout = 60
suspend_margin = 10
max_tasks = 2000
max_concurrent_compilations = 4
debug_emit_task_logs = false
"#;

        let _: Config = toml::from_str(toml).unwrap();
    }
}
