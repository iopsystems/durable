fn main() {
    #[cfg(feature = "bindgen")]
    generate();
}

#[cfg(feature = "bindgen")]
fn generate() {
    use std::path::Path;

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    durable_bindgen::generate(
        "wit",
        out_dir.join("bindings.rs"),
        "durable:core/import-sql",
    )
    .expect("failed to generate wit bindings");
}
