use eframe::egui;

pub struct FileMenuButton {}

impl FileMenuButton {
    pub fn render(ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            ui.label("TODO: Implement project/state saving");
        });
    }
}
