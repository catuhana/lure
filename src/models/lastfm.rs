use serde::{Deserialize, Serialize};

pub mod user {
    use super::{Deserialize, Serialize};

    pub mod get_recent_tracks {
        use std::borrow::Cow;

        use super::{Deserialize, Serialize};

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Data<'a> {
            pub recenttracks: RecentTracks<'a>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct RecentTracks<'a> {
            pub track: Vec<Track<'a>>,
            #[serde(rename = "@attr")]
            pub attr: RecentTracksAttr<'a>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Track<'a> {
            pub artist: Artist<'a>,
            pub streamable: Cow<'a, str>,
            pub image: Vec<Image<'a>>,
            pub mbid: Cow<'a, str>,
            pub album: Album<'a>,
            pub name: Cow<'a, str>,
            #[serde(rename = "@attr")]
            pub attr: Option<TrackAttr<'a>>,
            pub url: Cow<'a, str>,
            pub date: Option<Date<'a>>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Artist<'a> {
            pub mbid: Cow<'a, str>,
            #[serde(rename = "#text")]
            pub text: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Image<'a> {
            pub size: Cow<'a, str>,
            #[serde(rename = "#text")]
            pub text: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Album<'a> {
            pub mbid: Cow<'a, str>,
            #[serde(rename = "#text")]
            pub text: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct TrackAttr<'a> {
            pub nowplaying: Option<Cow<'a, str>>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        pub struct Date<'a> {
            pub uts: Cow<'a, str>,
            #[serde(rename = "#text")]
            pub text: Cow<'a, str>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct RecentTracksAttr<'a> {
            pub user: Cow<'a, str>,
            pub total_pages: Cow<'a, str>,
            pub page: Cow<'a, str>,
            pub per_page: Cow<'a, str>,
            pub total: Cow<'a, str>,
        }
    }
}
