fn main() {
    let task = durable::task();

    println!("Hello, {}!", task.name());
}
