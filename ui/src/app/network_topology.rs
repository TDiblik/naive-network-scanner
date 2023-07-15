use crossbeam::channel::{unbounded, Receiver, Sender};
use eframe::epaint::{Color32, Vec2};
use egui_graphs::{
    to_input_graph, Change, Graph, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use local_ip_address::local_ip;
use log::{debug, warn};
use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoNodeReferences},
    Directed,
};
use rand::Rng;
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

lazy_static! {
    pub static ref EGUI_GRAPH_SETTINGS_STYLE: SettingsStyle = SettingsStyle::new()
        .with_labels_always(true)
        .with_edge_radius_weight(0.0);
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
    pub ip: IpAddr, // ip == id ; has to be unique
    pub notes: String,
    pub is_localhost: bool, // True => node is a machine that's running this program
}
impl NetworkTopologyNode {
    pub fn new(ip: IpAddr, notes: String) -> Self {
        Self::new_internal(ip, notes, false)
    }

    pub fn new_my_pc() -> anyhow::Result<Self> {
        let my_local_ip = local_ip()?;
        Ok(Self::new_internal(
            my_local_ip,
            "This is the current pc.".to_string(),
            true,
        ))
    }

    fn new_internal(ip: IpAddr, notes: String, is_localhost: bool) -> Self {
        Self {
            ip,
            notes,
            is_localhost,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkTopologyEdge {}

pub type NetworkTopologyGraph =
    Arc<Mutex<Graph<NetworkTopologyNode, NetworkTopologyEdge, Directed>>>;
pub struct NetworkTopology {
    pub graph: NetworkTopologyGraph,
    pub graph_changes_sender: Sender<Change>,
    pub graph_changes_receiver: Receiver<Change>,
}

#[cfg(debug_assertions)]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<NetworkTopologyNode, NetworkTopologyEdge> =
            StableGraph::default();
        let graph = Arc::new(Mutex::new(to_input_graph(&graph_base)));
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
                "".to_string(),
            ),
            Some(Vec2::new(-200.0, 0.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.2"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.2"),
                "".to_string(),
            ),
            Some(Vec2::new(0.0, -200.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.3"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.3"),
                "".to_string(),
            ),
            Some(Vec2::new(200.0, 0.0)),
        );
        new_topology.add_node(
            NetworkTopologyNode::new(
                "192.168.0.4"
                    .parse()
                    .expect("Unable to parse valid ip 192.168.0.4"),
                "".to_string(),
            ),
            Some(Vec2::new(0.0, 200.0)),
        );
        new_topology
            .graph
            .lock()
            .unwrap()
            .node_references()
            .for_each(|s| debug!("{:?}", s));

        new_topology
    }
}

#[cfg(not(debug_assertions))]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<NetworkTopologyNode, NetworkTopologyEdge> =
            StableGraph::default();
        let graph = Arc::new(Mutex::new(to_input_graph(&graph_base)));

        let (graph_changes_sender, graph_changes_receiver) = unbounded();
        let mut new_topology = Self {
            graph,
            graph_changes_sender,
            graph_changes_receiver,
        };

        // TODO: Currentlly, egui_graph crashes for some reason, when I create new graph without nodes and then try to add some, so I decided to create dummy node. Try removing this line when it hits 1.0 / open issue / open PR. I don't want to deal with it rn.
        new_topology.add_node(
            NetworkTopologyNode::new(
                "0.0.0.0".parse().expect("Unable to parse valid ip 0.0.0.0"),
                "".to_string(),
            ),
            Some(Vec2::new(0.0, 0.0)),
        );

        new_topology
    }
}

pub type MaybeNetworkTopologyGraphNode =
    Option<(NodeIndex, egui_graphs::Node<NetworkTopologyNode>)>;
#[allow(dead_code)]
impl NetworkTopology {
    pub fn get_localhost_node(&mut self) -> MaybeNetworkTopologyGraphNode {
        Self::get_localhosts_node_generic(&mut self.graph)
    }

    pub fn get_localhosts_node_generic(
        graph: &mut NetworkTopologyGraph,
    ) -> MaybeNetworkTopologyGraphNode {
        let graph_lock = graph.lock().unwrap();
        let (node_index, node_value) = graph_lock
            .node_references()
            .find(|s| s.1.data().unwrap().is_localhost)?;

        Some((node_index, node_value.clone()))
    }

    pub fn get_node_by_ip(&mut self, ip: IpAddr) -> MaybeNetworkTopologyGraphNode {
        Self::get_node_by_ip_generic(&mut self.graph, ip)
    }

    pub fn get_node_by_ip_generic(
        graph: &mut NetworkTopologyGraph,
        ip: IpAddr,
    ) -> MaybeNetworkTopologyGraphNode {
        let graph_lock = graph.lock().unwrap();
        let (node_index, node_value) = graph_lock
            .node_references()
            .find(|s| s.1.data().unwrap().ip == ip)?;

        Some((node_index, node_value.clone()))
    }

    pub fn add_node(
        &mut self,
        new_topology_node: NetworkTopologyNode,
        location: Option<Vec2>,
    ) -> Option<NodeIndex> {
        Self::add_node_generic(&mut self.graph, new_topology_node, location)
    }

    pub fn add_node_generic(
        graph: &mut NetworkTopologyGraph,
        new_topology_node: NetworkTopologyNode,
        location: Option<Vec2>,
    ) -> Option<NodeIndex> {
        let mut rng = rand::thread_rng(); // TODO: could be optimized ? Idk if it's creating a new instance every time :/
        let spawn_location = location.unwrap_or(Vec2::new(
            rng.gen_range(-200.0..200.0),
            rng.gen_range(-200.0..200.0),
        ));

        let new_node = egui_graphs::Node::new(spawn_location, new_topology_node.clone())
            .with_label(new_topology_node.ip.to_string())
            .with_color(if new_topology_node.is_localhost {
                Color32::from_rgb(238, 108, 77) // TODO: Decide between (238, 108, 77) OR (152, 193, 217) OR (61, 90, 128)
            } else {
                Color32::from_rgb(200, 200, 200)
            });

        let mut graph_lock = graph.lock().unwrap();
        if graph_lock
            .node_references()
            .any(|s| s.1.data().unwrap().ip == new_topology_node.ip)
        {
            return None;
        }

        Some(graph_lock.add_node(new_node))
        // TODO: Graph should re-zoom to fit all
    }

    pub fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        weight: NetworkTopologyEdge,
    ) -> EdgeIndex {
        Self::add_edge_generic(&mut self.graph, from, to, weight)
    }

    pub fn add_edge_generic(
        graph: &mut NetworkTopologyGraph,
        from: NodeIndex,
        to: NodeIndex,
        weight: NetworkTopologyEdge,
    ) -> EdgeIndex {
        let new_edge = egui_graphs::Edge::new(weight);

        graph.lock().unwrap().add_edge(from, to, new_edge)
    }

    pub fn remove_edges_from_node(&mut self, from: NodeIndex) {
        Self::remove_edges_from_node_generic(&mut self.graph, from)
    }

    pub fn remove_edges_from_node_generic(graph: &mut NetworkTopologyGraph, from: NodeIndex) {
        let mut graph_lock = graph.lock().unwrap();
        for edge in graph_lock
            .edges(from)
            .map(|s| s.id())
            .collect::<Vec<EdgeIndex>>()
        {
            graph_lock.remove_edge(edge);
        }
    }
}
