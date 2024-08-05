use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Args, Debug)]
pub struct CommandArguments {
    /// Path of lure config file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Command for CommandArguments {
    fn run(&self) -> anyhow::Result<()> {
        println!("meow");

        Ok(())
    }
}
