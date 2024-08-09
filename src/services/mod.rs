#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
use crate::cli::start::ChannelData;

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
use tokio::sync::mpsc::Sender;

pub mod lastfm;
pub mod listenbrainz;

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
#[derive(Debug)]
pub struct TrackInfo {
    pub artist: String,
    pub name: String,
}

#[cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]
pub trait ServiceProvider: Sized {
    async fn initialise(&mut self) -> anyhow::Result<&Self>;

    async fn event_loop(&self, tx: Sender<ChannelData>) -> anyhow::Result<()>;
}
