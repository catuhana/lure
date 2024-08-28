#![cfg(feature = "services-listenbrainz")]

use reqwest::StatusCode;
use serde::Deserialize;
use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};
use tracing::{error, trace};

use crate::{cli::start::ChannelData, config::ListenbrainzServiceOptions};

use super::{ServiceProvider, TrackInfo};

mod models;

#[derive(Default, Debug)]
pub struct Listenbrainz {
    pub http_client: reqwest::Client,
    pub options: ListenbrainzServiceOptions,
}

impl Listenbrainz {
    async fn get_current_playing_track(&self) -> anyhow::Result<Option<TrackInfo>> {
        let url = format!(
            "{}/1/user/{}/playing-now",
            self.options.api_url, &self.options.username
        );

        let listens: models::user::playing_now::Data = self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await?
            .json()
            .await?;

        if let Some(track) = listens.payload.listens.first() {
            if track.playing_now {
                return Ok(Some(TrackInfo {
                    artist: track.track_metadata.artist_name.clone(),
                    name: track.track_metadata.track_name.clone(),
                }));
            }
        }

        Ok(None)
    }
}

impl ServiceProvider for Listenbrainz {
    fn initialise(&mut self) -> anyhow::Result<&Self> {
        Ok(self)
    }

    // TODO: Maybe turn this into a trait implementation? Since it
    // seems like it will look the same most of the time.
    fn track_check_loop(self, tx: mpsc::Sender<crate::cli::start::ChannelData>) {
        trace!("spawning task for `track_check_loop`");
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(self.options.check_interval.into()));

            trace!("looping `track_check_loop`");
            loop {
                interval.tick().await;

                let track = self.get_current_playing_track().await;
                match track {
                    Ok(track) => tx.send(ChannelData::Track(track)).await?,
                    Err(err) => {
                        error!("Listenbrainz API error: {err}");

                        tx.send(ChannelData::Exit(false)).await?;

                        break;
                    }
                }
            }
            trace!("got out of `track_check_loop` loop");

            Ok::<_, anyhow::Error>(())
        });
        trace!("spawned task for `track_check_loop`");
    }
}

#[derive(Deserialize)]
struct ListenbrainzError {
    error: String,
}

trait ReqwestResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::NOT_FOUND => {
                let error: ListenbrainzError = self.json().await?;
                anyhow::bail!("{}", error.error);
            }
            _ => anyhow::bail!(
                "Received an unexpected response from the Listenbrainz API: {}",
                self.text().await?
            ),
        }
    }
}
