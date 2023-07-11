use egui_dock::Tree;

use super::workspace_tab::WorkspaceTab;

pub struct TabsContext {
    pub tab_tree: Tree<WorkspaceTab>,
    pub default_tabs: Vec<WorkspaceTab>,
}

pub struct WorkspaceContext {
    pub app_state: AppState,
    pub ui_state: UIState,
}

pub struct AppState {
    pub name: String,
    pub age: u32,
}

pub struct UIState {
    pub open_tabs: Vec<WorkspaceTab>,
}
