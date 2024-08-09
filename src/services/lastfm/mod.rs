#![cfg(feature = "services-lastfm")]

mod models;

use reqwest::{ClientBuilder, StatusCode, Url};
use serde::Deserialize;
use tokio::{
    sync::mpsc::Sender,
    time::{interval, Duration},
};

use crate::{cli::start::ChannelData, config::LastFMServiceOptions};

use super::{ServiceProvider, TrackInfo};

#[derive(Default, Debug)]
pub struct LastFM {
    pub http_client: reqwest::Client,
    pub options: LastFMServiceOptions,
}

pub trait LastFMCompatibleServiceProvider: ServiceProvider {
    const USER_AGENT: &'static str = "reqwest/0.12 [lure]";
    const API_URL: &'static str;
}

impl LastFMCompatibleServiceProvider for LastFM {
    const API_URL: &'static str = "http://ws.audioscrobbler.com/2.0/";
}

impl LastFM {
    async fn get_current_playing_track(&self) -> anyhow::Result<Option<TrackInfo>> {
        let url = Url::parse_with_params(
            Self::API_URL,
            &[
                ("method", "user.getrecenttracks"),
                ("user", &self.options.username),
                ("api_key", &self.options.api_key),
                ("limit", "1"),
                ("format", "json"),
            ],
        )?;

        let recent_tracks: models::user::get_recent_tracks::Data = self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await?
            .json()
            .await?;

        if let Some(track) = recent_tracks.recenttracks.track.first() {
            if track
                .attr
                .as_ref()
                .is_some_and(|attr| attr.nowplaying.as_ref().is_some_and(|np| np == "true"))
            {
                return Ok(Some(TrackInfo {
                    artist: track.artist.text.to_string(),
                    name: track.name.to_string(),
                }));
            }
        }

        Ok(None)
    }
}

impl ServiceProvider for LastFM {
    async fn initialise(&mut self) -> anyhow::Result<&Self> {
        self.http_client = ClientBuilder::new().user_agent(Self::USER_AGENT).build()?;

        Ok(self)
    }

    async fn event_loop(&self, tx: Sender<ChannelData>) -> anyhow::Result<()> {
        let mut interval = interval(Duration::from_secs(self.options.check_interval.into()));
        loop {
            interval.tick().await;

            let track = self.get_current_playing_track().await;
            match track {
                Ok(track) => tx.send(ChannelData::Track(track)).await?,
                Err(err) => {
                    // TODO: Use tracing::error!
                    eprintln!("Last.fm API error: {err}");
                    tx.send(ChannelData::Exit(false)).await?;
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct LastFMError {
    message: String,
    #[serde(rename = "error", skip)]
    _error: usize,
}

trait ReqwestResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::FORBIDDEN => {
                let error_json = self.json::<LastFMError>().await?;
                anyhow::bail!("{}", error_json.message);
            }
            _ => anyhow::bail!(
                "Unexpected response from Last.fm api: {}",
                self.text().await?
            ),
        }
    }
}
