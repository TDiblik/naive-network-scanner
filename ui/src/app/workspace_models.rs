use egui_dock::Tree;

use super::{network_topology::NetworkTopology, workspace_tab::WorkspaceTab};

pub struct TabsContext {
    pub tab_tree: Tree<WorkspaceTab>,
    pub default_tabs: Vec<WorkspaceTab>,
}

pub struct WorkspaceContext {
    pub app_state: AppState,
    pub ui_state: UIState,
}

pub struct AppState {
    pub network_topology: NetworkTopology,
}

pub struct UIState {
    pub open_tabs: Vec<WorkspaceTab>,
    pub add_new_pc_window_state: AddNewPcWindowState,
}

#[derive(Default)]
pub struct AddNewPcWindowState {
    pub open: bool,
    pub new_ip: String,
    pub new_ip_validation_err: bool,
    pub notes: String,
}
