use tokio::{signal, sync::mpsc::UnboundedSender};

use crate::ChannelPayload;

pub struct Handler(UnboundedSender<ChannelPayload>);

impl Handler {
    pub const fn new(tx: UnboundedSender<ChannelPayload>) -> Self {
        Self(tx)
    }

    pub async fn handle(self) {
        tokio::spawn(async move {
            let ctrl_c = signal::ctrl_c();

            #[cfg(unix)]
            {
                use signal::unix::{signal, SignalKind};

                let mut sigterm =
                    signal(SignalKind::terminate()).expect("SIGTERM handler could not be created");

                tokio::select! {
                    _ = ctrl_c => {},
                    _ = sigterm.recv() => {}
                }
            }

            #[cfg(windows)]
            ctrl_c.await.expect("CTRL-C handler could not be created");

            self.0
                .send(ChannelPayload::Exit(true))
                .map_err(|err| tracing::error!("{err}"))
        });

        tracing::debug!("spawned exit signal handler");
    }
}
