#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod project_selector;
mod utils;
use eframe::egui;
use log::debug;

use crate::project_selector::ProjectSelectorWindoState;

fn main() -> anyhow::Result<()> {
    utils::env::init()?;
    utils::logging::init()?;
    // TODO: Perform runtime checks

    // Open project selector
    eframe::run_native(
        "TEEF - Project selector",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(550.0, 440.0)),
            resizable: false,
            ..Default::default()
        },
        Box::new(|_cc| Box::<ProjectSelectorWindoState>::default()),
    )
    .expect("Unable to render project selector.");

    // Open selected project

    Ok(())
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
