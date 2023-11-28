use confique::{toml, Config};
use tokio::fs;

#[derive(Config)]
pub struct Options {
    /// Which platform to check current listening from
    #[config(env = "LURE_PLATFORM")]
    pub platform: String,

    /// Revolt session token to set the status
    #[config(env = "LURE_REVOLT_SESSION_TOKEN")]
    pub session_token: String,

    /// Revolt status options to set
    #[config(nested)]
    pub status: StatusOptions,

    /// Last.fm platform specific options
    /// Can be skipped if this platform is not being used.
    #[cfg(feature = "lastfm")]
    #[config(nested)]
    pub lastfm: LastFMOptions,

    /// ListenBrainz platform specific options
    /// Can be skipped if this platform is not being used
    #[cfg(feature = "listenbrainz")]
    #[config(nested)]
    pub listenbrainz: ListenBrainzOptions,
}

impl Options {
    pub fn generate_config() -> String {
        toml::template::<Self>(&confique::toml::FormatOptions::default())
    }

    pub async fn create_config() -> anyhow::Result<()> {
        let config = Self::generate_config();
        let path = dirs::config_local_dir()
            .expect("unsupported operating system or platform")
            .join("lure")
            .join("config")
            .with_extension("toml");

        if path.try_exists()? {
            anyhow::bail!("configuration file already exists.")
        }

        fs::create_dir_all(&path.parent().unwrap()).await?;
        fs::write(&path, config).await?;

        println!("created a configuration file at `{}`", path.display());

        Ok(())
    }
}

#[derive(Config)]
pub struct StatusOptions {
    /// Status template to use when setting the status
    #[config(
        default = "🎵 Listening to %NAME% by %ARTIST%",
        env = "LURE_STATUS_TEMPLATE"
    )]
    pub template: String,

    /// Idle status message to use when not listening anything
    #[config(env = "LURE_STATUS_IDLE")]
    pub idle: Option<String>,
}

#[cfg(feature = "lastfm")]
#[derive(Config)]
pub struct LastFMOptions {
    /// Last.fm username to check current listening status from
    #[config(env = "LURE_LASTFM_USER")]
    pub user: Option<String>,

    /// Last.fm API key to be able to check current listening through API
    #[config(env = "LURE_LASTFM_API_KEY")]
    pub api_key: Option<String>,

    /// Check interval
    #[config(default = 12, env = "LURE_LASTFM_CHECK_INTERVAL")]
    pub check_interval: u64,
}

#[cfg(feature = "listenbrainz")]
#[derive(Config)]
pub struct ListenBrainzOptions {
    /// ListenBrainz username to check current listening status from
    #[config(env = "LURE_LISTENBRAINZ_USER")]
    pub user: Option<String>,

    /// ListenBrainz API URL to send the API requests to
    #[config(
        default = "https://api.listenbrainz.org",
        env = "LURE_LISTENBRAINZ_API_URL"
    )]
    pub api_url: String,

    /// Check interval
    #[config(default = 12, env = "LURE_LISTENBRAINZ_CHECK_INTERVAL")]
    pub check_interval: u64,
}
