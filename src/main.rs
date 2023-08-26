mod api;
mod config;

use std::time::Duration;

use config::Config;
use rive_models::{
    authentication::Authentication,
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let config = envy::prefixed("LR_").from_env::<Config>()?;
    let authetication = Authentication::SessionToken(config.token);
    let client = rive_http::Client::new(authetication);

    loop {
        // yeah imma genius
        if false {
            break;
        }

        let res = api::get_current_track(&config.api_key, &config.user).await;

        match res {
            Ok(res) => {
                let text = res
                    .map(|track| {
                        config
                            .template
                            .replace("%ARTIST%", &track.artist)
                            .replace("%NAME%", &track.name)
                    })
                    .or(config.idle.to_owned());

                let data = text.map_or(
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

        sleep(Duration::from_secs(config.delay)).await;
    }

    Ok(())
}
