use std::time::Duration;

use durable::notify;

fn main() {
    // Wait with a long timeout and no notification will ever be delivered. The
    // call is expected to time out and return `None`. The test harness drives a
    // custom clock past the deadline; the only way this workflow completes is if
    // the timer-based wakeup fallback fires (i.e. the task was suspended with a
    // `wakeup_at` and the runtime resolves that timer against the injected
    // clock).
    let result = notify::wait_with_timeout(Duration::from_secs(120));
    assert!(
        result.is_none(),
        "expected the wait to time out with no notification"
    );
}
