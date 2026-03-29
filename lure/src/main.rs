mod start;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(any(feature = "lastfm-service", feature = "listenbrainz-service"))]
    {
        let config_path = std::env::args().nth(1).map(std::path::PathBuf::from);
        Ok(start::run(config_path).await?)
    }

    #[cfg(not(any(feature = "lastfm-service", feature = "listenbrainz-service")))]
    {
        panic!("MEOW")
    }
}
