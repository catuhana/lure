use std::{future::Future, time::Duration};

use lure_core::{
    HTTPPlaybackAdapter, PlaybackService, PlaybackStatus, ServiceCustomError, TrackInfo,
};
use reqwest::{ClientBuilder, StatusCode, Url};
use secrecy::ExposeSecret as _;

pub mod config;
pub mod models;

pub struct Service {
    http_client: reqwest::Client,
    options: config::Options,
}

impl Service {
    pub fn try_new(options: config::Options) -> Result<Self, ServiceError> {
        Ok(Self {
            http_client: ClientBuilder::new().build()?,
            options,
        })
    }

    #[must_use]
    pub fn into_playback_service(self) -> impl PlaybackService {
        HTTPPlaybackAdapter(self)
    }
}

#[async_trait::async_trait]
impl lure_core::HTTPPlaybackService for Service {
    type Error = ServiceError;

    async fn get_current_playing_track(&self) -> Result<PlaybackStatus, Self::Error> {
        let url = Url::parse_with_params(
            "https://ws.audioscrobbler.com/2.0/",
            &[
                ("method", "user.getrecenttracks"),
                ("user", &self.options.username),
                ("api_key", self.options.api_key.expose_secret()),
                ("limit", "1"),
                ("format", "json"),
            ],
        )
        .map_err(|error| ServiceError::Anyhow(error.into()))?;

        match self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await
        {
            Ok(response) => {
                let mut recent_tracks: models::user::get_recent_tracks::Data =
                    response.json().await?;

                if let Some(track) = recent_tracks.recenttracks.track.first_mut()
                    && track
                        .attr
                        .as_ref()
                        .is_some_and(|attr| attr.nowplaying.as_ref().is_some_and(|np| *np))
                {
                    return Ok(PlaybackStatus::Playing(TrackInfo {
                        artist: std::mem::take(&mut track.artist.text),
                        title: std::mem::take(&mut track.name),
                    }));
                }
            }
            Err(error) => return Err(error),
        }

        Ok(PlaybackStatus::NotPlaying)
    }

    fn polling_interval(&self) -> Duration {
        Duration::from_secs(self.options.check_interval)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum APIError {
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
    #[error("Unexpected API error: {0}")]
    Unexpected(String),
}

pub type ServiceError = lure_core::ServiceError<APIError>;

impl ServiceCustomError for APIError {
    fn handle_error(&self) -> lure_core::ErrorSeverity {
        match self {
            Self::AuthenticationFailed | Self::InvalidAPIKey | Self::SuspendedAPIKey => {
                eprintln!("Fatal LastFM error: {self}");
                lure_core::ErrorSeverity::Fatal
            }
            _ => {
                eprintln!("Non-fatal LastFM error: {self}");
                lure_core::ErrorSeverity::Graceful
            }
        }
    }
}

pub trait HandleServiceAPIError: Sized {
    fn handle_user_friendly_error(self) -> impl Future<Output = Result<Self, ServiceError>>;
}

impl HandleServiceAPIError for reqwest::Response {
    async fn handle_user_friendly_error(self) -> Result<Self, ServiceError> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::FORBIDDEN => {
                let error: models::user::get_recent_tracks::Error = self.json().await?;
                match error.error {
                    4 => Err(APIError::AuthenticationFailed.into()),
                    8 => Err(APIError::OperationFailed.into()),
                    10 => Err(APIError::InvalidAPIKey.into()),
                    11 => Err(APIError::ServiceOffline.into()),
                    16 => Err(APIError::TemporaryError.into()),
                    26 => Err(APIError::SuspendedAPIKey.into()),
                    29 => Err(APIError::RateLimitExceeded.into()),
                    _ => Err(APIError::Unexpected(error.message).into()),
                }
            }
            _ => Err(
                APIError::Unexpected(format!("Unexpected status code: {}", self.status())).into(),
            ),
        }
    }
}
