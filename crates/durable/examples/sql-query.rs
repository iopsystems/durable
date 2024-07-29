use durable::sqlx;

fn main() {
    // First lets add some things to the event log.
    durable::print("message 1\n");
    durable::print("message 2\n");
    durable::print("message 3\n");

    let task_id = durable::task_id();
    let count = durable::sqlx::transaction("reading the event log", |mut conn| {
        let row = sqlx::query("SELECT COUNT(*) FROM event WHERE task_id = $1")
            .bind(task_id)
            .fetch_one(&mut conn)
            .expect("failed to make a query to the database");

        let count: i64 = row.get(0);
        count
    });

    durable::print(&format!("{count} events so far!\n"));
}
