use clap::Subcommand;

use super::Command;

#[derive(Subcommand, Debug)]
pub enum CommandSubcommands {
    /// Generate an example lure configuration file and print it.
    Generate,
}

impl Command for CommandSubcommands {
    fn run(&self) -> anyhow::Result<()> {
        println!("config");

        Ok(())
    }
}
