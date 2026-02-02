use std::{
    process::Command,
    sync::mpsc::{self, Receiver},
    thread::{self, sleep},
    time::Duration,
};

use eframe::egui;
use enum2egui::GuiInspect;
use quill_data_provider_lib::{EinkWindowSetting, load_window_settings};

use crate::style::style;

mod style;

fn save_settings(settings: &Vec<EinkWindowSetting>) -> Result<(), Box<dyn std::error::Error>> {
    for (i, set) in settings.iter().enumerate() {
        if set.app_id.is_empty() {
            return Err(format!("Id at index {} is empty", i).into());
        }
    }
    let home = std::env::var("HOME")?;
    let dir = format!("{}/.config/eink_window_settings", home);
    std::fs::create_dir_all(&dir)?;
    let path = std::path::Path::new(&dir).join("config.ron");
    let ron = ron::ser::to_string_pretty(settings, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, ron)?;
    Ok(())
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("EinkWindowSettings"),
        ..Default::default()
    };
    let (tx, rx) = mpsc::channel::<Vec<String>>();
    thread::spawn(move || {
        loop {
            let output = Command::new("niri")
                .args(["msg", "windows"])
                .output()
                .expect("failed to run command");
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            let mut windows: Vec<String> = Vec::new();
            for line in stdout.lines() {
                if line.contains("App ID: ") {
                    let mut line_mut = line.to_string().clone();
                    line_mut = line_mut.replace("App ID: \"", "");
                    line_mut.pop();
                    if line_mut.contains("App ID: (unset") || windows.contains(&line_mut) {
                        continue;
                    }
                    windows.push(line_mut);
                }
            }
            tx.send(windows).ok();
            sleep(Duration::from_secs(1));
        }
    });

    // So it opens the ones in the repo here. Yes, it does not support arm mac
    #[cfg(target_arch = "x86_64")]
    let path = "default/config.ron";
    #[cfg(not(target_arch = "x86_64"))]
    {
        let user = std::env::var("USER").unwrap_or_default();
        let path = format!("/home/{}/.config/eink_window_settings/config.ron", user);
    }

    println!("Path for settings is: {}", path);
    let settings = load_window_settings(path.to_string());

    let app = MyApp {
        settings: settings,
        windows_rx: rx,
        windows: Vec::new(),
        zoom_factor: 1.2,
        error: None,
    };

    eframe::run_native(
        "Eink window settings",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_zoom_factor(1.2);
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            cc.egui_ctx.set_style(style());

            Ok(Box::new(app))
        }),
    )
}

struct MyApp {
    settings: Vec<EinkWindowSetting>,
    windows_rx: Receiver<Vec<String>>,
    windows: Vec<String>,
    zoom_factor: f32,
    error: Option<String>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Save").clicked() {
                    self.error = save_settings(&self.settings).err().map(|e| e.to_string());
                }
                if ui.button("Zoom in").clicked() {
                    self.zoom_factor *= 1.2;
                    ctx.set_zoom_factor(self.zoom_factor);
                    ctx.request_repaint();
                }
                if ui.button("Zoom out").clicked() {
                    self.zoom_factor /= 1.2;
                    ctx.set_zoom_factor(self.zoom_factor);
                    ctx.request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            if let Ok(windows) = self.windows_rx.try_recv() {
                self.windows = windows;
            }
            ui.label(format!("Current window ID's:\n{}", self.windows.join("\n")));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                // .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.heading("Eink window settings");
                    ui.label("Values which are global, but will be set for the currently focused window (or defaults if not apply, for the focused window):
- Treshold level
- Dithering type
- Redraw delay
- Fast mode");
                    self.settings.ui_mut(ui);
                });
        });

        if let Some(err) = self.error.take() {
            let mut keep = true;

            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&err);
                    if ui.button("OK").clicked() {
                        keep = false;
                    }
                });

            if keep {
                self.error = Some(err);
            }
        }
    }
}
