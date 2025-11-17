use async_trait::async_trait;
use log::*;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::listener::SocketHandler;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerctlMetadata {
    name: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    art_url: Option<String>,
    status: Option<String>,
    length: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerOutput {
    name: String,
    title: String,
    artist: String,
    art_url: String,
    status: String,
    length: String,
    length_str: String,
}

async fn process_player_metadata(raw_json: &str) -> String {
    let raw_metadata: PlayerctlMetadata =
        serde_json::from_str(raw_json).unwrap_or_else(|e| {
            error!("Failed to parse playerctl JSON: {} - {}", e, raw_json);
            PlayerctlMetadata {
                name: None,
                title: None,
                artist: None,
                art_url: None,
                status: None,
                length: None,
            }
        });

    let name = raw_metadata.name.unwrap_or_default();
    let title = raw_metadata.title.unwrap_or_default();
    let artist = raw_metadata.artist.unwrap_or_default();
    let mut art_url = raw_metadata.art_url.unwrap_or_default();
    let status = raw_metadata.status.unwrap_or_default();
    let length_us: Option<u64> = raw_metadata.length.and_then(|s| s.parse::<u64>().ok());

    let length = if let Some(l) = length_us {
        ((l + 500_000) / 1_000_000).to_string()
    } else {
        "".to_string()
    };

    if art_url.starts_with("file://") {
        art_url = art_url.strip_prefix("file://").unwrap_or(&art_url).to_string();
    }

    let length_str = if let Some(l_us) = length_us {
        let total_seconds = l_us / 1_000_000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    } else {
        "".to_string()
    };

    let final_output = PlayerOutput {
        name,
        title,
        artist,
        art_url,
        status,
        length,
        length_str,
    };

    serde_json::to_string(&final_output).unwrap_or_else(|e| {
        error!("Failed to serialize player output: {}", e);
        "{}".to_string()
    })
}

pub struct PlayerListener;

#[async_trait]
impl SocketHandler for PlayerListener {
    const SOCKET_NAME: &'static str = "player";

    async fn start(&self, unix: &mut tokio::net::UnixStream) {
        info!("Starting PlayerListener");

        let mut cmd = Command::new("playerctl")
            .arg("metadata")
            .arg("-F")
            .arg("-f")
            .arg(r#"{"name":"{{playerName}}","title":"{{title}}","artist":"{{artist}}","artUrl":"{{mpris:artUrl}}","status":"{{status}}","length":"{{mpris:length}}"}"#)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn playerctl monitor command");

        let stdout = cmd.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout).lines();

        let initial_line = reader
            .next_line()
            .await
            .expect("Failed to read initial line from playerctl monitor")
            .expect("playerctl monitor did not output an initial line");
        let initial_state = process_player_metadata(&initial_line).await;
        self.send_unix(unix, initial_state.clone()).await;

        let mut previous_player_info = initial_state;

        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from playerctl monitor")
        {
            let current_player_info = process_player_metadata(&line).await;
            if previous_player_info != current_player_info {
                self.send_unix(unix, current_player_info.clone()).await;
                previous_player_info = current_player_info;
            } else {
                debug!("Player info is the same");
            }
        }
    }
}
