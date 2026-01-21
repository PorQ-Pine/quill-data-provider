use data_provider_lib::DriverMode;
use eframe::egui;
use enum2egui::{Gui, GuiInspect};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Eink window settings",
        options,
        Box::new(|_cc| {
            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Clone, Debug, PartialEq, Gui, Default)]
struct EinkWindowSetting {
    app_id: String,
    settings: DriverMode,
}

struct MyApp {
    settings: Vec<EinkWindowSetting>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Eink window settings");
            self.settings.ui_mut(ui);
        });
    }
}
