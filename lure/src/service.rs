use lure_types::PlaybackStatus;

use crate::start::ServiceError;

pub enum Service {
    LastFm(lure_lastfm_service::Service),
    ListenBrainz(lure_listenbrainz_service::Service),
}

impl Service {
    pub async fn poll(&self) -> Result<PlaybackStatus, ServiceError> {
        match self {
            Self::LastFm(s) => s.poll().await.map_err(ServiceError::LastFm),
            Self::ListenBrainz(s) => s.poll().await.map_err(ServiceError::ListenBrainz),
        }
    }

    pub const fn is_fatal_error(&self, error: &ServiceError) -> bool {
        match (self, error) {
            (Self::LastFm(_), ServiceError::LastFm(e)) => e.is_fatal(),
            (Self::ListenBrainz(_), ServiceError::ListenBrainz(e)) => e.is_fatal(),
            _ => true,
        }
    }
}
