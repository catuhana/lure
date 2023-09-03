use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: SubCommands,

    /// Revolt session token
    #[arg(long, env = "LURE_TOKEN", required = true)]
    pub token: String,

    /// Status template to show when listening
    #[arg(long, env = "LURE_STATUS_TEMPLATE", default_value_t = String::from("🎵 %ARTIST% – %NAME%"))]
    pub status_template: String,

    /// Status to show when not listening
    #[arg(long, env = "LURE_STATUS_IDLE")]
    pub status_idle: Option<String>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[cfg(feature = "lastfm")]
    /// Show listening status from Last.fm
    LastFM {
        /// Last.fm username
        #[arg(long, env = "LURE_LASTFM_USER", required = true)]
        user: String,
        /// Last.fm API key
        #[arg(long, env = "LURE_LASTFM_API_KEY", required = true)]
        api_key: String,
        /// Check interval
        #[arg(long, env = "LURE_LASTFM_CHECK_DELAY", default_value_t = 12)]
        check_interval: u64,
    },
    #[cfg(feature = "listenbrainz")]
    /// Show listening status from ListenBrainz
    ListenBrainz {
        /// ListenBrainz username
        #[arg(long, env = "LURE_LISTENBRAINZ_USER", required = true)]
        user: String,
        /// ListenBrainz custom API URL
        #[arg(
            long,
            env = "LURE_LISTENBRAINZ_API_URL",
            default_value_t = String::from("https://api.listenbrainz.org")
        )]
        api_url: String,
        /// Check interval
        #[arg(long, env = "LURE_LISTENBRAINZ_CHECK_DELAY", default_value_t = 12)]
        check_interval: u64,
    },
}