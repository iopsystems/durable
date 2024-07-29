use std::path::Path;

use anyhow::Context;
pub use anyhow::Result;
use wit_bindgen_core::wit_parser::Resolve;
use wit_bindgen_rust::{Opts, Ownership};

pub fn generate(
    source: impl AsRef<Path>,
    out: impl AsRef<Path>,
    world: impl AsRef<str>,
) -> anyhow::Result<()> {
    _generate(source.as_ref(), out.as_ref(), world.as_ref())
}

fn _generate(source: &Path, out: &Path, world: &str) -> anyhow::Result<()> {
    let opts = Opts {
        format: false,
        runtime_path: Some("wit_bindgen_rt".into()),
        ownership: Ownership::Owning,

        ..Default::default()
    };

    let mut resolve = Resolve::new();
    let (packages, paths) = resolve.push_dir(source)?;
    let world = resolve.select_world(&packages, Some(world))?;
    let mut generator = opts.build();

    for path in paths.iter() {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    let mut files = Default::default();
    generator.generate(&resolve, world, &mut files)?;

    let (_, src) = files.iter().next().unwrap();
    let src = std::str::from_utf8(src).unwrap();

    let file = syn::parse_str(&src).context("wit-bindgen emitted unparseable rust code")?;
    let src = prettyplease::unparse(&file);

    std::fs::write(out, src)
        .with_context(|| format!("failed to write bindings to `{}`", out.display()))?;

    Ok(())
}
