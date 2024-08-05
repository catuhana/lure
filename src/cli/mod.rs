use clap::{Parser, Subcommand};

mod config;
mod start;

pub trait Command {
    fn run(&self) -> anyhow::Result<()>;
}

#[derive(Parser, Debug)]
#[command(about, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Start lure.
    Start(start::CommandArguments),
    /// Lure configuration options
    #[command(subcommand)]
    Config(config::CommandSubcommands),
}
