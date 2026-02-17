use std::time::Duration;

use durable::notify;

fn main() {
    // First, attempt a wait with a short timeout that should expire (no notification pending).
    let result = notify::wait_with_timeout(Duration::from_millis(100));
    assert!(result.is_none(), "expected timeout with no notification");

    // Now notify ourselves so the next wait_with_timeout will succeed.
    let task = durable::task();
    notify::notify(task.id(), "test-event", &"hello timeout").unwrap();

    // Wait with a generous timeout - should receive the notification.
    let result = notify::wait_with_timeout(Duration::from_secs(30));
    let notif = result.expect("expected to receive a notification");
    assert_eq!(notif.event, "test-event");

    let data: String = notif.json().expect("invalid json data");
    assert_eq!(data, "hello timeout");
}
