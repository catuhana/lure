#![cfg(feature = "services-listenbrainz")]

use reqwest::StatusCode;
use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};
use tracing::{error, trace};

use crate::{cli::start::ChannelData, config::ListenBrainzServiceOptions};

use super::{ServiceProvider, TrackInfo};

mod models;

#[derive(Default, Debug)]
pub struct ListenBrainz {
    pub http_client: reqwest::Client,
    pub options: ListenBrainzServiceOptions,
}

impl ListenBrainz {
    async fn get_current_playing_track(&self) -> anyhow::Result<Option<TrackInfo>> {
        let url = format!(
            "{}/1/user/{}/playing-now",
            self.options.api_url, &self.options.username
        );

        match self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await
        {
            Ok(response) => {
                let listens: models::user::playing_now::Data = response.json().await?;

                if let Some(track) = listens.payload.listens.first() {
                    if track.playing_now {
                        return Ok(Some(TrackInfo {
                            artist: track.track_metadata.artist_name.clone(),
                            name: track.track_metadata.track_name.clone(),
                        }));
                    }
                }
            }
            Err(error) => match error {
                LastFMError::APIError(LastFMAPIError::NotFound()) => {
                    error!("User not found: {error}");
                }
                _ => return Err(error.into()),
            },
        }

        Ok(None)
    }
}

impl ServiceProvider for ListenBrainz {
    fn initialise(&mut self) -> anyhow::Result<&Self> {
        Ok(self)
    }

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
                    Err(error) => {
                        error!("ListenBrainz API error: {error}");

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

#[derive(thiserror::Error, Debug)]
enum LastFMError {
    #[error(transparent)]
    APIError(#[from] LastFMAPIError),
    #[error("Received an unexpected response from the ListenBrainz API: {0}")]
    UnexpectedAPIError(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
enum LastFMAPIError {
    #[error("User not found.")]
    NotFound(),
}

impl From<reqwest::Error> for LastFMError {
    fn from(error: reqwest::Error) -> Self {
        Self::Other(error.into())
    }
}

trait ResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self, LastFMError>;
}

impl ResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self, LastFMError> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::NOT_FOUND => Err(LastFMAPIError::NotFound().into()),
            _ => Err(LastFMError::UnexpectedAPIError(format!(
                "Unexpected HTTP status: {}",
                self.status()
            ))),
        }
    }
}
