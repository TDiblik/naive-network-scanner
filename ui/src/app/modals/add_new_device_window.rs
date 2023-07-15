use eframe::{
    egui,
    epaint::{Color32, Vec2},
};
use std::{net::IpAddr, str::FromStr};

use crate::{
    app::{network_topology::NetworkTopologyNode, workspace_models::WorkspaceContext},
    utils::{
        constants::{
            ACTION_SPACER, DEFAULT_SPACER, WORKSPACE_WINDOW_HEIGHT, WORKSPACE_WINDOW_WIDTH,
        },
        general::render_validation_err,
    },
};

const ADD_NEW_DEVICE_WINDOW_STARTING_POS: eframe::epaint::Pos2 = eframe::epaint::Pos2 {
    x: WORKSPACE_WINDOW_WIDTH / 2.0 - 150.0,
    y: WORKSPACE_WINDOW_HEIGHT / 2.0 - 150.0,
};

#[derive(Default)]
pub struct AddNewDeviceWindowState {
    pub open: bool,
    pub ip: String,
    pub ip_validation_err: bool,
    pub notes: String,
}

impl AddNewDeviceWindowState {
    pub fn render(egui_context: &egui::Context, app_context: &mut WorkspaceContext) {
        let mut should_show_window = app_context.ui_state.add_new_device_window_state.open;
        if !should_show_window {
            return;
        }

        egui::Window::new("Manually add a new device")
            .collapsible(false)
            .default_pos(ADD_NEW_DEVICE_WINDOW_STARTING_POS)
            .fixed_size(Vec2::new(275.0, 250.0))
            .open(&mut should_show_window)
            .show(egui_context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("IP Address");
                        ui.text_edit_singleline(
                            &mut app_context.ui_state.add_new_device_window_state.ip,
                        );
                    });
                    render_validation_err(
                        ui,
                        app_context
                            .ui_state
                            .add_new_device_window_state
                            .ip_validation_err,
                        "IP is not valid.",
                    );

                    ui.add_space(DEFAULT_SPACER);
                    ui.vertical(|ui| {
                        ui.label("Notes");
                        ui.text_edit_multiline(
                            &mut app_context.ui_state.add_new_device_window_state.notes,
                        );
                    });

                    ui.add_space(ACTION_SPACER);
                    if ui.button("Add").clicked() {
                        if let Ok(new_ip) =
                            IpAddr::from_str(&app_context.ui_state.add_new_device_window_state.ip)
                        {
                            app_context.app_state.network_topology.add_node(
                                NetworkTopologyNode::new(
                                    new_ip,
                                    app_context
                                        .ui_state
                                        .add_new_device_window_state
                                        .notes
                                        .clone(),
                                ),
                                None,
                            );
                            app_context.ui_state.add_new_device_window_state = Default::default();
                        } else {
                            app_context
                                .ui_state
                                .add_new_device_window_state
                                .ip_validation_err = true
                        }
                    }
                });
            });

        app_context.ui_state.add_new_device_window_state.open &= should_show_window;
    }
}
