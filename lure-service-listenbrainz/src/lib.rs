use core::future::Future;
use core::{task::Poll, time::Duration};

use futures::Stream;
use lure_service_common::{PlaybackStatus, Service as _, ServiceCustomError, TrackInfo};
use reqwest::{ClientBuilder, StatusCode};
use tokio::time::{Interval, interval};

pub struct Service {
    http_client: reqwest::Client,
    interval: Interval,
    options: lure_service_listenbrainz_config::Options,
}

impl Service {
    pub fn try_new(
        options: lure_service_listenbrainz_config::Options,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            http_client: ClientBuilder::new().build()?,
            interval: interval(Duration::from_secs(options.check_interval)),
            options,
        })
    }
}

#[async_trait::async_trait]
impl lure_service_common::Service for Service {
    async fn get_current_playing_track(&self) -> Result<PlaybackStatus, anyhow::Error> {
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
                let mut recent_tracks: listenbrainz_models::user::playing_now::Data =
                    response.json().await?;

                if let Some(track) = recent_tracks.payload.listens.first_mut() {
                    if track.playing_now {
                        return Ok(PlaybackStatus::Playing(TrackInfo {
                            artist: core::mem::take(&mut track.track_metadata.artist_name),
                            title: core::mem::take(&mut track.track_metadata.track_name),
                        }));
                    }
                }
            }
            Err(error) => return Err(error.into()),
        }

        Ok(PlaybackStatus::NotPlaying)
    }
}

impl Stream for Service {
    type Item = Result<PlaybackStatus, anyhow::Error>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        match self.interval.poll_tick(cx) {
            Poll::Ready(_) => Poll::Ready(Some(futures::executor::block_on(
                self.get_current_playing_track(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum APIError {
    #[error("User not found.")]
    NotFound,
    #[error("Unexpected API error: {0}")]
    Unexpected(String),
}

pub type ServiceError = lure_service_common::ServiceError<APIError>;

impl ServiceCustomError for APIError {
    fn handle_error(&self) -> lure_service_common::ErrorSeverity {
        if matches!(self, Self::NotFound) {
            eprintln!("Fatal ListenBrainz error: {self}");
            lure_service_common::ErrorSeverity::Fatal
        } else {
            eprintln!("Non-fatal ListenBrainz error: {self}");
            lure_service_common::ErrorSeverity::Graceful
        }
    }
}

pub trait HandleServiceAPIError: Sized {
    type Error: core::error::Error;

    fn handle_user_friendly_error(self) -> impl Future<Output = Result<Self, Self::Error>>;
}

impl HandleServiceAPIError for reqwest::Response {
    type Error = ServiceError;

    async fn handle_user_friendly_error(self) -> Result<Self, Self::Error> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::NOT_FOUND => Err(APIError::NotFound.into()),
            _ => Err(
                APIError::Unexpected(format!("Unexpected HTTP status: {}", self.status())).into(),
            ),
        }
    }
}
