use anyhow::Context;
use durable::notify::{self, notify};

fn main() -> anyhow::Result<()> {
    let task = durable::task();

    notify(task.id(), "test-event-1", "here's some event data")
        .context("failed to send test-event-1")?;
    notify(task.id(), "test-event-2", &12345i32).context("failed to send test-event-2")?;

    let notif1 = notify::wait();
    assert_eq!(notif1.event, "test-event-1");

    let data: String = notif1
        .json()
        .context("test-event-1 did not contain a valid json string")?;
    assert_eq!(data, "here's some event data");

    let notif2 = notify::wait();
    assert_eq!(notif2.event, "test-event-2");

    let data: i32 = notif2
        .json()
        .context("test-event-2 did not contain a valid json integer")?;
    assert_eq!(data, 12345);

    Ok(())
}
