// No rotation support yet, so the only reason this exist is for rotation support

use log::warn;
use tokio::process::Command;

const COMMAND: &str = r#"
lisgd -d /dev/input/by-path/platform-fe5e0000.i2c-event \
      -w 1872 -h 1404 \
      -g "2,LR,*,M,R,niri msg action focus-column-right" \
      -g "2,RL,*,M,R,niri msg action focus-column-left"
"#;

#[derive(Default)]
pub struct GesturesManager {
    // child: Option<tokio::process::Child>,
}

impl GesturesManager {
    pub async fn start(&mut self) {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(COMMAND)
            .spawn()
            .expect("Failed to spawn command");
        // self.child = Some(child);
        if let Ok(status) = child.wait().await {
            if !status.success() {
                warn!("Gestures exited and failed?");
            }
        }
    }
}

