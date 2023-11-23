use clap::Parser;
use confique::Config;
use rive_models::authentication::Authentication;
use tokio::sync;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
mod handlers;
mod listeners;
mod platforms;
mod rive;

use crate::cli::Arguments;
use crate::config::Options;
use crate::handlers::update;
use crate::listeners::exit;
#[cfg(feature = "lastfm")]
use crate::platforms::lastfm::{LastFM, LastFMPlatform};
#[cfg(feature = "listenbrainz")]
use crate::platforms::listenbrainz::ListenBrainz;
use crate::platforms::{Platform, Track};
use crate::rive::ClientExt;

#[derive(Clone, Debug)]
pub enum ChannelPayload {
    Data(Option<Track>),
    Exit(bool),
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

    match Arguments::parse().command {
        cli::Subcommands::Start(start) => {
            let options = Options::builder()
                .env()
                .file(start.config.unwrap_or_else(|| {
                    dirs::config_local_dir()
                        .expect("unsupported operating system or platform")
                        .join("lure")
                        .join("config")
                        .with_extension("toml")
                }))
                .load()?;

            let (tx, rx) = sync::mpsc::unbounded_channel::<ChannelPayload>();

            // TODO: move this to somewhere so this wont run before the platforms initialise
            let rive_client =
                rive_http::Client::new(Authentication::SessionToken(options.session_token));
            if rive_client.ping().await.is_err() {
                tx.send(ChannelPayload::Exit(false))?;
            }

            exit::Listener::new(tx.clone()).listen().await?;

            tokio::spawn(async move {
                match options.platform.to_lowercase().as_str() {
                    #[cfg(feature = "lastfm")]
                    "lastfm" => {
                        tracing::info!("starting lure using Last.fm listener");
                        let lastfm_options = options.lastfm;

                        if lastfm_options.user.is_none() {
                            anyhow::bail!("`user` value on `lastfm` listener is not specified.")
                        } else if lastfm_options.api_key.is_none() {
                            anyhow::bail!("`api_key` value on `lastfm` listener is not specified.")
                        }

                        LastFM {
                            user: lastfm_options.user.unwrap(),
                            api_key: lastfm_options.api_key.unwrap(),
                            ..Default::default()
                        }
                        .initialise()
                        .await
                        .unwrap()
                        .event_loop(tx.clone(), lastfm_options.check_interval)
                        .await?;
                    }
                    #[cfg(feature = "listenbrainz")]
                    "listenbrainz" => {
                        tracing::info!("starting lure using ListenBrainz listener");
                        let listenbrainz_options = options.listenbrainz;

                        if listenbrainz_options.user.is_none() {
                            anyhow::bail!("`user` value on `lastfm` listener is not specified.")
                        }

                        ListenBrainz {
                            user: listenbrainz_options.user.unwrap(),
                            api_url: listenbrainz_options.api_url,
                            ..Default::default()
                        }
                        .initialise()
                        .await
                        .unwrap()
                        .event_loop(tx.clone(), listenbrainz_options.check_interval)
                        .await?;

                    }
                    _ => tracing::error!("unknown `platform` value specified. supported values are `lastfm` and `listenbrainz`."),
                }

                Ok(())
            });

            update::Handler::new(rx)
                .handle(rive_client, options.status)
                .await;
        }
        cli::Subcommands::Config(config) => match config {
            cli::ConfigSubcommand::Generate { print } => {
                if print {
                    println!("{}", Options::generate_config());
                } else {
                    Options::create_config().await?;
                }
            }
        },
    }

    Ok(())
}
