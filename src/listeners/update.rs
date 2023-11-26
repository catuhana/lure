use rive_http::Client as RiveClient;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{config::StatusOptions, platforms::Track, rive::ClientExt, ChannelPayload};

pub struct Listener(UnboundedReceiver<ChannelPayload>);

impl Listener {
    pub const fn new(rx: UnboundedReceiver<ChannelPayload>) -> Self {
        Self(rx)
    }

    pub async fn listen(mut self, rive_client: RiveClient, status: StatusOptions) {
        tracing::debug!("spawning update listener");

        let mut previous_track: Option<Track> = None;
        while let Some(payload) = self.0.recv().await {
            match payload {
                ChannelPayload::Data(track) => {
                    if previous_track == track {
                        continue;
                    };

                    let status = track.as_ref().map_or_else(
                        || status.idle.clone(),
                        |track| {
                            Some(
                                status
                                    .template
                                    .replace("%ARTIST%", &track.artist)
                                    .replace("%NAME%", &track.name),
                            )
                        },
                    );

                    rive_client.set_status(status).await;
                    previous_track = track;
                }
                ChannelPayload::Exit(reset_status) => {
                    tracing::info!("stopping lure");

                    if reset_status {
                        rive_client.set_status(None).await;
                    }

                    break;
                }
            }
        }
    }
}
