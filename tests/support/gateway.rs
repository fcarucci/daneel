// SPDX-License-Identifier: Apache-2.0

use super::fixture::{GatewayPayload, MockAgent};
use serde_json::{Value, json};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tungstenite::{Message, accept};

pub(crate) struct MockGateway {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MockGateway {
    pub(crate) fn spawn(payload: GatewayPayload) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|error| format!("bind mock gateway: {error}"))?;
        let addr = listener
            .local_addr()
            .map_err(|error| format!("mock gateway local_addr: {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("mock gateway set_nonblocking: {error}"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if let Err(error) = handle_gateway_client(stream, &payload) {
                            eprintln!("mock gateway client error: {error}");
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(error) => {
                        eprintln!("mock gateway accept error: {error}");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            addr,
            stop,
            thread: Some(handle),
        })
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for MockGateway {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn handle_gateway_client(stream: TcpStream, payload: &GatewayPayload) -> Result<(), String> {
    let mut socket =
        accept(stream).map_err(|error| format!("websocket handshake failed: {error}"))?;

    loop {
        let message = match socket.read() {
            Ok(message) => message,
            Err(tungstenite::Error::ConnectionClosed) | Err(tungstenite::Error::AlreadyClosed) => {
                return Ok(());
            }
            Err(error) if is_client_disconnect(&error) => return Ok(()),
            Err(error) => return Err(format!("read gateway message: {error}")),
        };

        let Message::Text(text) = message else {
            continue;
        };
        let request: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse request json: {error}"))?;

        let response = match request.get("method").and_then(Value::as_str) {
            Some("connect") => {
                let response = connect_response(&request, payload);
                let _ = socket.send(Message::Text(health_event(payload).to_string().into()));
                response
            }
            Some("health") => health_response(&request, payload),
            Some(method) => error_response(&request, format!("Unsupported method {method}")),
            None => error_response(&request, "Missing method".to_string()),
        };

        socket
            .send(Message::Text(response.to_string().into()))
            .map_err(|error| format!("send gateway response: {error}"))?;
    }
}

fn is_client_disconnect(error: &tungstenite::Error) -> bool {
    matches!(error, tungstenite::Error::Io(io_error) if matches!(
        io_error.kind(),
        std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::UnexpectedEof
            | std::io::ErrorKind::BrokenPipe
    )) || error
        .to_string()
        .contains("Connection reset without closing handshake")
}

fn connect_response(request: &Value, payload: &GatewayPayload) -> Value {
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("connect");
    json!({
        "type": "res",
        "id": id,
        "ok": true,
        "payload": {
            "protocolVersion": 3,
            "stateVersion": payload.state_version,
            "uptimeMs": payload.uptime_ms,
            "snapshot": {
                "health": {
                    "ts": payload.snapshot_ts,
                    "defaultAgentId": "main",
                    "agents": payload.agents.iter().map(MockAgent::to_value).collect::<Vec<_>>(),
                    "bindings": payload.bindings,
                }
            }
        }
    })
}

fn health_response(request: &Value, payload: &GatewayPayload) -> Value {
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("health");
    json!({
        "type": "res",
        "id": id,
        "ok": true,
        "payload": {
            "status": "healthy",
            "stateVersion": payload.state_version,
            "uptimeMs": payload.uptime_ms
        }
    })
}

fn health_event(payload: &GatewayPayload) -> Value {
    json!({
        "type": "event",
        "event": "health",
        "payload": {
            "status": {
                "health": {
                    "state": "healthy"
                }
            },
            "stateVersion": payload.state_version,
            "uptimeMs": payload.uptime_ms
        }
    })
}

fn error_response(request: &Value, message: String) -> Value {
    let id = request.get("id").and_then(Value::as_str).unwrap_or("error");
    json!({
        "type": "res",
        "id": id,
        "ok": false,
        "error": {
            "message": message
        }
    })
}
