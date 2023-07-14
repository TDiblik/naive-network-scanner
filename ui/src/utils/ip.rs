use std::str::FromStr;

use log::debug;

use crate::{
    app::{
        network_topology::NetworkTopologyGraph,
        workspace_models::{AppState, StatusInfoRef, StatusMessage},
    },
    utils::icmp::send_icmp_echo_request_ping,
};

pub struct ScanningOptions {
    // TODO: Implemenet lol
    use_multithreading: bool,
    multithreading_number_of_threds: u8,
}

use pnet::packet::icmp::IcmpTypes;
pub fn scan_ip_range(
    mut graph_ref: NetworkTopologyGraph,
    status_info_ref: StatusInfoRef,
    // scanning_options: ScanningOptions,
) {
    std::thread::spawn(move || {
        let range_to_ping = ipnet::IpNet::from_str("192.168.0.0/24").unwrap();
        dbg!(&range_to_ping.hosts());

        let mut number_of_hosts = 0;
        for host in range_to_ping.hosts() {
            let Ok(answ) = send_icmp_echo_request_ping(host) else {
                AppState::log_to_status_generic(&status_info_ref, StatusMessage::Err("send_icmp_echo_request_ping returned error. Check logs for more info.".to_owned()));
                continue;
            };
            let Some(answ) = answ else {
                AppState::log_to_status_generic(&status_info_ref, StatusMessage::Warn(format!("{} ping timedout", host)));
                dbg!(answ);
                continue;
            };

            // dbg!(&answ.addr);
            // dbg!(answ.icmp_type == IcmpTypes::EchoReply);
            if answ.icmp_type == IcmpTypes::EchoReply {
                AppState::log_to_status_generic(
                    &status_info_ref,
                    StatusMessage::Info(format!("{} is reachable", host)),
                );
                number_of_hosts += 1;
            }
        }
        dbg!(number_of_hosts);

        // for host in range_to_ping.hosts() {
        //     debug!("Testing: {:?}", host);
        //     // TODO: Refactor into a function that returns bool and does not leak any implementation details.
        //     // TODO: The underlying library is slow as fuck. Replace asap.
        //     if let Ok(reply) = ping_rs::send_ping(
        //         &host,
        //         std::time::Duration::from_secs(5),
        //         &[0],
        //         Some(&ping_rs::PingOptions {
        //             ttl: 128,
        //             dont_fragment: true,
        //         }),
        //     ) {
        //         debug!("Ping {:?}", reply);
        //         NetworkTopology::add_node_generic(
        //             &mut graph_ref,
        //             NetworkTopologyNode::new(host, "".to_string()),
        //             None,
        //         );
        //     }
        // }
        debug!("Finished!");
    });
}
