use std::time::Duration;

use async_trait::async_trait;
use sqlx::postgres::PgNotification;

use super::{Event, EventSource};

/// The names of the PostgreSQL channels that [`PgEventSource`] listens on.
pub const CHANNELS: [&str; 4] = [
    "durable:task",
    "durable:task-suspend",
    "durable:notification",
    "durable:worker",
];

/// An [`EventSource`] that receives events via PostgreSQL LISTEN/NOTIFY.
///
/// This is the event source used by the worker by default. It listens on all
/// of the [`CHANNELS`] and parses their payloads into [`Event`]s. If the
/// underlying connection is lost then it will reconnect and emit
/// [`Event::Lagged`].
pub struct PgEventSource {
    listener: sqlx::postgres::PgListener,
}

impl PgEventSource {
    /// Create a new event source that listens using a connection from `pool`.
    pub async fn new(pool: &sqlx::PgPool) -> sqlx::Result<Self> {
        let mut listener = sqlx::postgres::PgListener::connect_with(pool).await?;

        listener.listen_all(CHANNELS).await?;

        Ok(Self { listener })
    }
}

#[async_trait]
impl EventSource for PgEventSource {
    async fn next(&mut self) -> anyhow::Result<Event> {
        fn parse_event<T, F>(name: &str, event: &PgNotification, func: F) -> Event
        where
            F: FnOnce(T) -> Event,
            T: serde::de::DeserializeOwned,
        {
            match serde_json::from_str(event.payload()) {
                Ok(payload) => func(payload),
                Err(e) => {
                    tracing::warn!("listener received an invalid `{name}` notification: {e}");
                    Event::Lagged
                }
            }
        }

        loop {
            break match self.listener.try_recv().await {
                Ok(Some(event)) => {
                    tracing::trace!("received event {}: {}", event.channel(), event.payload());

                    match event.channel() {
                        "durable:task" => Ok(parse_event("durable:task", &event, Event::Task)),
                        "durable:task-suspend" => Ok(parse_event(
                            "durable:task-suspend",
                            &event,
                            Event::TaskSuspend,
                        )),
                        "durable:notification" => Ok(parse_event(
                            "durable:notification",
                            &event,
                            Event::Notification,
                        )),
                        "durable:worker" => {
                            Ok(parse_event("durable:worker", &event, Event::Worker))
                        }
                        _ => continue,
                    }
                }
                Ok(None) => {
                    while let Err(e) = sqlx::query("SELECT 1").execute(&mut self.listener).await {
                        tracing::warn!("listener received an error: {e}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    Ok(Event::Lagged)
                }
                Err(mut e) => {
                    loop {
                        tracing::warn!("listener received an error: {e}");

                        match sqlx::query("SELECT 1").execute(&mut self.listener).await {
                            Ok(_) => break,
                            Err(err) => e = err,
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    Ok(Event::Lagged)
                }
            };
        }
    }
}
