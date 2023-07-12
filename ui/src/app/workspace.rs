use eframe::egui;
use log::{info, warn};
use petgraph::visit::IntoNodeReferences;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use super::{
    modals::add_new_device_window::AddNewDeviceWindowState,
    network_topology::{
        NetworkTopology, NetworkTopologyNode, EGUI_GRAPH_SETTINGS_INTERACTIONS,
        EGUI_GRAPH_SETTINGS_NAVIGATION, EGUI_GRAPH_SETTINGS_STYLE,
    },
    workspace_models::{AppState, TabsContext, UIState, WorkspaceContext},
    workspace_tab::{default_tabs, WorkspaceTab},
};

pub struct Workspace {
    // TODO: Allow dead code (id) for now, since I don't want to see the warning. Will be usefull in the future.
    #[allow(dead_code)]
    id: Uuid,
    tabs_context: TabsContext,
    context: WorkspaceContext,
}

impl Workspace {
    pub fn new(id: Uuid) -> Self {
        let tabs_context = default_tabs();
        let context = WorkspaceContext {
            app_state: AppState {
                network_topology: NetworkTopology::default(),
                status_info: Arc::new(Mutex::new("".to_string())),
            },
            ui_state: UIState {
                open_tabs: tabs_context.default_tabs.clone(),
                add_new_device_window_state: AddNewDeviceWindowState::default(),
            },
        };

        context
            .app_state
            .log_to_status(format!("Initialized new workspace {}!", id));
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
                // TODO: Refactor into separate file, so it does not polute workspace.rs
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

                // Modal windows
                let mut show_add_new_device_window =
                    self.context.ui_state.add_new_device_window_state.open;
                if show_add_new_device_window {
                    AddNewDeviceWindowState::render(
                        ctx,
                        &mut show_add_new_device_window,
                        &mut self.context,
                    );
                }
                self.context.ui_state.add_new_device_window_state.open &=
                    show_add_new_device_window;

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
            "meta_tab" => self.render_meta_tab(ui),
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

    fn render_meta_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add this computer").clicked() {
                self.app_state
                    .network_topology
                    .add_node(NetworkTopologyNode::new_my_pc().unwrap(), None);
                // TODO: Graph should re-zoom to fit all
            }
            if ui.button("Add a new device").clicked() {
                self.ui_state.add_new_device_window_state.open = true;
                // TODO: Graph should re-zoom to fit all
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
                let mut graph_ref = Arc::clone(&self.app_state.network_topology.graph);
                let status_info_ref = Arc::clone(&self.app_state.status_info);
                std::thread::spawn(move || {
                    let range_to_ping = ipnet::IpNet::from_str("192.168.0.0/24").unwrap();
                    dbg!(&range_to_ping.hosts());
                    let first = range_to_ping.hosts().clone().next();
                    let last = range_to_ping.hosts().last();
                    AppState::log_to_status_generic(
                        &status_info_ref,
                        format!(
                            "Starting ip scan from {} to {} ({})",
                            first.unwrap(),
                            last.unwrap(),
                            range_to_ping
                        ),
                    );
                    for host in range_to_ping.hosts() {
                        info!("Testing: {:?}", host);
                        if ping::ping(
                            host,
                            Some(std::time::Duration::from_millis(100)),
                            None,
                            None,
                            None,
                            None,
                        )
                        .is_ok()
                        {
                            info!("Found: {:?}", host);
                            AppState::log_to_status_generic(
                                &status_info_ref,
                                format!("IP {} responded to ping", host),
                            );

                            // if let Some(existing_node) = graph_ref
                            //     .clone()
                            //     .lock()
                            //     .unwrap()
                            //     .node_references()
                            //     .find(|s| s.1.data().unwrap().ip == host)
                            // {
                            //     graph_ref.lock().unwrap().add_edge(, b, weight)
                            //     existing_node
                            // } else {
                            //     NetworkTopology::add_node_generic(
                            //         &mut graph_ref,
                            //         NetworkTopologyNode::new(host, "".to_string()),
                            //         None,
                            //     );
                            // };
                            NetworkTopology::add_node_generic(
                                &mut graph_ref,
                                NetworkTopologyNode::new(host, "".to_string()),
                                None,
                            );
                        }
                    }
                    info!("Finished!");
                });
            }
        });
    }

    fn render_status_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(self.app_state.status_info.lock().unwrap().clone());
    }
}
