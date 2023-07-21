use std::net::IpAddr;

use eframe::egui;
use rand::random;

use crate::{
    app::workspace_models::WorkspaceContext, utils::constants::DEFAULT_WINDOW_STARTING_POS,
};

#[derive(Debug, Clone, Copy)]
pub struct DeviceWindowState {
    id: egui::Id,
    open: bool,
    ip: IpAddr,
}
impl DeviceWindowState {
    pub fn new(ip: IpAddr) -> Self {
        Self {
            // Using random instead of ip,
            // - because if I fuck up something, and there happen to be two same ips (should never happen) or windows, the window will still work
            // - I want to be able to open multiple windows for the same device at once.
            id: egui::Id::new(random::<u64>()),
            open: false,
            ip,
        }
    }

    pub fn show(&mut self) {
        self.open = true;
    }

    // pub fn hide(&mut self) {
    //     self.open = false;
    // }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }
}

impl DeviceWindowState {
    pub fn render(
        egui_context: &egui::Context,
        app_context: &mut WorkspaceContext,
        device_window_state_index: usize,
    ) {
        let device_window_state = app_context
            .ui_state
            .device_window_states
            .get_mut(device_window_state_index)
            .unwrap(); // safe to unwrap since this index is always valid

        let mut should_show_window_internal = device_window_state.open;
        if !should_show_window_internal {
            return;
        }

        egui::Window::new(format!("Device - {}", device_window_state.ip))
            .id(device_window_state.id)
            .collapsible(true)
            .default_pos(DEFAULT_WINDOW_STARTING_POS)
            .open(&mut should_show_window_internal)
            .show(egui_context, |ui| {
                // ui.vertical_centered(|ui| ui.label("suck my dick"))

                ui.add_space(15.0);
                // if ui.button("Delete").clicked() {
                //     app_context.app_state.network_topology.remove_edges_from_node(from)
                // }
            });

        device_window_state.open &= should_show_window_internal;
    }
}
