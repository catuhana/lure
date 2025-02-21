use std::path::PathBuf;

use crate::Command;
use lure_service_common::ServiceCustomError as _;

#[derive(Debug, clap::Args)]
pub struct Arguments {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Command for Arguments {
    type Error = ArgumentsError;

    #[cfg(any(feature = "service-lastfm", feature = "service-listenbrainz"))]
    async fn run(&self) -> Result<(), Self::Error> {
        const SECURE_CONFIG_KEYS: &[&str; 2] = &["session_token", "api_key"];

        use core::{pin::Pin, time::Duration};
        use std::path::Path;

        use figment::{
            Figment,
            providers::{Env, Format as _, Yaml},
        };
        use figment_file_provider_adapter::FileAdapter;
        use futures::{FutureExt as _, TryStreamExt as _};
        use lure_service_common::{PlaybackStatus, TrackInfo};
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

        // TODO: Find a better way to do this.
        let enabled_services = config.enabled_services();
        let mut service: Pin<Box<dyn lure_service_common::Service>> = match enabled_services.len() {
            0 => return Err(ArgumentsError::NoServicesEnabled),
            1 => match enabled_services.first() {
                #[cfg(feature = "service-lastfm")]
                Some(&"LastFM") => Box::pin(lure_service_lastfm::Service::try_new(
                    config.service.lastfm.unwrap(),
                )?),
                #[cfg(feature = "service-listenbrainz")]
                Some(&"ListenBrainz") => Box::pin(lure_service_listenbrainz::Service::try_new(
                    config.service.listenbrainz.unwrap(),
                )?),
                Some(_) | None => unreachable!(),
            },
            _ => {
                return Err(ArgumentsError::MoreThanOneServiceEnabled(
                    enabled_services.join(", "),
                ));
            }
        };

        let revolt_client = revolt_api::Client::try_new(
            config.revolt.api_url,
            &revolt_models::Authentication::SessionToken(config.revolt.session_token),
        )?;

        let first_status = revolt_client.get_status_text().await?;
        let mut previous_track: Option<TrackInfo> = None;

        let mut ctrl_c = Box::pin(tokio::signal::ctrl_c().fuse());

        loop {
            tokio::select! {
                _ = &mut ctrl_c => {
                    println!("Received Ctrl+C, exiting...");
                    revolt_client.set_status_text(first_status.clone()).await?;

                    break;
                },
                item = service.try_next() => {
                    match item {
                        Ok(None) => break,
                        Ok(Some(status)) => match status {
                            PlaybackStatus::Playing(track) if previous_track.as_ref().is_some_and(|prev| prev == &track) => {
                                continue;
                            }
                            PlaybackStatus::Playing(track) => {
                                let status = config
                                    .revolt
                                    .status
                                    .template
                                    .replace("%ARTIST%", &track.artist)
                                    .replace("%NAME%", &track.title);

                                match revolt_client.set_status_text(Some(status)).await {
                                    Ok(()) => previous_track = Some(track),
                                    Err(revolt_api::Error::ApiError(
                                        revolt_api::APIError::RateLimitExceeded(remaining)
                                    )) => sleep(Duration::from_millis(remaining)).await,
                                    Err(error) => return Err(error.into()),
                                }
                            }
                            PlaybackStatus::NotPlaying if previous_track.is_none() => continue,
                            PlaybackStatus::NotPlaying => {
                                match revolt_client.set_status_text(first_status.clone()).await {
                                    Ok(()) => previous_track = None,
                                    Err(revolt_api::Error::ApiError(
                                        revolt_api::APIError::RateLimitExceeded(remaining)
                                    )) => sleep(Duration::from_millis(remaining)).await,
                                    Err(error) => return Err(error.into()),
                                }
                            }
                        }
                        Err(error) => {
                            #[cfg(feature = "service-lastfm")]
                            if let Some(lure_service_lastfm::ServiceError::CustomError(api_error)) =
                                error.downcast_ref::<lure_service_lastfm::ServiceError>()
                            {
                                match api_error.handle_error() {
                                    lure_service_common::ErrorSeverity::Graceful => continue,
                                    lure_service_common::ErrorSeverity::Fatal => break,
                                }
                            }

                            #[cfg(feature = "service-listenbrainz")]
                            if let Some(lure_service_listenbrainz::ServiceError::CustomError(api_error)) =
                                error.downcast_ref::<lure_service_listenbrainz::ServiceError>()
                            {
                                match api_error.handle_error() {
                                    lure_service_common::ErrorSeverity::Graceful => continue,
                                    lure_service_common::ErrorSeverity::Fatal => break,
                                }
                            }

                            eprintln!("Unknown catastrophic error: {error}");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(not(any(feature = "service-lastfm", feature = "service-listenbrainz")))]
    async fn run(&self) -> Result<(), Self::Error> {
        Err(ArgumentsError::NoServiceFeaturesEnabled)
    }
}

#[cfg(any(feature = "service-lastfm", feature = "service-listenbrainz"))]
#[derive(Debug, thiserror::Error)]
pub enum ArgumentsError {
    #[error("More than one service ({0}) is enabled. Only one service can be enabled at a time.")]
    MoreThanOneServiceEnabled(String),
    #[error("None of the services are enabled. One service must be enabled.")]
    NoServicesEnabled,
    #[cfg(feature = "service-lastfm")]
    #[error(transparent)]
    LastFM(#[from] lure_service_lastfm::ServiceError),
    #[cfg(feature = "service-listenbrainz")]
    #[error(transparent)]
    ListenBrainz(#[from] lure_service_listenbrainz::ServiceError),
    #[error(transparent)]
    RevoltApi(#[from] revolt_api::Error),
    #[error(transparent)]
    Figment(#[from] figment::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[cfg(not(any(feature = "service-lastfm", feature = "service-listenbrainz")))]
#[derive(Debug, thiserror::Error)]
pub enum ArgumentsError {
    #[error(
        "None of the service features are enabled. At least one service feature must be enabled to use this command."
    )]
    NoServiceFeaturesEnabled,
}
