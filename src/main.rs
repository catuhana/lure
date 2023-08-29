use std::time::Duration;

use rive_models::{
    authentication::Authentication,
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};

mod config;
mod platforms;

use config::Config;
use platforms::{lastfm::LastFM, Platform};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = envy::prefixed("LR_").from_env::<Config>()?;
    let authetication = Authentication::SessionToken(config.token);
    let client = rive_http::Client::new(authetication);

    let lastfm = LastFM {
        user: config.user,
        api_key: config.api_key,
        ..Default::default()
    }
    .initialise()
    .await?;

    loop {
        let track = lastfm.get_current_track().await;

        match track {
            Ok(track) => {
                let status = track
                    .map(|track| {
                        config
                            .template
                            .replace("%ARTIST%", &track.artist)
                            .replace("%NAME%", &track.name)
                    })
                    .or(config.idle.to_owned());

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

                match client.edit_user(data).await {
                    Ok(_) => (),
                    Err(err) => println!("Revolt API error: {err}"),
                };
            }
            Err(err) => {
                println!("Last.fm API error: {err}");
            }
        }

        tokio::time::sleep(Duration::from_secs(config.delay)).await;
    }
}
