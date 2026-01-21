use anyhow::Result;
use enums::Requests;
use log::*;
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::broadcast,
};

const SOCKET_PATH: &str = "/tmp/eww_data/requests.socket";

pub async fn start_request_listener(tx: broadcast::Sender<Requests>) -> Result<()> {
    tokio::fs::create_dir_all("/tmp/eww_data").await.ok();

    if tokio::fs::metadata(SOCKET_PATH).await.is_ok() {
        tokio::fs::remove_file(SOCKET_PATH).await?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("Request listener started on {}", SOCKET_PATH);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                debug!("New client connected to request socket");
                handle_client(stream, tx.clone()).await?;
            }
            Err(e) => {
                error!("Failed to accept request client: {}", e);
            }
        }
    }
}

async fn handle_client(stream: UnixStream, tx: broadcast::Sender<Requests>) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;

    let request: Requests = postcard::from_bytes(&mut buf)?;
    debug!("Sending to broadcast: {:?}", request);
    tx.send(request)?;
    Ok(())
}
