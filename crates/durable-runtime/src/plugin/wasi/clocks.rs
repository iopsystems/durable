use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasmtime::component::Resource;

use super::WasiResources;
use crate::bindings::wasi;
use crate::bindings::wasi::clocks::monotonic_clock::{self as monotonic, Instant};
use crate::bindings::wasi::clocks::wall_clock::Datetime;
use crate::bindings::wasi::io::poll::Pollable;
use crate::plugin::PluginMapExt;
use crate::task::{Task, TransactionOptions};

const NS_PER_S: u64 = 1_000_000_000;

impl From<SerializableDateTime> for Datetime {
    fn from(value: SerializableDateTime) -> Self {
        Self {
            seconds: value.seconds,
            nanoseconds: value.nanoseconds,
        }
    }
}

impl wasi::clocks::wall_clock::Host for Task {
    async fn now(&mut self) -> wasmtime::Result<Datetime> {
        let options = TransactionOptions::new("wasi:clocks/wall-clock.now");
        self.state
            .maybe_do_transaction_sync(options, |_| {
                let now = SystemTime::now();
                let duration = now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO);

                Ok(Datetime::from(duration))
            })
            .await
    }

    async fn resolution(&mut self) -> wasmtime::Result<Datetime> {
        // The underlying clocks on the host don't necessarily have a consistent
        // resolution. This is especially true since the workflow may move between
        // hosts.
        //
        // Instead we just lie and say we've got a resolution of 1us.
        Ok(Duration::from_micros(1).into())
    }
}

impl wasi::clocks::monotonic_clock::Host for Task {
    async fn now(&mut self) -> wasmtime::Result<Instant> {
        let options = TransactionOptions::new("wasi:clocks/monotonic-clock.now");
        self.state
            .maybe_do_transaction_sync(options, |_| {
                let now = Utc::now();
                let timestamp = now.timestamp() as u64;
                let nanos = timestamp * NS_PER_S + now.timestamp_subsec_nanos() as u64;

                Ok(nanos)
            })
            .await
    }

    async fn resolution(&mut self) -> wasmtime::Result<monotonic::Duration> {
        // The underlying clocks on the host don't necessarily have a consistent
        // resolution. This is especially true since the workflow may move between
        // hosts.
        //
        // Instead we just lie and say we've got a resolution of 1us.
        Ok(1000)
    }

    async fn subscribe_instant(&mut self, when: Instant) -> wasmtime::Result<Resource<Pollable>> {
        let txn = self.state.transaction().map(|txn| txn.index());
        let timeout = DateTime::from_timestamp((when / NS_PER_S) as i64, (when % NS_PER_S) as u32)
            .unwrap_or(DateTime::<Utc>::MAX_UTC);

        let resources = self.plugins.expect_mut::<WasiResources>();
        let id = resources.pollables.insert(super::Pollable { timeout, txn });

        Ok(Resource::new_own(id as u32))
    }

    async fn subscribe_duration(
        &mut self,
        when: monotonic::Duration,
    ) -> wasmtime::Result<Resource<Pollable>> {
        let now = self.now().await?;
        self.subscribe_instant(now.saturating_add(when)).await
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Datetime")]
struct SerializableDateTime {
    seconds: u64,
    nanoseconds: u32,
}

impl Serialize for Datetime {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializableDateTime::serialize(self, ser)
    }
}

impl<'de> Deserialize<'de> for Datetime {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SerializableDateTime::deserialize(de)
    }
}

impl From<Duration> for Datetime {
    fn from(value: Duration) -> Self {
        Datetime {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos(),
        }
    }
}
