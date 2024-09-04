pub mod user {
    pub mod playing_now {
        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        pub struct Data {
            pub payload: Payload,
        }

        #[derive(Deserialize, Debug)]
        pub struct Payload {
            pub listens: Vec<Listen>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Listen {
            pub playing_now: bool,
            pub track_metadata: TrackMetadata,
        }

        #[derive(Deserialize, Debug)]
        pub struct TrackMetadata {
            pub artist_name: String,
            pub track_name: String,
        }
    }
}
