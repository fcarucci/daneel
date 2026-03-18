// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use serde_json::Value;

#[cfg(feature = "server")]
use crate::{
    adapter::GatewayAdapter,
    gateway::{connect_gateway, load_gateway_config},
    models::{
        gateway::GatewayStatusSnapshot,
        graph::{AgentEdge, AgentNode, AgentStatus},
        runtime::ActiveSessionRecord,
    },
};

#[cfg(feature = "server")]
#[derive(Clone, Debug, Default)]
pub struct OpenClawAdapter;

#[cfg(feature = "server")]
fn not_implemented<T>(method: &str) -> Result<T, String> {
    Err(format!(
        "OpenClawAdapter::{method}() is not implemented yet."
    ))
}

#[cfg(feature = "server")]
fn require_payload<'a>(payload: Option<&'a Value>, context: &str) -> Result<&'a Value, String> {
    payload.ok_or_else(|| format!("{context} did not include a payload."))
}

#[cfg(feature = "server")]
fn snapshot_agents(payload: &Value) -> Result<&Vec<Value>, String> {
    payload
        .get("snapshot")
        .ok_or_else(|| "Gateway connect payload did not include a snapshot.".to_string())?
        .get("health")
        .and_then(Value::as_object)
        .ok_or_else(|| "Gateway snapshot did not include health.".to_string())?
        .get("agents")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include agents.".to_string())
}

#[cfg(feature = "server")]
fn snapshot_bindings(payload: &Value) -> Result<&Vec<Value>, String> {
    payload
        .get("snapshot")
        .ok_or_else(|| "Gateway connect payload did not include a snapshot.".to_string())?
        .get("health")
        .and_then(Value::as_object)
        .ok_or_else(|| "Gateway snapshot did not include health.".to_string())?
        .get("bindings")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include bindings.".to_string())
}

#[cfg(feature = "server")]
fn map_heartbeat(agent: &Value) -> (bool, String) {
    let heartbeat = agent.get("heartbeat").unwrap_or(&Value::Null);
    let enabled = heartbeat
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let schedule = heartbeat
        .get("every")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    (enabled, schedule)
}

#[cfg(feature = "server")]
fn map_agent_node(agent: &Value) -> Result<AgentNode, String> {
    let id = agent
        .get("agentId")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenClaw agent payload is missing agentId.".to_string())?
        .to_string();
    let name = agent
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&id)
        .to_string();
    let is_default = agent
        .get("isDefault")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let (heartbeat_enabled, heartbeat_schedule) = map_heartbeat(agent);

    Ok(AgentNode {
        id,
        name,
        is_default,
        heartbeat_enabled,
        heartbeat_schedule,
        active_session_count: 0,
        latest_activity_age_ms: None,
        status: AgentStatus::Unknown,
    })
}

#[cfg(feature = "server")]
fn map_binding_edge(binding: &Value) -> Result<AgentEdge, String> {
    let source_id = binding
        .get("sourceAgentId")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenClaw binding payload is missing sourceAgentId.".to_string())?
        .to_string();
    let target_id = binding
        .get("targetAgentId")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenClaw binding payload is missing targetAgentId.".to_string())?
        .to_string();

    Ok(AgentEdge {
        source_id,
        target_id,
        kind: crate::models::graph::AgentEdgeKind::GatewayRouting,
    })
}

#[cfg(feature = "server")]
fn normalize_binding_edges(bindings: &[Value]) -> Result<Vec<AgentEdge>, String> {
    let mut edges: Vec<_> = bindings
        .iter()
        .map(map_binding_edge)
        .collect::<Result<Vec<_>, _>>()?;
    edges.sort_by(|left, right| {
        (&left.source_id, &left.target_id).cmp(&(&right.source_id, &right.target_id))
    });
    edges.dedup_by(|left, right| left.source_id == right.source_id && left.target_id == right.target_id);
    Ok(edges)
}

#[cfg(feature = "server")]
fn map_active_session_record(
    session: &Value,
    fallback_agent_id: Option<&str>,
) -> Result<ActiveSessionRecord, String> {
    let session_id = session
        .get("sessionId")
        .or_else(|| session.get("key"))
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenClaw session payload is missing sessionId.".to_string())?
        .to_string();
    let agent_id = session
        .get("agentId")
        .and_then(Value::as_str)
        .or(fallback_agent_id)
        .ok_or_else(|| "OpenClaw session payload is missing agentId.".to_string())?
        .to_string();
    let task = session
        .get("task")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let age_ms = session
        .get("ageMs")
        .or_else(|| session.get("age"))
        .and_then(Value::as_u64);

    Ok(ActiveSessionRecord {
        session_id,
        agent_id,
        task,
        age_ms,
    })
}

#[cfg(feature = "server")]
fn normalize_active_sessions(sessions: &[Value]) -> Result<Vec<ActiveSessionRecord>, String> {
    let mut records: Vec<_> = sessions
        .iter()
        .map(|session| map_active_session_record(session, None))
        .collect::<Result<Vec<_>, _>>()?;
    records.sort_by(|left, right| left.session_id.cmp(&right.session_id));
    records.dedup_by(|left, right| left.session_id == right.session_id);
    Ok(records)
}

#[cfg(feature = "server")]
#[async_trait]
impl GatewayAdapter for OpenClawAdapter {
    async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
        not_implemented("gateway_status")
    }

    async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
        let config = load_gateway_config()?;
        let (mut socket, connect_frame) = connect_gateway(&config, "connect-list-agents-1").await?;
        let _ = socket.close(None).await;

        let payload = require_payload(connect_frame.payload.as_ref(), "Gateway connect response")?;
        let agents = snapshot_agents(payload)?;

        agents.iter().map(map_agent_node).collect()
    }

    async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
        let config = load_gateway_config()?;
        let (mut socket, connect_frame) =
            connect_gateway(&config, "connect-list-bindings-1").await?;
        let _ = socket.close(None).await;

        let payload = require_payload(connect_frame.payload.as_ref(), "Gateway connect response")?;
        let bindings = snapshot_bindings(payload)?;

        normalize_binding_edges(bindings)
    }

    async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
        not_implemented("list_active_sessions")
    }

    async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
        not_implemented("list_agent_relationship_hints")
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
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
        OpenClawAdapter, map_active_session_record, map_agent_node, map_binding_edge,
        normalize_active_sessions, normalize_binding_edges,
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
        json!({
            "protocolVersion": 3,
            "stateVersion": 42,
            "uptimeMs": 123_456,
            "snapshot": {
                "health": {
                    "agents": agents,
                    "bindings": bindings
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

    #[tokio::test]
    #[serial]
    async fn list_agents_reads_nodes_from_gateway_snapshot() {
        let gateway = MockGateway::spawn(gateway_snapshot_payload(json!([
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
        ]), json!([])))
        .expect("spawn mock gateway");
        let tempdir = tempdir().expect("create tempdir");
        let config_path = write_openclaw_config(tempdir.path(), gateway.addr.port())
            .expect("write openclaw config");
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
        let config_path = write_openclaw_config(tempdir.path(), gateway.addr.port())
            .expect("write openclaw config");
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
}
