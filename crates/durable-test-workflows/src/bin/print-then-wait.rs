use durable::notify;

fn main() {
    // Record a transaction first so the test harness can tell when the task
    // has started running.
    println!("waiting for a notification");

    let notif = notify::wait();

    print!("event: {}\ndata:  {}\n", notif.event, notif.data.get());
}
