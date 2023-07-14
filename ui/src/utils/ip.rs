use std::{
    net::IpAddr,
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info};
use petgraph::visit::IntoNodeReferences;

use crate::app::{
    network_topology::{NetworkTopology, NetworkTopologyGraph, NetworkTopologyNode},
    workspace_models::{AppState, StatusInfo},
};

pub struct ScanningOptions {
    // TODO: Implemenet lol
    use_multithreading: bool,
    multithreading_number_of_threds: u8,
}

pub fn scan_ip_range(
    mut graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfo,
    // scanning_options: ScanningOptions,
) {
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
            debug!("Testing: {:?}", host);
            // TODO: Refactor into a function that returns bool and does not leak any implementation details.
            // TODO: The underlying library is slow as fuck. Replace asap.
            if let Ok(reply) = ping_rs::send_ping(
                &host,
                std::time::Duration::from_secs(5),
                &[0],
                Some(&ping_rs::PingOptions {
                    ttl: 128,
                    dont_fragment: true,
                }),
            ) {
                debug!("Ping {:?}", reply);

                // if let Some(existing_node) = graph_ref
                //     .clone()
                //     .lock()
                //     .unwrap()
                //     .node_references()
                //     .find(|s| s.1.data().unwrap().ip == host)
                // {
                //     // graph_ref.lock().unwrap().add_edge(
                //     //     existing_node.0,
                //     //     my_pc_index.clone().unwrap().0,
                //     //     egui_graphs::Edge::new(()),
                //     // );
                //     // debug!("Add edge");
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
        debug!("Finished!");

        AppState::log_to_status_generic(&status_info_ref, "Finished ip scan".to_owned());
    });
}
