use rive_http::Client;
use rive_models::{
    data::EditUserData,
    user::{FieldsUser, UserStatus},
};

#[async_trait::async_trait]
pub trait ClientExt {
    async fn set_status(&self, status: Option<String>) -> ();
}

#[async_trait::async_trait]
impl ClientExt for Client {
    async fn set_status(&self, status: Option<String>) {
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
            Err(err) => println!("Revolt API error: {err}"),
        };
    }
}
