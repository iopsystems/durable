use std::path::Path;

fn main() -> durable_bindgen::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    durable_bindgen::generate(
        "wit",
        out_dir.join("bindings.rs"),
        "durable:core/import-core",
    )?;

    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or(String::new());
    if target_family == "wasm" {
        cc::Build::new()
            .file("src/ctor.c")
            .compile("durable-ctor");
    }

    Ok(())
}
