use std::path::{Path, PathBuf};

use anyhow::Context;
use durable_migrate::{EmbedOptions, Migrator};
use wasmtime_wit_bindgen::*;
use wit_parser::Resolve;

fn generate(
    path: impl AsRef<Path>,
    out: impl AsRef<Path>,
    world: &str,
    opts: &Opts,
) -> anyhow::Result<()> {
    _generate(path.as_ref(), out.as_ref(), world, opts)
}

fn _generate(path: &Path, out: &Path, world: &str, opts: &Opts) -> anyhow::Result<()> {
    let mut resolve = Resolve::new();
    let (packages, paths) = resolve.push_dir(path)?;
    let world = resolve.select_world(packages, Some(world))?;
    let bindings = opts.generate(&resolve, world)?;

    for path in paths.iter() {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    std::fs::write(out, &bindings)
        .with_context(|| format!("failed to write bindings to `{}`", out.display()))?;

    Ok(())
}

fn generate_bindings(out_dir: &Path) -> anyhow::Result<()> {
    // Method implementations that should be sync.
    //
    // In general sync interfaces are more efficient so it is better to make things
    // sync where possible.
    let sync = [
        // These ones are from durable itself.
        "task-id",
        "task-name",
        "task-data",
        "task-created-at",
        "abort",
        // And these ones are from the various wasi p2 interfaces that we export.
        "[method]error.to-debug-string",
        "[method]input-stream.read",
        "[method]input-stream.blocking-read",
        "[method]input-stream.skip",
        "[method]input-stream.blocking-skip",
        "[method]input-stream.subscribe",
        "[method]output-stream.check-write",
        "[method]output-stream.flush",
        "[method]output-stream.blocking-flush",
        "[method]output-stream.subscribe",
        "[method]output-stream.splice",
        "[method]output-stream.blocking-splice",
        "get-environment",
        "get-arguments",
        "initial-cwd",
        "exit",
        "get-stdin",
        "get-stdout",
        "get-stderr",
        "resolution",
    ];

    let opts = Opts {
        rustfmt: false,
        tracing: true,
        trappable_imports: TrappableImports::All,
        async_: AsyncConfig::AllExceptImports(
            sync.into_iter().map(|item| item.to_string()).collect(),
        ),
        ..Default::default()
    };

    let output = out_dir.join("bindings.rs");
    generate("wit", output, "durable:core/imports", &opts)?;

    Ok(())
}

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
    generate_bindings(&out_dir).context("failed to generate wasmtime wit bindings")?;
    generate_migrations(&out_dir).context("failed to generate database migrations")?;

    Ok(())
}
