use serde::{Deserialize, Serialize};

pub mod user {
    use super::{Deserialize, Serialize};

    pub mod playing_now {
        use std::borrow::Cow;

        use super::{Deserialize, Serialize};

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Data<'a> {
            pub payload: Payload<'a>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Payload<'a> {
            pub count: u32,
            pub listens: Vec<Listen<'a>>,
            pub playing_now: bool,
            pub user_id: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Listen<'a> {
            pub playing_now: bool,
            pub track_metadata: TrackMetadata<'a>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct TrackMetadata<'a> {
            pub additional_info: AdditionalInfo<'a>,
            pub artist_name: Cow<'a, str>,
            pub release_name: Option<Cow<'a, str>>,
            pub track_name: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct AdditionalInfo<'a> {
            pub duration: u32,
            pub music_service_name: Cow<'a, str>,
            pub origin_url: Cow<'a, str>,
            pub submission_client: Cow<'a, str>,
            pub submission_client_version: Cow<'a, str>,
        }
    }
}
