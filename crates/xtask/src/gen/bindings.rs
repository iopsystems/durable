use std::path::PathBuf;

use durable_bindgen::Options;

#[derive(Debug, clap::Args)]
pub struct Bindings {}

struct Generator {
    workspace_root: PathBuf,
}

impl Bindings {
    pub fn run(self) -> anyhow::Result<()> {
        let workspace_root = crate::workspace_root()?;
        let generator = Generator { workspace_root };

        generator.generate_for_crate(
            "durable-core",
            "durable:core/import-core",
            Options::new().with("wasi:clocks/wall-clock@0.2.0"),
        )?;
        generator.generate_for_crate("durable-http", "durable:core/import-http", Options::new())?;
        generator.generate_for_crate(
            "durable-sqlx",
            "durable:core/import-sql",
            Options::new()
                .with_additional_derive_attribute("serde::Serialize")
                .with_additional_derive_attribute("serde::Deserialize"),
        )?;

        Ok(())
    }
}

impl Generator {
    fn generate_for_crate(&self, name: &str, world: &str, options: Options) -> anyhow::Result<()> {
        let crate_dir = self.workspace_root.join("crates").join(name);
        let wit_dir = crate_dir.join("wit");
        let output = crate_dir.join("src/bindings.rs");

        durable_bindgen::generate(&wit_dir, &output, world, options)?;

        Ok(())
    }
}
