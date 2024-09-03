use clap::Parser as _;
use cli::Command as _;

mod cli;
mod config;
mod revolt;
mod services;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::log::set_up()?;

    match cli::Cli::parse().subcommand {
        #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
        cli::Subcommands::Start(start) => start.run().await,
        cli::Subcommands::Config(config) => config.run().await,
    }
}
