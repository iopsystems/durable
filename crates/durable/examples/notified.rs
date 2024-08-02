use durable::Notification;

fn main() {
    println!("Waiting for a `task-exit` notification...");

    loop {
        let notif: Notification = durable::notification();

        println!(
            "Got `{}` notification with data: {}",
            notif.event,
            notif.data.get()
        );

        if notif.event == "task-exit" {
            break;
        }
    }
}
