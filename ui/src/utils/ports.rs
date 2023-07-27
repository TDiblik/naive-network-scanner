use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
};

pub fn is_port_open_using_tcp_stream(
    ip: IpAddr,
    port: u16,
    connection_timeout_ms: u64,
    should_banner_grab: bool,
    should_fuzz: bool,
    read_write_timeout_ms: u64,
) -> anyhow::Result<bool> {
    let target = SocketAddr::new(ip, port);
    match TcpStream::connect_timeout(&target, Duration::from_millis(connection_timeout_ms)) {
        Ok(mut connected_socket) => {
            if !should_banner_grab && !should_fuzz {
                return Ok(true);
            }

            println!("TCP Port {} is open", port);
            connected_socket
                .set_write_timeout(Some(Duration::from_millis(read_write_timeout_ms)))?;
            _ = connected_socket
                .set_read_timeout(Some(Duration::from_millis(read_write_timeout_ms)));

            // grab banner
            let mut buffer = Vec::new();
            connected_socket.read_to_end(&mut buffer);
            println!(
                "{:?}",
                buffer
                    .iter()
                    .map(|s| char::from_u32((*s).into()).unwrap())
                    .collect::<String>()
            );

            // try send random data and grab again
            connected_socket.write_all(b"info\n");
            connected_socket.write_all(b"help\n");
            connected_socket.write_all(b"version\n");
            let mut buffer = Vec::new();
            connected_socket.read_to_end(&mut buffer);
            println!(
                "{:?}",
                buffer
                    .iter()
                    .map(|s| char::from_u32((*s).into()).unwrap())
                    .collect::<String>()
            );

            true
        }
        Err(e) => false,
    }
}

fn fuzz_port(socket: &mut TcpStream, port: u16) {}
