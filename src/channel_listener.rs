use rive_http::Client as RiveClient;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{config::StatusOptions, platforms::Track, rive::ClientExt, ChannelMessage};

pub async fn listen(
    mut rx: UnboundedReceiver<ChannelMessage>,
    rive_client: RiveClient,
    status: StatusOptions,
) {
    tracing::debug!("spawning update listener");

    let mut previous_track: Option<Track> = None;
    while let Some(payload) = rx.recv().await {
        match payload {
            ChannelMessage::Track(track) => {
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
            ChannelMessage::Exit(reset_status) => {
                tracing::info!("stopping lure");

                if reset_status {
                    rive_client.set_status(None).await;
                }

                break;
            }
        }
    }
}
