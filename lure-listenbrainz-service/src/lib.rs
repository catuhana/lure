use core::future::Future;
use core::time::Duration;

use lure_core::{HTTPPlaybackAdapter, PlaybackStatus, ServiceCustomError, TrackInfo};
use reqwest::{ClientBuilder, StatusCode};

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
    pub fn into_playback_service(self) -> impl lure_core::PlaybackService {
        HTTPPlaybackAdapter(self)
    }
}

#[async_trait::async_trait]
impl lure_core::HTTPPlaybackService for Service {
    type Error = ServiceError;

    async fn get_current_playing_track(&self) -> Result<PlaybackStatus, Self::Error> {
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
                let mut recent_tracks: models::user::playing_now::Data = response.json().await?;

                if let Some(track) = recent_tracks.payload.listens.first_mut()
                    && track.playing_now
                {
                    return Ok(PlaybackStatus::Playing(TrackInfo {
                        artist: core::mem::take(&mut track.track_metadata.artist_name),
                        title: core::mem::take(&mut track.track_metadata.track_name),
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

#[derive(thiserror::Error, Debug)]
pub enum APIError {
    #[error("User not found.")]
    NotFound,
    #[error("Unexpected API error: {0}")]
    Unexpected(String),
}

pub type ServiceError = lure_core::ServiceError<APIError>;

impl ServiceCustomError for APIError {
    fn handle_error(&self) -> lure_core::ErrorSeverity {
        if matches!(self, Self::NotFound) {
            eprintln!("Fatal ListenBrainz error: {self}");
            lure_core::ErrorSeverity::Fatal
        } else {
            eprintln!("Non-fatal ListenBrainz error: {self}");
            lure_core::ErrorSeverity::Graceful
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
            StatusCode::NOT_FOUND => Err(APIError::NotFound.into()),
            _ => Err(
                APIError::Unexpected(format!("Unexpected HTTP status: {}", self.status())).into(),
            ),
        }
    }
}
