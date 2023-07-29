use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
};

use log::{error, info};

use crate::utils::constants::PORT_FUZZING_COMMANDS;

use super::ip::{FuzzingResults, Port, ScanIpPortsConfig};

// bool => whether the connection was successfully established
// Option<String> => banner
// FuzzingResults => output of fuzzing
pub fn is_port_open_using_tcp_stream(
    ip: IpAddr,
    port: Port,
    config: &ScanIpPortsConfig,
) -> (bool, Option<String>, FuzzingResults) {
    let target = SocketAddr::new(ip, port);
    match TcpStream::connect_timeout(&target, Duration::from_millis(config.connection_timeout_ms)) {
        Ok(mut connected_socket) => {
            if !config.should_banner_grab && !config.should_fuzz {
                return (true, None, None);
            }

            if let Err(e) = connected_socket
                .set_read_timeout(Some(Duration::from_millis(config.read_write_timeout_ms)))
            {
                error!(
                    "An error occurred while setting connected socket read timeout: {}",
                    e
                );
                return (true, None, None);
            }
            if let Err(e) = connected_socket
                .set_write_timeout(Some(Duration::from_millis(config.read_write_timeout_ms)))
            {
                error!(
                    "An error occurred while setting connected socket write timeout: {}",
                    e
                );
                return (true, None, None);
            }

            // grab banner
            let mut banner: String = String::new();
            if config.should_banner_grab {
                let mut buffer = Vec::new();
                if let Err(e) = connected_socket.read_to_end(&mut buffer) {
                    error!(
                        "An error occurred while setting connected socket write timeout: {}",
                        e
                    );
                }
                banner = buffer
                    .iter()
                    .map(|s| char::from_u32((*s).into()).unwrap())
                    .collect::<String>();
            }

            // fuzzing
            let mut fuzzing_results = vec![];
            if config.should_fuzz {
                for command in PORT_FUZZING_COMMANDS {
                    connected_socket.write_all(command);
                    let mut buffer = Vec::new();
                    connected_socket.read_to_end(&mut buffer);
                    fuzzing_results.push(
                        buffer
                            .iter()
                            .map(|s| char::from_u32((*s).into()).unwrap())
                            .collect::<String>(),
                    );
                }
            }

            (true, Some(banner), Some(fuzzing_results))
        }
        Err(e) => {
            info!("Unable to connect to port {}, error: {}", port, e);
            (false, None, None)
        }
    }
}
