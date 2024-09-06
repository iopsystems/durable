use sqlx::postgres::PgListener;
use tabled::settings::formatting::AlignmentStrategy;
use tabled::settings::object::Segment;
use tabled::settings::{Alignment, Margin, Modify, Padding, Style};
use tabled::{Table, Tabled};

use crate::CommonOptions;

/// Print events that have been emitted by a task.
#[derive(Debug, clap::Parser)]
pub struct Events {
    /// The id of the task we want to see the logs for.
    pub task: i64,
}

#[derive(Tabled)]
struct Event {
    index: i64,
    label: String,
    value: String,
}

impl Events {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;

        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("durable:task-complete").await?;

        let exists = sqlx::query_scalar!("SELECT id FROM durable.task WHERE id = $1", self.task)
            .fetch_optional(&pool)
            .await?
            .is_some();

        if !exists {
            anyhow::bail!("unable to find task with id {}", self.task);
        }

        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT
                index,
                label,
                value::text as "value!"
             FROM durable.event
            WHERE task_id = $1
            ORDER BY index ASC
            "#,
            self.task,
        )
        .fetch_all(&mut listener)
        .await?;

        let mut table = Table::new(events);
        table
            .with(
                Modify::new(Segment::all())
                    .with(Alignment::left())
                    .with(AlignmentStrategy::PerLine),
            )
            .with(Style::blank())
            .with(Margin::new(0, 0, 0, 0))
            .with(Padding::new(0, 0, 0, 0));

        println!("{table}");

        Ok(())
    }
}
