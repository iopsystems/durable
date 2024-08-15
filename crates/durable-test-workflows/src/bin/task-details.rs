fn main() {
    let task = durable::task();

    print!(
        "\
Task Details:
    id:   {}
    name: {}
    data: {}
",
        task.id(),
        task.name(),
        task.raw_data().get()
    );
}
