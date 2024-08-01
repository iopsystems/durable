use std::time::{Duration, Instant};

fn main() {
    let now = Instant::now();
    let name = durable::task_name();

    println!("task {name} started!");

    for _ in 0..20 {
        std::thread::sleep(Duration::from_secs(1));

        println!("{}s have elapsed", now.elapsed().as_secs_f32());
    }
}
