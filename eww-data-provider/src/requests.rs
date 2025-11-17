use enums::Requests;
use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
    sync::broadcast,
};
use anyhow::Result;
use log::*;

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


async fn handle_client(mut stream: UnixStream, tx: broadcast::Sender<Requests>) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    let request: Requests = postcard::from_bytes_cobs(&mut buf)?;
    tx.send(request)?;
    Ok(())
}
