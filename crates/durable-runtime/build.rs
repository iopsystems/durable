use std::path::{Path, PathBuf};

use anyhow::Context;
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
    let world = resolve.select_world(&packages, Some(world))?;
    let bindings = opts.generate(&resolve, world)?;

    for path in paths.iter() {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    let file = syn::parse_str::<syn::File>(&bindings) //
        .context("generated bindings were not valid rust")?;
    let bindings = prettyplease::unparse(&file);

    std::fs::write(out, &bindings)
        .with_context(|| format!("failed to write bindings to `{}`", out.display()))?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let opts = Opts {
        rustfmt: false,
        tracing: true,
        trappable_imports: TrappableImports::All,
        async_: AsyncConfig::AllExceptImports(
            ["task-id", "task-name", "task-data", "abort"]
                .into_iter()
                .map(|item| item.to_string())
                .collect(),
        ),
        ..Default::default()
    };

    generate(
        "wit",
        out_dir.join("bindings.rs"),
        "durable:core/imports",
        &opts,
    )?;

    Ok(())
}
