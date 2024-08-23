use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::MetadataCommand;

/// Publish already packaged crates from the target directory.
///
/// This will avoid publishing crates that are already contained within the
/// registry.
#[derive(Debug, clap::Args)]
pub struct Publish {
    /// A path to an existing gcsfuse mount for the registry.
    ///
    /// If not provided then the registry will be mounted automatically.
    #[arg(long)]
    repository: Option<PathBuf>,
}

impl Publish {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("failed to run `cargo metatada`")?;

        let sh = xshell::Shell::new()?;
        sh.change_dir(&metadata.workspace_root);

        let repodir;
        let (repository, _guard): (&Path, _) = match &self.repository {
            Some(repo) => (repo, None),
            None => {
                repodir = sh.create_temp_dir()?;
                let repo = repodir.path();
                xshell::cmd!(sh, "gcsfuse --log-format text systemslab-cargo {repo}").run()?;

                let guard = scopeguard::guard((), {
                    let sh = &sh;
                    move |_| {
                        if let Err(e) = xshell::cmd!(sh, "fusermount -u {repo}").run() {
                            eprintln!("failed to unmount {}: {e}", repo.display());
                        }
                    }
                });

                (repo, Some(guard))
            }
        };

        let package_dir = metadata.target_directory.join("package/*.crate");
        for package in glob::glob(package_dir.as_str())? {
            let path = package?;

            let filename = match path.file_name() {
                Some(filename) => filename,
                None => anyhow::bail!("crate path `{}` has no filename", path.display()),
            };
            let filename = match filename.to_str() {
                Some(filename) => filename,
                None => anyhow::bail!(
                    "filname of path `{}` was not a valid utf8 string",
                    path.display()
                ),
            };

            // Skip incomplete temporary files.
            if filename.starts_with(".") {
                continue;
            }

            let basename = filename
                .strip_suffix(".crate")
                .expect("filename did not end with `.crate`?");
            let (name, version) = match basename.rsplit_once("-") {
                Some(tuple) => tuple,
                None => {
                    anyhow::bail!("`{filename}` did not the format expected from package names")
                }
            };

            if name.len() < 4 {
                // These use a different index format than the one we look for.
                anyhow::bail!(
                    "cargo xtask publish doesn't properly support crate names with <4 characters \
                     at this time"
                );
            }

            let ab = &name[0..2];
            let cd = &name[2..4];

            let index = format!("crates/{ab}/{cd}/{name}/{version}.crate");
            let index = repository.join(&index);

            if index.exists() {
                println!("Skipping {name} {version}");
                continue;
            }

            println!("Publishing {name} {version}");
            xshell::cmd!(sh, "margo add {path} --registry {repository}").run()?;
        }

        println!("Generating HTML index");
        xshell::cmd!(sh, "margo generate-html --registry {repository}").run()?;

        Ok(())
    }
}
