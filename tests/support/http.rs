// SPDX-License-Identifier: Apache-2.0

use super::process::RunningProcess;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    thread,
    time::{Duration, Instant},
};

pub const APP_START_TIMEOUT: Duration = Duration::from_secs(180);
pub const PAGE_TIMEOUT: Duration = Duration::from_secs(75);
pub const SSE_TIMEOUT: Duration = Duration::from_secs(20);

pub fn wait_for_http_ready(port: u16, app: &mut RunningProcess) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let started = Instant::now();

    while started.elapsed() < APP_START_TIMEOUT {
        app.assert_still_running();
        if TcpStream::connect_timeout(&address, Duration::from_millis(250)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }

    panic!(
        "Timed out waiting for http://127.0.0.1:{port}.\nCaptured logs:\n{}",
        app.log_output()
    );
}

pub fn wait_for_backend_route_ready(port: u16, path: &str, app: &mut RunningProcess) {
    let started = Instant::now();
    let mut last_response = String::new();

    while started.elapsed() < APP_START_TIMEOUT {
        app.assert_still_running();
        match fetch_http_response(port, path, "text/event-stream") {
            Ok(response) => {
                last_response = response.clone();
                if response_starts_successfully(&response) {
                    return;
                }
            }
            Err(error) => last_response = error,
        }

        thread::sleep(Duration::from_millis(250));
    }

    panic!(
        "Timed out waiting for backend route http://127.0.0.1:{port}{path}.\nLast response:\n{last_response}\nCaptured logs:\n{}",
        app.log_output()
    );
}

pub fn read_http_until(
    port: u16,
    path: &str,
    accept: &str,
    required: &[&str],
    forbidden: &[&str],
    process: &mut RunningProcess,
) -> String {
    let started = Instant::now();
    let mut last_response = String::new();
    let mut last_error = String::new();

    while started.elapsed() < PAGE_TIMEOUT {
        process.assert_still_running();
        match fetch_http_response(port, path, accept) {
            Ok(response) => {
                last_response = response.clone();
                if matches_expectations(&response, required, forbidden) {
                    return response;
                }
            }
            Err(error) => last_error = error,
        }

        thread::sleep(Duration::from_millis(250));
    }

    panic!(
        "Timed out waiting for HTTP response at http://127.0.0.1:{port}{path}.\nRequired: {required:?}\nForbidden: {forbidden:?}\nLast request error:\n{last_error}\nLast response:\n{last_response}\nApp logs:\n{}",
        process.log_output()
    );
}

pub fn read_sse_until(
    port: u16,
    path: &str,
    required: &[&str],
    forbidden: &[&str],
    process: &mut RunningProcess,
) -> String {
    let started = Instant::now();
    let mut last_stream = String::new();
    let mut last_error = String::new();

    while started.elapsed() < SSE_TIMEOUT {
        process.assert_still_running();
        match open_request_stream(port, path, "text/event-stream") {
            Ok(mut stream) => {
                let mut buffer = [0_u8; 4096];
                let mut captured = String::new();
                while started.elapsed() < SSE_TIMEOUT {
                    process.assert_still_running();
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            last_error =
                                "SSE stream closed before the expected event arrived.".to_string();
                            break;
                        }
                        Ok(read) => {
                            captured.push_str(&String::from_utf8_lossy(&buffer[..read]));
                            last_stream = captured.clone();

                            if matches_expectations(&captured, required, forbidden) {
                                return captured;
                            }
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            continue;
                        }
                        Err(error) => {
                            last_error = format!("Could not read SSE stream: {error}");
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                last_error = format!("Could not open SSE endpoint: {error}");
            }
        }

        thread::sleep(Duration::from_millis(250));
    }

    panic!(
        "Timed out waiting for SSE event at http://127.0.0.1:{port}{path}.\nRequired: {required:?}\nForbidden: {forbidden:?}\nLast connection error:\n{last_error}\nLast stream data:\n{last_stream}\nApp logs:\n{}",
        process.log_output()
    );
}

pub fn matches_expectations(response: &str, required: &[&str], forbidden: &[&str]) -> bool {
    required.iter().all(|text| response.contains(text))
        && !forbidden.iter().any(|text| response.contains(text))
}

pub fn response_starts_successfully(response: &str) -> bool {
    response.starts_with("HTTP/1.1 200 OK") || response.starts_with("HTTP/1.1 204 No Content")
}

pub fn with_query_param(url: &str, key: &str, value: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}{key}={value}")
}

fn open_request_stream(port: u16, path: &str, accept: &str) -> Result<TcpStream, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(1))
        .map_err(|error| format!("Could not connect to http://127.0.0.1:{port}{path}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|error| format!("Could not set read timeout for {path}: {error}"))?;
    write_http_request(&mut stream, port, path, accept)?;
    Ok(stream)
}

fn write_http_request(
    stream: &mut TcpStream,
    port: u16,
    path: &str,
    accept: &str,
) -> Result<(), String> {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept: {accept}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("Could not write HTTP request for {path}: {error}"))
}

fn fetch_http_response(port: u16, path: &str, accept: &str) -> Result<String, String> {
    let mut stream = open_request_stream(port, path, accept)?;
    read_stream_to_string(&mut stream, port, path)
}

fn read_stream_to_string(stream: &mut TcpStream, port: u16, path: &str) -> Result<String, String> {
    let started = Instant::now();
    let mut buffer = [0_u8; 4096];
    let mut captured = String::new();

    while started.elapsed() < Duration::from_secs(2) {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => captured.push_str(&String::from_utf8_lossy(&buffer[..read])),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(error) => {
                return Err(format!("Could not read HTTP response for {path}: {error}"));
            }
        }
    }

    if captured.is_empty() {
        Err(format!(
            "No response was received from http://127.0.0.1:{port}{path} before timeout."
        ))
    } else {
        Ok(captured)
    }
}
