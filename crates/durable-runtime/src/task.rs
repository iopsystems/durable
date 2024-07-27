use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Method;
use serde_json::value::RawValue;
use sqlx::types::Json;

use crate::bindings::durable::http::*;
use crate::bindings::durable::{self, *};
use crate::error::AbortError;
use crate::worker::{SharedState, TaskData};

#[derive(Copy, Clone, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "task_state", rename_all = "snake_case")]
pub enum TaskState {
    Active,
    Complete,
    Failed,
}

pub(crate) struct WorkflowState {
    shared: Arc<SharedState>,
    task_id: i64,
    index: i32,
    name: String,
    data: Box<RawValue>,
    worker_id: i64,

    txn: Option<CurrentTxn>,
}

impl WorkflowState {
    pub fn new(shared: Arc<SharedState>, task: TaskData, worker_id: i64) -> Self {
        Self {
            shared,
            task_id: task.id,
            index: 0,
            name: task.name,
            data: task.data.0,
            worker_id,

            txn: None,
        }
    }
}

struct CurrentTxn {
    label: String,
    conn: Option<sqlx::Transaction<'static, sqlx::Postgres>>,

    /// Whether an error occurred while attempting to run the transaction.
    txerr: bool,
}

impl WorkflowState {
    fn assert_in_transaction(&self, op: &str) -> anyhow::Result<()> {
        if !self.txn.is_some() {
            anyhow::bail!("attempted to run impure function `{op}` outside of a transaction")
        }

        Ok(())
    }

    async fn enter(
        &mut self,
        label: String,
        is_txn: bool,
    ) -> anyhow::Result<Option<Box<RawValue>>> {
        if let Some(txn) = &self.txn {
            anyhow::bail!(
                "attempted to start transaction {label:?} while already within transaction {:?}",
                txn.label
            );
        }

        let mut tx = self.shared.pool.begin().await?;
        let record = sqlx::query!(
            r#"
            SELECT
                label,
                value as "value: Json<Box<RawValue>>"
             FROM event
            WHERE task_id = $1
              AND index = $2
            "#,
            self.task_id,
            self.index
        )
        .fetch_optional(&mut *tx)
        .await?;

        let record = match record {
            Some(record) => record,
            None => {
                let mut txn = CurrentTxn {
                    label,
                    txerr: false,
                    conn: None,
                };
                if is_txn {
                    txn.conn = Some(tx);
                } else {
                    tx.commit().await?;
                }

                self.txn = Some(txn);

                return Ok(None);
            }
        };

        if record.label != label {
            anyhow::bail!(
                "workflow execution is non-deterministic: stored event at index {} has label {:?} \
                 but the workflow requested {:?}",
                self.index,
                record.label,
                label
            );
        }

        self.index += 1;

        Ok(Some(record.value.0))
    }

    async fn exit(&mut self, data: &RawValue) -> anyhow::Result<()> {
        let mut txn = match self.txn.take() {
            Some(txn) => txn,
            None => anyhow::bail!("attempted to exit a transaction without having entered one"),
        };

        let mut tx = match txn.conn.take() {
            Some(tx) if !txn.txerr => tx,
            _ => self.shared.pool.begin().await?,
        };

        let running_on = sqlx::query!("SELECT running_on FROM task WHERE id = $1", self.task_id)
            .fetch_one(&mut *tx)
            .await?
            .running_on;
        if running_on != Some(self.worker_id) {
            // This task is no longer running on the current worker. Don't commit anything,
            // and abort the task.
            return Err(anyhow::Error::new(AbortError));
        }

        sqlx::query!(
            r#"
            INSERT INTO event(task_id, index, label, value)
            VALUES ($1, $2, $3, $4)
            "#,
            self.task_id,
            self.index,
            txn.label,
            Json(data) as Json<&RawValue>
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.index += 1;

        Ok(())
    }

    async fn http_impl(&mut self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let method = Method::from_bytes(request.method.as_bytes())?;
        let timeout = request
            .timeout
            .map(Duration::from_nanos)
            .unwrap_or(self.shared.config.max_http_timeout)
            .min(self.shared.config.max_http_timeout);

        let url = reqwest::Url::parse(&request.url) //
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        let mut builder = self.shared.client.request(method, url).timeout(timeout);

        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        for header in request.headers {
            let name = HeaderName::from_bytes(&header.name.as_bytes())?;
            let value = HeaderValue::from_bytes(&header.value)?;

            builder = builder.header(name, value);
        }

        let response = builder.send().await?;

        Ok(HttpResponse {
            status: response.status().as_u16(),
            headers: response
                .headers()
                .iter()
                .map(|(name, value)| HttpHeader {
                    name: name.as_str().to_owned(),
                    value: value.as_bytes().to_owned(),
                })
                .collect(),
            body: response.bytes().await?.to_vec(),
        })
    }
}

#[async_trait]
impl CoreImports for WorkflowState {
    fn task_name(&mut self) -> anyhow::Result<String> {
        Ok(self.name.clone())
    }

    fn task_data(&mut self) -> anyhow::Result<String> {
        Ok(self.data.get().to_owned())
    }

    fn abort(&mut self, message: String) -> anyhow::Result<()> {
        anyhow::bail!("task aborted: {message}")
    }

    async fn transaction_enter(
        &mut self,
        label: String,
        is_txn: bool,
    ) -> anyhow::Result<Option<String>> {
        let data = self.enter(label, is_txn).await?;
        let data = data.map(|value| value.get().to_owned());

        Ok(data)
    }

    async fn transaction_exit(&mut self, data: String) -> anyhow::Result<()> {
        let data: &RawValue = serde_json::from_str(&data) //
            .context("provided data was not a valid json string")?;
        self.exit(data).await?;

        Ok(())
    }

    async fn print(&mut self, message: String) -> anyhow::Result<()> {
        self.assert_in_transaction("print")?;

        println!("{message}");
        Ok(())
    }
}

#[async_trait]
impl durable::http::Host for WorkflowState {
    async fn http(
        &mut self,
        request: HttpRequest,
    ) -> anyhow::Result<Result<HttpResponse, HttpError>> {
        self.assert_in_transaction("http")?;

        Ok(self.http_impl(request).await)
    }
}
