#![cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]

use std::path::PathBuf;

use clap::Args;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use tokio::{signal, sync::mpsc};
use tracing::trace;

use crate::services::TrackInfo;
use crate::{config, services::ServiceProvider};

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

        // TODO: Use default from config.rs
        let config_path = self
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from("config.yaml"));

        let config: config::Config = Figment::new()
            .merge(Yaml::file(&config_path))
            .merge(Env::raw().split("__"))
            .merge(FileAdapter::wrap(Yaml::file(config_path)).only(&["api_key"]))
            .merge(FileAdapter::wrap(Env::raw().split("__")).only(&["session_token"]))
            .extract()?;

        let (tx, rx) = mpsc::channel::<ChannelData>(1);

        exit_handler(tx.clone());

        match config.enable {
            Some(enabled_service) => match enabled_service {
                #[cfg(feature = "services-lastfm")]
                config::Services::LastFM => {
                    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
                    {
                        if config.services.lastfm.is_none() {
                            anyhow::bail!("No Last.fm config specified, even though it's enabled.")
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

                    service.initialise()?;
                    service.track_check_loop(tx);

                    channel_listener(rx).await?;
                }
                #[cfg(feature = "services-listenbrainz")]
                config::Services::Listenbrainz => {
                    #[cfg(all(feature = "services-lastfm", feature = "services-listenbrainz"))]
                    if config.services.listenbrainz.is_none() {
                        anyhow::bail!("No Listenbrainz config specified, even though it's enabled.")
                    }

                    let mut service = crate::services::listenbrainz::Listenbrainz {
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

                    service.initialise()?;
                    service.track_check_loop(tx);

                    channel_listener(rx).await?;
                }
            },
            None => return Ok(()),
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
async fn channel_listener(mut rx: mpsc::Receiver<ChannelData>) -> anyhow::Result<()> {
    trace!("looping `channel_listener`");
    while let Some(data) = rx.recv().await {
        match data {
            ChannelData::Track(track) => {
                dbg!(track);
            }
            ChannelData::Exit(graceful) => {
                if graceful {
                    println!("graceful exit");
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

            let mut sigterm =
                signal(SignalKind::terminate()).expect("SIGTERM handler could not be created");

            tokio::select! {
                _ = ctrl_c => {},
                _ = sigterm.recv() => {}
            }
        }

        #[cfg(windows)]
        ctrl_c.await.expect("CTRL-C handler could not be created");

        tx.send(ChannelData::Exit(true))
            .await
            .expect("CTRL-C handler could not be created");
    });
    trace!("spawned task for `exit_handler`");
}
