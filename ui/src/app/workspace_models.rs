use std::sync::{Arc, Mutex};

use crate::utils::constants::LINE_ENDING;

use super::{
    modals::{
        add_new_device_window::AddNewDeviceWindowState, generic_info_window::GenericInfoWindowState,
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

pub type StatusInfo = Arc<Mutex<String>>;

#[derive(Clone)]
pub enum StatusMessage {
    Info(String),
    Warn(String),
    Err(String),
}
impl From<StatusMessage> for String {
    fn from(value: StatusMessage) -> Self {
        match value {
            StatusMessage::Info(s) => s,
            StatusMessage::Warn(s) => s,
            StatusMessage::Err(s) => s,
        }
    }
}

pub struct AppState {
    pub network_topology: NetworkTopology,
    pub status_info: StatusInfo,
}
impl AppState {
    pub fn log_to_status(&self, info_to_append: StatusMessage) {
        Self::log_to_status_generic(&self.status_info, info_to_append);
    }

    pub fn log_to_status_generic(status_info_ref: &StatusInfo, info_to_append: StatusMessage) {
        let mut new_log_line = chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();
        new_log_line.push_str(": ");
        let line_to_append: String = info_to_append.clone().into();
        new_log_line.push_str(&line_to_append);
        new_log_line.push_str(LINE_ENDING);

        match info_to_append {
            StatusMessage::Info(_) => info!("{}", new_log_line),
            StatusMessage::Warn(_) => warn!("{}", new_log_line),
            StatusMessage::Err(_) => error!("{}", new_log_line),
        }

        status_info_ref.lock().unwrap().push_str(&new_log_line);
    }
}

pub struct UIState {
    pub open_tabs: Vec<WorkspaceTab>,
    pub add_new_device_window_state: AddNewDeviceWindowState,
    pub add_this_computer_state: GenericInfoWindowState,
}
