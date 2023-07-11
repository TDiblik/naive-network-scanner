use eframe::egui;
use egui_dock::{DockArea, TabViewer, Tree};

use super::workspace_tab::{default_tab_tree, WorkspaceTab};

pub struct Workspace {
    tab_tree: Tree<WorkspaceTab>,
    context: WorkspaceContext,
}

impl Default for Workspace {
    fn default() -> Self {
        let tab_tree = default_tab_tree();
        let context = WorkspaceContext {
            name: "Arthur".to_owned(),
            age: 42,
        };

        Self { tab_tree, context }
    }
}

impl eframe::App for Workspace {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                // TODO: Implement
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {});
                    ui.menu_button("View", |ui| {
                        // allow certain tabs to be toggled
                        // for tab in &["File Browser", "Asset Manager"] {
                        //     if ui
                        //         .selectable_label(self.context.open_tabs.contains(*tab), *tab)
                        //         .clicked()
                        //     {
                        //         if let Some(index) = self.tree.find_tab(&tab.to_string()) {
                        //             self.tree.remove_tab(index);
                        //             self.context.open_tabs.remove(*tab);
                        //         } else {
                        //             self.tree.push_to_focused_leaf(tab.to_string());
                        //         }

                        //         ui.close_menu();
                        //     }
                        // }
                        ui.selectable_label(true, "abc");
                    });
                });

                DockArea::new(&mut self.tab_tree)
                    .show_close_buttons(true)
                    .show_add_buttons(false)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(false)
                    .show_inside(ui, &mut self.context);
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}

pub struct WorkspaceContext {
    name: String,
    age: u32,
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
        // self.open_tabs.remove(tab);
        true
    }
}
