use clap::Parser;
use cli::SubCommands;
use rive_models::authentication::Authentication;
use tokio::sync;

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
    let cli = cli::Args::parse();

    let (tx, mut rx) = sync::mpsc::unbounded_channel::<ChannelPayload>();
    ExitHandler::new(tx.clone()).handle().await;

    let rive_client = rive_http::Client::new(Authentication::SessionToken(cli.token));

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

    while let Some(payload) = rx.recv().await {
        match payload {
            ChannelPayload::Data(track) => {
                let status = track
                    .map(|track| {
                        cli.status_template
                            .replace("%ARTIST%", &track.artist)
                            .replace("%NAME%", &track.name)
                    })
                    .or_else(|| cli.status_idle.to_owned());

                rive_client.set_status(status).await;
            }
            ChannelPayload::Exit => {
                rive_client.set_status(None).await;
                break;
            }
        }
    }

    Ok(())
}
