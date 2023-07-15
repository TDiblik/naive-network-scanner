use log::info;
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
pub fn ping_ip_range(
    mut graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfoRef,
    ips_to_ping: Vec<IpAddr>,
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
        let mut number_of_hosts = 0;
        let localhost_node_index =
            match NetworkTopology::get_localhosts_node_generic(&mut graph_ref) {
                Some(s) => Some(s.0),
                None => None,
            };

        if let Some(localhost) = localhost_node_index {
            NetworkTopology::remove_edges_from_node_generic(&mut graph_ref, localhost);
        }

        for ip in ips_to_ping {
            let Ok(answ) = send_icmp_echo_request_ping(ip) else {
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
                NetworkTopology::add_edge_generic(
                    &mut graph_ref,
                    localhost,
                    target_node_index,
                    NetworkTopologyEdge::default(),
                );
            }

            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(format!("{} is reachable", ip)),
            );
            number_of_hosts += 1;
        }
        dbg!(number_of_hosts);
    });
}
