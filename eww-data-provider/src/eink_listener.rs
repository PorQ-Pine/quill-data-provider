use enums::Requests;
use log::{debug, error, info, warn};
use std::time::{Duration, Instant};
use tokio::{process::Command, time::sleep};

use crate::eink::{get_eww_screen_config, refresh_screen};

pub struct EinkListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
}

impl EinkListener {
    pub async fn start(&mut self) {
        info!("Starting EinkListener");
        loop {
            if let Ok(data) = self.channel_rx.recv().await {
                match data {
                    Requests::ScreenRefresh => {
                        refresh_screen().await;
                    }
                    Requests::ScreenSettings => {
                        debug!("Got screen settings");
                        let screen_settings = get_eww_screen_config().await;
                        debug!("Screen settings: {:?}", screen_settings);
                    }
                    _ => {}
                }
            } else {
                error!("Failed to recv");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
