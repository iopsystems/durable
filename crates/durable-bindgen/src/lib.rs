use std::path::Path;

use anyhow::Context;
pub use anyhow::Result;
use wit_bindgen_core::source::Files;
use wit_bindgen_core::wit_parser::Resolve;
use wit_bindgen_core::WorldGenerator;
use wit_bindgen_rust::{Opts, Ownership, WithOption};

#[derive(Clone)]
pub struct Options(pub Opts);

impl Options {
    pub fn new() -> Self {
        Self(Opts {
            format: true,
            runtime_path: Some("wit_bindgen_rt".into()),
            ownership: Ownership::Borrowing {
                duplicate_if_necessary: true,
            },
            ..Default::default()
        })
    }

    pub fn with(mut self, module: impl Into<String>) -> Self {
        self.0.with.push((module.into(), WithOption::Generate));
        self
    }

    pub fn with_additional_derive_attribute(mut self, attr: impl Into<String>) -> Self {
        self.0.additional_derive_attributes.push(attr.into());
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate(
    source: impl AsRef<Path>,
    out: impl AsRef<Path>,
    world: impl AsRef<str>,
    options: Options,
) -> anyhow::Result<()> {
    _generate(source.as_ref(), out.as_ref(), world.as_ref(), options)
}

fn _generate(source: &Path, out: &Path, world: &str, options: Options) -> anyhow::Result<()> {
    let mut resolve = Resolve::new();
    let (package, paths) = resolve.push_dir(source)?;
    let world = resolve.select_world(&[package], Some(world))?;
    let mut generator = options.0.build();

    if std::env::var_os("OUT_DIR").is_some() {
        for path in paths.paths() {
            println!("cargo::rerun-if-changed={}", path.display());
        }
    }

    let mut files = Files::default();
    generator.generate(&resolve, world, &mut files)?;

    let (_, src) = files.iter().next().unwrap();
    let src = std::str::from_utf8(src).unwrap();

    let file = syn::parse_str(src).context("wit-bindgen emitted unparseable rust code")?;
    let src = prettyplease::unparse(&file);

    std::fs::write(out, src)
        .with_context(|| format!("failed to write bindings to `{}`", out.display()))?;

    Ok(())
}
