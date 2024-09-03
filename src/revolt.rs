#![cfg(any(feature = "services-lastfm", feature = "services-listenbrainz"))]

use std::str::FromStr;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, StatusCode,
};
use rive_models::{
    authentication::Authentication,
    data::EditUserData,
    user::{FieldsUser, User, UserStatus},
};
use tracing::trace;

pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: String::from("https://api.revolt.chat"),
        }
    }
}

impl HttpClient {
    pub fn try_new(api_url: String, authentication: &Authentication) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str(&authentication.header_key())?,
            HeaderValue::from_str(&authentication.value())?,
        );
        let reqwest_client = Client::builder().default_headers(headers);

        Ok(Self {
            client: reqwest_client.build()?,
            base_url: api_url,
        })
    }

    pub async fn set_status(&self, status: Option<String>) -> anyhow::Result<()> {
        tracing::info!("updating Revolt status to {:?}", &status);

        let data = status.map_or_else(
            || EditUserData {
                remove: Some(vec![FieldsUser::StatusText]),
                ..Default::default()
            },
            |text| EditUserData {
                status: Some(UserStatus {
                    text: Some(text),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        let response = self
            .client
            .patch(format!("{}/users/@me", self.base_url))
            .json(&data)
            .send()
            .await?
            .handle_user_friendly_error()
            .await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to update the Revolt status: {}", response.status())
        }

        tracing::debug!("updated Revolt status");

        Ok(())
    }

    pub async fn get_status(&self) -> anyhow::Result<Option<String>> {
        trace!("fetching user data from Revolt API (get_status)...");

        let response = self
            .client
            .get(format!("{}/users/@me", self.base_url))
            .send()
            .await?
            .handle_user_friendly_error()
            .await?;

        let user_data: User = response.json().await?;
        let status = user_data.status.and_then(|status| status.text);

        trace!("successfully fetched the Revolt status");

        Ok(status)
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        trace!("fetching user data from Revolt API (ping)...");

        let response = self
            .client
            .get(format!("{}/users/@me", self.base_url))
            .send()
            .await?
            .handle_user_friendly_error()
            .await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to ping the Revolt API: {}", response.status())
        }

        trace!("successfully pinged the Revolt API");

        Ok(())
    }
}

trait ReqwestResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self> {
        dbg!(&self);
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::TOO_MANY_REQUESTS => {
                anyhow::bail!("Hit Revolt API rate limit. Please try again some time later.")
            }
            StatusCode::UNAUTHORIZED => {
                anyhow::bail!("Revolt API authentication failed. Please check your credentials.")
            }
            _ => anyhow::bail!("Unexpected error: {}", self.text().await?),
        }
    }
}
