use std::time::Duration;

use crate::listener::SocketHandler;
use async_trait::async_trait;
use enums::Requests;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::{process::Command, time::sleep};

pub struct VirtualKeyboardListener {
    pub channel: tokio::sync::broadcast::Receiver<Requests>,
}

impl VirtualKeyboardListener {
    pub async fn start(&mut self) {
        info!("Starting DunstListener");
        loop {
            if let Ok(data) = self.channel.recv().await {
                if data == Requests::VirtualKeyboard {
                    let mut tries = 1;
                    while tries < 7 {
                        let should_be = !Self::is_visible().await;
                        Self::set_visible(should_be).await;
                        sleep(Duration::from_millis(25 * tries)).await;
                        if should_be == Self::is_visible().await {
                            info!("Keyboard set as it should!");
                            break;
                        } else {
                            warn!("Keyboard did not listen, retrying...");
                        }
                        tries += 1;
                    }
                    if tries >= 7 {
                        error!("Keyboard did not respond");
                    }
                }
            } else {
                error!("Failed to recv");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn is_visible() -> bool {
        let output = Command::new("busctl")
            .args(&[
                "get-property",
                "--user",
                "sm.puri.OSK0",
                "/sm/puri/OSK0",
                "sm.puri.OSK0",
                "Visible",
            ])
            .output()
            .await
            .expect("failed to execute command");

        let result = String::from_utf8_lossy(&output.stdout);
        result.trim().split_whitespace().nth(1) == Some("true")
    }

    async fn set_visible(visible: bool) {
        let value = if visible { "true" } else { "false" };
        Command::new("busctl")
            .args(&[
                "call",
                "--user",
                "sm.puri.OSK0",
                "/sm/puri/OSK0",
                "sm.puri.OSK0",
                "SetVisible",
                "b",
                value,
            ])
            .status()
            .await
            .expect("failed to execute command");
    }
}
