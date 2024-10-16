#![cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]

use serde::Deserialize;

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Which service to enable for checking your listening status.
    pub enable: Option<Services>,
    /// Configuration for the services.
    pub services: ServiceOptions,
    /// Configuration for Revolt.
    pub revolt: RevoltOptions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Services {
    /// `Last.fm` service.
    #[cfg(feature = "services-lastfm")]
    LastFM,
    /// `ListenBrainz` service.
    #[cfg(feature = "services-listenbrainz")]
    ListenBrainz,
}

#[derive(Deserialize, Debug, Default)]
pub struct ServiceOptions {
    /// Options for the `Last.fm` service.
    #[cfg(all(feature = "services-lastfm", not(feature = "services-listenbrainz")))]
    pub lastfm: LastFMServiceOptions,
    /// Options for the `Last.fm` service.
    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
    pub lastfm: Option<LastFMServiceOptions>,

    /// Options for the `ListenBrainz` service.
    #[cfg(all(feature = "services-listenbrainz", not(feature = "services-lastfm")))]
    pub listenbrainz: ListenBrainzServiceOptions,
    /// Options for the `ListenBrainz` service.
    #[cfg(all(feature = "services-listenbrainz", feature = "services-lastfm"))]
    pub listenbrainz: Option<ListenBrainzServiceOptions>,
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Deserialize, Debug)]
pub struct RevoltOptions {
    /// Status options.
    #[serde(default = "RevoltStatusOptions::default")]
    pub status: RevoltStatusOptions,
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
            status: RevoltStatusOptions::default(),
            api_url: default_revolt_api_url(),
            session_token: String::default(),
        }
    }
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Deserialize, Debug)]
pub struct RevoltStatusOptions {
    /// The status text to set.
    #[serde(default = "default_revolt_status_template")]
    pub template: String,
    /// The status emoji to set.
    pub idle: Option<String>,
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
impl Default for RevoltStatusOptions {
    fn default() -> Self {
        Self {
            template: default_revolt_status_template(),
            idle: None,
        }
    }
}

#[cfg(feature = "services-lastfm")]
#[derive(Deserialize, Debug)]
pub struct LastFMServiceOptions {
    /// `Last.fm` username to check for listening activity.
    pub username: String,
    /// `Last.fm` API key to use for checking listening activity.
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
pub struct ListenBrainzServiceOptions {
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
impl Default for ListenBrainzServiceOptions {
    fn default() -> Self {
        Self {
            username: String::default(),
            api_url: default_listenbrainz_api_url(),
            check_interval: default_check_interval(),
        }
    }
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
fn default_revolt_status_template() -> String {
    String::from("🎵 Listening to %NAME% by %ARTIST%")
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
