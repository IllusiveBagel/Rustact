use std::time::Duration;

use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::events::{FrameworkEvent, is_ctrl_c, map_terminal_event};

use super::dispatcher::AppMessage;

pub fn spawn_terminal_events(tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut events = EventStream::new();
        while let Some(event) = events.next().await {
            match event {
                Ok(evt) => {
                    if let Some(mapped) = map_terminal_event(evt) {
                        let shutdown = is_ctrl_c(&mapped);
                        if tx.send(AppMessage::ExternalEvent(mapped)).await.is_err() {
                            break;
                        }
                        if shutdown {
                            let _ = tx.send(AppMessage::Shutdown).await;
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    })
}

pub fn spawn_tick_loop(tx: mpsc::Sender<AppMessage>, rate: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(rate);
        loop {
            ticker.tick().await;
            if tx
                .send(AppMessage::ExternalEvent(FrameworkEvent::Tick))
                .await
                .is_err()
            {
                break;
            }
        }
    })
}

pub fn spawn_shutdown_watcher(tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            let _ = tx.send(AppMessage::Shutdown).await;
        }
    })
}
