use anyhow::Context;
use rand::RngCore;

use crate::bindings::wasi;
use crate::task::{Task, TransactionOptions};

#[async_trait::async_trait]
impl wasi::random::random::Host for Task {
    async fn get_random_bytes(&mut self, len: u64) -> wasmtime::Result<Vec<u8>> {
        let config = self.state.config();
        if len as usize > config.max_returned_buffer_len {
            anyhow::bail!("get-random-bytes requested more bytes than permitted by config");
        }

        let options = TransactionOptions::new("wasi:random/random.get-random-bytes");
        self.state
            .maybe_do_transaction_sync(options, move |_| {
                let mut data = Vec::with_capacity(len as usize);
                getrandom::fill_uninit(data.spare_capacity_mut())
                    .context("get-random-bytes: failed to call getrandom")?;

                // SAFETY: getrandom_uninit returned successfully so all bytes in the spare
                //         capacity of the vector are initialized.
                unsafe { data.set_len(data.capacity()) };

                Ok(data)
            })
            .await
    }

    async fn get_random_u64(&mut self) -> wasmtime::Result<u64> {
        let options = TransactionOptions::new("wasi:random/random.get-random-u64");
        self.state
            .maybe_do_transaction_sync(options, move |_| {
                let mut data = [0u8; std::mem::size_of::<u64>()];

                getrandom::fill(&mut data).context("get-random-u64: failed to call getrandom")?;

                Ok(u64::from_ne_bytes(data))
            })
            .await
    }
}

#[async_trait::async_trait]
impl wasi::random::insecure::Host for Task {
    async fn get_insecure_random_bytes(&mut self, len: u64) -> wasmtime::Result<Vec<u8>> {
        let config = self.state.config();
        if len as usize > config.max_returned_buffer_len {
            anyhow::bail!(
                "get-insecure-random-bytes requested more bytes than permitted by config"
            );
        }

        let options = TransactionOptions::new("wasi:random/random.get-insecure-random-bytes");
        self.state
            .maybe_do_transaction_sync(options, move |_| {
                let mut data = vec![0u8; len as usize];
                rand::rng().fill_bytes(&mut data);
                Ok(data)
            })
            .await
    }

    async fn get_insecure_random_u64(&mut self) -> wasmtime::Result<u64> {
        let options = TransactionOptions::new("wasi:random/random.get-insecure-random-u64");
        self.state
            .maybe_do_transaction_sync(options, move |_| Ok(rand::rng().next_u64()))
            .await
    }
}

#[async_trait::async_trait]
impl wasi::random::insecure_seed::Host for Task {
    async fn insecure_seed(&mut self) -> wasmtime::Result<(u64, u64)> {
        // This needs to be something that is consistent between hosts so we
        // implement it by hashing the task name and task id using a hasher that
        // is not random (so not the one in std).

        let task_id = self.state.task_id() as u64;
        let state = ahash::RandomState::with_seed(task_id as usize);
        let hash = state.hash_one(self.state.task_name());

        Ok((hash, task_id))
    }
}
