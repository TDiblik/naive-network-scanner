use eframe::egui;
use log::info;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::utils::general::add_localhost_pc;

use super::{
    menu_bar::{file_menu_button::FileMenuButton, view_menu_button::ViewMenuButton},
    modals::{
        add_new_device_window::AddNewDeviceWindowState,
        generic_info_window::GenericInfoWindowState, scan_ip_range_window::ScanIpRangeWindowState,
    },
    network_topology::{
        NetworkTopology, EGUI_GRAPH_SETTINGS_INTERACTIONS, EGUI_GRAPH_SETTINGS_NAVIGATION,
        EGUI_GRAPH_SETTINGS_STYLE,
    },
    workspace_models::{
        AppState, StatusInfo, StatusMessage, TabsContext, UIState, WorkspaceContext,
    },
    workspace_tab::{default_tabs, WorkspaceTab},
};

pub struct Workspace {
    // TODO: Allow dead code (id) for now, since I don't want to see the warning. Will be usefull in the future.
    #[allow(dead_code)]
    pub id: Uuid,
    pub tabs_context: TabsContext,
    pub context: WorkspaceContext,
}

impl Workspace {
    pub fn new(id: Uuid) -> Self {
        let tabs_context = default_tabs();
        let context = WorkspaceContext {
            app_state: AppState {
                network_topology: NetworkTopology::default(),
                status_info: Arc::new(Mutex::new(StatusInfo::default())),
            },
            ui_state: UIState {
                open_tabs: tabs_context.default_tabs.clone(),
                add_new_device_window_state: AddNewDeviceWindowState::default(),
                add_this_computer_window_state: GenericInfoWindowState::new(
                    "Cannot add this computer",
                ),
                scan_ip_range_window_state: ScanIpRangeWindowState::default(),
            },
        };

        context.app_state.log_to_status(StatusMessage::Info(format!(
            "Initialized new workspace {}!",
            id
        )));
        Self {
            id,
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
                // Top menu bars
                egui::menu::bar(ui, |ui| {
                    FileMenuButton::render(ui);
                    ViewMenuButton::render(ui, self);
                });

                // "Add new device" modal window
                AddNewDeviceWindowState::render(ctx, &mut self.context);
                // "Add this computer" info modal window
                GenericInfoWindowState::render(
                    ctx,
                    &mut self.context.ui_state.add_this_computer_window_state,
                );
                // "IP Range Scanning options" model window
                ScanIpRangeWindowState::render(ctx, &mut self.context);

                // Docking
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
            // TODO (chore): Order by default alignment
            "general_tab" => self.render_general_tab(ui),
            "topology_overview_tab" => self.render_topology_overview_tab(ui),
            "discovery_inside_tab" => self.render_discovery_inside_tab(ui),
            "status_tab" => self.render_status_tab(ui),
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
    // TODO (chore): Order render functions by default alignment

    fn render_general_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add this computer").clicked() {
                add_localhost_pc(self);
            }
            if ui.button("Add a new device").clicked() {
                self.ui_state.add_new_device_window_state.open = true;
            }
        });
    }

    fn render_topology_overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.add(
            &mut egui_graphs::GraphView::new(
                &mut self.app_state.network_topology.graph.lock().unwrap(),
            )
            .with_styles(&EGUI_GRAPH_SETTINGS_STYLE)
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
            let egui_graphs::Change::Node(node) = change else { continue; };
            let egui_graphs::ChangeNode::Clicked { id: node_id } = node else { continue; };
            let graph_lock = self.app_state.network_topology.graph.lock().unwrap();
            let Some(node) = graph_lock.node_weight(node_id) else { continue; };
            // TODO: Open windows with device and it's settings.
            info!("{:?}", node)
        }
    }

    fn render_discovery_inside_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Scan IP Range").clicked() {
                self.ui_state.scan_ip_range_window_state.open = true;
            }
        });
    }

    fn render_status_tab(&mut self, ui: &mut egui::Ui) {
        let mut status_info_lock = self.app_state.status_info.lock().unwrap();
        ui.label(status_info_lock.text.clone());
        if status_info_lock.scroll_on_next_render {
            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            status_info_lock.scroll_on_next_render = false;
        }
    }
}
