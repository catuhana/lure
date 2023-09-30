use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    /// Start lure
    Start(StartSubcommand),

    /// Manage lure configuration file
    #[command(subcommand)]
    Config(ConfigSubcommand),
}

#[derive(Args)]
pub struct StartSubcommand {
    /// Custom lure config file path to load
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Generate lure configuration file
    Generate {
        /// Print the config to stdout instead of saving it
        #[arg(short, long)]
        print: bool,
    },
}
