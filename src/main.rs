use clap::Parser as _;
use cli::Command as _;

mod cli;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match cli::Cli::parse().subcommand {
        cli::Subcommands::Start(start) => start.run().await,
        cli::Subcommands::Config(config) => config.run().await,
    }
}
