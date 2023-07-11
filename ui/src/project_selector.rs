use eframe::egui;

#[derive(Default)]
pub struct ProjectSelector {}

impl eframe::App for ProjectSelector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Project selector");
            });
        });
    }
}
