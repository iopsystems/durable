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
        durable_bindgen::Options::new()
            .with_additional_derive_attribute("serde::Serialize")
            .with_additional_derive_attribute("serde::Deserialize"),
    )
    .expect("failed to generate wit bindings");
}
