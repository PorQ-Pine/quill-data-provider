use tokio::process::Command;

// Only matters when:
// DriverMode is Fast
// Normal, Y2 and Y1
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y 0
enum Dithering {
    Bayer,       //0
    BlueNoise16, // 1
    BlueNoise32, // 2
}

// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y 0
enum DriverMode {
    Normal(BitDepth), //0
    Fast(Dithering),  // 1
                      // Doesn't work for me
                      // Zero, // 8
}

// RenderHints
// Only matters in Normal mode
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s "Y1|T|R"
enum BitDepth {
    Y1(Conversion),
    Y2(Conversion, Redraw),
    Y4(Redraw),
}

enum Conversion {
    Tresholding,          // T
    Dithering(Dithering), // D
}

enum Redraw {
    FastDrawing,        // R
    DisableFastDrawing, // r
}

enum ScreenOptions {
    // busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh
    FullRefresh,
    ScreenMode(DriverMode),
}

async fn run_cmd(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let out = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub async fn refresh_screen() {
    run_cmd("busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh").await;
}

#[derive(Debug)]
pub struct EwwScreenConfig {
    dithering_bayer: bool,
    dithering_blue_noise16: bool,
    dithering_blue_noise32: bool,
    driver_normal: bool,
    driver_fast: bool,
    bitdepth_y1: bool,
    bitdepth_y2: bool,
    bitdepth_y4: bool,
    conv_tresholding: bool,
    conv_dithering_bayer: bool,
    redraw_fastdrawing: bool,
    redraw_disablefastdrawing: bool,
}

async fn eww_get(string: &str) -> bool {
    let final_line = format!("eww get {}", string);
    let line = run_cmd(&final_line).await;
    line.trim().parse().unwrap()
}

pub async fn get_eww_screen_config() -> EwwScreenConfig {
    EwwScreenConfig {
        dithering_bayer: eww_get("dithering_bayer").await,
        dithering_blue_noise16: eww_get("dithering_bluenoise16").await,
        dithering_blue_noise32: eww_get("dithering_bluenoise32").await,
        driver_normal: eww_get("driver_normal_mode").await,
        driver_fast: eww_get("driver_fast_mode").await,
        bitdepth_y1: eww_get("bitdepth_y1").await,
        bitdepth_y2: eww_get("bitdepth_y2").await,
        bitdepth_y4: eww_get("bitdepth_y4").await,
        conv_tresholding: eww_get("conversion_tresholding").await,
        conv_dithering_bayer: eww_get("conversion_dithering").await,
        redraw_fastdrawing: eww_get("redraw_fast_drawing").await,
        redraw_disablefastdrawing: eww_get("redraw_disabled").await,
    }
}
