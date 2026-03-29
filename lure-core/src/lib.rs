use std::sync::Arc;

use futures::Stream;

#[derive(Debug, PartialEq, Eq)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing(TrackInfo),
    NotPlaying,
}

pub enum ErrorSeverity {
    Graceful,
    Fatal,
}

pub trait PlaybackService: Send + Sync {
    fn into_stream(
        self,
    ) -> Box<dyn Stream<Item = Result<PlaybackStatus, anyhow::Error>> + Send + Unpin>;
}

#[async_trait::async_trait]
pub trait HTTPPlaybackService: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get_current_playing_track(&self) -> Result<PlaybackStatus, Self::Error>;

    fn polling_interval(&self) -> std::time::Duration;
}

pub trait WebSocketPlaybackService:
    Stream<Item = Result<PlaybackStatus, anyhow::Error>> + Send + Sync + Unpin + 'static
{
}

pub struct HTTPPlaybackAdapter<T: HTTPPlaybackService>(pub T);

impl<T: HTTPPlaybackService + Unpin> PlaybackService for HTTPPlaybackAdapter<T>
where
    T::Error: Into<anyhow::Error>,
{
    fn into_stream(
        self,
    ) -> Box<dyn Stream<Item = Result<PlaybackStatus, anyhow::Error>> + Send + Unpin> {
        let polling_interval = self.0.polling_interval();
        let service = Arc::new(self.0);

        let stream = futures_util::stream::unfold(
            (tokio::time::interval(polling_interval), service),
            |(mut interval, service)| async move {
                interval.tick().await;

                let result = match service.get_current_playing_track().await {
                    Ok(status) => Ok(status),
                    Err(err) => Err(err.into()),
                };

                Some((result, (interval, service)))
            },
        );

        Box::new(Box::pin(stream))
    }
}

pub struct WebSocketPlaybackAdapter<T: WebSocketPlaybackService>(pub T);

impl<T: WebSocketPlaybackService> PlaybackService for WebSocketPlaybackAdapter<T> {
    fn into_stream(
        self,
    ) -> Box<dyn Stream<Item = Result<PlaybackStatus, anyhow::Error>> + Send + Unpin> {
        Box::new(self.0)
    }
}

pub trait ServiceCustomError: std::error::Error + Send + Sync + 'static {
    fn handle_error(&self) -> ErrorSeverity;
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError<T: ServiceCustomError> {
    #[error(transparent)]
    CustomError(T),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl<T: ServiceCustomError> From<T> for ServiceError<T> {
    fn from(error: T) -> Self {
        Self::CustomError(error)
    }
}
