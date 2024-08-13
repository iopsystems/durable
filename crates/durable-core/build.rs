fn main() {
    #[cfg(feature = "bindgen")]
    generate_bindings();

    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or(String::new());
    if target_family == "wasm" {
        cc::Build::new().file("src/ctor.c").compile("durable-ctor");
    }
}

#[cfg(feature = "bindgen")]
fn generate_bindings() {
    use std::path::Path;

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    durable_bindgen::generate(
        "wit",
        out_dir.join("bindings.rs"),
        "durable:core/import-core",
        durable_bindgen::Options::new().with("wasi:clocks/wall-clock@0.2.0"),
    )
    .expect("failed to compile the bindings");
}
