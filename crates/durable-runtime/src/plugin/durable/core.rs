use anyhow::Context;
use serde_json::value::RawValue;

use crate::bindings::durable::core::core::Host;
use crate::plugin::{Task, TransactionOptions};

#[async_trait::async_trait]
impl Host for Task {
    fn task_id(&mut self) -> anyhow::Result<i64> {
        Ok(self.state.task_id())
    }

    fn task_name(&mut self) -> anyhow::Result<String> {
        Ok(self.state.task_name().to_owned())
    }

    fn task_data(&mut self) -> anyhow::Result<String> {
        Ok(self.state.task_data().get().to_owned())
    }

    fn abort(&mut self, message: String) -> anyhow::Result<()> {
        anyhow::bail!("task aborted: {message}")
    }

    async fn transaction_enter(
        &mut self,
        label: String,
        database: bool,
    ) -> anyhow::Result<Option<String>> {
        let options = TransactionOptions::new(label).database(database);
        let data = self.state.enter(options).await?.map(|v| v.get().to_owned());

        Ok(data)
    }

    async fn transaction_exit(&mut self, data: String) -> anyhow::Result<()> {
        let data: &RawValue = serde_json::from_str(&data) //
            .context("provided data was not valid json")?;
        self.state.exit(data).await?;

        Ok(())
    }

    async fn print(&mut self, message: String) -> anyhow::Result<()> {
        self.state.assert_in_transaction("print")?;
        println!("{message}");
        Ok(())
    }
}
