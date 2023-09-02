use rive_http::Client;
use rive_models::{
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};

#[async_trait::async_trait]
pub trait ClientExt {
    async fn ping(&self) -> Option<()>;
    async fn set_status(&self, status: Option<String>) -> ();
}

#[async_trait::async_trait]
impl ClientExt for Client {
    async fn ping(&self) -> Option<()> {
        tracing::debug!("pinging Revolt API");

        match self.fetch_self().await {
            Ok(_) => Some(()),
            Err(error) => {
                if error.to_string().contains("Unauthenticated") {
                    tracing::error!("provided session token is not valid")
                } else {
                    tracing::error!("an unexpected API error ocurred: {error}")
                }

                None
            }
        }
    }

    async fn set_status(&self, status: Option<String>) {
        tracing::info!("updating revolt status to {:?}", &status);

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

        match self.edit_user(data).await {
            Ok(_) => (),
            Err(err) => tracing::error!("Revolt API error: {err}"),
        };
    }
}
