#![cfg(feature = "listenbrainz")]

use std::time::Duration;

use reqwest::Client as ReqwestClient;
use tokio::{sync::mpsc::UnboundedSender, time};

use super::{Platform, Track};

use crate::models::listenbrainz;
use crate::ChannelMessage;

#[derive(Default)]
pub struct ListenBrainz {
    pub client: ReqwestClient,
    pub api_url: String,
    pub user: String,
}

impl ListenBrainz {
    pub async fn event_loop(
        self,
        tx: UnboundedSender<ChannelMessage>,
        check_interval: u64,
    ) -> anyhow::Result<()> {
        let mut interval = time::interval(Duration::from_secs(check_interval));
        loop {
            interval.tick().await;

            let track = self.get_current_track().await;
            match track {
                Ok(track) => tx.send(ChannelMessage::Track(track))?,
                Err(err) => tracing::error!("ListenBrainz API error: {err}"),
            }
        }
    }
}

#[async_trait::async_trait]
impl Platform for ListenBrainz {
    type Platform = Self;

    async fn initialise(self) -> anyhow::Result<Self::Platform> {
        Ok(self)
    }

    async fn get_current_track(&self) -> anyhow::Result<Option<Track>> {
        let url = format!("{}/1/user/{}/playing-now", self.api_url, &self.user);

        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;

        let json = response
            .json::<listenbrainz::user::playing_now::Data>()
            .await?;

        if let Some(track) = json.payload.listens.first() {
            if track.playing_now {
                return Ok(Some(Track {
                    artist: track.track_metadata.artist_name.to_string(),
                    name: track.track_metadata.track_name.to_string(),
                }));
            }
        }

        Ok(None)
    }
}
