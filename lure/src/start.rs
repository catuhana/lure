#![cfg(any(feature = "lastfm-service", feature = "listenbrainz-service"))]

use std::path::{Path, PathBuf};
use std::time::Duration;

use figment::{
    Figment,
    providers::{Env, Format as _, Yaml},
};
use figment_file_provider_adapter::FileAdapter;
use futures::FutureExt as _;
use lure_config::ServiceOptions;
use lure_types::{PlaybackStatus, TrackInfo};
use tokio::time::sleep;

#[cfg(feature = "lastfm-service")]
use crate::service::Service;

pub async fn run(config_path: Option<PathBuf>) -> Result<(), RunError> {
    const SECURE_CONFIG_KEYS: &[&str; 2] = &["session_token", "api_key"];

    let config_path = config_path
        .as_ref()
        .map_or_else(|| Path::new("config.yaml"), PathBuf::as_path);

    let config: lure_config::Config = Figment::new()
        .merge(Yaml::file(config_path))
        .merge(Env::prefixed("LURE_").split("__"))
        .merge(FileAdapter::wrap(Yaml::file(config_path)).only(SECURE_CONFIG_KEYS))
        .merge(FileAdapter::wrap(Env::prefixed("LURE_").split("__")).only(SECURE_CONFIG_KEYS))
        .extract()?;

    let service = match config.service {
        ServiceOptions {
            lastfm: Some(config),
            listenbrainz: None,
        } => Service::LastFm(
            lure_lastfm_service::Service::try_new(config).map_err(ServiceError::from)?,
        ),
        ServiceOptions {
            lastfm: None,
            listenbrainz: Some(config),
        } => Service::ListenBrainz(
            lure_listenbrainz_service::Service::try_new(config).map_err(ServiceError::from)?,
        ),
        ServiceOptions {
            lastfm: Some(_),
            listenbrainz: Some(_),
        } => {
            return Err(RunError::MoreThanOneServiceEnabled(
                "lastfm and listenbrainz".to_string(),
            ));
        }
        ServiceOptions {
            lastfm: None,
            listenbrainz: None,
        } => return Err(RunError::NoServicesEnabled),
    };

    let stoat_client = lure_stoat_api::Client::try_new(
        config.stoat.api_url,
        &lure_stoat_models::Authentication::SessionToken(config.stoat.session_token),
    )?;

    let first_status = stoat_client.get_status_text().await?;
    let mut previous_track: Option<TrackInfo> = None;

    let mut ctrl_c = Box::pin(tokio::signal::ctrl_c().fuse());

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                println!("Received Ctrl+C, exiting...");
                break;
            }
            result = service.poll() => match result {
                Ok(PlaybackStatus::Playing(track)) => {
                    if previous_track.as_ref().is_some_and(|prev| prev == &track) {
                        continue;
                    }

                    let status_text = config
                        .stoat
                        .status
                        .template
                        .replace("%ARTIST%", &track.artist)
                        .replace("%NAME%", &track.title);

                    match stoat_client.set_status_text(Some(status_text)).await {
                        Ok(()) => previous_track = Some(track),
                        Err(lure_stoat_api::Error::ApiError(
                            lure_stoat_api::APIError::RateLimitExceeded(remaining)
                        )) => sleep(Duration::from_millis(remaining)).await,
                        Err(error) => return Err(error.into()),
                    }
                }
                Ok(PlaybackStatus::NotPlaying) => {
                    if previous_track.is_none() {
                        continue;
                    }

                    match stoat_client.set_status_text(first_status.clone()).await {
                        Ok(()) => previous_track = None,
                        Err(lure_stoat_api::Error::ApiError(
                            lure_stoat_api::APIError::RateLimitExceeded(remaining)
                        )) => sleep(Duration::from_millis(remaining)).await,
                        Err(error) => return Err(error.into()),
                    }
                }
                Err(error) => {
                    if service.is_fatal_error(&error) {
                        eprintln!("Fatal error: {error}");
                        break;
                    }

                    eprintln!("Non-fatal error, retrying: {error}");
                }
            }
        }
    }

    stoat_client.set_status_text(first_status).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[cfg(feature = "lastfm-service")]
    #[error(transparent)]
    LastFm(#[from] lure_lastfm_service::ServiceError),
    #[cfg(feature = "listenbrainz-service")]
    #[error(transparent)]
    ListenBrainz(#[from] lure_listenbrainz_service::ServiceError),
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("More than one service ({0}) is enabled. Only one service can be enabled at a time.")]
    MoreThanOneServiceEnabled(String),
    #[error("No services are enabled. One service must be enabled.")]
    NoServicesEnabled,
    #[error(transparent)]
    StoatApi(#[from] lure_stoat_api::Error),
    #[error(transparent)]
    Figment(#[from] figment::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Service(#[from] ServiceError),
}
