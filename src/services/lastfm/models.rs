pub mod user {
    use serde::Deserialize;

    pub mod get_recent_tracks {
        use super::Deserialize;

        #[derive(Deserialize, Debug)]
        pub struct Data {
            pub recenttracks: RecentTracks,
        }

        #[derive(Deserialize, Debug)]
        pub struct RecentTracks {
            pub track: Vec<Track>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Track {
            pub artist: Artist,
            pub name: String,
            #[serde(rename = "@attr")]
            pub attr: Option<TrackAttr>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Artist {
            #[serde(rename = "#text")]
            pub text: String,
        }

        #[derive(Deserialize, Debug)]
        pub struct TrackAttr {
            pub nowplaying: Option<String>,
        }
    }
}
