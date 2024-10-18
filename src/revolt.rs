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
use tracing::{error, trace};

#[derive(thiserror::Error, Debug)]
pub enum RevoltAPIError {
    #[error("Revolt API authentication failed. Please check your credentials.")]
    AuthenticationFailed,
    #[error("Revolt API rate limit exceeded.")]
    RateLimitExceeded(u128),
    #[error("Revolt API returned an unexpected error: {0}")]
    Unknown(StatusCode),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

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

    pub async fn set_status(&self, status: Option<String>) -> anyhow::Result<(), RevoltAPIError> {
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

        self.client
            .patch(format!("{}/users/@me", self.base_url))
            .json(&data)
            .send()
            .await?
            .handle_return_error()
            .await?;

        tracing::debug!("updated Revolt status");

        Ok(())
    }

    pub async fn get_status(&self) -> anyhow::Result<Option<String>, RevoltAPIError> {
        trace!("fetching user data from Revolt API (`get_status`)...");

        let response = self
            .client
            .get(format!("{}/users/@me", self.base_url))
            .send()
            .await?
            .handle_return_error()
            .await?;

        let user_data: User = response.json().await?;
        let status = user_data.status.and_then(|status| status.text);

        trace!("successfully fetched the Revolt status");

        Ok(status)
    }

    pub async fn ping(&self) -> anyhow::Result<(), RevoltAPIError> {
        trace!("fetching user data from Revolt API (`ping`)...");

        self.client
            .get(format!("{}/users/@me", self.base_url))
            .send()
            .await?
            .handle_return_error()
            .await?;

        trace!("successfully pinged the Revolt API");

        Ok(())
    }
}

impl From<reqwest::Error> for RevoltAPIError {
    fn from(error: reqwest::Error) -> Self {
        Self::Other(error.into())
    }
}

trait ResponseExt: Sized {
    async fn handle_return_error(self) -> anyhow::Result<Self, RevoltAPIError>;
}

impl ResponseExt for reqwest::Response {
    async fn handle_return_error(self) -> anyhow::Result<Self, RevoltAPIError> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::UNAUTHORIZED => Err(RevoltAPIError::AuthenticationFailed),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = self
                    .headers()
                    .get("X-Ratelimit-Reset-After")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u128>().ok())
                    .unwrap_or(0);

                Err(RevoltAPIError::RateLimitExceeded(retry_after))
            }
            status => Err(RevoltAPIError::Unknown(status)),
        }
    }
}
