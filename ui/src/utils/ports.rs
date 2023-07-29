use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    thread,
    time::Duration,
};

use log::{error, info};

use crate::utils::constants::PORT_FUZZING_COMMANDS;

use super::ip::{BannerGrabResult, FuzzingResults, Port, ScanIpPortsConfig};

// bool => whether the connection was successfully established
pub fn is_port_open_using_tcp_stream(
    ip: IpAddr,
    port: Port,
    config: &ScanIpPortsConfig,
) -> (bool, BannerGrabResult, FuzzingResults) {
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
                let grabbed_output =
                    read_everything_from_socket(&connected_socket, config.read_write_timeout_ms);
                match grabbed_output {
                    Ok(grabbed_output) => {
                        banner = socket_vec_buffer_to_string(grabbed_output);
                    }
                    Err(e) => {
                        error!("An error occurred while banner grabbing: {}", e);
                    }
                }
            }

            // fuzzing
            let mut fuzzing_results = vec![];
            if config.should_fuzz {
                for command in PORT_FUZZING_COMMANDS {
                    if let Err(e) = connected_socket.write_all(command) {
                        error!(
                            "An error occurred while writing \"{}\" to socket: {}",
                            socket_buffer_to_string(command),
                            e
                        );
                        continue;
                    }
                    if let Err(e) = connected_socket.flush() {
                        error!(
                            "An error occurred while flushing after \"{}\" to socket. Continuing since this could severy corrupt output: {}",
                            socket_buffer_to_string(command),
                            e
                        );
                        continue;
                    }

                    let grabbed_output = read_everything_from_socket(
                        &connected_socket,
                        config.read_write_timeout_ms,
                    );
                    match grabbed_output {
                        Ok(grabbed_output) => {
                            fuzzing_results.push(socket_vec_buffer_to_string(grabbed_output));
                        }
                        Err(e) => {
                            error!("An error occurred while reading from socket after sending fuzz command \"{}\": {}", socket_buffer_to_string(command), e);
                        }
                    }
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

fn read_everything_from_socket(
    mut socket: &TcpStream,
    read_timeout_ms: u64,
) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut buffer = [0; 1024];
    let mut potentionally_finished = false;
    loop {
        match socket.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Append the read data to the result vector.
                data.extend_from_slice(&buffer[..bytes_read]);
                potentionally_finished = false;
            }
            // The stream has reached its end.
            Ok(_) => {
                return Ok(data);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if potentionally_finished {
                    return Ok(data);
                }
                thread::sleep(std::time::Duration::from_millis(read_timeout_ms));
                potentionally_finished = true;
            }
            // Other error occured
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}

fn socket_buffer_to_string(buffer: &[u8]) -> String {
    buffer
        .iter()
        .map(|s| char::from_u32((*s).into()).unwrap())
        .collect::<String>()
}

fn socket_vec_buffer_to_string(buffer: Vec<u8>) -> String {
    buffer
        .iter()
        .map(|s| char::from_u32((*s).into()).unwrap())
        .collect::<String>()
}
