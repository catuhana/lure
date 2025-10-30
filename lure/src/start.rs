use std::path::PathBuf;

use crate::Command;

#[derive(Debug, clap::Args)]
pub struct Arguments {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Command for Arguments {
    type Error = ArgumentsError;

    #[cfg(any(feature = "lastfm-service", feature = "listenbrainz-service"))]
    async fn run(&self) -> Result<(), Self::Error> {
        const SECURE_CONFIG_KEYS: &[&str; 2] = &["session_token", "api_key"];

        use core::time::Duration;
        use std::path::Path;

        use figment::{
            Figment,
            providers::{Env, Format as _, Yaml},
        };
        use figment_file_provider_adapter::FileAdapter;
        use futures::{FutureExt as _, TryStreamExt as _};
        use lure_core::{PlaybackService as _, PlaybackStatus, ServiceCustomError as _, TrackInfo};
        use tokio::time::sleep;

        let config_path = self
            .config
            .as_ref()
            .map_or_else(|| Path::new("config.yaml"), PathBuf::as_path);

        let config: lure_config::Config = Figment::new()
            .merge(Yaml::file(config_path))
            .merge(Env::prefixed("LURE_").split("__"))
            .merge(FileAdapter::wrap(Yaml::file(config_path)).only(SECURE_CONFIG_KEYS))
            .merge(FileAdapter::wrap(Env::prefixed("LURE_").split("__")).only(SECURE_CONFIG_KEYS))
            .extract()?;

        let enabled_services = config.enabled_services();
        let mut service_stream = match enabled_services.len() {
            0 => return Err(ArgumentsError::NoServicesEnabled),
            1 => match enabled_services.first() {
                #[cfg(feature = "lastfm-service")]
                Some(&"Last.fm") => {
                    lure_lastfm_service::Service::try_new(config.service.lastfm.unwrap())?
                        .into_playback_service()
                        .into_stream()
                }
                #[cfg(feature = "listenbrainz-service")]
                Some(&"ListenBrainz") => lure_listenbrainz_service::Service::try_new(
                    config.service.listenbrainz.unwrap(),
                )?
                .into_playback_service()
                .into_stream(),
                Some(_) | None => unreachable!(),
            },
            _ => {
                return Err(ArgumentsError::MoreThanOneServiceEnabled(
                    enabled_services.join(", "),
                ));
            }
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
                },
                item = service_stream.try_next() => {
                    match item {
                        Ok(None) => unreachable!(),
                        Ok(Some(status)) => match status {
                            PlaybackStatus::Playing(track) if previous_track.as_ref().is_some_and(|prev| prev == &track) => {
                                continue;
                            }
                            PlaybackStatus::Playing(track) => {
                                let status = config
                                    .stoat
                                    .status
                                    .template
                                    .replace("%ARTIST%", &track.artist)
                                    .replace("%NAME%", &track.title);

                                match stoat_client.set_status_text(Some(status)).await {
                                    Ok(()) => previous_track = Some(track),
                                    Err(lure_stoat_api::Error::ApiError(
                                        lure_stoat_api::APIError::RateLimitExceeded(remaining)
                                    )) => sleep(Duration::from_millis(remaining)).await,
                                    Err(error) => return Err(error.into()),
                                }
                            }
                            PlaybackStatus::NotPlaying if previous_track.is_none() => continue,
                            PlaybackStatus::NotPlaying => {
                                match stoat_client.set_status_text(first_status.clone()).await {
                                    Ok(()) => previous_track = None,
                                    Err(lure_stoat_api::Error::ApiError(
                                        lure_stoat_api::APIError::RateLimitExceeded(remaining)
                                    )) => sleep(Duration::from_millis(remaining)).await,
                                    Err(error) => return Err(error.into()),
                                }
                            }
                        }
                        Err(error) => {
                            #[cfg(feature = "lastfm-service")]
                            if let Some(lure_lastfm_service::ServiceError::CustomError(api_error)) =
                                error.downcast_ref::<lure_lastfm_service::ServiceError>()
                            {
                                match api_error.handle_error() {
                                    lure_core::ErrorSeverity::Graceful => continue,
                                    lure_core::ErrorSeverity::Fatal => break,
                                }
                            }

                            #[cfg(feature = "listenbrainz-service")]
                            if let Some(lure_listenbrainz_service::ServiceError::CustomError(api_error)) =
                                error.downcast_ref::<lure_listenbrainz_service::ServiceError>()
                            {
                                match api_error.handle_error() {
                                    lure_core::ErrorSeverity::Graceful => continue,
                                    lure_core::ErrorSeverity::Fatal => break,
                                }
                            }

                            eprintln!("Unknown catastrophic error: {error}");
                            break;
                        }
                    }
                }
            }
        }

        stoat_client.set_status_text(first_status.clone()).await?;

        Ok(())
    }

    #[cfg(not(any(feature = "lastfm-service", feature = "listenbrainz-service")))]
    async fn run(&self) -> Result<(), Self::Error> {
        Err(ArgumentsError::NoServiceFeaturesEnabled)
    }
}

#[cfg(any(feature = "lastfm-service", feature = "listenbrainz-service"))]
#[derive(Debug, thiserror::Error)]
pub enum ArgumentsError {
    #[error("More than one service ({0}) is enabled. Only one service can be enabled at a time.")]
    MoreThanOneServiceEnabled(String),
    #[error("None of the services are enabled. One service must be enabled.")]
    NoServicesEnabled,
    #[cfg(feature = "lastfm-service")]
    #[error(transparent)]
    LastFM(#[from] lure_lastfm_service::ServiceError),
    #[cfg(feature = "listenbrainz-service")]
    #[error(transparent)]
    ListenBrainz(#[from] lure_listenbrainz_service::ServiceError),
    #[error(transparent)]
    StoatApi(#[from] lure_stoat_api::Error),
    #[error(transparent)]
    Figment(#[from] figment::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[cfg(not(any(feature = "lastfm-service", feature = "listenbrainz-service")))]
#[derive(Debug, thiserror::Error)]
pub enum ArgumentsError {
    #[error(
        "None of the service features are enabled. At least one service feature must be enabled to use this command."
    )]
    NoServiceFeaturesEnabled,
}
