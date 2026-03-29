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
