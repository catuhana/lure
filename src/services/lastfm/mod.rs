#![cfg(feature = "services-lastfm")]

mod models;

use reqwest::{ClientBuilder, StatusCode, Url};
use serde::Deserialize;
use tokio::{
    sync::mpsc::Sender,
    time::{interval, Duration},
};
use tracing::{error, trace};

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

        match self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await
        {
            Ok(response) => {
                let recent_tracks: models::user::get_recent_tracks::Data = response.json().await?;

                if let Some(track) = recent_tracks.recenttracks.track.first() {
                    if track
                        .attr
                        .as_ref()
                        .is_some_and(|attr| attr.nowplaying.as_ref().is_some_and(|np| np == "true"))
                    {
                        return Ok(Some(TrackInfo {
                            artist: track.artist.text.clone(),
                            name: track.name.clone(),
                        }));
                    }
                }
            }
            Err(error) => match error {
                LastFMError::APIError(LastFMAPIError::RateLimitExceeded) => {
                    error!("rate limit exceeded, skipping update");
                }
                LastFMError::APIError(LastFMAPIError::TemporaryError) => {
                    error!("temporary API error occurred, skipping update");
                }
                LastFMError::APIError(LastFMAPIError::OperationFailed) => {
                    error!("something went wrong with Last.fm API, skipping update");
                }
                _ => return Err(error.into()),
            },
        }

        Ok(None)
    }
}

impl ServiceProvider for LastFM {
    fn initialise(&mut self) -> anyhow::Result<&Self> {
        trace!("initialising self fields");
        self.http_client = ClientBuilder::new().user_agent(Self::USER_AGENT).build()?;
        trace!("initialised self fields");

        Ok(self)
    }

    fn track_check_loop(self, tx: Sender<ChannelData>) {
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
                        error!("Last.fm API error: {error}");

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
    #[error("Received an unexpected response from the Last.fm API: {0}")]
    UnexpectedAPIError(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
enum LastFMAPIError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Something went wrong with Last.fm API")]
    OperationFailed,
    #[error("Provided API key is invalid")]
    InvalidAPIKey,
    #[error("API is temporarily offline")]
    ServiceOffline,
    #[error("A temporary error occurred")]
    TemporaryError,
    #[error("API key has been suspended")]
    SuspendedAPIKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
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
            StatusCode::FORBIDDEN => {
                #[derive(Deserialize)]
                struct JSONError {
                    message: String,
                    error: u64,
                }

                let error = self.json::<JSONError>().await?;
                match error.error {
                    4 => Err(LastFMAPIError::AuthenticationFailed.into()),
                    8 => Err(LastFMAPIError::OperationFailed.into()),
                    10 => Err(LastFMAPIError::InvalidAPIKey.into()),
                    11 => Err(LastFMAPIError::ServiceOffline.into()),
                    16 => Err(LastFMAPIError::TemporaryError.into()),
                    26 => Err(LastFMAPIError::SuspendedAPIKey.into()),
                    29 => Err(LastFMAPIError::RateLimitExceeded.into()),
                    _ => Err(LastFMError::UnexpectedAPIError(error.message)),
                }
            }
            _ => Err(LastFMError::UnexpectedAPIError(format!(
                "Unexpected HTTP status: {0}",
                self.status()
            ))),
        }
    }
}
