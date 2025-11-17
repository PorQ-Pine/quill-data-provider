// src/network.rs
use async_trait::async_trait;
use log::*;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use serde::{Deserialize, Serialize};

use crate::listener::SocketHandler;

#[derive(Debug, Serialize, Deserialize)]
struct NetworkInfo {
    essid: String,
    signal: String,
}

async fn get_network_info() -> String {
    let mut essid = String::new();
    let mut signal = String::new();

    // Get signal
    let nmcli_wifi_output = Command::new("nmcli")
        .arg("-f")
        .arg("in-use,signal")
        .arg("dev")
        .arg("wifi")
        .output()
        .await
        .expect("Failed to execute nmcli dev wifi");

    let nmcli_wifi_stdout = String::from_utf8_lossy(&nmcli_wifi_output.stdout);
    if let Some(line) = nmcli_wifi_stdout.lines().find(|l| l.contains('*')) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            signal = parts[1].to_string();
        }
    }

    // Get ESSID
    let nmcli_conn_output = Command::new("nmcli")
        .arg("-t")
        .arg("-f")
        .arg("NAME")
        .arg("connection")
        .arg("show")
        .arg("--active")
        .output()
        .await
        .expect("Failed to execute nmcli connection show --active");

    let nmcli_conn_stdout = String::from_utf8_lossy(&nmcli_conn_output.stdout);
    if let Some(line) = nmcli_conn_stdout.lines().next() {
        essid = line.trim().trim_matches('"').to_string();
    }

    let network_info = NetworkInfo { essid, signal };
    serde_json::to_string(&network_info).unwrap_or_else(|e| {
        error!("Failed to serialize network info: {}", e);
        "{\"essid\": \"\", \"signal\": \"\"}".to_string()
    })
}

pub struct NetworkListener;

#[async_trait]
impl SocketHandler for NetworkListener {
    const SOCKET_NAME: &'static str = "network";

    async fn start(&self, unix: &mut tokio::net::UnixStream) {
        info!("Starting NetworkListener");

        let mut previous_network_info = get_network_info().await;
        self.send_unix(unix, previous_network_info.clone()).await;

        let mut cmd = Command::new("ip")
            .arg("monitor")
            .arg("link")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn ip monitor link command");

        let stdout = cmd.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from ip monitor link")
        {
            // debug!("ip monitor link line: {}", line);
            // Re-fetch network info on any link change
            let current_network_info = get_network_info().await;
            if previous_network_info != current_network_info {
                self.send_unix(unix, current_network_info.clone()).await;
                previous_network_info = current_network_info;
            } else {
                // debug!("Network info is the same");
            }
        }
    }
}
