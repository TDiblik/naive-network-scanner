use log::info;
use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences};
use pnet::packet::icmp::IcmpTypes;
use std::net::IpAddr;

use crate::{
    app::{
        network_topology::{
            NetworkTopology, NetworkTopologyEdge, NetworkTopologyGraph, NetworkTopologyGraphNode,
            NetworkTopologyNode, PortInfo,
        },
        workspace_models::{AppState, StatusInfoRef, StatusMessage},
    },
    utils::icmp::send_icmp_echo_request_ping,
};

use super::{constants::ALL_COMMON_PORTS, ports::is_port_open_using_tcp_stream};

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
        let localhost_node_index = match NetworkTopology::get_localhosts_node(&mut graph_ref) {
            Some(s) => Some(s.0),
            None => None,
        };

        if reset_connectivity_status {
            if let Some(localhost) = localhost_node_index {
                NetworkTopology::remove_edges_from_node(&mut graph_ref, localhost);
            }
        }

        for ip in ips_to_ping {
            unreachable_ips.push(ip);
            let answ = send_icmp_echo_request_ping(ip, ping_timeout_ms, ping_checkup_ms);
            if answ.is_err() {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Err(format!(
                        "send_icmp_echo_request_ping returned an error => {:?}",
                        answ
                    )),
                );
                continue;
            }
            let Some(answ) = answ.unwrap() else {
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

            let target_node_index = match NetworkTopology::get_node_by_ip(&mut graph_ref, ip) {
                Some((node_index, _)) => node_index,
                None => NetworkTopology::add_node(
                    &mut graph_ref,
                    NetworkTopologyNode::new(ip, "".to_string(), None),
                    None,
                )
                .unwrap(), // safe to unwrape, since we're 100% sure the node does not exist yet.
            };
            if let Some(localhost) = localhost_node_index {
                if !graph_ref
                    .lock()
                    .unwrap()
                    .contains_edge(localhost, target_node_index)
                {
                    NetworkTopology::add_edge(
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

pub fn update_hostname_list(
    mut graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfoRef,
    ips: Vec<IpAddr>,
) {
    std::thread::spawn(move || {
        let nodes_to_test = NetworkTopology::get_all_nodes_except_localhost(&mut graph_ref)
            .iter()
            .filter(|s| ips.contains(&s.1.data().unwrap().ip))
            .cloned()
            .collect::<Vec<NetworkTopologyGraphNode>>();

        if nodes_to_test.is_empty() {
            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(
                    "Didn't find intercept of ips and nodes in graph. Not performing hostname resolution against graph.".to_owned(),
                ),
            );
            return;
        }
        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info(format!(
                "Initiating addr lookup against {} hosts.",
                nodes_to_test.len()
            )),
        );
        info!(
            "Starting addr lookup on following nodes: {:?}",
            nodes_to_test
        );

        for node in nodes_to_test {
            let ip_to_test = node.1.data().unwrap().ip;
            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(format!(
                    "Performing DNS lookup-up for hostname for {}",
                    ip_to_test
                )),
            );

            let Ok(new_hostname) = dns_lookup::lookup_addr(&ip_to_test) else {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Info(format!("Unable to determine hostname for {}", ip_to_test)),
                );
                continue;
            };

            let mut graph_lock = graph_ref.lock().unwrap();
            let node_to_update = graph_lock.node_weight_mut(node.0).unwrap();
            let mut new_data = node_to_update.data().unwrap().clone();
            new_data.hostname = new_hostname.clone();
            node_to_update.set_data(Some(new_data));
            drop(graph_lock);

            NetworkTopology::update_node_label(&mut graph_ref, node.0);
            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(format!(
                    "Hostname for {} is \"{}\"",
                    ip_to_test, new_hostname
                )),
            );
        }

        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info("Finished addr lookup.".to_string()),
        );
    });
}

// TODO: Implement option for multi threading
// TODO: Implement option to change pc mac address for each ping
pub type Port = u16;
pub type BannerGrabResult = Option<String>;
#[derive(Debug, Clone)]
pub struct FuzzingResult {
    pub command: String,
    pub result: String,
    pub result_raw: Vec<u8>,
}
pub type FuzzingResults = Option<Vec<FuzzingResult>>;
pub struct ScanIpPortsConfig {
    pub connection_timeout_ms: u64,
    pub should_banner_grab: bool,
    pub should_fuzz: bool,
    pub read_write_timeout_ms: u64,
}
pub fn scap_ip_ports(
    graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfoRef,
    ip: IpAddr,
    ports: Vec<Port>,
    node_index: NodeIndex,
    config: ScanIpPortsConfig,
) {
    std::thread::spawn(move || {
        let mut reachable_ports = vec![];

        for port in ports {
            let port_info_raw = is_port_open_using_tcp_stream(ip, port, &config);
            if !port_info_raw.0 {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Info(format!("Port {port} is unreachable.")),
                );
                continue;
            }

            let (possible_port_service, possible_service_usefull_info) =
                recognize_port_service(&port, &port_info_raw.1, &port_info_raw.2);

            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Info(format!(
                    "Port {port} is reachable, possible service guess: {possible_port_service}."
                )),
            );
            if config.should_banner_grab {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Info(format!(
                        "Port {port} - banner grab result: {:?}",
                        port_info_raw.1
                    )),
                );
            }
            if config.should_fuzz {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Info(format!(
                        "Port {port} - fuzzing results: {:?}",
                        port_info_raw.2
                    )),
                );
            }

            reachable_ports.push(PortInfo::new(
                port,
                port_info_raw.1.clone(),
                port_info_raw.2.clone(),
                possible_port_service,
                possible_service_usefull_info,
            ));
        }

        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info(format!(
                "Finished scanning. Found {} reachable ports ({}). Proceeding to update the node...",
                reachable_ports.len(),
                reachable_ports
                    .iter()
                    .map(|s| s.number.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
        );

        let mut graph_lock = graph_ref.lock().unwrap();
        let node_to_update = graph_lock.node_weight_mut(node_index);
        if node_to_update.is_none() {
            AppState::log_to_status_generic(
                &status_info_ref,
                StatusMessage::Warn("Unable to update the node after port scan. The results got destroyed. Sorry :(".to_string()),
            );
            return;
        }
        let node_to_update = node_to_update.unwrap();
        let mut new_data = node_to_update.data().unwrap().clone();
        for opened_port in reachable_ports {
            new_data.opened_ports.push(opened_port);
        }
        node_to_update.set_data(Some(new_data));
        drop(graph_lock);
        AppState::log_to_status_generic(
            &status_info_ref,
            StatusMessage::Info("Successfully updated the data on the node.".to_string()),
        );
    });
}

#[allow(unused_variables)]
fn recognize_port_service(
    port: &Port,
    banner: &BannerGrabResult,
    fuzzing_results: &FuzzingResults,
) -> (String, Option<String>) {
    let possible_port = ALL_COMMON_PORTS.iter().find(|s| s.0 == *port);

    // TODO: Implement recognition based on banner (and/or) fuzzing results. Then remove the [allow(unused_variables)]

    possible_port
        .map(|s| (s.1.to_string(), Some(s.2.to_string())))
        .unwrap_or(("unknown".to_string(), None))
}
