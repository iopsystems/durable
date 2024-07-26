#[cfg(feature = "regenerate")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sh = xshell::Shell::new()?;
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=wit/durable.wit");

    xshell::cmd!(sh, "wit-bindgen rust wit/durable.wit --out-dir {out_dir}").run()?;

    Ok(())
}

#[cfg(not(feature = "regenerate"))]
fn main() {}
