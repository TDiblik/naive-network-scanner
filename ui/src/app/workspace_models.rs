use std::sync::{Arc, Mutex};

use crate::utils::constants::LINE_ENDING;

use super::{
    modals::{
        add_new_device_window::AddNewDeviceWindowState, device_window_state::DeviceWindowState,
        generic_info_window::GenericInfoWindowState, scan_ip_range_window::ScanIpRangeWindowState,
    },
    network_topology::NetworkTopology,
    workspace_tab::WorkspaceTab,
};
use egui_dock::Tree;
use log::{error, info, warn};

pub struct TabsContext {
    pub tab_tree: Tree<WorkspaceTab>,
    pub default_tabs: Vec<WorkspaceTab>,
}

pub struct WorkspaceContext {
    pub app_state: AppState,
    pub ui_state: UIState,
}

#[derive(Default)]
pub struct StatusInfo {
    pub text_to_batch: String,
    pub text_to_render: String,
    pub scroll_on_next_render: bool,
}
pub type StatusInfoRef = Arc<Mutex<StatusInfo>>;

#[derive(Clone)]
pub enum StatusMessage {
    Info(String),
    Warn(String),
    Err(String),
}

pub struct AppState {
    pub network_topology: NetworkTopology,
    pub status_info: StatusInfoRef,
}
impl AppState {
    pub fn log_to_status(&self, info_to_append: StatusMessage) {
        Self::log_to_status_generic(&self.status_info, info_to_append);
    }

    pub fn log_to_status_generic(status_info_ref: &StatusInfoRef, info_to_append: StatusMessage) {
        let now = chrono::Local::now();
        let value_to_append = match info_to_append {
            StatusMessage::Info(s) => {
                info!("{s}");
                s
            }
            StatusMessage::Warn(s) => {
                warn!("{s}");
                s
            }
            StatusMessage::Err(s) => {
                error!("{s}");
                s
            }
        };

        status_info_ref
            .lock()
            .unwrap()
            .text_to_batch
            .push_str(&format!(
                "{}: {}{}",
                now.format("%H:%M:%S%.3f"),
                value_to_append,
                LINE_ENDING
            ));
    }
}

pub struct UIState {
    pub open_tabs: Vec<WorkspaceTab>,
    pub add_new_device_window_state: AddNewDeviceWindowState,
    pub add_this_computer_window_state: GenericInfoWindowState,
    pub scan_ip_range_window_state: ScanIpRangeWindowState,
    pub device_window_states: Vec<DeviceWindowState>,
}
