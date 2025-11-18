use std::time::Duration;

use async_trait::async_trait;
use log::*;
use tokio::process::Command;
use tokio::time::sleep;

use crate::listener::SocketHandler;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BluetoothStatus {
    on: bool,
    name: String,
    signal: String,
}

pub async fn get_bt() -> String {
    let mut status = BluetoothStatus {
        on: false,
        name: "".to_string(),
        signal: "".to_string(),
    };

    let output = Command::new("bluetoothctl")
        .arg("show")
        .output()
        .await
        .expect("Failed to execute bluetoothctl show");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let power_line = stdout.lines().find(|line| line.contains("Powered:"));

    if let Some(line) = power_line {
        if line.contains("yes") {
            status.on = true;
        }
    }

    if !status.on {
        return serde_json::to_string(&status).unwrap();
    }

    let devices_output = Command::new("bluetoothctl")
        .arg("devices")
        .output()
        .await
        .expect("Failed to execute bluetoothctl devices");

    let devices_stdout = String::from_utf8_lossy(&devices_output.stdout);
    let mut connected_mac: Option<String> = None;

    for line in devices_stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let mac = parts[1].to_string();
            let info_output = Command::new("bluetoothctl")
                .arg("info")
                .arg(&mac)
                .output()
                .await
                .expect("Failed to execute bluetoothctl info");

            let info_stdout = String::from_utf8_lossy(&info_output.stdout);
            if info_stdout.contains("Connected: yes") {
                connected_mac = Some(mac);
                break;
            }
        }
    }

    if let Some(mac) = connected_mac {
        let info_output = Command::new("bluetoothctl")
            .arg("info")
            .arg(&mac)
            .output()
            .await
            .expect("Failed to execute bluetoothctl info");

        let info_stdout = String::from_utf8_lossy(&info_output.stdout);

        if let Some(name_line) = info_stdout.lines().find(|line| line.contains("Name:")) {
            status.name = name_line
                .trim_start_matches("Name:")
                .trim()
                .trim_matches('"')
                .to_string();
        }

        if let Some(rssi_line) = info_stdout.lines().find(|line| line.contains("RSSI:")) {
            let parts: Vec<&str> = rssi_line.split_whitespace().collect();
            if parts.len() >= 2 {
                status.signal = parts[1].to_string();
            }
        }
    }

    serde_json::to_string(&status).unwrap()
}

pub struct BluetoothListener;

#[async_trait]
impl SocketHandler for BluetoothListener {
    const SOCKET_NAME: &'static str = "bluetooth";

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
        info!("Starting Bluetooth listener");
        // Idk

        /*
               sleep(Duration::from_secs(5)).await;

               let mut cmd = Command::new("bluetoothctl")
                   .arg("--monitor")
                   .stdout(std::process::Stdio::piped())
                   .spawn()
                   .expect("Failed to spawn bluetoothctl monitor command");

               let stdout = cmd.stdout.take().expect("Failed to take stdout");
               let mut reader = BufReader::new(stdout).lines();
        */

        /*
        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from bluetoothctl monitor")
        {
            debug!("bluetoothctl monitor line: {}", line);
            if line.contains("Powered") || line.contains("Connected") || line.contains("RSSI:") || line.contains("PowerState") {
                info!("Bluetooth event detected: {}", line);
                sleep(Duration::from_millis(200)).await;
                let bt_status = get_bt().await;
                if previous_bt != bt_status {
                    self.send_unix(unix, bt_status.clone()).await;
                    previous_bt = bt_status;
                } else {
                    debug!("Bluetooth info is the same");
                }
            }
        }
        */
        let mut last_bluetooth_line = String::new();
        loop {
            let output = Command::new("rfkill")
                .output()
                .await
                .expect("failed to execute rfkill");
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(bt_line) = output_str.lines().find(|l| l.contains("bluetooth")) {
                if bt_line != last_bluetooth_line {
                    debug!("Bluetooth changed: {}", bt_line);
                    self.send_unix(unix, get_bt().await).await;

                    last_bluetooth_line = bt_line.to_string();
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}
