use crate::listener::SocketHandler;
use async_trait::async_trait;
use log::*;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub struct VolumeListener;

#[async_trait]
impl SocketHandler for VolumeListener {
    const SOCKET_NAME: &'static str = "volume";

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
        info!("Starting VolumeListener");

        async fn get_current_volume() -> String {
            let output = Command::new("pamixer")
                .arg("--get-volume-human")
                .output()
                .await
                .expect("Failed to execute pamixer command");
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_end_matches('%')
                .to_string()
        }

        let mut previous_volume = get_current_volume().await;
        self.send_unix(unix, previous_volume.clone()).await;

        let mut cmd = Command::new("pactl")
            .arg("subscribe")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn pactl subscribe command");

        let stdout = cmd.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from pactl subscribe")
        {
            if line.contains("on sink") {
                // info!("Volume change event detected");
                let current_volume = get_current_volume().await;
                if previous_volume != current_volume {
                    self.send_unix(unix, current_volume.clone()).await;
                    previous_volume = current_volume;
                } else {
                    debug!("Volume is the same");
                }
            }
        }
    }
}
