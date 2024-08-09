use clap::{Parser, Subcommand};

pub mod config;
pub mod start;

pub trait Command {
    async fn run(&self) -> anyhow::Result<()>;
}

#[derive(Parser, Debug)]
#[command(about, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
    /// Start lure.
    Start(start::CommandArguments),
    /// Lure configuration options
    #[command(subcommand)]
    Config(config::CommandSubcommands),
}
