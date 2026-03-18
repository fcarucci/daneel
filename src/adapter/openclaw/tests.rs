// SPDX-License-Identifier: Apache-2.0

use std::{
    env, fs,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use serde_json::json;
use serial_test::serial;
use tempfile::tempdir;
use tungstenite::{Message, accept};

use crate::{
    adapter::GatewayAdapter,
    models::graph::{AgentEdgeKind, AgentStatus},
};

use super::{
    OpenClawAdapter,
    mapping::{
        map_active_session_record, map_agent_node, map_binding_edge, normalize_active_sessions,
        normalize_binding_edges,
    },
};

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = env::var(key).ok();
        unsafe {
            env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(previous) => unsafe { env::set_var(self.key, previous) },
            None => unsafe { env::remove_var(self.key) },
        }
    }
}

struct MockGateway {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MockGateway {
    fn spawn(connect_payload: serde_json::Value) -> Result<Self, String> {
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
                        let _ = handle_gateway_client(stream, &connect_payload);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            addr,
            stop,
            thread: Some(handle),
        })
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

fn handle_gateway_client(
    stream: TcpStream,
    connect_payload: &serde_json::Value,
) -> Result<(), String> {
    let mut socket = accept(stream).map_err(|error| format!("handshake failed: {error}"))?;
    let message = socket
        .read()
        .map_err(|error| format!("read request: {error}"))?;
    let Message::Text(text) = message else {
        return Err("expected text connect request".to_string());
    };
    let request: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("parse request: {error}"))?;
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("connect-test");

    socket
        .send(Message::Text(
            json!({
                "type": "res",
                "id": id,
                "ok": true,
                "payload": connect_payload,
            })
            .to_string()
            .into(),
        ))
        .map_err(|error| format!("send response: {error}"))?;

    Ok(())
}

fn write_openclaw_config(
    tempdir: &std::path::Path,
    port: u16,
) -> Result<std::path::PathBuf, String> {
    let config_path = tempdir.join("openclaw.json");
    fs::write(
        &config_path,
        serde_json::to_vec_pretty(&json!({
            "gateway": {
                "port": port,
                "auth": { "token": "test-token" }
            }
        }))
        .expect("serialize config"),
    )
    .map_err(|error| format!("write config: {error}"))?;
    Ok(config_path)
}

fn gateway_snapshot_payload(
    agents: serde_json::Value,
    bindings: serde_json::Value,
) -> serde_json::Value {
    gateway_snapshot_payload_with_sessions(agents, bindings, json!([]))
}

fn gateway_snapshot_payload_with_sessions(
    agents: serde_json::Value,
    bindings: serde_json::Value,
    sessions: serde_json::Value,
) -> serde_json::Value {
    json!({
        "protocolVersion": 3,
        "stateVersion": 42,
        "uptimeMs": 123_456,
        "snapshot": {
            "health": {
                "agents": agents,
                "bindings": bindings,
                "activeSessions": sessions
            }
        }
    })
}

#[test]
fn openclaw_agent_json_maps_to_agent_node() {
    let node = map_agent_node(&json!({
        "agentId": "planner",
        "name": "Planner",
        "isDefault": true,
        "heartbeat": {
            "enabled": true,
            "every": "15m"
        }
    }))
    .expect("map agent node");

    assert_eq!(node.id, "planner");
    assert_eq!(node.name, "Planner");
    assert!(node.is_default);
    assert!(node.heartbeat_enabled);
    assert_eq!(node.heartbeat_schedule, "15m");
    assert_eq!(node.active_session_count, 0);
    assert_eq!(node.latest_activity_age_ms, None);
    assert_eq!(node.status, AgentStatus::Unknown);
}

#[test]
fn unknown_fields_do_not_break_agent_mapping() {
    let node = map_agent_node(&json!({
        "agentId": "calendar",
        "name": "Calendar",
        "heartbeat": {
            "enabled": true,
            "every": "30m",
            "model": "ignored-model"
        },
        "extra": {
            "nested": ["noise", 1, true]
        }
    }))
    .expect("map noisy agent node");

    assert_eq!(node.id, "calendar");
    assert_eq!(node.name, "Calendar");
    assert_eq!(node.heartbeat_schedule, "30m");
}

#[test]
fn missing_optional_fields_fall_back_safely() {
    let node =
        map_agent_node(&json!({ "agentId": "health-coach" })).expect("map sparse agent node");

    assert_eq!(node.id, "health-coach");
    assert_eq!(node.name, "health-coach");
    assert!(!node.is_default);
    assert!(!node.heartbeat_enabled);
    assert_eq!(node.heartbeat_schedule, "");
    assert_eq!(node.active_session_count, 0);
    assert_eq!(node.latest_activity_age_ms, None);
    assert_eq!(node.status, AgentStatus::Unknown);
}

#[test]
fn missing_agent_id_returns_a_clear_error() {
    let error = map_agent_node(&json!({
        "name": "Broken Agent",
        "heartbeat": {
            "enabled": true,
            "every": "5m"
        }
    }))
    .expect_err("reject missing agent id");

    assert!(error.contains("missing agentId"));
}

#[test]
fn binding_payload_maps_to_gateway_routing_edge() {
    let edge = map_binding_edge(&json!({
        "sourceAgentId": "main",
        "targetAgentId": "planner",
        "bindingType": "routes_to"
    }))
    .expect("map binding edge");

    assert_eq!(edge.source_id, "main");
    assert_eq!(edge.target_id, "planner");
    assert_eq!(edge.kind, AgentEdgeKind::GatewayRouting);
}

#[test]
fn openclaw_session_json_maps_to_active_session_record() {
    let session = map_active_session_record(
        &json!({
            "sessionId": "session-1",
            "agentId": "planner",
            "task": "plan route",
            "ageMs": 500
        }),
        None,
    )
    .expect("map active session");

    assert_eq!(session.session_id, "session-1");
    assert_eq!(session.agent_id, "planner");
    assert_eq!(session.task.as_deref(), Some("plan route"));
    assert_eq!(session.age_ms, Some(500));
}

#[test]
fn multiple_sessions_for_different_agents_map_correctly() {
    let sessions = [
        map_active_session_record(
            &json!({
                "sessionId": "session-1",
                "agentId": "planner",
                "ageMs": 500
            }),
            None,
        )
        .expect("map planner session"),
        map_active_session_record(
            &json!({
                "sessionId": "session-2",
                "agentId": "calendar",
                "task": "check inbox",
                "age": 250
            }),
            None,
        )
        .expect("map calendar session"),
    ];

    assert_eq!(sessions[0].agent_id, "planner");
    assert_eq!(sessions[0].age_ms, Some(500));
    assert_eq!(sessions[1].agent_id, "calendar");
    assert_eq!(sessions[1].task.as_deref(), Some("check inbox"));
    assert_eq!(sessions[1].age_ms, Some(250));
}

#[test]
fn missing_optional_session_fields_fall_back_safely() {
    let session = map_active_session_record(
        &json!({
            "sessionId": "session-1",
            "agentId": "planner"
        }),
        None,
    )
    .expect("map sparse session");

    assert_eq!(session.session_id, "session-1");
    assert_eq!(session.agent_id, "planner");
    assert_eq!(session.task, None);
    assert_eq!(session.age_ms, None);
}

#[test]
fn unknown_session_fields_do_not_break_mapping() {
    let session = map_active_session_record(
        &json!({
            "sessionId": "session-1",
            "agentId": "planner",
            "task": "plan route",
            "extra": { "nested": true }
        }),
        None,
    )
    .expect("map noisy session");

    assert_eq!(session.session_id, "session-1");
    assert_eq!(session.agent_id, "planner");
    assert_eq!(session.task.as_deref(), Some("plan route"));
}

#[test]
fn missing_required_session_identity_returns_a_clear_error() {
    let error = map_active_session_record(
        &json!({
            "sessionId": "session-1"
        }),
        None,
    )
    .expect_err("reject session without agent id");

    assert!(error.contains("missing agentId"));
}

#[test]
fn duplicate_session_ids_are_normalized_deterministically() {
    let sessions = normalize_active_sessions(&[
        json!({ "sessionId": "session-2", "agentId": "calendar", "ageMs": 600 }),
        json!({ "sessionId": "session-1", "agentId": "planner", "ageMs": 500 }),
        json!({ "sessionId": "session-1", "agentId": "planner", "ageMs": 400 }),
    ])
    .expect("normalize duplicate sessions");

    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].session_id, "session-1");
    assert_eq!(sessions[0].age_ms, Some(500));
    assert_eq!(sessions[1].session_id, "session-2");
}

#[test]
fn unknown_session_agent_references_still_map_safely() {
    let session = map_active_session_record(
        &json!({
            "sessionId": "session-1",
            "agentId": "ghost-agent",
            "ageMs": 900
        }),
        None,
    )
    .expect("map session with unknown agent reference");

    assert_eq!(session.agent_id, "ghost-agent");
    assert_eq!(session.age_ms, Some(900));
}

#[tokio::test]
#[serial]
async fn list_agents_reads_nodes_from_gateway_snapshot() {
    let gateway = MockGateway::spawn(gateway_snapshot_payload(
        json!([
            {
                "agentId": "main",
                "name": "Main",
                "isDefault": true,
                "heartbeat": {
                    "enabled": true,
                    "every": "15m"
                }
            },
            {
                "agentId": "planner"
            }
        ]),
        json!([]),
    ))
    .expect("spawn mock gateway");
    let tempdir = tempdir().expect("create tempdir");
    let config_path =
        write_openclaw_config(tempdir.path(), gateway.addr.port()).expect("write openclaw config");
    let _guard = EnvVarGuard::set(
        "OPENCLAW_CONFIG_PATH",
        config_path.to_str().expect("config path as utf-8"),
    );

    let agents = OpenClawAdapter
        .list_agents()
        .await
        .expect("list agents through gateway");

    assert_eq!(agents.len(), 2);
    assert_eq!(agents[0].id, "main");
    assert_eq!(agents[0].name, "Main");
    assert!(agents[0].is_default);
    assert_eq!(agents[0].heartbeat_schedule, "15m");
    assert_eq!(agents[1].id, "planner");
    assert_eq!(agents[1].name, "planner");
}

#[test]
fn empty_bindings_normalize_to_empty_edges() {
    let edges = normalize_binding_edges(&[]).expect("normalize empty bindings");
    assert!(edges.is_empty());
}

#[test]
fn duplicate_bindings_are_normalized_deterministically() {
    let edges = normalize_binding_edges(&[
        json!({ "sourceAgentId": "main", "targetAgentId": "planner" }),
        json!({ "sourceAgentId": "main", "targetAgentId": "planner", "bindingType": "routes_to" }),
        json!({ "sourceAgentId": "calendar", "targetAgentId": "main" }),
    ])
    .expect("normalize duplicate bindings");

    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].source_id, "calendar");
    assert_eq!(edges[0].target_id, "main");
    assert_eq!(edges[1].source_id, "main");
    assert_eq!(edges[1].target_id, "planner");
}

#[tokio::test]
#[serial]
async fn list_agent_bindings_reads_edges_from_gateway_snapshot() {
    let gateway = MockGateway::spawn(gateway_snapshot_payload(
        json!([]),
        json!([
            { "sourceAgentId": "main", "targetAgentId": "planner" },
            { "sourceAgentId": "calendar", "targetAgentId": "main" },
            { "sourceAgentId": "main", "targetAgentId": "planner", "bindingType": "routes_to" }
        ]),
    ))
    .expect("spawn mock gateway");
    let tempdir = tempdir().expect("create tempdir");
    let config_path =
        write_openclaw_config(tempdir.path(), gateway.addr.port()).expect("write openclaw config");
    let _guard = EnvVarGuard::set(
        "OPENCLAW_CONFIG_PATH",
        config_path.to_str().expect("config path as utf-8"),
    );

    let edges = OpenClawAdapter
        .list_agent_bindings()
        .await
        .expect("list bindings through gateway");

    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].source_id, "calendar");
    assert_eq!(edges[0].target_id, "main");
    assert_eq!(edges[1].source_id, "main");
    assert_eq!(edges[1].target_id, "planner");
    assert_eq!(edges[0].kind, AgentEdgeKind::GatewayRouting);
    assert_eq!(edges[1].kind, AgentEdgeKind::GatewayRouting);
}

#[tokio::test]
#[serial]
async fn list_active_sessions_reads_sessions_from_gateway_snapshot() {
    let gateway = MockGateway::spawn(gateway_snapshot_payload_with_sessions(
        json!([]),
        json!([]),
        json!([
            { "sessionId": "session-2", "agentId": "calendar", "task": "check inbox", "ageMs": 600 },
            { "sessionId": "session-1", "agentId": "planner", "ageMs": 500 },
            { "sessionId": "session-1", "agentId": "planner", "ageMs": 400 },
            { "sessionId": "session-3", "agentId": "ghost-agent", "ageMs": 900 }
        ]),
    ))
    .expect("spawn mock gateway");
    let tempdir = tempdir().expect("create tempdir");
    let config_path =
        write_openclaw_config(tempdir.path(), gateway.addr.port()).expect("write openclaw config");
    let _guard = EnvVarGuard::set(
        "OPENCLAW_CONFIG_PATH",
        config_path.to_str().expect("config path as utf-8"),
    );

    let sessions = OpenClawAdapter
        .list_active_sessions()
        .await
        .expect("list active sessions through gateway");

    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].session_id, "session-1");
    assert_eq!(sessions[0].agent_id, "planner");
    assert_eq!(sessions[0].age_ms, Some(500));
    assert_eq!(sessions[1].session_id, "session-2");
    assert_eq!(sessions[1].task.as_deref(), Some("check inbox"));
    assert_eq!(sessions[2].agent_id, "ghost-agent");
}

#[tokio::test]
#[serial]
async fn list_active_sessions_falls_back_to_agent_recent_entries() {
    let gateway = MockGateway::spawn(gateway_snapshot_payload(
        json!([
            {
                "agentId": "planner",
                "sessions": {
                    "recent": [
                        { "key": "session-2", "age": 700 },
                        { "key": "session-1", "age": 500 }
                    ]
                }
            },
            {
                "agentId": "calendar",
                "sessions": {
                    "recent": [
                        { "key": "session-1", "age": 400 },
                        { "key": "session-3", "task": "check inbox", "age": 300 }
                    ]
                }
            }
        ]),
        json!([]),
    ))
    .expect("spawn mock gateway");
    let tempdir = tempdir().expect("create tempdir");
    let config_path =
        write_openclaw_config(tempdir.path(), gateway.addr.port()).expect("write openclaw config");
    let _guard = EnvVarGuard::set(
        "OPENCLAW_CONFIG_PATH",
        config_path.to_str().expect("config path as utf-8"),
    );

    let sessions = OpenClawAdapter
        .list_active_sessions()
        .await
        .expect("list fallback sessions through gateway");

    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].session_id, "session-1");
    assert_eq!(sessions[0].agent_id, "planner");
    assert_eq!(sessions[0].age_ms, Some(500));
    assert_eq!(sessions[2].session_id, "session-3");
    assert_eq!(sessions[2].agent_id, "calendar");
    assert_eq!(sessions[2].task.as_deref(), Some("check inbox"));
}
