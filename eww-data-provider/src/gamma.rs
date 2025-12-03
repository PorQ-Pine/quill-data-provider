use std::time::Duration;

use enums::Requests;
use log::{debug, error, info};
use tokio::{
    process::{Child, Command},
    time::sleep,
};

pub struct GammaListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
    pub child: Option<Child>,
    pub current_gamma: u8,
}

impl GammaListener {
    pub async fn start(&mut self) {
        info!("Starting GammaListener");

        // Initial
        self.toggle_gamma(10).await;

        loop {
            if let Ok(data) = self.channel_rx.recv().await {
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
    }

    pub async fn toggle_gamma(&mut self, gamma_level: u8) {
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
