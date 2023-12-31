use std::{net::IpAddr, sync::Arc};

use eframe::{egui::{self, ScrollArea, TextStyle}, epaint::Vec2};
use egui_extras::{TableBuilder, Column};
use petgraph::stable_graph::NodeIndex;
use rand::random;

use crate::{
    app::{network_topology::NetworkTopology, workspace_models::WorkspaceContext},
    utils::{
        constants::{
            ACTION_SPACER, ALL_COMMON_PORTS, ALL_COMMON_PORTS_LENGHT, DEFAULT_SPACER,
            DEFAULT_WINDOW_STARTING_POS, MOST_COMMON_PORTS, TRASH_ICON,
        },
        general::{render_validation_err, render_numeric_textbox},
        ip::{scap_ip_ports, update_hostname_list, Port, ScanIpPortsConfig},
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum SubWindowType {
    Info,
    Ports,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PortScanSubWindowType {
    Range,
    AllCommon,
    Manual,
}

type AllCommonPortsParsed = Vec<(Port, String, String)>;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceWindowState {
    window_id: egui::Id,
    window_id_raw: u64,
    open: bool,
    ip: IpAddr,
    node_index: NodeIndex,

    subwindow_selected: SubWindowType,

    port_scan_subwindow_selected: PortScanSubWindowType,
    port_scan_subwindow_range_from: String,
    port_scan_subwindow_range_from_validation_err: bool,
    port_scan_subwindow_range_to: String,
    port_scan_subwindow_range_to_validation_err: bool,
    port_scan_subwindow_all_common_ports: AllCommonPortsParsed,
    port_scan_subwindow_manual_ports: Vec<String>,
    port_scan_subwindow_manual_ports_validation_err: Vec<String>,
    port_scan_settings_connection_timeout_ms: String,
    port_scan_settings_should_banner_grab: bool,
    port_scan_settings_should_fuzz: bool,
    port_scan_settings_read_write_timeout_ms: String,
    
    should_show_port_scan_window: bool,
    scan_results_window_id: egui::Id,
    scan_results_window_id_raw: u64
}
impl DeviceWindowState {
    pub fn new(ip: IpAddr, node_index: NodeIndex) -> Self {
        // Using random instead of ip,
        // - because if I fuck up something, and there happen to be two same ips (should never happen), the window will still work
        // - I want to be able to open multiple windows for the same device at once.
        let window_id_raw = random::<u64>();
        let scan_results_window_id_raw = random::<u64>();
        Self {
            window_id: egui::Id::new(window_id_raw),
            window_id_raw,
            open: false,
            ip,
            node_index,

            subwindow_selected: SubWindowType::Info,

            port_scan_subwindow_selected: PortScanSubWindowType::Range,
            port_scan_subwindow_range_from: "1".to_owned(),
            port_scan_subwindow_range_from_validation_err: false,
            port_scan_subwindow_range_to: "1023".to_owned(),
            port_scan_subwindow_range_to_validation_err: false,
            port_scan_subwindow_all_common_ports: Self::get_mapped_all_common_ports(),
            port_scan_subwindow_manual_ports: MOST_COMMON_PORTS.map(|s| s.to_string()).to_vec(),
            port_scan_subwindow_manual_ports_validation_err: vec![],
            port_scan_settings_connection_timeout_ms: "50".to_owned(),
            port_scan_settings_should_banner_grab: true,
            port_scan_settings_should_fuzz: false,
            port_scan_settings_read_write_timeout_ms: "250".to_owned(),
            
            should_show_port_scan_window: false,
            scan_results_window_id: egui::Id::new(scan_results_window_id_raw),
            scan_results_window_id_raw
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

    fn get_mapped_all_common_ports() -> AllCommonPortsParsed {
        ALL_COMMON_PORTS.map(|s| (s.0, s.1.to_string(), s.2.to_string())).to_vec()
    }

    pub fn render(
        egui_context: &egui::Context,
        app_context: &mut WorkspaceContext,
        device_window_state_index: usize,
    ) {
        let (
            window_id,
            // window_id_raw,
            device_ip,
            device_node_index,
            mut should_show_window_internal,
            should_show_port_scan_window,
            scan_results_window_id
        ) = {
            let current_window = app_context
                .ui_state
                .device_window_states
                .get(device_window_state_index)
                .unwrap();
            (
                current_window.window_id,
                // current_window.window_id_raw,
                current_window.ip,
                current_window.node_index,
                current_window.open,
                current_window.should_show_port_scan_window,
                current_window.scan_results_window_id
            )
        };
        if !should_show_window_internal {
            return;
        }
        
        if should_show_port_scan_window {
            egui::Window::new(format!("Port scan results - {}", device_ip))
                .id(scan_results_window_id)
                .collapsible(true)
                .default_pos(DEFAULT_WINDOW_STARTING_POS)
                .resizable(true)
                .min_height(500.0)
                .open(
                    &mut app_context
                        .ui_state
                        .device_window_states
                        .get_mut(device_window_state_index)
                        .unwrap()
                        .should_show_port_scan_window
                )
                .show(egui_context, |ui| {
                    let graph_lock =
                        &mut app_context.app_state.network_topology.graph.lock().unwrap();
                    let node_info = graph_lock.node_weight_mut(device_node_index).unwrap(); // safe to unwrap since this CANNOT be None;
                    let mut new_node_data = node_info.data().unwrap().clone();

                    let mut index_to_delete = None;

                    ScrollArea::horizontal().show(ui, |ui| {
                        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::auto())
                            .column(Column::initial(100.0).range(40.0..=300.0))
                            .column(Column::initial(100.0).at_least(40.0).resizable(true))
                            .column(Column::initial(100.0).at_least(40.0).resizable(true))
                            .column(Column::remainder())
                            .min_scrolled_height(0.0);

                        table.header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Port");
                            });
                            header.col(|ui| {
                                ui.strong("Possible service");
                            });
                            header.col(|ui| {
                                ui.strong("Banner");
                            });
                            header.col(|ui| {
                                ui.strong("Fuzzing results");
                            });
                            header.col(|ui| {
                                ui.strong("Actions");
                            });
                        })
                        .body(|mut body| {
                            for (i, port) in new_node_data.opened_ports.iter().enumerate() {
                                body.row(text_height, |mut row| {
                                    row.col(|ui| {
                                        ui.label(port.number.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(port.possible_service_name.clone()).on_hover_text(port.possible_service_usefull_info.clone().unwrap_or("".to_string()));
                                    });
                                    row.col(|ui| {
                                        match port.banner.clone() {
                                            Some(banner) if !banner.is_empty() => {
                                                ui.label("found").on_hover_text(banner);
                                            }
                                            _ => {
                                                ui.label("not found");
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        match port.fuzzing_results.clone() {
                                            Some(fuzzing_results) if !fuzzing_results.is_empty() => {
                                                let popup_text = fuzzing_results
                                                        .iter()
                                                        .map(
                                                            |s| if !s.result_raw.is_empty() {
                                                                format!("\"{}\" => \n{}\n--------\n", s.command.trim_end(), s.result)
                                                            } else {
                                                                "".to_string()
                                                            }
                                                        )
                                                        .collect::<String>();
                                                ui.label("found").on_hover_text(popup_text);
                                            },
                                            _ => {
                                                ui.label("not found");
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        if ui.button(TRASH_ICON).clicked() {
                                            index_to_delete = Some(i);
                                        }
                                    });
                                });
                            }
                        });
                        if let Some(index_to_delete) = index_to_delete {
                            new_node_data.opened_ports.remove(index_to_delete);
                        }
                        node_info.set_data(Some(new_node_data));
                    });
                });
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
                        "General info",
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
                    match app_context.ui_state.device_window_states[device_window_state_index]
                        .subwindow_selected
                    {
                        SubWindowType::Info => {
                            let graph_lock =
                                &mut app_context.app_state.network_topology.graph.lock().unwrap();
                            let node_info = graph_lock.node_weight_mut(device_node_index).unwrap(); // safe to unwrap since this CANNOT be None;
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
                        SubWindowType::Ports => {
                            let window_binding = &mut app_context.ui_state.device_window_states
                                [device_window_state_index];
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut window_binding.port_scan_subwindow_selected,
                                    PortScanSubWindowType::Range,
                                    "Range (inclusive)",
                                );
                                ui.selectable_value(
                                    &mut window_binding.port_scan_subwindow_selected,
                                    PortScanSubWindowType::AllCommon,
                                    "Common ports",
                                );
                                ui.selectable_value(
                                    &mut window_binding.port_scan_subwindow_selected,
                                    PortScanSubWindowType::Manual,
                                    "Manually input ports",
                                );
                            });
                            ui.separator();
                            match window_binding.port_scan_subwindow_selected {
                                PortScanSubWindowType::Range => {
                                    ui.horizontal(|ui| {
                                        ui.label("From port");
                                        ui.text_edit_singleline(
                                            &mut window_binding.port_scan_subwindow_range_from,
                                        );
                                    });
                                    render_validation_err(
                                        ui,
                                        window_binding
                                            .port_scan_subwindow_range_from_validation_err,
                                        "Port is not valid (must be 0 - 65535).",
                                    );

                                    ui.add_space(DEFAULT_SPACER);
                                    ui.horizontal(|ui| {
                                        ui.label("To port     ");
                                        ui.text_edit_singleline(
                                            &mut window_binding.port_scan_subwindow_range_to,
                                        );
                                    });
                                    render_validation_err(
                                        ui,
                                        window_binding.port_scan_subwindow_range_to_validation_err,
                                        "Port is not valid (must be 0 - 65535).",
                                    );
                                }
                                PortScanSubWindowType::AllCommon => {
                                    let mut index_to_delete = None;
                                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                                        ui,
                                        ui.text_style_height(&TextStyle::Body) / 999.0,
                                        ALL_COMMON_PORTS_LENGHT,
                                        |ui, _| {
                                            // TODO: Every second row should have #202020 as a background color
                                            for (i, port) in window_binding
                                                .port_scan_subwindow_all_common_ports
                                                .iter()
                                                .enumerate()
                                            {
                                                ui.horizontal(|ui| {
                                                    ui.label(port.0.to_string());
                                                    ui.horizontal(|ui| {
                                                        ui.set_width(ui.available_width() * 0.85);
                                                        ui.label(port.1.clone()).on_hover_text(port.2.clone());
                                                    });
                                                    ui.add_space(ACTION_SPACER);
                                                    if ui.button(TRASH_ICON).clicked() {
                                                        index_to_delete = Some(i);
                                                    }
                                                });
                                            }
                                        },
                                    );
                                    if let Some(index_to_delete) = index_to_delete {
                                        window_binding
                                            .port_scan_subwindow_all_common_ports
                                            .remove(index_to_delete);
                                    }
                                    if window_binding.port_scan_subwindow_all_common_ports.len()
                                        != ALL_COMMON_PORTS_LENGHT
                                        && ui.button("Refresh list of ports").clicked()
                                    {
                                        window_binding.port_scan_subwindow_all_common_ports =
                                            Self::get_mapped_all_common_ports();
                                    }
                                }
                                PortScanSubWindowType::Manual => {
                                    let mut index_to_delete = None;
                                    for (i, port) in window_binding
                                        .port_scan_subwindow_manual_ports
                                        .iter_mut()
                                        .enumerate()
                                    {
                                        ui.horizontal(|ui| {
                                            ui.set_width(ui.available_width() * 0.85);
                                            ui.text_edit_singleline(port);
                                            if ui.button(TRASH_ICON).clicked() {
                                                index_to_delete = Some(i);
                                            }
                                        });
                                    }
                                    if let Some(index_to_delete) = index_to_delete {
                                        window_binding
                                            .port_scan_subwindow_manual_ports
                                            .remove(index_to_delete);
                                    }

                                    for err in window_binding
                                        .port_scan_subwindow_manual_ports_validation_err
                                        .iter()
                                    {
                                        render_validation_err(
                                            ui,
                                            true,
                                            &format!(
                                                "{} is not valid port (must be 0 - 65535).",
                                                err
                                            ),
                                        );
                                    }

                                    ui.add_space(DEFAULT_SPACER);
                                    if ui.button("+").clicked() {
                                        window_binding
                                            .port_scan_subwindow_manual_ports
                                            .push("".to_owned());
                                    }
                                }
                            }

                            ui.separator();
                            ui.add_space(DEFAULT_SPACER);
                            ui.horizontal(|ui| {
                                ui.label("Connection timeout (ms)");
                                render_numeric_textbox(
                                    ui,
                                    &mut window_binding.port_scan_settings_connection_timeout_ms
                                );
                            });

                            ui.add_space(DEFAULT_SPACER);
                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut window_binding.port_scan_settings_should_banner_grab,
                                    "Should try banner grabbing",
                                );
                            });

                            ui.add_space(DEFAULT_SPACER);
                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut window_binding.port_scan_settings_should_fuzz,
                                    "Should try fuzzing (could take a while...)",
                                ).on_hover_text("You probably want to set read/write timeout to extremelly small amount of time.");
                            });

                            if window_binding.port_scan_settings_should_banner_grab || window_binding.port_scan_settings_should_fuzz {
                                ui.add_space(DEFAULT_SPACER);
                                ui.horizontal(|ui| {
                                    ui.label("Socket read/write timeout (ms)");
                                    render_numeric_textbox(
                                        ui,
                                        &mut window_binding.port_scan_settings_read_write_timeout_ms
                                    );
                                });
                            }

                            ui.separator();
                            if ui.button("Start scan").clicked() {
                                let ports_to_try = match window_binding
                                    .port_scan_subwindow_selected
                                {
                                    PortScanSubWindowType::Range => {
                                        window_binding.port_scan_subwindow_range_from_validation_err = false;
                                        window_binding.port_scan_subwindow_range_to_validation_err = false;
                                        let from_port_res = window_binding.port_scan_subwindow_range_from.parse::<Port>();
                                        let to_port_res = window_binding.port_scan_subwindow_range_to.parse::<Port>();
                                        match (from_port_res, to_port_res) {
                                            (Ok(from_port), Ok(to_port)) => {
                                                Some((from_port..=to_port).collect::<Vec<Port>>())
                                            },
                                            (Err(_), Ok(_)) => {
                                                window_binding.port_scan_subwindow_range_from_validation_err = true;
                                                None
                                            },
                                            (Ok(_), Err(_)) => {
                                                window_binding.port_scan_subwindow_range_to_validation_err = true;
                                                None
                                            },
                                            (Err(_), Err(_)) => {
                                                window_binding.port_scan_subwindow_range_from_validation_err = true;
                                                window_binding.port_scan_subwindow_range_to_validation_err = true;
                                                None
                                            },
                                        }
                                    }
                                    PortScanSubWindowType::AllCommon => Some(
                                        window_binding
                                            .port_scan_subwindow_all_common_ports
                                            .iter()
                                            .map(|s| s.0)
                                            .collect(),
                                    ),
                                    PortScanSubWindowType::Manual => {
                                        let mut ports_to_try = vec![];
                                        let mut are_all_valid = true;

                                        window_binding
                                            .port_scan_subwindow_manual_ports_validation_err =
                                            vec![];
                                        for port in &window_binding.port_scan_subwindow_manual_ports
                                        {
                                            if let Ok(port_to_add) = port.parse::<Port>() {
                                                ports_to_try.push(port_to_add);
                                            } else {
                                                are_all_valid = false;
                                                window_binding
                                                    .port_scan_subwindow_manual_ports_validation_err
                                                    .push(port.to_string());
                                            }
                                        }

                                        if are_all_valid {
                                            Some(ports_to_try)
                                        } else {
                                            None
                                        }
                                    }
                                };
                                
                                if let Some(ports_to_try) = ports_to_try {
                                    scap_ip_ports(
                                        Arc::clone(&app_context.app_state.network_topology.graph),
                                        Arc::clone(&app_context.app_state.status_info),
                                        device_ip, 
                                        ports_to_try, 
                                        device_node_index,
                                        ScanIpPortsConfig {
                                            connection_timeout_ms: window_binding.port_scan_settings_connection_timeout_ms.parse().unwrap_or(1),
                                            should_banner_grab: window_binding.port_scan_settings_should_banner_grab,
                                            should_fuzz: window_binding.port_scan_settings_should_fuzz,
                                            read_write_timeout_ms: window_binding.port_scan_settings_read_write_timeout_ms.parse().unwrap_or(1),
                                        }
                                    );
                                }
                            }
                            
                            ui.add_space(ACTION_SPACER);
                            if ui.button("Show results").clicked() {
                                window_binding.should_show_port_scan_window = true;
                            }
                        }
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
                                        .map(|s| s.clone())
                                        .collect();
                                }
                            });
                        }
                    }
                })
            });

        // At this point, id is no longer guaranteed to be valid.
        let Some(possible_window_ref) = app_context.ui_state.device_window_states.get_mut(device_window_state_index) else {
            return;
        };
        possible_window_ref.open &= should_show_window_internal;
    }
}
