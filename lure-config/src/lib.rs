#![cfg(any(feature = "lastfm-service", feature = "listenbrainz-service"))]

pub mod revolt;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub service: ServiceOptions,
    pub revolt: revolt::Options,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServiceOptions {
    #[cfg(feature = "lastfm-service")]
    pub lastfm: Option<lure_lastfm_service_config::Options>,
    #[cfg(feature = "listenbrainz-service")]
    pub listenbrainz: Option<lure_listenbrainz_service_config::Options>,
}

impl Config {
    #[must_use]
    pub fn enabled_services(&self) -> Vec<&str> {
        let mut services = Vec::new();

        #[cfg(feature = "lastfm-service")]
        if let Some(lastfm) = &self.service.lastfm {
            if lastfm.enable {
                services.push("Last.fm");
            }
        }

        #[cfg(feature = "listenbrainz-service")]
        if let Some(listenbrainz) = &self.service.listenbrainz {
            if listenbrainz.enable {
                services.push("ListenBrainz");
            }
        }

        services
    }
}
