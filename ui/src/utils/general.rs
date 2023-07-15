use eframe::epaint::Color32;
use petgraph::visit::IntoNodeReferences;

use crate::app::{network_topology::NetworkTopologyNode, workspace_models::WorkspaceContext};

pub fn add_localhost_pc(app_context: &mut WorkspaceContext) {
    let new_localhost = NetworkTopologyNode::new_my_pc();

    if new_localhost.is_err() {
        app_context.ui_state.add_this_computer_window_state.open = false;
        app_context.ui_state.add_this_computer_window_state.text = "Unable to get information required to create localhost. This is probably, because you're not connected to any network.".to_string();
        return;
    }

    if app_context
        .app_state
        .network_topology
        .graph
        .lock()
        .unwrap()
        .node_references()
        .any(|s| s.1.data().unwrap().is_localhost)
    {
        app_context.ui_state.add_this_computer_window_state.open = true;
        app_context.ui_state.add_this_computer_window_state.text = "Unable to create new instance of this computer, since it already exists. Before creating a new one, please make sure to remove the old instance.".to_string();
        return;
    }

    app_context
        .app_state
        .network_topology
        .add_node(new_localhost.unwrap(), None);
}

pub fn render_validation_err(ui: &mut eframe::egui::Ui, is_invalid: bool, message: &str) {
    if is_invalid {
        ui.colored_label(Color32::RED, message);
    }
}
