use std::path::{Path, PathBuf};

use anyhow::Context;
use durable_migrate::{EmbedOptions, Migrator};

fn generate_migrations(out_dir: &Path) -> anyhow::Result<()> {
    let migrator = Migrator::from_dir("migrations")?;
    let embed = migrator.embed(&EmbedOptions::default());
    let output = out_dir.join("migrations.rs");
    std::fs::write(&output, &embed)?;

    println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");

    Ok(())
}

fn set_sqlx_offline() {
    let development = std::env::var_os("DURABLE_DEVELOPMENT")
        .map(|var| var == "1" || var == "true")
        .unwrap_or(false);

    if !development {
        println!("cargo::rustc-env=SQLX_OFFLINE=true");
        println!("cargo::rustc-env=SQLX_OFFLINE_DIR=.sqlx");
    }
}

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    set_sqlx_offline();
    generate_migrations(&out_dir).context("failed to generate database migrations")?;

    Ok(())
}
