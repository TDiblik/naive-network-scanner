use std::{
    net::IpAddr,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use log::error;
use rand::random;

use pnet::packet::{
    icmp::{
        echo_reply::EchoReplyPacket,
        echo_request::{IcmpCodes, MutableEchoRequestPacket},
        IcmpCode, IcmpType, IcmpTypes,
    },
    ip::IpNextHeaderProtocols,
    util::checksum,
    Packet,
};
use pnet::transport::icmp_packet_iter;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::{transport_channel, TransportProtocol};

const ICMP_SIZE: usize = 64;

struct PingStatus {
    got_reply: bool,
    reply: Option<EchoReplyInfo>,
    stringified_err: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EchoReplyInfo {
    pub addr: IpAddr,
    pub rtt: Duration,

    pub identifier: u16,
    pub icmp_code: IcmpCode,
    pub icmp_type: IcmpType,

    pub raw_packet: Vec<u8>,
    pub raw_payload: Vec<u8>,
    pub checksum: u16,
}

// Could be optimized by sending multiple pings at once, but I don't really care about perfomance atm
pub fn send_icmp_echo_request_ping(
    address: IpAddr,
    ping_timeout_ms: u64,
    ping_checkup_ms: u64,
) -> anyhow::Result<Option<EchoReplyInfo>> {
    let protocol = Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp));
    let (mut tx, mut rx) = match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return Err(e.into()),
    };

    let mut icmp_header: [u8; ICMP_SIZE] = [0; ICMP_SIZE];
    let icmp_packet = create_icmp_echo_request_packet(&mut icmp_header);
    let sent_at = Arc::new(RwLock::new(Instant::now()));
    tx.send_to(icmp_packet, address)?;

    let status: Arc<Mutex<PingStatus>> = Arc::new(Mutex::new(PingStatus {
        got_reply: false,
        reply: None,
        stringified_err: None,
    }));

    let closure_status = Arc::clone(&status);
    let closure_sent_at = Arc::clone(&sent_at);
    thread::spawn(move || {
        let mut iter = icmp_packet_iter(&mut rx);
        match iter.next() {
            Ok((packet, addr)) => {
                if let Some(echo_reply) = EchoReplyPacket::new(packet.packet()) {
                    let mut new_status = closure_status.lock().unwrap();
                    new_status.reply = Some(EchoReplyInfo {
                        addr,
                        rtt: Instant::now().duration_since(*closure_sent_at.read().unwrap()),
                        identifier: echo_reply.get_identifier(),
                        icmp_code: echo_reply.get_icmp_code(),
                        icmp_type: echo_reply.get_icmp_type(),
                        raw_packet: echo_reply.packet().into(),
                        raw_payload: echo_reply.payload().into(),
                        checksum: echo_reply.get_checksum(),
                    });
                    new_status.got_reply = true;
                }
            }
            Err(e) => {
                error!(
                    "An error occurred while reading icmp echo request ping: {}",
                    e
                );
                closure_status.lock().unwrap().stringified_err = Some(e.to_string());
            }
        }
    });

    let ping_timeout = Duration::from_millis(ping_timeout_ms);
    let ping_checkup = Duration::from_millis(ping_checkup_ms);
    loop {
        thread::sleep(ping_checkup);

        let status_lock = status.lock().unwrap();
        if let Some(err_text) = status_lock.stringified_err.clone() {
            return Err(anyhow!(err_text));
        }

        if status_lock.got_reply || Instant::now() - *sent_at.read().unwrap() > ping_timeout {
            return Ok(status_lock.reply.clone());
        }
    }
}

fn create_icmp_echo_request_packet(icmp_header: &mut [u8]) -> MutableEchoRequestPacket<'_> {
    let mut icmp_packet = MutableEchoRequestPacket::new(icmp_header).unwrap();
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_icmp_code(IcmpCodes::NoCode);
    icmp_packet.set_identifier(random::<u16>());
    icmp_packet.set_sequence_number(1);
    icmp_packet.set_checksum(checksum(icmp_packet.packet(), 1));

    icmp_packet
}
