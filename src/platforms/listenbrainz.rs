// TODO: Maybe port this to ListenBrainz's
// socket.io WebSocket server?

#![cfg(feature = "listenbrainz")]

use std::time::Duration;

use reqwest::{Client as ReqwestClient, ClientBuilder, Url};
use tokio::{sync::mpsc::UnboundedSender, time};

use super::{Platform, Track};
use crate::ChannelPayload;

#[derive(Default)]
pub struct ListenBrainz {
    pub client: ReqwestClient,
    pub api_url: Option<&'static str>,
    pub user: String,
}

#[async_trait::async_trait]
pub trait ListenBrainzPlatform: Platform {
    const API_URL: &'static str;

    async fn event_loop(self, tx: UnboundedSender<ChannelPayload>, check_interval: u64);
}

#[async_trait::async_trait]
impl ListenBrainzPlatform for ListenBrainz {
    const API_URL: &'static str = "https://api.listenbrainz.org";

    async fn event_loop(self, tx: UnboundedSender<ChannelPayload>, check_interval: u64) {
        let mut interval = time::interval(Duration::from_secs(check_interval));
        loop {
            interval.tick().await;

            let track = self.get_current_track().await;
            match track {
                Ok(track) => {
                    if tx.send(ChannelPayload::Data(track)).is_err() {
                        tracing::error!("receiver dropped");
                        break;
                    }
                }
                Err(err) => tracing::error!("ListenBrainz API error: {err}"),
            }
        }
    }
}

#[async_trait::async_trait]
impl Platform for ListenBrainz {
    type Platform = Self;

    async fn initialise(mut self) -> anyhow::Result<Self::Platform> {
        self.client = ClientBuilder::new().build()?;
        Ok(self)
    }

    async fn get_current_track(&self) -> anyhow::Result<Option<Track>> {
        let url = Url::parse(
            format!(
                "{}/1/user/{}/playing-now",
                self.api_url.unwrap_or(Self::API_URL),
                &self.user
            )
            .as_str(),
        )?;

        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;

        let json = response.json::<serde_json::Value>().await?;
        let track = &json["payload"]["listens"][0];
        let track_metadata = &track["track_metadata"];

        if track
            .get("playing_now")
            .is_some_and(|playing| playing.as_bool().unwrap())
        {
            return Ok(Some(Track {
                artist: track_metadata["artist_name"].as_str().unwrap().to_string(),
                name: track_metadata["track_name"].as_str().unwrap().to_string(),
            }));
        }

        Ok(None)
    }
}
