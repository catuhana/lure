pub mod stoat;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub service: ServiceOptions,
    pub stoat: stoat::Options,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ServiceOptions {
    pub lastfm: Option<lure_lastfm_service::config::Options>,
    pub listenbrainz: Option<lure_listenbrainz_service::config::Options>,
}
