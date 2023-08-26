#[derive(Debug, Clone)]
pub struct Track {
    pub artist: String,
    pub name: String,
}

pub async fn get_current_track(api_key: &str, user: &str) -> anyhow::Result<Option<Track>> {
    let url = reqwest::Url::parse_with_params(
        "http://ws.audioscrobbler.com/2.0/",
        &[
            ("method", "user.getrecenttracks"),
            ("user", user),
            ("api_key", api_key),
            ("limit", "1"),
            ("format", "json"),
        ],
    )?;

    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;

    let json = response.json::<serde_json::Value>().await?;
    let track = &json["recenttracks"]["track"][0];

    // wtf
    if track.get("@attr").is_some_and(|a| {
        a.get("nowplaying")
            .is_some_and(|np| np.as_str().unwrap() == "true")
    }) {
        return Ok(Some(Track {
            artist: track["artist"]["#text"].as_str().unwrap().to_string(),
            name: track["name"].as_str().unwrap().to_string(),
        }));
    }

    Ok(None)
}
