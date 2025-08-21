use std::time::Duration;

use chrono::{DateTime, Utc};
use wasi::io::poll::Pollable;
use wasi::io::streams::{InputStream, OutputStream, StreamError};
use wasmtime::component::Resource;

use super::WasiResources;
use crate::bindings::wasi;
use crate::plugin::PluginMapExt;
use crate::task::TransactionOptions;
use crate::Task;

/// The duration that the task will be woken in advance of the timer becoming
/// ready.
const SUSPEND_PREWAKE: Duration = Duration::from_secs(10);

impl wasi::io::error::HostError for Task {
    async fn to_debug_string(
        &mut self,
        res: Resource<wasi::io::error::Error>,
    ) -> wasmtime::Result<String> {
        let resources = self.plugins.expect::<WasiResources>();
        let error = &resources.errors[res.rep() as usize];

        Ok(error.to_string())
    }

    async fn drop(&mut self, res: Resource<wasi::io::error::Error>) -> wasmtime::Result<()> {
        let resources = self.plugins.expect_mut::<WasiResources>();
        resources.errors.remove(res.rep() as usize);
        Ok(())
    }
}

impl wasi::io::error::Host for Task {}

// Stub implementation of input streams.
//
// Workflows have no input streams. However, in order to support the CLI
// environment we
impl wasi::io::streams::HostInputStream for Task {
    async fn read(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<Vec<u8>, StreamError>> {
        // Workflows have no input streams. We model this by always indicating that
        // the stream is closed.
        Ok(Err(StreamError::Closed))
    }

    async fn blocking_read(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<Vec<u8>, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    async fn skip(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    async fn blocking_skip(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    async fn subscribe(
        &mut self,
        _: Resource<InputStream>,
    ) -> wasmtime::Result<Resource<Pollable>> {
        Ok(Resource::new_own(u32::MAX))
    }

    async fn drop(&mut self, _: Resource<InputStream>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl wasi::io::streams::HostOutputStream for Task {
    async fn check_write(
        &mut self,
        _: Resource<OutputStream>,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Ok(u64::MAX))
    }

    async fn write(
        &mut self,
        stream: Resource<OutputStream>,
        contents: Vec<u8>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        if stream.rep() != 1 {
            return Ok(Err(StreamError::Closed));
        }

        let options = TransactionOptions::new("wasi:io/streams.output-stream.write");
        self.state
            .maybe_do_transaction_sync(options, move |state| {
                let txn = state.transaction_mut().unwrap();
                let utf8 = String::from_utf8_lossy(&contents);
                txn.write_logs(&utf8);
                Ok(())
            })
            .await?;

        Ok(Ok(()))
    }

    async fn blocking_write_and_flush(
        &mut self,
        stream: Resource<OutputStream>,
        contents: Vec<u8>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        self.write(stream, contents).await
    }

    async fn flush(
        &mut self,
        stream: Resource<OutputStream>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        Ok(match stream.rep() {
            1 => Ok(()),
            _ => Err(StreamError::Closed),
        })
    }

    async fn blocking_flush(
        &mut self,
        stream: Resource<OutputStream>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        self.flush(stream).await
    }

    async fn subscribe(
        &mut self,
        _: Resource<OutputStream>,
    ) -> wasmtime::Result<Resource<Pollable>> {
        Ok(Resource::new_own(u32::MAX))
    }

    async fn write_zeroes(
        &mut self,
        stream: Resource<OutputStream>,
        len: u64,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        if stream.rep() != 1 {
            return Ok(Err(StreamError::Closed));
        }

        let written = self
            .state
            .transaction()
            .map(|txn| txn.logs.len())
            .unwrap_or(0);
        let remaining = self
            .state
            .config()
            .max_log_bytes_per_transaction
            .saturating_sub(written);

        let zeroes = vec![0u8; remaining.min(len.try_into().unwrap_or(usize::MAX))];
        self.write(stream, zeroes).await
    }

    async fn blocking_write_zeroes_and_flush(
        &mut self,
        stream: Resource<OutputStream>,
        len: u64,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        self.write_zeroes(stream, len).await
    }

    async fn splice(
        &mut self,
        _dst: Resource<OutputStream>,
        _src: Resource<InputStream>,
        _len: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    async fn blocking_splice(
        &mut self,
        _dst: Resource<OutputStream>,
        _src: Resource<InputStream>,
        _len: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    async fn drop(&mut self, _: Resource<OutputStream>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl wasi::io::streams::Host for Task {}

impl wasi::io::poll::HostPollable for Task {
    async fn ready(&mut self, pollable: Resource<Pollable>) -> wasmtime::Result<bool> {
        // Pollable is for a stream, so it is always ready.
        if pollable.rep() == u32::MAX {
            return Ok(true);
        }

        let resources = self.plugins.expect::<WasiResources>();
        let pollable = resources.pollables[pollable.rep() as usize];

        let options = TransactionOptions::new("wasi:io/pollable.pollable.ready");
        self.state
            .maybe_do_transaction_sync(options, move |state| {
                let txn = state.transaction_mut().unwrap();

                match pollable.txn {
                    Some(index) if index != txn.index() => anyhow::bail!(
                        "attempted to use a pollable outside the transaction it was created in"
                    ),
                    _ => (),
                }

                let now = Utc::now();
                Ok(pollable.timeout <= now)
            })
            .await
    }

    async fn block(&mut self, pollable: Resource<Pollable>) -> wasmtime::Result<()> {
        // Pollable is for a stream, so it is always ready.
        if pollable.rep() == u32::MAX {
            return Ok(());
        }

        let resources = self.plugins.expect::<WasiResources>();
        let pollable = resources.pollables[pollable.rep() as usize];
        let suspend_timeout = self.state.config().suspend_timeout;
        let suspend_margin = self.state.config().suspend_margin;

        let entered = match self.state.transaction() {
            Some(_) => false,
            None => {
                let options = TransactionOptions::new("wasi:io/poll.pollable.block");
                if let Some(result) = self.state.enter(options).await? {
                    return Ok(result);
                }

                true
            }
        };

        let txn = self.state.transaction_mut().unwrap();
        let is_external = match pollable.txn {
            Some(index) if index != txn.index() => anyhow::bail!(
                "attempted to use a pollable outside the transaction it was created in"
            ),
            Some(_) => false,
            None => true,
        };

        let now = Utc::now();
        let delta = pollable
            .timeout
            .signed_duration_since(now)
            .to_std()
            .unwrap_or(Duration::ZERO);

        if is_external && delta > suspend_timeout + suspend_margin {
            // Avoid holding on to a db connection if we are suspending this task anyway
            txn.take_conn();

            let mut conn = self.state.pool().acquire().await?;
            let status = self
                .state
                .suspend(&mut conn, Some(pollable.timeout))
                .await?;

            return Err(status.into());
        }

        tokio::time::sleep(delta).await;

        if entered {
            self.state.exit(&()).await?;
        }

        Ok(())
    }

    async fn drop(&mut self, pollable: Resource<Pollable>) -> wasmtime::Result<()> {
        // All stream pollables are shared so no drops necessary.
        if pollable.rep() == u32::MAX {
            return Ok(());
        }

        let resources = self.plugins.expect_mut::<WasiResources>();
        resources.pollables.remove(pollable.rep() as _);

        Ok(())
    }
}

impl wasi::io::poll::Host for Task {
    async fn poll(&mut self, pollables: Vec<Resource<Pollable>>) -> wasmtime::Result<Vec<u32>> {
        if pollables.len() > u32::MAX as usize {
            anyhow::bail!("poll called with more than 2^32 pollables");
        }

        // No need to do a transaction if it is only stream pollables
        if pollables.iter().all(|p| p.rep() == u32::MAX) {
            return Ok(pollables
                .iter()
                .enumerate()
                .map(|(idx, _)| idx as u32)
                .collect());
        }

        let entered = match self.state.transaction() {
            Some(_) => false,
            None => {
                let options = TransactionOptions::new("wasi:io/poll.poll");
                if let Some(result) = self.state.enter(options).await? {
                    return Ok(result);
                }

                true
            }
        };

        let suspend_timeout = self.state.config().suspend_timeout;
        let resources = self.plugins.expect::<WasiResources>();
        let txn = self.state.transaction_mut().unwrap();

        // Check whether any of the pollables was created within the current
        // transaction. If this is the case then we can't suspend the task because it
        // might be sleeping based on an impure current time acquired from within the
        // transaction.
        let mut has_internal = false;
        for pollable in pollables.iter() {
            let pollable = resources.pollables[pollable.rep() as _];

            match pollable.txn {
                Some(index) if index != txn.index() => anyhow::bail!(
                    "attempted to use a pollable outside the transaction it was created in"
                ),
                Some(_) => {
                    has_internal = true;
                    break;
                }
                None => (),
            }
        }

        let mut ready = Vec::new();
        loop {
            let now = Utc::now();
            let mut wakeup: Option<DateTime<Utc>> = None;

            for (idx, pollable) in pollables.iter().enumerate() {
                if pollable.rep() == u32::MAX {
                    ready.push(idx as u32);
                    continue;
                }

                let pollable = resources.pollables[pollable.rep() as _];
                if pollable.timeout <= now {
                    ready.push(idx as u32);
                    continue;
                }

                wakeup = Some(
                    wakeup
                        .map(|wakeup| wakeup.min(pollable.timeout))
                        .unwrap_or(pollable.timeout),
                );
            }

            if !ready.is_empty() {
                break;
            }

            let wakeup = match wakeup {
                Some(wakeup) => wakeup,
                None => anyhow::bail!("no pollables were ready but no wakeup time was computed"),
            };
            let duration = wakeup
                .signed_duration_since(now)
                .to_std()
                .unwrap_or(Duration::ZERO);

            if !has_internal && duration > suspend_timeout + SUSPEND_PREWAKE {
                // Avoid holding on to a db connection if we are suspending this task anyway
                let _ = txn.take_conn();

                let mut conn = self.state.pool().acquire().await?;
                let status = self.state.suspend(&mut conn, Some(wakeup)).await?;

                return Err(status.into());
            }

            tracing::trace!("blocking poll for {}", humantime::format_duration(duration));
            tokio::time::sleep(duration).await;
        }

        if entered {
            self.state.exit(&ready).await?;
        }

        Ok(ready)
    }
}
