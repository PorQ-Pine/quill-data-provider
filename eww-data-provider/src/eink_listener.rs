use enums::Requests;
use log::{debug, error, info};
use std::time::Duration;
use tokio::time::sleep;

use crate::eink::{
    eww_screen_config_to_enum, get_eww_screen_config, refresh_screen, set_screen_settings, DEFAULT_TRESHOLDING_LEVEL,
};

pub struct EinkListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
    // pub gamma_channel_tx: tokio::sync::mpsc::Sender<GammaControl>,
}

impl EinkListener {
    pub async fn start(&mut self) {
        info!("Starting EinkListener");
        debug!("Setting initial settings");
        // Perfect defaults, middle ground between speed and look
        set_screen_settings(
            crate::eink::DriverMode::Normal(crate::eink::BitDepth::Y2(
                crate::eink::Conversion::Tresholding(DEFAULT_TRESHOLDING_LEVEL),
                crate::eink::Redraw::FastDrawing(25),
            )),
            // &mut self.gamma_channel_tx,
        )
        .await;

        async fn screen_settings_call(_quick: bool) {
            debug!("Got screen settings");
            let screen_settings = get_eww_screen_config().await;
            debug!("Screen settings: {:?}", screen_settings);
            let enum_screen_settings = eww_screen_config_to_enum(&screen_settings).await;
            debug!("Enum screen settings: {:#?}", enum_screen_settings);
            set_screen_settings(
                enum_screen_settings,
                // &mut self.gamma_channel_tx
                // quick
            )
            .await;
        }

        loop {
            if let Ok(data) = self.channel_rx.recv().await {
                match data {
                    Requests::ScreenRefresh => {
                        refresh_screen().await;
                    }
                    Requests::ScreenSettings => {
                        screen_settings_call(false).await;
                    }
                    Requests::SmallScreenSettings => {
                        screen_settings_call(true).await;
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
