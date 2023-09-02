use clap::Parser;
use cli::SubCommands;
use rive_models::authentication::Authentication;
use tokio::sync;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod handlers;
mod platforms;
mod rive;

use crate::handlers::ExitHandler;
#[cfg(feature = "lastfm")]
use crate::platforms::{
    lastfm::{LastFM, LastFMPlatform},
    Platform, Track,
};
use crate::rive::ClientExt;

#[derive(Clone, Debug)]
pub enum ChannelPayload {
    Data(Option<Track>),
    Exit,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lure=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = cli::Args::parse();

    let (tx, mut rx) = sync::mpsc::unbounded_channel::<ChannelPayload>();
    let rive_client = rive_http::Client::new(Authentication::SessionToken(cli.token));

    if rive_client.ping().await.is_none() {
        tx.send(ChannelPayload::Exit)?;
    }

    ExitHandler::new(tx.clone()).handle().await;

    // TODO: Write a handler for that, similar to ExitHandler.
    // Requires some trait work to do.
    tokio::spawn(async move {
        match cli.command {
            SubCommands::LastFM {
                user,
                api_key,
                check_interval,
            } => {
                LastFM {
                    user,
                    api_key,
                    ..Default::default()
                }
                .initialise()
                .await
                .unwrap()
                .event_loop(tx.clone(), check_interval)
                .await;
            }
        }
    });

    let mut previous_track: Option<Track> = None;
    while let Some(payload) = rx.recv().await {
        match payload {
            ChannelPayload::Data(track) => {
                if previous_track == track {
                    continue;
                };

                let status = track
                    .as_ref()
                    .map(|track| {
                        cli.status_template
                            .replace("%ARTIST%", &track.artist)
                            .replace("%NAME%", &track.name)
                    })
                    .or_else(|| cli.status_idle.clone());

                rive_client.set_status(status).await;
                previous_track = track;
            }
            ChannelPayload::Exit => {
                tracing::info!("stopping lure");
                rive_client.set_status(None).await;
                break;
            }
        }
    }

    Ok(())
}
