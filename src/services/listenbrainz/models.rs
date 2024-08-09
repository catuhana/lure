pub mod user {
    use serde::Deserialize;

    pub mod playing_now {
        use std::borrow::Cow;

        use super::Deserialize;

        #[derive(Deserialize, Debug)]
        pub struct Data<'a> {
            pub payload: Payload<'a>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Payload<'a> {
            pub listens: Vec<Listen<'a>>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Listen<'a> {
            pub playing_now: bool,
            pub track_metadata: TrackMetadata<'a>,
        }

        #[derive(Deserialize, Debug)]
        pub struct TrackMetadata<'a> {
            pub artist_name: Cow<'a, str>,
            pub track_name: Cow<'a, str>,
        }
    }
}
