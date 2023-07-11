use eframe::egui;
use log::info;

use super::{
    network_topology::{
        NetworkTopology, EGUI_GRAPH_SETTINGS_INTERACTIONS, EGUI_GRAPH_SETTINGS_NAVIGATION,
    },
    workspace_models::{AppState, TabsContext, UIState, WorkspaceContext},
    workspace_tab::{default_tabs, WorkspaceTab},
};

pub struct Workspace {
    tabs_context: TabsContext,
    context: WorkspaceContext,
}

impl Default for Workspace {
    fn default() -> Self {
        let tabs_context = default_tabs();
        let context = WorkspaceContext {
            app_state: AppState {
                network_topology: NetworkTopology::default(),
            },
            ui_state: UIState {
                open_tabs: tabs_context.default_tabs.clone(),
            },
        };

        Self {
            tabs_context,
            context,
        }
    }
}

impl eframe::App for Workspace {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: Implement custom frame https://github.com/emilk/egui/tree/master/examples/custom_window_frame
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        ui.label("TODO: Implement project/state saving");
                    });
                    ui.menu_button("View", |ui| {
                        for default_tab in self.tabs_context.default_tabs.iter() {
                            let open_tab_index = self
                                .context
                                .ui_state
                                .open_tabs
                                .iter()
                                .position(|s| s.id == default_tab.id);

                            if ui
                                .selectable_label(
                                    open_tab_index.is_some(),
                                    default_tab.title.clone(),
                                )
                                .clicked()
                            {
                                if let Some(open_tab_index) = open_tab_index {
                                    self.tabs_context.tab_tree.remove_tab(
                                        self.tabs_context.tab_tree.find_tab(default_tab).unwrap(),
                                    );
                                    self.context.ui_state.open_tabs.remove(open_tab_index);
                                } else {
                                    self.tabs_context
                                        .tab_tree
                                        .push_to_focused_leaf(default_tab.clone());
                                    self.context.ui_state.open_tabs.push(default_tab.clone());
                                }
                                ui.close_menu();
                            }
                        }
                    });
                });

                let mut dock_style = egui_dock::Style::from_egui(ui.style());
                dock_style.separator.extra = 50.0;
                egui_dock::DockArea::new(&mut self.tabs_context.tab_tree)
                    .show_close_buttons(true)
                    .show_add_buttons(false)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(false)
                    .style(dock_style)
                    .show_inside(ui, &mut self.context);
            });
    }

    // TODO: Ask to save state if newly serialized state != saved state
    // https://github.com/emilk/egui/blob/master/examples/confirm_exit/src/main.rs
}

impl egui_dock::TabViewer for WorkspaceContext {
    type Tab = WorkspaceTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.id.as_str() {
            "meta_tab" => self.render_meta_tab(ui),
            "topology_overview_tab" => self.render_topology_overview_tab(ui),
            // "Simple Demo" => self.simple_demo(ui),
            // "Style Editor" => self.style_editor(ui),
            _ => {
                ui.label("TODO");
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title.clone().into()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        let position_to_delete = self.ui_state.open_tabs.iter().position(|s| s.id == tab.id);
        if let Some(position_to_delete) = position_to_delete {
            self.ui_state.open_tabs.remove(position_to_delete);
        }

        true
    }
}

impl WorkspaceContext {
    pub fn render_meta_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.button("abc");
        });
    }

    pub fn render_topology_overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.add(
            &mut egui_graphs::GraphView::new(&mut self.app_state.network_topology.graph)
                .with_interactions(&EGUI_GRAPH_SETTINGS_INTERACTIONS)
                .with_navigations(&EGUI_GRAPH_SETTINGS_NAVIGATION)
                .with_changes(&self.app_state.network_topology.graph_changes_sender),
        );
        for change in self
            .app_state
            .network_topology
            .graph_changes_receiver
            .try_iter()
        {
            info!("{:?}", change)
        }
    }
}
