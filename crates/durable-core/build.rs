fn main() {
    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    if target_family == "wasm" {
        cc::Build::new().file("src/ctor.c").compile("durable-ctor");
    }
}
