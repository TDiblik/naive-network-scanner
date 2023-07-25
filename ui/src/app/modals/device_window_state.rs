use std::{net::IpAddr, sync::Arc};

use eframe::{egui, epaint::Vec2};
use petgraph::stable_graph::NodeIndex;
use rand::random;

use crate::{
    app::{network_topology::NetworkTopology, workspace_models::WorkspaceContext},
    utils::{
        constants::{ACTION_SPACER, DEFAULT_SPACER, DEFAULT_WINDOW_STARTING_POS},
        ip::update_hostname_list,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum SubWindowType {
    Info,
    Ports,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceWindowState {
    window_id: egui::Id,
    open: bool,
    ip: IpAddr,
    node_index: NodeIndex,
    subwindow_selected: SubWindowType,
}
impl DeviceWindowState {
    pub fn new(ip: IpAddr, node_index: NodeIndex) -> Self {
        Self {
            // Using random instead of ip,
            // - because if I fuck up something, and there happen to be two same ips (should never happen), the window will still work
            // - I want to be able to open multiple windows for the same device at once.
            window_id: egui::Id::new(random::<u64>()),
            open: false,
            ip,
            node_index,
            subwindow_selected: SubWindowType::Info,
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
        let (window_id, device_ip, device_node_index, mut should_show_window_internal) = {
            let current_window = app_context
                .ui_state
                .device_window_states
                .get(device_window_state_index)
                .unwrap();
            (
                current_window.window_id,
                current_window.ip,
                current_window.node_index,
                current_window.open,
            )
        };
        if !should_show_window_internal {
            return;
        }

        egui::Window::new(format!("Device - {}", device_ip))
            .id(window_id)
            .collapsible(true)
            .default_pos(DEFAULT_WINDOW_STARTING_POS)
            .fixed_size(Vec2::new(275.0, 250.0))
            .open(&mut should_show_window_internal)
            .show(egui_context, |ui| {
                ui.horizontal(|ui| {
                    let subwindow_selected_binding = &mut app_context.ui_state.device_window_states
                        [device_window_state_index]
                        .subwindow_selected;
                    ui.selectable_value(
                        subwindow_selected_binding,
                        SubWindowType::Info,
                        "Range (inclusive)",
                    );
                    ui.selectable_value(
                        subwindow_selected_binding,
                        SubWindowType::Ports,
                        "Opened ports",
                    );
                    ui.selectable_value(
                        subwindow_selected_binding,
                        SubWindowType::Actions,
                        "Actions",
                    );
                });
                ui.separator();

                ui.vertical_centered(|ui| {
                    match app_context.ui_state.device_window_states[device_window_state_index].subwindow_selected {
                        SubWindowType::Info => {
                            let graph_lock = &mut app_context.app_state.network_topology.graph.lock().unwrap();
                            let Some(node_info) = graph_lock.node_weight_mut(device_node_index) else {
                                app_context.ui_state.device_window_states[device_window_state_index].open = false;
                                return;
                            };
                            let mut new_node_data = node_info.data().unwrap().clone();

                            ui.add_space(DEFAULT_SPACER);
                            ui.horizontal(|ui| {
                                ui.label("IP Address");
                                ui.add_enabled_ui(false, |ui| {
                                    ui.text_edit_singleline(&mut new_node_data.ip.to_string())
                                });
                            });

                            ui.add_space(DEFAULT_SPACER);
                            ui.horizontal(|ui| {
                                ui.label("Hostname (optional)");

                                let mut new_hostname = new_node_data.hostname.clone();
                                ui.text_edit_singleline(&mut new_hostname);
                                new_node_data.hostname = new_hostname;

                                if ui.button("get").clicked() {
                                    update_hostname_list(
                                        Arc::clone(&app_context.app_state.network_topology.graph),
                                        Arc::clone(&app_context.app_state.status_info),
                                        vec![new_node_data.ip],
                                    );
                                }
                            });

                            ui.add_space(DEFAULT_SPACER);
                            ui.vertical(|ui| {
                                ui.label("Notes (optional)");

                                let mut new_notes = new_node_data.notes.clone();
                                ui.text_edit_multiline(&mut new_notes);
                                new_node_data.notes = new_notes;
                            });

                            node_info.set_data(Some(new_node_data));
                        }
                        SubWindowType::Ports => {}
                        SubWindowType::Actions => {
                            ui.vertical_centered_justified(|ui| {
                                ui.add_space(ACTION_SPACER);
                                if ui.button("Delete").clicked() {
                                    NetworkTopology::remove_node(
                                        &mut app_context.app_state.network_topology.graph,
                                        device_node_index,
                                    );
                                    app_context.ui_state.device_window_states = app_context
                                        .ui_state
                                        .device_window_states
                                        .iter_mut()
                                        .filter(|s| s.ip() != device_ip)
                                        .map(|s| *s)
                                        .collect();
                                }
                            });
                        }
                    }
                })
            });

        // At this point, id is no longer guaranteed to be valid.
        let Some(possibly_window_still_exists) = app_context.ui_state.device_window_states.get_mut(device_window_state_index) else {
            return;
        };
        possibly_window_still_exists.open &= should_show_window_internal;
    }
}
