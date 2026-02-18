use std::time::Duration;

use durable::notify;

fn main() {
    // Wait with a very long timeout. The test harness will deliver a
    // notification while we are blocked here. If the wake-up path works, this
    // should return well before the timeout.
    let result = notify::wait_with_timeout(Duration::from_secs(120));
    let notif = result.expect("expected to receive a notification before timeout");
    assert_eq!(notif.event, "wakeup");
}
