use clap::Subcommand;

use super::Command;

#[derive(Subcommand, Debug)]
pub enum CommandSubcommands {
    /// Generate an example lure configuration file and print it.
    Generate,
    // TODO: Implement `revolt get-session-token`
}

impl Command for CommandSubcommands {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Generate => {
                print!("{}", include_str!("../../resources/config.example.yaml"));
            }
        }

        Ok(())
    }
}
