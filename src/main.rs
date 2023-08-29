use std::time::Duration;

use clap::Parser;
use cli::SubCommands;
use rive_models::{
    authentication::Authentication,
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};

mod cli;
mod platforms;

#[cfg(feature = "lastfm")]
use platforms::{lastfm::LastFM, Platform};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Args::parse();

    match cli.command {
        SubCommands::LastFM {
            user,
            api_key,
            check_delay,
        } => {
            let last_fm = LastFM {
                user,
                api_key,
                ..Default::default()
            }
            .initialise()
            .await?;

            let revolt_client = rive_http::Client::new(Authentication::SessionToken(cli.token));

            loop {
                let track = last_fm.get_current_track().await;

                match track {
                    Ok(track) => {
                        let status = track
                            .map(|track| {
                                cli.status_template
                                    .replace("%ARTIST%", &track.artist)
                                    .replace("%NAME%", &track.name)
                            })
                            .or_else(|| cli.status_idle.to_owned());

                        let data = status.map_or(
                            EditUserData {
                                remove: Some(vec![FieldsUser::StatusText]),
                                ..Default::default()
                            },
                            |text| EditUserData {
                                status: Some(UserStatus {
                                    text: Some(text),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        );

                        match revolt_client.edit_user(data).await {
                            Ok(_) => (),
                            Err(err) => println!("Revolt API error: {err}"),
                        };
                    }
                    Err(err) => {
                        println!("Last.fm API error: {err}");
                    }
                }

                tokio::time::sleep(Duration::from_secs(check_delay)).await;
            }
        }
    }
}
