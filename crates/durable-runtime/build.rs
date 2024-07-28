fn main() {
    println!("cargo::rerun-if-changed=../durable-core/wit/durable.wit");
}
