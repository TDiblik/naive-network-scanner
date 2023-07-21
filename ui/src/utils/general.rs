use eframe::{egui::Ui, epaint::Color32};
use petgraph::visit::IntoNodeReferences;

use crate::app::{
    network_topology::{NetworkTopology, NetworkTopologyNode},
    workspace_models::WorkspaceContext,
};

pub fn add_localhost_pc(app_context: &mut WorkspaceContext) {
    let new_localhost = NetworkTopologyNode::new_my_pc();

    if new_localhost.is_err() {
        app_context.ui_state.add_this_computer_window_state.show("Unable to get information required to create localhost. This is probably, because you're not connected to any network.".to_string());
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
        app_context.ui_state.add_this_computer_window_state.show("Unable to create new instance of this computer, since it already exists. Before creating a new one, please make sure to remove the old instance.".to_string());
        return;
    }

    let new_localhost = new_localhost.unwrap();
    if NetworkTopology::add_node(
        &mut app_context.app_state.network_topology.graph,
        new_localhost.clone(),
        None,
    )
    .is_none()
    {
        app_context.ui_state.add_this_computer_window_state.show(format!("Unable to create new instance of this computer, since your IP ({}) already exists as a node. Before creating a new one, please make sure to remove the old instance.", new_localhost.ip));
    }
}

pub fn render_validation_err(ui: &mut Ui, is_invalid: bool, message: &str) {
    if is_invalid {
        ui.colored_label(Color32::RED, message);
    }
}

pub fn render_numeric_textbox(ui: &mut Ui, input: &mut String) {
    if ui.text_edit_singleline(input).changed() {
        *input = input.chars().filter(|s| s.is_numeric()).collect::<String>();
    }
}
