// TODO: Maybe port this to ListenBrainz's
// socket.io WebSocket server?

#![cfg(feature = "listenbrainz")]

use std::time::Duration;

use reqwest::Client as ReqwestClient;
use tokio::{sync::mpsc::UnboundedSender, time};

use super::{Platform, Track};
use crate::ChannelPayload;

#[derive(Default)]
pub struct ListenBrainz {
    pub client: ReqwestClient,
    pub api_url: String,
    pub user: String,
}

impl ListenBrainz {
    pub async fn event_loop(
        self,
        tx: UnboundedSender<ChannelPayload>,
        check_interval: u64,
    ) -> anyhow::Result<()> {
        let mut interval = time::interval(Duration::from_secs(check_interval));
        loop {
            interval.tick().await;

            let track = self.get_current_track().await;
            match track {
                Ok(track) => tx.send(ChannelPayload::Data(track))?,
                Err(err) => tracing::error!("ListenBrainz API error: {err}"),
            }
        }
    }
}

impl Platform for ListenBrainz {
    type Platform = Self;

    async fn initialise(self) -> anyhow::Result<Self::Platform> {
        Ok(self)
    }

    async fn get_current_track(&self) -> anyhow::Result<Option<Track>> {
        let url = format!("{}/1/user/{}/playing-now", self.api_url, &self.user);

        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;

        let json = response.json::<serde_json::Value>().await?;
        let track = &json["payload"]["listens"][0];

        if track
            .get("playing_now")
            .is_some_and(|playing| playing.as_bool().unwrap())
        {
            return Ok(Some(Track {
                artist: track["track_metadata"]["artist_name"]
                    .as_str()
                    .unwrap()
                    .into(),
                name: track["track_metadata"]["track_name"]
                    .as_str()
                    .unwrap()
                    .into(),
            }));
        }

        Ok(None)
    }
}
