use durable::sqlx;

fn main() {
    // First lets add some things to the event log.
    println!("message 1");
    println!("message 2");
    println!("message 3");

    let task_id = durable::task_id();
    let count = durable::sqlx::transaction("reading the event log", |mut conn| {
        let row = sqlx::query("SELECT COUNT(*) FROM durable.event WHERE task_id = $1")
            .bind(task_id)
            .fetch_one(&mut conn)
            .expect("failed to make a query to the database");

        let count: i64 = row.get(0);
        count
    });

    println!("{count} events so far!");
}
