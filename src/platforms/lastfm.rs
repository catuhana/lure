#![cfg(feature = "lastfm")]

use std::time::Duration;

use reqwest::{Client as ReqwestClient, ClientBuilder, Url};
use tokio::{sync::mpsc::UnboundedSender, time};

use super::{Platform, Track};
use crate::ChannelPayload;

#[derive(Default)]
pub struct LastFM {
    pub client: ReqwestClient,
    pub user_agent: Option<&'static str>,
    pub api_url: Option<&'static str>,
    pub user: String,
    pub api_key: String,
}

#[async_trait::async_trait]
pub trait LastFMPlatform: Platform {
    const USER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";
    const API_URL: &'static str;

    async fn event_loop(self, event_tx: UnboundedSender<ChannelPayload>, check_interval: u64);
}

#[async_trait::async_trait]
impl LastFMPlatform for LastFM {
    const API_URL: &'static str = "https://ws.audioscrobbler.com/2.0";

    async fn event_loop(self, event_tx: UnboundedSender<ChannelPayload>, check_interval: u64) {
        let mut interval = time::interval(Duration::from_secs(check_interval));
        loop {
            let track = self.get_current_track().await;
            match track {
                Ok(track) => {
                    if event_tx.send(ChannelPayload::Data(track)).is_err() {
                        eprintln!("receiver dropped");
                        break;
                    }
                }
                Err(err) => println!("Last.fm API error: {err}"),
            }

            interval.tick().await;
        }
    }
}

#[async_trait::async_trait]
impl Platform for LastFM {
    type Platform = Self;

    async fn initialise(mut self) -> anyhow::Result<Self::Platform> {
        self.client = ClientBuilder::new()
            .user_agent(self.user_agent.unwrap_or(Self::USER_AGENT))
            .build()?;
        Ok(self)
    }

    async fn get_current_track(&self) -> anyhow::Result<Option<Track>> {
        let url = Url::parse_with_params(
            self.api_url.unwrap_or(Self::API_URL),
            &[
                ("method", "user.getrecenttracks"),
                ("user", &self.user),
                ("api_key", &self.api_key),
                ("limit", "1"),
                ("format", "json"),
            ],
        )?;

        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;

        let json = response.json::<serde_json::Value>().await?;
        let track = &json["recenttracks"]["track"][0];

        if track.get("@attr").is_some_and(|a| {
            a.get("nowplaying")
                .is_some_and(|np| np.as_str().unwrap() == "true")
        }) {
            return Ok(Some(Track {
                artist: track["artist"]["#text"].as_str().unwrap().to_string(),
                name: track["name"].as_str().unwrap().to_string(),
            }));
        }

        Ok(None)
    }
}
