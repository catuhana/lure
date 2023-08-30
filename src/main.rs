use clap::Parser;
use cli::SubCommands;
use rive_models::authentication::Authentication;
use tokio::signal;
use tokio::sync::watch;

mod cli;
mod platforms;
mod rive;

#[cfg(feature = "lastfm")]
use crate::platforms::{lastfm::LastFM, Platform, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Args::parse();

    let status = Status {
        template: cli.status_template,
        idle: cli.status_idle,
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    tokio::spawn(async move {
        let ctrl_c = signal::ctrl_c();

        #[cfg(unix)]
        {
            use signal::unix::{signal, SignalKind};

            let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler could not be created");

            tokio::select! {
                _ = ctrl_c => {},
                _ = sigterm.recv() => {}
            }
        }

        #[cfg(windows)]
        ctrl_c.await.expect("CTRL-C handler could not be created");

        shutdown_tx.send(true)
    });

    match cli.command {
        SubCommands::LastFM {
            user,
            api_key,
            check_interval,
        } => {
            let rive_client = rive_http::Client::new(Authentication::SessionToken(cli.token));

            LastFM {
                user,
                api_key,
                ..Default::default()
            }
            .initialise()
            .await?
            .event_loop(rive_client, status, shutdown_rx, check_interval)
            .await;
        }
    }

    Ok(())
}
