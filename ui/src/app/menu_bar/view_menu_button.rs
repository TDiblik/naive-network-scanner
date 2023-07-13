use eframe::egui;

use crate::app::workspace::Workspace;

pub struct ViewMenuButton {}

impl ViewMenuButton {
    pub fn render(ui: &mut egui::Ui, workspace_state: &mut Workspace) {
        ui.menu_button("View", |ui| {
            for default_tab in workspace_state.tabs_context.default_tabs.iter() {
                let open_tab_index = workspace_state
                    .context
                    .ui_state
                    .open_tabs
                    .iter()
                    .position(|s| s.id == default_tab.id);

                if ui
                    .selectable_label(open_tab_index.is_some(), default_tab.title.clone())
                    .clicked()
                {
                    if let Some(open_tab_index) = open_tab_index {
                        workspace_state.tabs_context.tab_tree.remove_tab(
                            workspace_state
                                .tabs_context
                                .tab_tree
                                .find_tab(default_tab)
                                .unwrap(),
                        );
                        workspace_state
                            .context
                            .ui_state
                            .open_tabs
                            .remove(open_tab_index);
                    } else {
                        workspace_state
                            .tabs_context
                            .tab_tree
                            .push_to_focused_leaf(default_tab.clone());
                        workspace_state
                            .context
                            .ui_state
                            .open_tabs
                            .push(default_tab.clone());
                    }
                    ui.close_menu();
                }
            }
        });
    }
}
