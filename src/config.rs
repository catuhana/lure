use serde::Deserialize;

fn default_template() -> String {
    "🎵 %ARTIST% – %NAME%".to_string()
}

fn default_delay() -> u64 {
    5
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub token: String,
    pub api_key: String,
    pub user: String,
    #[serde(default = "default_delay")]
    pub delay: u64,
    #[serde(default = "default_template")]
    pub template: String,
    pub idle: Option<String>,
}
