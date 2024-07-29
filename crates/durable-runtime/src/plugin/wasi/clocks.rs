use std::time::{Duration, SystemTime};

use wasmtime::component::Resource;

use super::WasiResources;
use crate::bindings::wasi;
use crate::bindings::wasi::clocks::monotonic_clock::{self as monotonic, Instant};
use crate::bindings::wasi::clocks::wall_clock::Datetime;
use crate::bindings::wasi::io::poll::Pollable;
use crate::plugin::PluginMapExt;
use crate::task::{Task, TransactionOptions};

#[derive(Serialize, Deserialize)]
struct SerializableDateTime {
    seconds: u64,
    nanoseconds: u32,
}

impl From<SerializableDateTime> for Datetime {
    fn from(value: SerializableDateTime) -> Self {
        Self {
            seconds: value.seconds,
            nanoseconds: value.nanoseconds,
        }
    }
}

#[async_trait::async_trait]
impl wasi::clocks::wall_clock::Host for Task {
    async fn now(&mut self) -> wasmtime::Result<Datetime> {
        let options = TransactionOptions::new("wasi:clocks/wall-clock.now");
        self.state
            .maybe_do_transaction(options, |_| {
                Box::pin(async move {
                    let now = SystemTime::now();
                    let duration = now
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or(Duration::ZERO);

                    Ok(SerializableDateTime {
                        seconds: duration.as_secs(),
                        nanoseconds: duration.subsec_nanos(),
                    })
                })
            })
            .await
            .map(From::from)
    }

    fn resolution(&mut self) -> wasmtime::Result<Datetime> {
        // The underlying clocks on the host don't necessarily have a consistent
        // resolution. This is especially true since the workflow may move between
        // hosts.
        //
        // Instead we just lie and say we've got a resolution of 1us.
        Ok(Datetime {
            seconds: 0,
            nanoseconds: 1000,
        })
    }
}

#[async_trait::async_trait]
impl wasi::clocks::monotonic_clock::Host for Task {
    async fn now(&mut self) -> wasmtime::Result<Instant> {
        let options = TransactionOptions::new("wasi:clocks/monotonic-clock.now");
        self.state
            .maybe_do_transaction(options, |_| {
                Box::pin(async move {
                    let now = SystemTime::now();
                    let duration = now
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or(Duration::ZERO);

                    let instant = duration.as_nanos().try_into().unwrap_or(u64::MAX);
                    Ok(instant)
                })
            })
            .await
    }

    fn resolution(&mut self) -> wasmtime::Result<monotonic::Duration> {
        // The underlying clocks on the host don't necessarily have a consistent
        // resolution. This is especially true since the workflow may move between
        // hosts.
        //
        // Instead we just lie and say we've got a resolution of 1us.
        Ok(1000)
    }

    async fn subscribe_instant(&mut self, when: Instant) -> wasmtime::Result<Resource<Pollable>> {
        let txn = self.state.transaction().map(|txn| txn.index());
        let timeout = SystemTime::UNIX_EPOCH + Duration::from_nanos(when);

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
