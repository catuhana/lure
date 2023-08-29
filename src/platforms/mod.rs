pub mod lastfm;

#[derive(Debug, Clone)]
pub struct Track {
    pub artist: String,
    pub name: String,
}

#[async_trait::async_trait]
pub trait Platform {
    type Platform;

    async fn initialise(self) -> anyhow::Result<Self::Platform>;
    async fn get_current_track(&self) -> anyhow::Result<Option<Track>>;
}
