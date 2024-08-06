use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// Which service to enable for checking your listening status.
    #[cfg(any(feature = "services_lastfm", feature = "services_listenbrainz"))]
    pub enable: Option<Services>,
    /// Configuration for the services.
    #[cfg(any(feature = "services_lastfm", feature = "services_listenbrainz"))]
    pub services: ServiceOptions,
    /// Configuration for Revolt.
    pub revolt: RevoltOptions,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Services {
    /// Last.fm service.
    #[cfg(feature = "services_lastfm")]
    LastFM,
    /// Listenbrainz service.
    #[cfg(feature = "services_listenbrainz")]
    Listenbrainz,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceOptions {
    /// Common options for all services.
    #[serde(default = "default_common_service_options")]
    pub _common: Option<CommonServiceOptions>,
    /// Options for the Last.fm service.
    #[cfg(feature = "services_lastfm")]
    pub lastfm: Option<LastFMServiceOptions>,
    /// Options for the Listenbrainz service.
    #[cfg(feature = "services_listenbrainz")]
    pub listenbrainz: Option<ListenbrainzServiceOptions>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RevoltOptions {
    /// The API URL of the instance.
    #[serde(default = "default_revolt_api_url")]
    pub api_url: String,
    /// The session token of the account.
    pub session_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommonServiceOptions {
    /// Interval in seconds to check for new listening activity.
    #[serde(default = "default_check_interval")]
    pub check_interval: u8,
}

#[cfg(feature = "services_lastfm")]
#[derive(Deserialize, Serialize, Debug)]
pub struct LastFMServiceOptions {
    /// Last.fm username to check for listening activity.
    pub username: String,
    /// Last.fm API key to use for checking listening activity.
    pub api_key: String,
}

#[cfg(feature = "services_listenbrainz")]
#[derive(Deserialize, Serialize, Debug)]
pub struct ListenbrainzServiceOptions {
    /// ListenBrainz username to check for listening activity.
    pub username: String,

    /// ListenBrainz API URL to use for checking listening activity.
    #[serde(default = "default_listenbrainz_api_url")]
    pub api_url: String,
}

const fn default_common_service_options() -> Option<CommonServiceOptions> {
    Some(CommonServiceOptions {
        check_interval: default_check_interval(),
    })
}

fn default_revolt_api_url() -> String {
    String::from("https://api.revolt.chat")
}

const fn default_check_interval() -> u8 {
    16
}

#[cfg(feature = "services_listenbrainz")]
fn default_listenbrainz_api_url() -> String {
    String::from("https://api.listenbrainz.org")
}
