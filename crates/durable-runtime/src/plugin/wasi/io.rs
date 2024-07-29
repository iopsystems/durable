use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use wasi::io::poll::Pollable;
use wasi::io::streams::{InputStream, OutputStream, StreamError};
use wasmtime::component::Resource;

use super::WasiResources;
use crate::bindings::wasi;
use crate::plugin::PluginMapExt;
use crate::task::TransactionOptions;
use crate::Task;

impl wasi::io::error::HostError for Task {
    fn to_debug_string(
        &mut self,
        res: Resource<wasi::io::error::Error>,
    ) -> wasmtime::Result<String> {
        let resources = self.plugins.expect::<WasiResources>();
        let error = &resources.errors[res.rep() as usize];

        Ok(error.to_string())
    }

    fn drop(&mut self, res: Resource<wasi::io::error::Error>) -> wasmtime::Result<()> {
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
#[async_trait]
impl wasi::io::streams::HostInputStream for Task {
    fn read(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<Vec<u8>, StreamError>> {
        // Workflows have no input streams. We model this by always indicating that
        // the stream is closed.
        Ok(Err(StreamError::Closed))
    }

    fn blocking_read(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<Vec<u8>, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    fn skip(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    fn blocking_skip(
        &mut self,
        _: Resource<InputStream>,
        _: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    fn subscribe(&mut self, _: Resource<InputStream>) -> wasmtime::Result<Resource<Pollable>> {
        Ok(Resource::new_own(u32::MAX))
    }

    fn drop(&mut self, _: Resource<InputStream>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl wasi::io::streams::HostOutputStream for Task {
    fn check_write(
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
            .maybe_do_transaction(options, move |txn| {
                Box::pin(async move {
                    let utf8 = String::from_utf8_lossy(&contents);
                    txn.write_logs(&utf8);
                    Ok(())
                })
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

    fn flush(
        &mut self,
        stream: Resource<OutputStream>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        Ok(match stream.rep() {
            1 => Ok(()),
            _ => Err(StreamError::Closed),
        })
    }

    fn blocking_flush(
        &mut self,
        stream: Resource<OutputStream>,
    ) -> wasmtime::Result<Result<(), StreamError>> {
        self.flush(stream)
    }

    fn subscribe(&mut self, _: Resource<OutputStream>) -> wasmtime::Result<Resource<Pollable>> {
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

    fn splice(
        &mut self,
        _dst: Resource<OutputStream>,
        _src: Resource<InputStream>,
        _len: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    fn blocking_splice(
        &mut self,
        _dst: Resource<OutputStream>,
        _src: Resource<InputStream>,
        _len: u64,
    ) -> wasmtime::Result<Result<u64, StreamError>> {
        Ok(Err(StreamError::Closed))
    }

    fn drop(&mut self, _: Resource<OutputStream>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl wasi::io::streams::Host for Task {}

#[async_trait]
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
            .maybe_do_transaction_sync(options, move |txn| {
                match pollable.txn {
                    Some(index) if index != txn.index() => anyhow::bail!(
                        "attempted to use a pollable outside the transaction it was created in"
                    ),
                    _ => (),
                }

                let now = SystemTime::now();
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

        let options = TransactionOptions::new("wasi:io/poll.pollable.block");
        self.state
            .maybe_do_transaction(options, move |txn| {
                Box::pin(async move {
                    match pollable.txn {
                        Some(index) if index != txn.index() => anyhow::bail!(
                            "attempted to use a pollable outside the transaction it was created in"
                        ),
                        _ => (),
                    }

                    let now = SystemTime::now();

                    if let Ok(delta) = now.duration_since(pollable.timeout) {
                        // TODO: We may want some sort of suspended tasks for tasks that block for a
                        //       long time. For now this just gets translated to a tokio sleep.
                        tokio::time::sleep(delta).await;
                    }

                    Ok(())
                })
            })
            .await
    }

    fn drop(&mut self, pollable: Resource<Pollable>) -> wasmtime::Result<()> {
        // All stream pollables are shared so no drops necessary.
        if pollable.rep() == u32::MAX {
            return Ok(());
        }

        let resources = self.plugins.expect_mut::<WasiResources>();
        resources.pollables.remove(pollable.rep() as _);

        Ok(())
    }
}

#[async_trait]
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

        let resources = self.plugins.expect::<WasiResources>();
        let mut ready = Vec::new();

        while ready.is_empty() {
            let now = SystemTime::now();
            let mut wakeup = now;

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

                wakeup = wakeup.min(pollable.timeout);
            }

            if ready.is_empty() {
                let duration = wakeup.duration_since(now).unwrap_or(Duration::ZERO);
                tokio::time::sleep(duration).await;
            }
        }

        if entered {
            self.state.exit(&ready).await?;
        }

        Ok(ready)
    }
}
