use lure_types::PlaybackStatus;

use crate::start::ServiceError;

pub enum Service {
    #[cfg(feature = "lastfm-service")]
    LastFm(lure_lastfm_service::Service),
    #[cfg(feature = "listenbrainz-service")]
    ListenBrainz(lure_listenbrainz_service::Service),
}

impl Service {
    pub async fn poll(&self) -> Result<PlaybackStatus, ServiceError> {
        match self {
            #[cfg(feature = "lastfm-service")]
            Self::LastFm(s) => s.poll().await.map_err(ServiceError::LastFm),
            #[cfg(feature = "listenbrainz-service")]
            Self::ListenBrainz(s) => s.poll().await.map_err(ServiceError::ListenBrainz),
        }
    }

    pub const fn is_fatal_error(&self, error: &ServiceError) -> bool {
        match (self, error) {
            #[cfg(feature = "lastfm-service")]
            (Self::LastFm(_), ServiceError::LastFm(e)) => e.is_fatal(),
            #[cfg(feature = "listenbrainz-service")]
            (Self::ListenBrainz(_), ServiceError::ListenBrainz(e)) => e.is_fatal(),
            _ => true,
        }
    }
}
