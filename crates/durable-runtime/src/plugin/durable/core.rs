use std::time::{Duration, SystemTime};

use anyhow::Context;
use serde_json::value::RawValue;

use crate::bindings::durable::core::core::Host;
use crate::bindings::wasi::clocks::wall_clock::Datetime;
use crate::task::{Task, TransactionOptions};

impl Host for Task {
    async fn task_id(&mut self) -> anyhow::Result<i64> {
        Ok(self.state.task_id())
    }

    async fn task_name(&mut self) -> anyhow::Result<String> {
        Ok(self.state.task_name().to_owned())
    }

    async fn task_data(&mut self) -> anyhow::Result<String> {
        Ok(self.state.task_data().get().to_owned())
    }

    async fn task_created_at(&mut self) -> anyhow::Result<Datetime> {
        let created_at = self.state.task_created_at();
        let systime: SystemTime = created_at.into();

        let duration = systime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);

        Ok(Datetime {
            seconds: duration.as_secs(),
            nanoseconds: duration.subsec_nanos(),
        })
    }

    async fn transaction_enter(
        &mut self,
        label: String,
        database: bool,
    ) -> anyhow::Result<Option<String>> {
        let options = TransactionOptions::new(label).database(database);
        let data: Option<Box<RawValue>> = self.state.enter(options).await?;
        let data = data.map(|v| v.get().to_owned());

        let txn = self.state.transaction().map(|txn| txn.index());
        self.resources.set_txn(txn);

        Ok(data)
    }

    async fn transaction_exit(&mut self, data: String) -> anyhow::Result<()> {
        let data: &RawValue = serde_json::from_str(&data) //
            .context("provided data was not valid json")?;
        self.state.exit(data).await?;

        let txn = self.state.transaction().map(|txn| txn.index());
        self.resources.set_txn(txn);

        Ok(())
    }
}
