use eframe::egui;
use egui_dock::{DockArea, TabViewer};

use super::{
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
                name: "Arthur".to_owned(),
                age: 42,
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

                DockArea::new(&mut self.tabs_context.tab_tree)
                    .show_close_buttons(true)
                    .show_add_buttons(false)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(false)
                    .show_inside(ui, &mut self.context);
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // TODO: Ask to save state if newly serialized state != saved state
    }
}

impl TabViewer for WorkspaceContext {
    type Tab = WorkspaceTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.id {
            // "Simple Demo" => self.simple_demo(ui),
            // "Style Editor" => self.style_editor(ui),
            _ => {
                ui.label(tab.title.clone());
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
