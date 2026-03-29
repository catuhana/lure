use std::time::Duration;

use lure_types::{PlaybackStatus, TrackInfo};
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

    pub async fn poll(&self) -> Result<PlaybackStatus, ServiceError> {
        tokio::time::sleep(Duration::from_secs(self.options.check_interval)).await;

        let url = format!(
            "{}/1/user/{}/playing-now",
            self.options.api_url, self.options.username
        );

        let response = self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await?;

        let mut data: models::user::playing_now::Data = response.json().await?;

        if let Some(track) = data.payload.listens.first_mut()
            && track.playing_now
        {
            return Ok(PlaybackStatus::Playing(TrackInfo {
                artist: std::mem::take(&mut track.track_metadata.artist_name),
                title: std::mem::take(&mut track.track_metadata.track_name),
            }));
        }

        Ok(PlaybackStatus::NotPlaying)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum APIError {
    #[error("User not found.")]
    NotFound,
    #[error("Unexpected API error: {0}")]
    Unexpected(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    Api(#[from] APIError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl ServiceError {
    pub const fn is_fatal(&self) -> bool {
        matches!(self, Self::Api(APIError::NotFound))
    }
}

trait HandleUserFriendlyError: Sized {
    async fn handle_user_friendly_error(self) -> Result<Self, ServiceError>;
}

impl HandleUserFriendlyError for reqwest::Response {
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
