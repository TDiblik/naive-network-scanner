use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
};

use log::{error, info};

use crate::utils::constants::PORT_FUZZING_COMMANDS;

use super::ip::{BannerGrabResult, FuzzingResult, FuzzingResults, Port, ScanIpPortsConfig};

// Hey, future me, I have no fucking idea why I have to do this windows vs linux shit,
// but it doesn't for the love of good want to work without it :(

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
                #[cfg(target_os = "windows")]
                {
                    let mut buffer = Vec::new();
                    // for some reason windows throws error even though it succeeds
                    _ = connected_socket.read_to_end(&mut buffer);
                    banner = socket_buffer_to_string(&buffer);
                }

                #[cfg(target_os = "linux")]
                {
                    let grabbed_output = read_everything_from_socket(
                        &connected_socket,
                        config.read_write_timeout_ms,
                    );
                    match grabbed_output {
                        Ok(grabbed_output) => {
                            banner = socket_buffer_to_string(&grabbed_output);
                        }
                        Err(e) => {
                            error!("An error occurred while banner grabbing: {}", e);
                        }
                    }
                }
            }

            // fuzzing
            let mut fuzzing_results = vec![];
            if config.should_fuzz {
                for command in PORT_FUZZING_COMMANDS {
                    let command_stringified = socket_buffer_to_string(command);
                    if let Err(e) = connected_socket.write_all(command) {
                        error!(
                            "An error occurred while writing \"{}\" to socket: {}",
                            command_stringified, e
                        );
                        continue;
                    }
                    if let Err(e) = connected_socket.flush() {
                        error!(
                            "An error occurred while flushing after \"{}\" to socket. Continuing since this could severy corrupt output: {}",
                            command_stringified,
                            e
                        );
                        continue;
                    }

                    #[cfg(target_os = "windows")]
                    {
                        let mut buffer = Vec::new();
                        // for some reason windows throws error even though it succeeds
                        _ = connected_socket.read_to_end(&mut buffer);
                        if !buffer.is_empty() {
                            fuzzing_results.push(FuzzingResult {
                                command: command_stringified,
                                result: socket_buffer_to_string(&buffer),
                                result_raw: buffer,
                            });
                        }
                    }

                    #[cfg(target_os = "linux")]
                    {
                        let grabbed_output = read_everything_from_socket(
                            &connected_socket,
                            config.read_write_timeout_ms,
                        );
                        match grabbed_output {
                            Ok(grabbed_output) if !grabbed_output.is_empty() => {
                                fuzzing_results.push(FuzzingResult {
                                    command: command_stringified,
                                    result: socket_buffer_to_string(&grabbed_output),
                                    result_raw: grabbed_output,
                                });
                            }
                            Err(e) => {
                                error!("An error occurred while reading from socket after sending fuzz command \"{}\": {}", command_stringified, e);
                            }
                            _ => {}
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

#[cfg(target_os = "linux")]
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
                std::thread::sleep(std::time::Duration::from_millis(read_timeout_ms));
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
