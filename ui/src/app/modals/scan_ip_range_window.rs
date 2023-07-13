// use eframe::{
//     egui,
//     epaint::{Color32, Vec2},
// };
// use std::{net::IpAddr, str::FromStr};

// use crate::{
//     app::{network_topology::NetworkTopologyNode, workspace_models::WorkspaceContext},
//     utils::constants::{WORKSPACE_WINDOW_HEIGHT, WORKSPACE_WINDOW_WIDTH},
// };

// const SCAN_IP_RANGE_WINDOW_STARTING_POS: eframe::epaint::Pos2 = eframe::epaint::Pos2 {
//     x: WORKSPACE_WINDOW_WIDTH / 2.0 - 150.0,  // TODO: Edit
//     y: WORKSPACE_WINDOW_HEIGHT / 2.0 - 150.0, // TODO: Edit
// };

// #[derive(Default)]
// pub struct ScanIpRangeWindowState {
//     pub open: bool,
// }

// impl ScanIpRangeWindowState {
//     pub fn render(
//         egui_context: &egui::Context,
//         open_window_ref: &mut bool,
//         app_context: &mut WorkspaceContext,
//     ) {
//         egui::Window::new("Manually add a new device")
//             .collapsible(false)
//             .default_pos(SCAN_IP_RANGE_WINDOW_STARTING_POS)
//             .fixed_size(Vec2::new(275.0, 250.0))
//             .open(open_window_ref)
//             .show(egui_context, |ui| {
//                 ui.vertical_centered(|ui| {
//                     ui.horizontal(|ui| {
//                         ui.label("IP Address");
//                         ui.text_edit_singleline(
//                             &mut app_context.ui_state.add_new_device_window_state.ip,
//                         );
//                     });
//                     if app_context
//                         .ui_state
//                         .add_new_device_window_state
//                         .ip_validation_err
//                     {
//                         ui.colored_label(Color32::RED, "IP is not valid.");
//                     }
//                     ui.add_space(5.0);

//                     ui.vertical(|ui| {
//                         ui.label("Notes");
//                         ui.text_edit_multiline(
//                             &mut app_context.ui_state.add_new_device_window_state.notes,
//                         );
//                     });
//                     ui.add_space(10.0);
//                     if ui.button("Add").clicked() {
//                         if let Ok(new_ip) =
//                             IpAddr::from_str(&app_context.ui_state.add_new_device_window_state.ip)
//                         {
//                             app_context.app_state.network_topology.add_node(
//                                 NetworkTopologyNode::new(
//                                     new_ip,
//                                     app_context
//                                         .ui_state
//                                         .add_new_device_window_state
//                                         .notes
//                                         .clone(),
//                                 ),
//                                 None,
//                             );
//                             app_context.ui_state.add_new_device_window_state = Default::default();
//                         } else {
//                             app_context
//                                 .ui_state
//                                 .add_new_device_window_state
//                                 .ip_validation_err = true
//                         }
//                     }
//                 });
//             });
//     }
// }
