use eframe::egui;
use enum2egui::{Gui, GuiInspect};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

// The mess of connected enums is so we know what affects when, so:
// - We can set only what's needed
// - We show only what can be changed

// Only matters when:
// DriverMode is Fast
// Normal, Y2 and Y1
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y 0
#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum Dithering {
    #[default]
    Bayer, // 0
    BlueNoise16, // 1
    BlueNoise32, // 2
}

impl std::fmt::Display for Dithering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y 0
#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum DriverMode {
    Normal(#[enum2egui(label = "Bit depth")] BitDepth), // 0
    Fast(#[enum2egui(label = "Dithering type")] Dithering), // 1
                                                        // Doesn't work for me
                                                        // Zero, // 8
}

impl std::fmt::Display for DriverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for DriverMode {
    fn default() -> Self {
        DriverMode::Normal(BitDepth::Y2(
            Conversion::Tresholding,
            Redraw::DisableFastDrawing,
        ))
    }
}

// RenderHints
// Only matters in Normal mode
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s "Y1|T|R"
#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum BitDepth {
    Y1(#[enum2egui(label = "Conversion")] Conversion, TresholdLevel),
    Y2(
        #[enum2egui(label = "Conversion")] Conversion,
        #[enum2egui(label = "Fast redraw")] Redraw,
    ),
    Y4(#[enum2egui(label = "Fast redraw")] Redraw),
}

impl Default for BitDepth {
    fn default() -> Self {
        BitDepth::Y2(Conversion::default(), Redraw::default())
    }
}

impl std::fmt::Display for BitDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum Conversion {
    Tresholding, // T, + level
    Dithering(#[enum2egui(label = "Dithering type")] Dithering),       // D
}

impl Default for Conversion {
    fn default() -> Self {
        Conversion::Tresholding
    }
}

impl std::fmt::Display for Conversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// So the configurator works well...
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum TresholdLevel {
    _2,
    _3,
    _4,
    _5,
    _6,
    #[default]
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
}

impl TresholdLevel {
    pub async fn set(&self) {
        /*
        if level < 2 || level > 15 {
            error!("Wrong treshold level");
        }
        */
        let level: u8 = match self {
            TresholdLevel::_2 => 2,
            TresholdLevel::_3 => 3,
            TresholdLevel::_4 => 4,
            TresholdLevel::_5 => 5,
            TresholdLevel::_6 => 6,
            TresholdLevel::_7 => 7,
            TresholdLevel::_8 => 8,
            TresholdLevel::_9 => 9,
            TresholdLevel::_10 => 10,
            TresholdLevel::_11 => 11,
            TresholdLevel::_12 => 12,
            TresholdLevel::_13 => 13,
            TresholdLevel::_14 => 14,
            TresholdLevel::_15 => 15,
        };

        if let Err(e) = tokio::fs::write(
            "/sys/module/rockchip_ebc_blit_neon/parameters/y4_threshold_y1",
            level.to_string(),
        )
        .await
        {
            error!("Failed to set threshold: {}", e);
        }
    }

    /*
    pub async fn set_tresholding_level(level: u8, show_gui: bool) {
        let converted = 2 + ((level.saturating_sub(1) as f32 / 99.0) * 13.0).round() as u8;
        Self::set_tresholding_level_internal(converted).await;

        if show_gui {
            run_cmd(&format!(
                "eww --no-daemonize update thresholding_level_value_real={}",
                converted
            ))
            .await;
        }
    }
    */
}

impl std::fmt::Display for TresholdLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum Redraw {
    FastDrawing(#[enum2egui(label = "")] RedrawOptions), // R
    #[default]
    DisableFastDrawing,                    // r
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub struct RedrawOptions {
    #[enum2egui(label = "\nRedraw delay (10-300 is reasonable)")]
    delay: u16,
}

impl RedrawOptions {
    pub async fn set(&self) {
        run_cmd(&format!("busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 RedrawDelay q {}", self.delay)).await;
    }
}

impl Default for RedrawOptions {
    fn default() -> Self {
        Self { delay: 25 }
    }
}

impl std::fmt::Display for Redraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Impl
impl DriverMode {
    pub async fn set(&self) {
        let string = match self {
            DriverMode::Normal(_bit_depth) => "0",
            DriverMode::Fast(_dithering) => "1",
        };
        let line = format!(
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y {}",
            string
        );
        run_cmd(&line).await;
    }
}

impl Dithering {
    pub async fn set(&self) {
        let string = match self {
            Dithering::Bayer => "0",
            Dithering::BlueNoise16 => "1",
            Dithering::BlueNoise32 => "2",
        };
        let line = format!(
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y {}",
            string
        );
        run_cmd(&line).await;
    }
}

pub async fn run_cmd(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    debug!("Running run_cmd as: {} {:?}", parts[0], &parts[1..]);
    let out = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[derive(Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub struct EinkWindowSetting {
    pub app_id: String,
    pub settings: DriverMode,
}

pub fn load_settings(user: String) -> Result<Vec<EinkWindowSetting>, Box<dyn std::error::Error>> {
    let path = format!("/home/{}/.config/eink_window_settings/config.ron", user);
    let contents = std::fs::read_to_string(path)?;
    let settings: Vec<EinkWindowSetting> = ron::from_str(&contents)?;
    Ok(settings)
}
