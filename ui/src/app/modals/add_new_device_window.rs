use eframe::{egui, epaint::Vec2};
use std::{net::IpAddr, str::FromStr, sync::Arc};

use crate::{
    app::{
        network_topology::{NetworkTopology, NetworkTopologyNode},
        workspace_models::WorkspaceContext,
    },
    utils::{
        constants::{ACTION_SPACER, DEFAULT_SPACER, DEFAULT_WINDOW_STARTING_POS},
        general::render_validation_err,
        icmp::{
            DEFAULT_PING_ENSURED_CONNECTIVITY_CHECKUP_MS,
            DEFAULT_PING_ENSURED_CONNECTIVITY_TIMEOUT_MS,
        },
        ip::ping_ip_list,
    },
};

#[derive(Default)]
pub struct AddNewDeviceWindowState {
    pub open: bool,
    pub ip: String,
    pub ip_validation_err: bool,
    pub hostname: String,
    pub notes: String,
    pub ip_already_exists_err: bool,
    pub ping_after_creation: bool,
}

impl AddNewDeviceWindowState {
    pub fn render(egui_context: &egui::Context, app_context: &mut WorkspaceContext) {
        let mut should_show_window = app_context.ui_state.add_new_device_window_state.open;
        if !should_show_window {
            return;
        }

        egui::Window::new("Manually add a new device")
            .collapsible(false)
            .default_pos(DEFAULT_WINDOW_STARTING_POS)
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
                    ui.horizontal(|ui| {
                        ui.label("Hostname (optional)");
                        ui.text_edit_singleline(
                            &mut app_context.ui_state.add_new_device_window_state.hostname,
                        );
                    });

                    ui.add_space(DEFAULT_SPACER);
                    ui.vertical(|ui| {
                        ui.label("Notes (optional)");
                        ui.text_edit_multiline(
                            &mut app_context.ui_state.add_new_device_window_state.notes,
                        );
                    });

                    render_validation_err(
                        ui,
                        app_context
                            .ui_state
                            .add_new_device_window_state
                            .ip_already_exists_err,
                        "IP already exists as a node.",
                    );

                    ui.add_space(DEFAULT_SPACER);
                    ui.checkbox(
                        &mut app_context
                            .ui_state
                            .add_new_device_window_state
                            .ping_after_creation,
                        "Send ping after creation",
                    );

                    ui.add_space(ACTION_SPACER);
                    if ui.button("Add").clicked() {
                        app_context
                            .ui_state
                            .add_new_device_window_state
                            .ip_validation_err = false;
                        app_context
                            .ui_state
                            .add_new_device_window_state
                            .ip_already_exists_err = false;
                        if let Ok(new_ip) =
                            IpAddr::from_str(&app_context.ui_state.add_new_device_window_state.ip)
                        {
                            if NetworkTopology::add_node(
                                &mut app_context.app_state.network_topology.graph,
                                NetworkTopologyNode::new(
                                    new_ip,
                                    app_context
                                        .ui_state
                                        .add_new_device_window_state
                                        .notes
                                        .clone(),
                                    Some(
                                        app_context
                                            .ui_state
                                            .add_new_device_window_state
                                            .hostname
                                            .clone(),
                                    ),
                                ),
                                None,
                            )
                            .is_none()
                            {
                                app_context
                                    .ui_state
                                    .add_new_device_window_state
                                    .ip_already_exists_err = true;
                                return;
                            }
                            if app_context
                                .ui_state
                                .add_new_device_window_state
                                .ping_after_creation
                            {
                                ping_ip_list(
                                    Arc::clone(&app_context.app_state.network_topology.graph),
                                    Arc::clone(&app_context.app_state.status_info),
                                    vec![new_ip],
                                    DEFAULT_PING_ENSURED_CONNECTIVITY_TIMEOUT_MS,
                                    DEFAULT_PING_ENSURED_CONNECTIVITY_CHECKUP_MS,
                                    false,
                                    false,
                                );
                            }
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
