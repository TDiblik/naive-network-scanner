use log::info;
use petgraph::visit::IntoNodeReferences;
use pnet::packet::icmp::IcmpTypes;
use std::net::IpAddr;

use crate::{
    app::{
        network_topology::{
            NetworkTopology, NetworkTopologyEdge, NetworkTopologyGraph, NetworkTopologyNode,
        },
        workspace_models::{AppState, StatusInfoRef, StatusMessage},
    },
    utils::icmp::send_icmp_echo_request_ping,
};

// TODO: Implement option for multi threading
// TODO: Implement option to change pc mac address for each ping
pub fn ping_ip_list(
    mut graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfoRef,
    ips_to_ping: Vec<IpAddr>,
    ping_timeout_ms: u64,
    ping_checkup_ms: u64,
    remove_connectivity_status_when_unreachable: bool,
    reset_connectivity_status: bool,
) {
    if ips_to_ping.is_empty() {
        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info("Didn't receive any ips to ping. Not performing ping.".to_owned()),
        );
        return;
    }
    AppState::log_to_status_generic(
        &status_info_ref,
        StatusMessage::Info(format!(
            "Initiating ip ping against {} hosts.",
            ips_to_ping.len()
        )),
    );
    info!("Starting ip scan on following ips: {:?}", ips_to_ping);

    std::thread::spawn(move || {
        let mut reachable_ips = vec![];
        let mut unreachable_ips = vec![];
        let localhost_node_index =
            match NetworkTopology::get_localhosts_node_generic(&mut graph_ref) {
                Some(s) => Some(s.0),
                None => None,
            };

        if reset_connectivity_status {
            if let Some(localhost) = localhost_node_index {
                NetworkTopology::remove_edges_from_node_generic(&mut graph_ref, localhost);
            }
        }

        for ip in ips_to_ping {
            unreachable_ips.push(ip);
            let Ok(answ) = send_icmp_echo_request_ping(ip, ping_timeout_ms, ping_checkup_ms) else {
                AppState::log_to_status_generic(&status_info_ref, StatusMessage::Err("send_icmp_echo_request_ping returned error. Check logs for more info.".to_owned()));
                continue;
            };
            let Some(answ) = answ else {
                AppState::log_to_status_generic(&status_info_ref, StatusMessage::Warn(format!("{} ping timed out", ip)));
                continue;
            };
            if answ.icmp_type == IcmpTypes::DestinationUnreachable {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Warn(format!("{} is unreachable", ip)),
                );
                continue;
            }
            if answ.icmp_type != IcmpTypes::EchoReply {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Warn(format!(
                        "{} ping returned {{ adr \"{}\" , icmp type \"{}\" , icmp code \"{}\" }}. Likely unreacheable.",
                        ip, answ.addr, answ.icmp_type.0, answ.icmp_code.0
                    )),
                );
                continue;
            }

            let target_node_index =
                match NetworkTopology::get_node_by_ip_generic(&mut graph_ref, ip) {
                    Some((node_index, _)) => node_index,
                    None => NetworkTopology::add_node_generic(
                        &mut graph_ref,
                        NetworkTopologyNode::new(ip, "".to_string()),
                        None,
                    ),
                };
            if let Some(localhost) = localhost_node_index {
                if !graph_ref
                    .lock()
                    .unwrap()
                    .contains_edge(localhost, target_node_index)
                {
                    NetworkTopology::add_edge_generic(
                        &mut graph_ref,
                        localhost,
                        target_node_index,
                        NetworkTopologyEdge::default(),
                    );
                }
            }

            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(format!("{} is reachable", ip)),
            );
            reachable_ips.push(ip);
            unreachable_ips.pop();
        }

        if remove_connectivity_status_when_unreachable {
            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info("Removing connectivity status to unreachable IPs".to_owned()),
            );
            if let Some(localhost) = localhost_node_index {
                let mut graph_lock = graph_ref.lock().unwrap();
                for unreachable_ip in unreachable_ips {
                    if let Some((unreachable_ip_index, _)) = graph_lock
                        .node_references()
                        .find(|s| s.1.data().unwrap().ip == unreachable_ip)
                    {
                        if let Some(edge) = graph_lock.find_edge(localhost, unreachable_ip_index) {
                            graph_lock.remove_edge(edge);
                        }
                    }
                }
            }
        }

        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info(format!(
                "Finished scanning. Found {} reachable IPs",
                reachable_ips.len()
            )),
        );
    });
}
