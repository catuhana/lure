#![cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]

use std::path::{Path, PathBuf};

use clap::Args;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use rive_models::authentication::Authentication;
use tokio::{signal, sync::mpsc};
use tracing::{debug, trace};

use crate::{
    config::{self, RevoltStatusOptions},
    services::ServiceProvider,
};
use crate::{revolt, services::TrackInfo};

use super::Command;

#[derive(Args, Debug)]
pub struct CommandArguments {
    /// Path of lure config file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Command for CommandArguments {
    async fn run(&self) -> anyhow::Result<()> {
        trace!("`start` subcommand");

        let config_path = self
            .config
            .as_ref()
            .map_or_else(|| Path::new("config.yaml"), PathBuf::as_path);

        let config: config::Config = Figment::new()
            .merge(Yaml::file(config_path))
            .merge(Env::prefixed("LURE_").split("__"))
            .merge(FileAdapter::wrap(Yaml::file(config_path)).only(&["session_token", "api_key"]))
            .merge(
                FileAdapter::wrap(Env::prefixed("LURE_").split("__"))
                    .only(&["session_token", "api_key"]),
            )
            .extract()?;

        let (tx, rx) = mpsc::channel::<ChannelData>(1);

        exit_handler(tx.clone());

        match config.enable {
            Some(enabled_service) => match enabled_service {
                // TODO: Create a macro for this.
                #[cfg(feature = "services-lastfm")]
                config::Services::LastFM => {
                    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
                    {
                        if config.services.lastfm.is_none() {
                            anyhow::bail!("Last.fm is enabled, but no configuration is provided.")
                        }
                    }

                    let mut service = crate::services::lastfm::LastFM {
                        #[cfg(all(
                            feature = "services-lastfm",
                            not(feature = "services-listenbrainz")
                        ))]
                        options: config.services.lastfm,
                        #[cfg(all(
                            feature = "services-lastfm",
                            feature = "services-listenbrainz"
                        ))]
                        options: config.services.lastfm.unwrap(),
                        ..Default::default()
                    };

                    let revolt_client = revolt::HttpClient::try_new(
                        config.revolt.api_url,
                        &Authentication::SessionToken(config.revolt.session_token),
                    )?;
                    revolt_client.ping().await?;

                    service.initialise()?;
                    service.track_check_loop(tx);

                    channel_listener(rx, revolt_client, config.revolt.status).await?;
                }
                #[cfg(feature = "services-listenbrainz")]
                config::Services::ListenBrainz => {
                    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
                    if config.services.listenbrainz.is_none() {
                        anyhow::bail!("ListenBrainz is enabled, but no configuration is provided.")
                    }

                    let mut service = crate::services::listenbrainz::ListenBrainz {
                        #[cfg(all(
                            feature = "services-listenbrainz",
                            not(feature = "services-lastfm")
                        ))]
                        options: config.services.listenbrainz,
                        #[cfg(all(
                            feature = "services-listenbrainz",
                            feature = "services-lastfm"
                        ))]
                        options: config.services.listenbrainz.unwrap(),
                        ..Default::default()
                    };

                    let revolt_client = revolt::HttpClient::try_new(
                        config.revolt.api_url,
                        &Authentication::SessionToken(config.revolt.session_token),
                    )?;
                    revolt_client.ping().await?;

                    service.initialise()?;
                    service.track_check_loop(tx);

                    channel_listener(rx, revolt_client, config.revolt.status).await?;
                }
            },
            None => anyhow::bail!(
                "No service is enabled. Please enable a service in the configuration file."
            ),
        }

        Ok(())
    }
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Debug)]
pub enum ChannelData {
    Track(Option<TrackInfo>),
    Exit(bool),
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
async fn channel_listener(
    mut rx: mpsc::Receiver<ChannelData>,
    revolt_client: revolt::HttpClient,
    revolt_status: RevoltStatusOptions,
) -> anyhow::Result<()> {
    use tokio::time;

    trace!("looping `channel_listener`");

    let first_status = revolt_client.get_status().await?;
    let mut previous_track: Option<TrackInfo> = None;

    while let Some(data) = rx.recv().await {
        match data {
            ChannelData::Track(track) if track.is_some() => {
                if previous_track == track {
                    debug!(
                        "current track `{track:?}` is the same as previous track `{previous_track:?}`, skipping status update"
                    );
                    continue;
                }

                let status = track.as_ref().map_or_else(
                    || revolt_status.idle.clone(),
                    |track| {
                        Some(
                            revolt_status
                                .template
                                .replace("%ARTIST%", &track.artist)
                                .replace("%NAME%", &track.name),
                        )
                    },
                );

                match revolt_client.set_status(status).await {
                    Ok(()) => {
                        previous_track = track;
                    }
                    Err(error) => {
                        if let revolt::RevoltAPIError::RateLimitExceeded(remaining) = error {
                            tracing::warn!("rate limit exceeded, waiting until the time limit is over to update status...");
                            time::sleep(time::Duration::from_millis(remaining.try_into()?)).await;
                        } else {
                            tracing::error!("error occurred while updating status: {:?}", error);
                            return Err(error.into());
                        }
                    }
                }
            }
            ChannelData::Track(track) if track.is_none() => {
                if previous_track.is_none() {
                    debug!("no track to update, skipping status update");
                    continue;
                }

                debug!("no track to update, reverting to previous status");
                match revolt_client.set_status(first_status.clone()).await {
                    Ok(()) => {
                        previous_track = None;
                    }
                    Err(error) => match error {
                        revolt::RevoltAPIError::RateLimitExceeded(_remaining) => {
                            tracing::warn!("rate limit exceeded, skipping status update");
                            continue;
                        }
                        _ => {
                            return Err(error.into());
                        }
                    },
                }
            }
            ChannelData::Track(_) => unreachable!("unexpected `ChannelData::Track` variant"),
            ChannelData::Exit(graceful) => {
                tracing::info!("stopping lure");

                if graceful {
                    loop {
                        match revolt_client.set_status(first_status.clone()).await {
                            Ok(()) => break,
                            Err(error) => match error {
                                revolt::RevoltAPIError::RateLimitExceeded(remaining) => {
                                    if remaining > 0 {
                                        tracing::warn!("rate limit exceeded, waiting until the time limit is over to revert status...");
                                        time::sleep(time::Duration::from_millis(
                                            remaining.try_into()?,
                                        ))
                                        .await;
                                    }
                                }
                                _ => {
                                    return Err(error.into());
                                }
                            },
                        }
                    }
                }

                break;
            }
        }
    }

    trace!("got out of `channel_listener` loop");

    Ok(())
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
fn exit_handler(tx: mpsc::Sender<ChannelData>) {
    trace!("spawning task for `exit_handler`");
    tokio::spawn(async move {
        let ctrl_c = signal::ctrl_c();

        #[cfg(unix)]
        {
            use signal::unix::{signal, SignalKind};

            let mut sigterm = signal(SignalKind::terminate())
                .expect("SIGTERM signal handler could not be created");

            tokio::select! {
                _ = ctrl_c => {},
                _ = sigterm.recv() => {}
            }
        }

        #[cfg(windows)]
        ctrl_c
            .await
            .expect("CTRL-C signal handler could not be created");

        tx.send(ChannelData::Exit(true))
            .await
            .expect("CTRL-C response could not be sent");
    });
    trace!("spawned task for `exit_handler`");
}
