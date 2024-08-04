use clap::{Args, Parser, Subcommand};

mod start;

pub trait Command {
    type Arguments: Args;

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
}
