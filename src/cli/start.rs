use std::path::PathBuf;

use clap::Args;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};

use crate::config;

use super::Command;

#[derive(Args, Debug)]
pub struct CommandArguments {
    /// Path of lure config file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Command for CommandArguments {
    fn run(&self) -> anyhow::Result<()> {
        let config_path = self
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from("config.yaml"));

        let config = Figment::new()
            .merge(Yaml::file(config_path))
            .merge(Env::raw().split("__"))
            .extract::<config::Config>()?;

        dbg!(config);

        Ok(())
    }
}
