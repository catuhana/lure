use clap::Parser as _;
use cli::Command as _;

mod cli;
mod config;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match cli::Cli::parse().subcommand {
        #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
        cli::Subcommands::Start(start) => start.run().await,
        cli::Subcommands::Config(config) => config.run().await,
    }
}
