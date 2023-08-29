use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: SubCommands,

    #[arg(long, env = "LR_TOKEN", required = true)]
    pub token: String,

    #[arg(long, env = "LR_STATUS_TEMPLATE", default_value_t = String::from("🎵 %ARTIST% – %NAME%"))]
    pub status_template: String,

    #[arg(long, env = "LR_STATUS_IDLE")]
    pub status_idle: Option<String>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[cfg(feature = "lastfm")]
    LastFM {
        #[arg(long, env = "LR_LASTFM_USER", required = true)]
        user: String,
        #[arg(long, env = "LR_LASTFM_API_KEY", required = true)]
        api_key: String,
        #[arg(long, env = "LR_LASTFM_CHECK_DELAY", default_value_t = 6)]
        check_delay: u64,
    },
}
