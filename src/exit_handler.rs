use tokio::{signal, sync::mpsc::UnboundedSender};

use crate::ChannelMessage;

pub fn handle(tx: UnboundedSender<ChannelMessage>) {
    tracing::debug!("spawning exit signal handler");

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

        tx.send(ChannelMessage::Exit(true))
            .expect("CTRL-C handler could not be created");
    });

    tracing::debug!("spawned exit signal handler");
}
