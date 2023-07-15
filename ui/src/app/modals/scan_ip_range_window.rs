use eframe::{
    egui,
    epaint::{Color32, Vec2},
};
use ipnet::IpNet;
use std::{
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
    sync::Arc,
};

use crate::{
    app::workspace_models::WorkspaceContext,
    utils::{
        constants::{WORKSPACE_WINDOW_HEIGHT, WORKSPACE_WINDOW_WIDTH},
        ip::ping_ip_range,
    },
};

const SCAN_IP_RANGE_WINDOW_STARTING_POS: eframe::epaint::Pos2 = eframe::epaint::Pos2 {
    x: WORKSPACE_WINDOW_WIDTH / 2.0 - 150.0,  // TODO: Edit
    y: WORKSPACE_WINDOW_HEIGHT / 2.0 - 150.0, // TODO: Edit
};

pub struct ScanIpRangeWindowState {
    pub open: bool,
    pub input_type: IpInputType,

    pub range_ip_from: String,
    pub range_ip_from_validation_err: bool,
    pub range_ip_to: String,
    pub range_ip_to_validation_err: bool,
    pub cidr_notation: String,
    pub cidr_notation_validation_err: bool,

    pub manual_ips: Vec<String>,
    pub manual_ips_validation_err: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IpInputType {
    Range,
    CIDRNotation,
    Manual,
}

impl Default for ScanIpRangeWindowState {
    fn default() -> Self {
        Self {
            open: true,
            // open: false,
            input_type: IpInputType::Range,
            range_ip_from: "192.168.0.0".to_owned(),
            range_ip_from_validation_err: false,
            range_ip_to: "192.168.0.255".to_owned(),
            range_ip_to_validation_err: false,

            cidr_notation: "192.168.0.0/24".to_owned(),
            cidr_notation_validation_err: false,

            manual_ips: vec!["192.168.0.1".to_owned()],
            manual_ips_validation_err: vec![],
        }
    }
}

impl ScanIpRangeWindowState {
    pub fn render(egui_context: &egui::Context, app_context: &mut WorkspaceContext) {
        let mut should_show_window = app_context.ui_state.scan_ip_range_window_state.open;
        if !should_show_window {
            return;
        }

        egui::Window::new("IP Range Scanning options")
            .collapsible(false)
            .default_pos(SCAN_IP_RANGE_WINDOW_STARTING_POS)
            .fixed_size(Vec2::new(275.0, 250.0))
            .open(&mut should_show_window)
            .show(egui_context, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut app_context.ui_state.scan_ip_range_window_state.input_type,
                        IpInputType::Range,
                        "Range (inclusive)",
                    );
                    ui.selectable_value(
                        &mut app_context.ui_state.scan_ip_range_window_state.input_type,
                        IpInputType::CIDRNotation,
                        "CIDR Notation",
                    );
                    ui.selectable_value(
                        &mut app_context.ui_state.scan_ip_range_window_state.input_type,
                        IpInputType::Manual,
                        "Manually input IPs",
                    );
                });
                ui.separator();

                ui.vertical_centered(|ui| {
                    match app_context.ui_state.scan_ip_range_window_state.input_type {
                        IpInputType::Range => {
                            ui.horizontal(|ui| {
                                ui.label("From IP Address");
                                ui.text_edit_singleline(
                                    &mut app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .range_ip_from,
                                );
                            });
                            if app_context
                                .ui_state
                                .scan_ip_range_window_state
                                .range_ip_from_validation_err
                            {
                                ui.colored_label(Color32::RED, "IP is not valid.");
                            }
                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.label("To IP Address     ");
                                ui.text_edit_singleline(
                                    &mut app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .range_ip_to,
                                );
                            });
                            if app_context
                                .ui_state
                                .scan_ip_range_window_state
                                .range_ip_to_validation_err
                            {
                                ui.colored_label(Color32::RED, "IP is not valid.");
                            }
                        }
                        IpInputType::CIDRNotation => {
                            ui.horizontal(|ui| {
                                ui.label("CIDR Notation");
                                ui.text_edit_singleline(
                                    &mut app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .cidr_notation,
                                );
                            });
                            if app_context
                                .ui_state
                                .scan_ip_range_window_state
                                .cidr_notation_validation_err
                            {
                                ui.colored_label(Color32::RED, "CIDR notation is not valid.");
                            }
                        }
                        IpInputType::Manual => {
                            let mut index_to_delete = None;
                            for (i, ip) in app_context
                                .ui_state
                                .scan_ip_range_window_state
                                .manual_ips
                                .iter_mut()
                                .enumerate()
                            {
                                ui.horizontal(|ui| {
                                    ui.set_width(ui.available_width() * 0.85);
                                    ui.text_edit_singleline(ip);
                                    if ui.button("🗑").clicked() {
                                        index_to_delete = Some(i);
                                    }
                                });
                            }
                            if let Some(index_to_delete) = index_to_delete {
                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .manual_ips
                                    .remove(index_to_delete);
                            }

                            for err in app_context
                                .ui_state
                                .scan_ip_range_window_state
                                .manual_ips_validation_err
                                .iter()
                            {
                                ui.colored_label(Color32::RED, format!("{} is not valid IP.", err));
                            }

                            ui.add_space(5.0);
                            if ui.button("+").clicked() {
                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .manual_ips
                                    .push("".to_owned());
                            }
                        }
                    }
                    ui.add_space(5.0);
                    ui.separator();

                    if ui.button("Start scan").clicked() {
                        let ips_to_scan: Option<Vec<IpAddr>> = match app_context
                            .ui_state
                            .scan_ip_range_window_state
                            .input_type
                        {
                            IpInputType::Range => {
                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .range_ip_from_validation_err = false;
                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .range_ip_to_validation_err = false;

                                let from_ip_res = IpAddr::from_str(
                                    &app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .range_ip_from,
                                );
                                let to_ip_res = IpAddr::from_str(
                                    &app_context.ui_state.scan_ip_range_window_state.range_ip_to,
                                );

                                match (from_ip_res, to_ip_res) {
                                    (Ok(from_ip), Ok(to_ip)) => {
                                        let mut ips_to_ping = vec![];
                                        let mut from = u128::from_be_bytes(
                                            match from_ip {
                                                IpAddr::V4(s) => s.to_ipv6_mapped(),
                                                IpAddr::V6(s) => s,
                                            }
                                            .octets(),
                                        );
                                        let to = u128::from_be_bytes(
                                            match to_ip {
                                                IpAddr::V4(s) => s.to_ipv6_mapped(),
                                                IpAddr::V6(s) => s,
                                            }
                                            .octets(),
                                        );

                                        while from <= to {
                                            ips_to_ping.push(Ipv6Addr::from(from));
                                            from += 1;
                                        }
                                        Some(
                                            ips_to_ping
                                                .iter()
                                                .map(|s| {
                                                    if let Some(ipv4) = s.to_ipv4() {
                                                        IpAddr::V4(ipv4)
                                                    } else {
                                                        IpAddr::V6(*s)
                                                    }
                                                })
                                                .collect(),
                                        )
                                    }
                                    (Err(_), Ok(_)) => {
                                        app_context
                                            .ui_state
                                            .scan_ip_range_window_state
                                            .range_ip_from_validation_err = true;
                                        None
                                    }
                                    (Ok(_), Err(_)) => {
                                        app_context
                                            .ui_state
                                            .scan_ip_range_window_state
                                            .range_ip_to_validation_err = true;
                                        None
                                    }
                                    (Err(_), Err(_)) => {
                                        app_context
                                            .ui_state
                                            .scan_ip_range_window_state
                                            .range_ip_from_validation_err = true;
                                        app_context
                                            .ui_state
                                            .scan_ip_range_window_state
                                            .range_ip_to_validation_err = true;
                                        None
                                    }
                                }
                            }
                            IpInputType::CIDRNotation => {
                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .cidr_notation_validation_err = false;
                                if let Ok(ips_to_ping) = IpNet::from_str(
                                    &app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .cidr_notation,
                                ) {
                                    Some(ips_to_ping.hosts().collect::<Vec<IpAddr>>())
                                } else {
                                    app_context
                                        .ui_state
                                        .scan_ip_range_window_state
                                        .cidr_notation_validation_err = true;
                                    None
                                }
                            }
                            IpInputType::Manual => {
                                let mut ips_to_ping = vec![];
                                let mut are_all_valid = true;

                                app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .manual_ips_validation_err = vec![];
                                for ip in app_context
                                    .ui_state
                                    .scan_ip_range_window_state
                                    .manual_ips
                                    .iter()
                                {
                                    if let Ok(ip_to_add) = IpAddr::from_str(ip) {
                                        ips_to_ping.push(ip_to_add);
                                    } else {
                                        are_all_valid = false;
                                        app_context
                                            .ui_state
                                            .scan_ip_range_window_state
                                            .manual_ips_validation_err
                                            .push(ip.to_string());
                                    }
                                }

                                if are_all_valid {
                                    Some(ips_to_ping)
                                } else {
                                    None
                                }
                            }
                        };

                        if let Some(ips_to_ping) = ips_to_scan {
                            ping_ip_range(
                                Arc::clone(&app_context.app_state.network_topology.graph),
                                Arc::clone(&app_context.app_state.status_info),
                                ips_to_ping,
                            );
                            app_context.ui_state.scan_ip_range_window_state.open = false;
                        }
                    }
                });
            });

        app_context.ui_state.scan_ip_range_window_state.open &= should_show_window;
    }
}
