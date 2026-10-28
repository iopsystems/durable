use std::time::Duration;

use durable::notify;

fn main() {
    // Nothing sends this task a notification, so the wait times out. That
    // gives the test harness a few seconds to change the task's owner before
    // the timeout is recorded.
    let result = notify::wait_with_timeout(Duration::from_secs(2));
    assert!(result.is_none(), "expected the wait to time out");
}
