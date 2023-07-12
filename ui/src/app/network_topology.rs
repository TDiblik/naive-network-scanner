use crossbeam::channel::{unbounded, Receiver, Sender};
use eframe::epaint::{Color32, Vec2};
use egui_graphs::{
    to_input_graph, Change, Graph, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use local_ip_address::local_ip;
use log::{info, warn};
use petgraph::{stable_graph::StableGraph, visit::IntoNodeReferences, Directed};
use rand::Rng;
use std::net::IpAddr;

lazy_static! {
    pub static ref EGUI_GRAPH_SETTINGS_STYLE: SettingsStyle =
        SettingsStyle::new().with_labels_always(true);
    pub static ref EGUI_GRAPH_SETTINGS_INTERACTIONS: SettingsInteraction =
        SettingsInteraction::new()
            .with_clicking_enabled(true)
            .with_dragging_enabled(true)
            .with_selection_enabled(false);
    pub static ref EGUI_GRAPH_SETTINGS_NAVIGATION: SettingsNavigation = SettingsNavigation::new()
        .with_fit_to_screen_enabled(false)
        .with_zoom_and_pan_enabled(true)
        .with_screen_padding(0.25)
        .with_zoom_speed(0.1);
}

#[derive(Debug, Clone)]
pub struct NetworkTopologyNode {
    pub id: IpAddr, // ip serves as id, since you cannot have multiple in one network
    pub is_my_pc: bool,
}
impl NetworkTopologyNode {
    pub fn new(ip: IpAddr) -> Self {
        Self::new_internal(ip, false)
    }

    pub fn new_my_pc() -> anyhow::Result<Self> {
        let my_local_ip = local_ip()?;
        Ok(Self::new_internal(my_local_ip, true))
    }

    fn new_internal(ip: IpAddr, is_my_pc: bool) -> Self {
        Self { id: ip, is_my_pc }
    }
}

pub type NetworkTopologyGraph = Graph<NetworkTopologyNode, (), Directed>;
pub struct NetworkTopology {
    pub graph: NetworkTopologyGraph,
    pub graph_changes_sender: Sender<Change>,
    pub graph_changes_receiver: Receiver<Change>,
}

#[cfg(debug_assertions)]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<NetworkTopologyNode, ()> = StableGraph::default();
        let graph = to_input_graph(&graph_base);
        let (graph_changes_sender, graph_changes_receiver) = unbounded();
        let mut new_topology = Self {
            graph,
            graph_changes_sender,
            graph_changes_receiver,
        };

        if let Ok(new_my_pc_node) = NetworkTopologyNode::new_my_pc() {
            new_topology.add_node(new_my_pc_node, Some(Vec2::new(0.0, 0.0)));
        } else {
            warn!("Unable to create new_my_pc_node for development purposes. This means that local_ip function likelly does not work atm.");
        }
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.1"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.1"),
            ),
            Some(Vec2::new(-200.0, 0.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.2"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.2"),
            ),
            Some(Vec2::new(0.0, -200.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.3"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.3"),
            ),
            Some(Vec2::new(200.0, 0.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.4"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.4"),
            ),
            Some(Vec2::new(0.0, 200.0)),
        );
        new_topology
            .graph
            .node_references()
            .for_each(|s| info!("{:?}", s));

        new_topology
    }
}

#[cfg(not(debug_assertions))]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<NetworkTopologyNode, ()> = StableGraph::default();
        let graph = to_input_graph(&graph_base);

        let (graph_changes_sender, graph_changes_receiver) = unbounded();
        let mut new_topology = Self {
            graph,
            graph_changes_sender,
            graph_changes_receiver,
        };

        // TODO: Currentlly, egui_graph crashes for some reason, when I create new graph without nodes and then try to add some, so I decided to create dummy node. Try removing this line when it hits 1.0 / open issue / open PR. I don't want to deal with it rn.
        new_topology.add_node(
            NetworkTopologyNode::new("0.0.0.0".parse().unwrap()),
            Some(Vec2::new(0.0, 0.0)),
        );

        new_topology
    }
}

impl NetworkTopology {
    pub fn add_node(&mut self, new_topology_node: NetworkTopologyNode, location: Option<Vec2>) {
        let mut rng = rand::thread_rng(); // TODO: could be optimized ? Idk if it's creating a new instance every time :/
        let spawn_location = location.unwrap_or(Vec2::new(
            rng.gen_range(-200.0..200.0),
            rng.gen_range(-200.0..200.0),
        ));

        let new_node = egui_graphs::Node::new(spawn_location, new_topology_node.clone())
            .with_label(new_topology_node.id.to_string())
            .with_color(if new_topology_node.is_my_pc {
                Color32::from_rgb(238, 108, 77) // TODO: Decide between (238, 108, 77) OR (152, 193, 217) OR (61, 90, 128)
            } else {
                Color32::from_rgb(200, 200, 200)
            });
        self.graph.add_node(new_node);
    }
}
