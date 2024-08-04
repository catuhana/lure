use clap::Parser as _;
use cli::Command as _;

mod cli;

fn main() -> anyhow::Result<()> {
    match cli::Cli::parse().subcommand {
        cli::Subcommands::Start(start) => start.run(),
    }
}
