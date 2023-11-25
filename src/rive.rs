use rive_http::Client;
use rive_models::{
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};

#[async_trait::async_trait]
pub trait ClientExt {
    async fn ping(&self) -> anyhow::Result<(), anyhow::Error>;
    async fn set_status(&self, status: Option<String>) -> ();
}

#[async_trait::async_trait]
impl ClientExt for Client {
    async fn ping(&self) -> anyhow::Result<(), anyhow::Error> {
        tracing::debug!("pinging Revolt API");

        let result = match self.fetch_self().await {
            Ok(_) => Ok(()),
            Err(error) => {
                if error.to_string().contains("Unauthenticated") {
                    anyhow::bail!("provided session token is not valid");
                }

                anyhow::bail!("an unexpected API error occurred: {error}");
            }
        };
        tracing::debug!("pinging Revolt API completed");

        result
    }

    async fn set_status(&self, status: Option<String>) {
        tracing::info!("updating Revolt status to {:?}", &status);

        let data = status.map_or(
            EditUserData {
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

        match self.edit_user(&data).await {
            Ok(_) => (),
            Err(err) => tracing::error!("Revolt API error: {err}"),
        };

        tracing::info!("updated Revolt status");
    }
}
