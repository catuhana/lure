#![cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Which service to enable for checking your listening status.
    #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
    pub enable: Option<Services>,
    /// Configuration for the services.
    #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
    pub services: ServiceOptions,
    #[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
    /// Configuration for Revolt.
    pub revolt: RevoltOptions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Services {
    /// Last.fm service.
    #[cfg(feature = "services-lastfm")]
    LastFM,
    /// Listenbrainz service.
    #[cfg(feature = "services-listenbrainz")]
    Listenbrainz,
}

#[derive(Deserialize, Debug, Default)]
pub struct ServiceOptions {
    /// Options for the Last.fm service.
    #[cfg(all(feature = "services-lastfm", not(feature = "services-listenbrainz")))]
    pub lastfm: LastFMServiceOptions,
    /// Options for the Last.fm service.
    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
    pub lastfm: Option<LastFMServiceOptions>,

    /// Options for the Listenbrainz service.
    #[cfg(all(feature = "services-listenbrainz", not(feature = "services-lastfm")))]
    pub listenbrainz: ListenbrainzServiceOptions,
    /// Options for the Listenbrainz service.
    #[cfg(all(feature = "services-listenbrainz", feature = "services-lastfm"))]
    pub listenbrainz: Option<ListenbrainzServiceOptions>,
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Deserialize, Debug)]
pub struct RevoltOptions {
    /// The API URL of the instance.
    #[serde(default = "default_revolt_api_url")]
    pub api_url: String,
    /// The session token of the account.
    pub session_token: String,
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
impl Default for RevoltOptions {
    fn default() -> Self {
        Self {
            api_url: default_revolt_api_url(),
            session_token: String::default(),
        }
    }
}

#[cfg(feature = "services-lastfm")]
#[derive(Deserialize, Debug)]
pub struct LastFMServiceOptions {
    /// Last.fm username to check for listening activity.
    pub username: String,
    /// Last.fm API key to use for checking listening activity.
    pub api_key: String,
    /// Interval in seconds to check for listening activity.
    #[serde(default = "default_check_interval")]
    pub check_interval: u8,
}

#[cfg(feature = "services-lastfm")]
impl Default for LastFMServiceOptions {
    fn default() -> Self {
        Self {
            username: String::default(),
            api_key: String::default(),
            check_interval: default_check_interval(),
        }
    }
}

#[cfg(feature = "services-listenbrainz")]
#[derive(Deserialize, Debug)]
pub struct ListenbrainzServiceOptions {
    /// `ListenBrainz` username to check for listening activity.
    pub username: String,
    /// `ListenBrainz` API URL to use for checking listening activity.
    #[serde(default = "default_listenbrainz_api_url")]
    pub api_url: String,
    /// Interval in seconds to check for listening activity.
    #[serde(default = "default_check_interval")]
    pub check_interval: u8,
}

#[cfg(feature = "services-listenbrainz")]
impl Default for ListenbrainzServiceOptions {
    fn default() -> Self {
        Self {
            username: String::default(),
            api_url: default_listenbrainz_api_url(),
            check_interval: default_check_interval(),
        }
    }
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
fn default_revolt_api_url() -> String {
    String::from("https://api.revolt.chat")
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
const fn default_check_interval() -> u8 {
    16
}

#[cfg(feature = "services-listenbrainz")]
fn default_listenbrainz_api_url() -> String {
    String::from("https://api.listenbrainz.org")
}
