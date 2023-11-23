use tokio::{signal, sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::ChannelPayload;

pub struct Listener(UnboundedSender<ChannelPayload>);

impl Listener {
    pub const fn new(tx: UnboundedSender<ChannelPayload>) -> Self {
        Self(tx)
    }

    pub async fn listen(self) -> anyhow::Result<()> {
        tracing::debug!("spawning exit signal listener");

        let _: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let ctrl_c = signal::ctrl_c();

            #[cfg(unix)]
            {
                use signal::unix::{signal, SignalKind};

                let mut sigterm = signal(SignalKind::terminate())?;

                tokio::select! {
                    _ = ctrl_c => {},
                    _ = sigterm.recv() => {}
                }
            }

            #[cfg(windows)]
            ctrl_c.await?;

            self.0.send(ChannelPayload::Exit(true))?;

            Ok(())
        });

        tracing::debug!("spawned exit signal listener");

        Ok(())
    }
}
