use std::time::{Duration, Instant};

fn main() {
    let task = durable::task_name();

    println!("{task}: sleeping for 120s!");

    let start = Instant::now();
    std::thread::sleep(Duration::from_secs(120));

    let elapsed = start.elapsed();
    println!("{task}: slept for {}s", elapsed.as_secs_f64());
}
