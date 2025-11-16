pub mod listener;
pub mod consts;
pub mod battery;

use listener::SocketHandler;
use battery::{BatteryStateListener, BatteryPercentListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let battery_state = BatteryStateListener;
    tokio::spawn(async move {
        battery_state.start(battery_state.open_socket().await).await;
    });

    let battery_percent = BatteryPercentListener;
    tokio::spawn(async move {
        battery_percent.start(battery_percent.open_socket().await).await;
    });

    tokio::signal::ctrl_c().await.expect("failed to listen for event");
    Ok(())
}
