use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Generate {}

struct Generator {
    workspace_root: PathBuf,
}

impl Generate {
    pub fn run(self) -> anyhow::Result<()> {
        let workspace_root = crate::workspace_root()?;
        let generator = Generator { workspace_root };

        generator.generate_for_crate("durable-core", "durable:core/import-core")?;
        generator.generate_for_crate("durable-http", "durable:core/import-http")?;
        generator.generate_for_crate("durable-sqlx", "durable:core/import-sql")?;

        Ok(())
    }
}

impl Generator {
    fn generate_for_crate(&self, name: &str, world: &str) -> anyhow::Result<()> {
        let crate_dir = self.workspace_root.join("crates").join(name);
        let wit_dir = crate_dir.join("wit");
        let output = crate_dir.join("src/bindings.rs");

        durable_bindgen::generate(&wit_dir, &output, world)?;

        Ok(())
    }
}
