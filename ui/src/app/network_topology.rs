use crossbeam::channel::{unbounded, Receiver, Sender};
use eframe::epaint::Vec2;
use egui_graphs::{to_input_graph, Change, Graph, SettingsInteraction, SettingsNavigation};
use log::info;
use petgraph::{stable_graph::StableGraph, visit::IntoNodeReferences, Directed};
use rand::Rng;

pub type NetworkTopologyGraph = Graph<(), (), Directed>;

lazy_static! {
    pub static ref EGUI_GRAPH_SETTINGS_INTERACTIONS: SettingsInteraction =
        SettingsInteraction::new()
            .with_clicking_enabled(true)
            .with_dragging_enabled(true)
            .with_selection_enabled(false);
    pub static ref EGUI_GRAPH_SETTINGS_NAVIGATION: SettingsNavigation = SettingsNavigation::new()
        .with_fit_to_screen_enabled(false)
        .with_zoom_and_pan_enabled(true)
        .with_zoom_speed(0.1);
}

pub struct NetworkTopology {
    pub graph: NetworkTopologyGraph,
    pub graph_changes_sender: Sender<Change>,
    pub graph_changes_receiver: Receiver<Change>,
}

#[cfg(debug_assertions)]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<(), ()> = StableGraph::default();
        let graph = to_input_graph(&graph_base);
        let (graph_changes_sender, graph_changes_receiver) = unbounded();
        let mut new = Self {
            graph,
            graph_changes_sender,
            graph_changes_receiver,
        };

        new.add_node(None);
        new.add_node(None);
        new.add_node(None);
        new.graph.node_references().for_each(|s| info!("{:?}", s));
        new
    }
}

#[cfg(not(debug_assertions))]
impl Default for NetworkTopology {
    fn default() -> Self {
        let graph_base: StableGraph<(), ()> = StableGraph::default();
        let graph = to_input_graph(&graph_base);
        let (graph_changes_sender, graph_changes_receiver) = unbounded();
        Self {
            graph,
            graph_changes_sender,
            graph_changes_receiver,
        }
    }
}

impl NetworkTopology {
    pub fn add_node(&mut self, location: Option<Vec2>) {
        let mut rng = rand::thread_rng(); // could be optimized
        let spawn_location = location.unwrap_or(Vec2::new(
            rng.gen_range(0.0..500.0),
            rng.gen_range(0.0..500.0),
        ));
        self.graph
            .add_node(egui_graphs::Node::new(spawn_location, ()));
    }
}
