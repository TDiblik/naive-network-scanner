use eframe::egui;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use uuid::Uuid;

#[derive(Default)]
pub struct ProjectSelectorWindoState {
    projects: Vec<ProjectRow>,
}

pub struct ProjectRow {
    id: Uuid,
    name: String,
}

impl eframe::App for ProjectSelectorWindoState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Project selector");
            });
        });
    }
}
