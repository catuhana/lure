#![cfg(feature = "lastfm")]

use std::time::Duration;

use reqwest::{Client as ReqwestClient, ClientBuilder, Url};
use tokio::{sync::mpsc::UnboundedSender, time};

use super::{Platform, Track};

use crate::models::lastfm;
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
    const USER_AGENT: &'static str = "reqwest/0.11 [lure]";
    const API_URL: &'static str;

    async fn event_loop(
        self,
        tx: UnboundedSender<ChannelPayload>,
        check_interval: u64,
    ) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl LastFMPlatform for LastFM {
    const API_URL: &'static str = "https://ws.audioscrobbler.com/2.0";

    async fn event_loop(
        self,
        tx: UnboundedSender<ChannelPayload>,
        check_interval: u64,
    ) -> anyhow::Result<()> {
        let mut interval = time::interval(Duration::from_secs(check_interval));
        loop {
            interval.tick().await;

            let track = self.get_current_track().await;
            match track {
                Ok(track) => tx.send(ChannelPayload::Track(track))?,
                Err(err) => tracing::error!("Last.fm API error: {err}"),
            }
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

        let json = response
            .json::<lastfm::user::get_recent_tracks::Payload>()
            .await?;

        if let Some(track) = json.recenttracks.track.first() {
            if track
                .attr
                .as_ref()
                .is_some_and(|attr| attr.nowplaying.as_ref().is_some_and(|np| np == "true"))
            {
                return Ok(Some(Track {
                    artist: track.artist.text.to_string(),
                    name: track.name.to_string(),
                }));
            }
        }

        Ok(None)
    }
}
