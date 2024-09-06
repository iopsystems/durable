use durable::notify;

fn main() {
    let notif = notify::wait();

    print!(
        "event: {}\ndata:  {}\n",
        notif.event,
        notif.data.get(),
    );
}
