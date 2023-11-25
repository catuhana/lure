pub mod lastfm;
pub mod listenbrainz;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Track {
    pub artist: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub template: String,
    pub idle: Option<String>,
}

#[async_trait::async_trait]
pub trait Platform {
    type Platform;

    async fn initialise(self) -> anyhow::Result<Self::Platform>;
    async fn get_current_track(&self) -> anyhow::Result<Option<Track>>;
}
