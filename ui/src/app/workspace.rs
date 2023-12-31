use eframe::egui;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::{
    app::modals::device_window_state::DeviceWindowState,
    utils::{
        general::add_localhost_pc,
        icmp::{
            DEFAULT_PING_ENSURED_CONNECTIVITY_CHECKUP_MS,
            DEFAULT_PING_ENSURED_CONNECTIVITY_TIMEOUT_MS,
        },
        ip::{ping_ip_list, update_hostname_list},
    },
};

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
                device_window_states: vec![],
            },
        };

        context.app_state.log_to_status(StatusMessage::Info(format!(
            "Initialized new workspace {}!",
            id
        )));

        let logging_thread_arc = Arc::clone(&context.app_state.status_info);
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(5));

            let mut status_info_lock = logging_thread_arc.lock().unwrap();
            let text_to_append = status_info_lock.text_to_batch.clone();
            if !text_to_append.is_empty() {
                if status_info_lock.text_to_render.len() > 75_000 {
                    status_info_lock.text_to_render = "".to_owned();
                }
                status_info_lock.text_to_render.push_str(&text_to_append);
                status_info_lock.scroll_on_next_render = true;
                status_info_lock.text_to_batch = "".to_owned();
            }
        });

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
                egui::menu::bar(ui, |ui| {
                    FileMenuButton::render(ui);
                    ViewMenuButton::render(ui, self);
                });

                AddNewDeviceWindowState::render(ctx, &mut self.context);
                GenericInfoWindowState::render(
                    ctx,
                    &mut self.context.ui_state.add_this_computer_window_state,
                );
                ScanIpRangeWindowState::render(ctx, &mut self.context);
                for i in 0..self.context.ui_state.device_window_states.len() {
                    DeviceWindowState::render(ctx, &mut self.context, i);
                }

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
            "discovery_shared_tab" => self.render_discovery_shared_tab(ui),
            "discovery_inside_tab" => self.render_discovery_inside_tab(ui),
            "topology_overview_tab" => self.render_topology_overview_tab(ui),
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

// TODO (chore): Order render functions by default alignment
impl WorkspaceContext {
    fn render_general_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add this computer").clicked() {
                add_localhost_pc(self);
            }
            if ui.button("Add a new device").clicked() {
                self.ui_state.add_new_device_window_state.open = true;
            }
            if ui
                .button("Refresh connection status to all devices (soft)")
                .clicked()
            {
                ping_ip_list(
                    Arc::clone(&self.app_state.network_topology.graph),
                    Arc::clone(&self.app_state.status_info),
                    NetworkTopology::get_all_ips_except_localhost(
                        &mut self.app_state.network_topology.graph,
                    ),
                    DEFAULT_PING_ENSURED_CONNECTIVITY_TIMEOUT_MS,
                    DEFAULT_PING_ENSURED_CONNECTIVITY_CHECKUP_MS,
                    true,
                    false,
                );
            }
            if ui
                .button("Refresh connection status to all devices (hard)")
                .clicked()
            {
                ping_ip_list(
                    Arc::clone(&self.app_state.network_topology.graph),
                    Arc::clone(&self.app_state.status_info),
                    NetworkTopology::get_all_ips_except_localhost(
                        &mut self.app_state.network_topology.graph,
                    ),
                    DEFAULT_PING_ENSURED_CONNECTIVITY_TIMEOUT_MS,
                    DEFAULT_PING_ENSURED_CONNECTIVITY_CHECKUP_MS,
                    false,
                    true,
                );
            }
        });
    }

    fn render_discovery_shared_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Retrieve hostnames (all devices)").clicked() {
                update_hostname_list(
                    Arc::clone(&self.app_state.network_topology.graph),
                    Arc::clone(&self.app_state.status_info),
                    NetworkTopology::get_all_nodes_except_localhost(
                        &mut self.app_state.network_topology.graph,
                    )
                    .iter()
                    .map(|s| s.1.data().unwrap().ip)
                    .collect(),
                );
            }
        });
    }

    fn render_discovery_inside_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Scan IP Range").clicked() {
                self.ui_state.scan_ip_range_window_state.open = true;
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
            let node_data = node.data().unwrap();

            let mut should_add_new_window = true;

            if self
                .ui_state
                .device_window_states
                .iter()
                .filter(|s| s.ip() == node_data.ip)
                .any(|s| !s.is_open())
            {
                self.ui_state
                    .device_window_states
                    .iter_mut()
                    .filter(|s| s.ip() == node_data.ip)
                    .for_each(|s| s.show());
                should_add_new_window = false;
            }

            if should_add_new_window {
                let mut new_window = DeviceWindowState::new(node_data.ip, node_id);
                new_window.show();
                self.ui_state.device_window_states.push(new_window);
            }
        }
    }

    fn render_status_tab(&mut self, ui: &mut egui::Ui) {
        let mut status_info_lock = self.app_state.status_info.lock().unwrap();
        ui.label(status_info_lock.text_to_render.clone());
        if status_info_lock.scroll_on_next_render {
            status_info_lock.scroll_on_next_render = false;
            drop(status_info_lock);
            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
        }
    }
}
