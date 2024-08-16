use std::ffi::OsString;

use anyhow::Context;
use cargo_metadata::MetadataCommand;

#[derive(Debug, clap::Args)]
pub struct Package {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    args: Vec<OsString>,
}

impl Package {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let mut packages = Vec::new();
        for package in metadata.workspace_packages() {
            match &package.publish {
                Some(registries) if registries.is_empty() => continue,
                _ => (),
            }

            packages.push(&*package.name);
        }

        let sh = xshell::Shell::new()?;
        sh.change_dir(&metadata.workspace_root);

        let mut cmd = xshell::cmd!(sh, "cargo package");
        for package in packages {
            cmd = cmd.arg("-p").arg(package);
        }

        cmd = cmd.args(self.args);
        cmd.run()?;

        Ok(())
    }
}
