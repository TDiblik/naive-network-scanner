use eframe::{egui, epaint::Vec2};

use crate::utils::constants::DEFAULT_WINDOW_STARTING_POS;

pub struct GenericInfoWindowState {
    open: bool,
    title: String,
    text: String,
}
impl GenericInfoWindowState {
    pub fn new(title: &str) -> Self {
        Self {
            open: false,
            title: title.to_string(),
            text: "".to_string(),
        }
    }

    pub fn show(&mut self, text: String) {
        self.open = true;
        self.text = text;
    }

    pub fn hide(&mut self) {
        self.open = false;
        self.text = "".to_owned();
    }
}

impl GenericInfoWindowState {
    pub fn render(egui_context: &egui::Context, generic_state: &mut GenericInfoWindowState) {
        let mut should_show_window_internal = generic_state.open;
        if !should_show_window_internal {
            return;
        }

        egui::Window::new(generic_state.title.as_str())
            .collapsible(false)
            .default_pos(DEFAULT_WINDOW_STARTING_POS)
            .fixed_size(Vec2::new(275.0, 250.0))
            .open(&mut should_show_window_internal)
            .show(egui_context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(generic_state.text.as_str());
                    if ui.button("Ok").clicked() {
                        generic_state.hide();
                    }
                })
            });

        generic_state.open &= should_show_window_internal;
    }
}
