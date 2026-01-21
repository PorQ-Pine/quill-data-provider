use std::time::Duration;
use enums::Requests;
use log::{debug, error, info};
use tokio::{
    process::{Child, Command},
    time::sleep,
};

pub const DEFAULT_GAMMA: u8 = 10;

pub enum GammaControl {
    Force(u8),
    PreviousValue,
}

pub struct GammaListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
    pub internal_channel_rx: tokio::sync::mpsc::Receiver<GammaControl>,
    pub child: Option<Child>,
    pub current_gamma: u8,
}

impl GammaListener {
    pub async fn start(&mut self) {
        info!("Starting GammaListener");

        // Initial
        self.toggle_gamma(10).await;

        loop {
            tokio::select! {
                res = self.channel_rx.recv() => {
                    self.handle_channel_rx(res).await;
                }
                Some(gamma_value) = self.internal_channel_rx.recv() => {
                    match gamma_value {
                        GammaControl::Force(value) => {
                            let previous = self.current_gamma;
                            self.handle_internal_channel_rx(value).await;
                            self.current_gamma = previous;
                        },
                        GammaControl::PreviousValue => {
                            self.handle_internal_channel_rx(self.current_gamma).await
                        },
                    }
                }
            }
        }
    }

    pub async fn handle_internal_channel_rx(&mut self, gamma_value: u8) {
        self.toggle_gamma(gamma_value).await;
    }

    pub async fn handle_channel_rx(
        &mut self,
        res: Result<Requests, tokio::sync::broadcast::error::RecvError>,
    ) {
        if let Ok(data) = res {
            match data {
                Requests::GammaLevel => {
                    let res = Command::new("eww")
                        .arg("--no-daemonize")
                        .arg("get")
                        .arg("thresholding_level_value")
                        .output()
                        .await;
                    if let Ok(value) = &res {
                        let vec = &value.stdout;
                        let str = String::from_utf8_lossy(&vec).to_string();
                        if let Ok(gamma_level) = str.trim().parse() {
                            self.toggle_gamma(gamma_level).await;
                        } else {
                            error!("Failed to parse gamma level: {:?}", res);
                        }
                    } else {
                        error!("Failed to get thresholding_level_value");
                    }
                    sleep(Duration::from_millis(25)).await;
                }
                _ => {}
            }
        }
    }

    pub async fn toggle_gamma(&mut self, gamma_level: u8) {
        // Not working yet ;/
        return;
        
        if self.current_gamma == gamma_level {
            return;
        }
        if let Some(mut child) = self.child.take() {
            if child.kill().await.is_err() {
                Command::new("killall")
                    .arg("-9")
                    .arg("gammastep")
                    .status()
                    .await
                    .ok();
            };
        }

        let f = gamma_level as f32 / 10.0;
        debug!("Setting gamma level to: {}", f);
        let child = Command::new("gammastep")
            .arg("-l")
            .arg("0.0:0.0")
            .arg("-o")
            .arg("-g")
            .arg(format!("{:.1}:{:.1}:{:.1}", f, f, f))
            .spawn()
            .expect("failed to start gammastep");

        self.child.replace(child);
        self.current_gamma = gamma_level;
    }
}
