mod bindings;

#[derive(Debug, clap::Args)]
pub struct Gen {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Bindings(self::bindings::Bindings),
    // MigrationSql(self::migration_sql::MigrationSql),
}

impl Gen {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Bindings(cmd) => cmd.run(),
        }
    }
}
