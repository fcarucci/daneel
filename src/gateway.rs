// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(feature = "server"), allow(dead_code))]

use dioxus::prelude::{ServerFnError, dioxus_fullstack, server};

#[cfg(feature = "server")]
use crate::app_state::{server_app_state, server_gateway_config};
#[cfg(feature = "server")]
use crate::graph_service::{self, GraphAssemblyInputs};
use crate::models::{
    agents::AgentOverviewSnapshot, gateway::GatewayStatusSnapshot, graph::AgentGraphSnapshot,
};

mod agents;
mod config;
mod parse;
mod session_store;
mod status;
mod ws;

pub const GATEWAY_STATUS_ENDPOINT: &str = "gateway/status";
pub const AGENT_OVERVIEW_ENDPOINT: &str = "agents/overview";
pub const AGENT_GRAPH_SNAPSHOT_ENDPOINT: &str = "graph/snapshot";

#[cfg(feature = "server")]
pub(crate) use config::LoadedGatewayConfig;
pub(crate) use config::{DEFAULT_GATEWAY_URL, load_gateway_config};

#[cfg(feature = "server")]
pub(crate) use ws::{connect_gateway, connect_request, wait_for_response};

#[server(endpoint = "gateway/status")]
pub async fn get_gateway_status() -> Result<GatewayStatusSnapshot, ServerFnError> {
    debug_assert_eq!(GATEWAY_STATUS_ENDPOINT, "gateway/status");
    Ok(load_gateway_status().await)
}

#[server(endpoint = "agents/overview")]
pub async fn get_agent_overview() -> Result<AgentOverviewSnapshot, ServerFnError> {
    debug_assert_eq!(AGENT_OVERVIEW_ENDPOINT, "agents/overview");
    agents::load_agent_overview()
        .await
        .map_err(ServerFnError::new)
}

#[server(endpoint = "graph/snapshot")]
pub async fn get_agent_graph_snapshot() -> Result<AgentGraphSnapshot, ServerFnError> {
    debug_assert_eq!(AGENT_GRAPH_SNAPSHOT_ENDPOINT, "graph/snapshot");
    load_agent_graph_snapshot()
        .await
        .map_err(ServerFnError::new)
}

async fn load_gateway_status() -> GatewayStatusSnapshot {
    #[cfg(feature = "server")]
    {
        let config = match server_gateway_config() {
            Ok(config) => config,
            Err(error) => {
                return degraded_gateway_status(
                    DEFAULT_GATEWAY_URL.to_string(),
                    "Gateway configuration unavailable",
                    error,
                );
            }
        };

        match status::fetch_gateway_status_via_websocket(&config).await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                degraded_gateway_status(config.ws_url.clone(), "Gateway connection failed", error)
            }
        }
    }

    #[cfg(not(feature = "server"))]
    {
        let config = match load_gateway_config() {
            Ok(config) => config,
            Err(error) => {
                return degraded_gateway_status(
                    DEFAULT_GATEWAY_URL.to_string(),
                    "Gateway configuration unavailable",
                    error,
                );
            }
        };

        degraded_gateway_status(
            config.ws_url,
            "Gateway status is only available from the server build",
            "Run Daneel in fullstack mode so server functions can connect to OpenClaw.".to_string(),
        )
    }
}

fn degraded_gateway_status(
    gateway_url: String,
    summary: impl Into<String>,
    detail: impl Into<String>,
) -> GatewayStatusSnapshot {
    GatewayStatusSnapshot::degraded(gateway_url, summary, detail)
}

#[cfg(feature = "server")]
async fn load_agent_graph_snapshot() -> Result<AgentGraphSnapshot, String> {
    let state = server_app_state()?;
    load_agent_graph_snapshot_with(state.adapter(), snapshot_timestamp_ms()).await
}

#[cfg(feature = "server")]
async fn load_agent_graph_snapshot_with(
    adapter: &impl crate::adapter::GatewayAdapter,
    snapshot_ts: u64,
) -> Result<AgentGraphSnapshot, String> {
    let (agents, gateway_edges, active_sessions, hint_edges) = tokio::join!(
        adapter.list_agents(),
        adapter.list_agent_bindings(),
        adapter.list_active_sessions(),
        adapter.list_agent_relationship_hints(),
    );

    Ok(graph_service::assemble_graph_snapshot(
        GraphAssemblyInputs {
            agents: agents?,
            gateway_edges: gateway_edges?,
            active_sessions: active_sessions?,
            hint_edges: hint_edges.unwrap_or_default(),
        },
        snapshot_ts,
    ))
}

#[cfg(feature = "server")]
fn snapshot_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use async_trait::async_trait;

    use crate::adapter::GatewayAdapter;
    use crate::models::{
        graph::{AgentEdge, AgentEdgeKind, AgentNode, AgentStatus},
        runtime::ActiveSessionRecord,
    };

    use crate::models::gateway::GatewayLevel;

    use super::{
        AGENT_GRAPH_SNAPSHOT_ENDPOINT, connect_request, load_agent_graph_snapshot_with,
        load_gateway_config, snapshot_timestamp_ms, status,
    };

    #[derive(Clone, Debug)]
    struct MockAdapter {
        agents: Vec<AgentNode>,
        bindings: Vec<AgentEdge>,
        sessions: Vec<ActiveSessionRecord>,
        hints: Result<Vec<AgentEdge>, String>,
    }

    impl Default for MockAdapter {
        fn default() -> Self {
            Self {
                agents: Vec::new(),
                bindings: Vec::new(),
                sessions: Vec::new(),
                hints: Ok(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl GatewayAdapter for MockAdapter {
        async fn gateway_status(
            &self,
        ) -> Result<crate::models::gateway::GatewayStatusSnapshot, String> {
            Ok(crate::models::gateway::GatewayStatusSnapshot {
                connected: true,
                level: GatewayLevel::Healthy,
                summary: "healthy".to_string(),
                detail: "mock".to_string(),
                gateway_url: "ws://127.0.0.1:18789/".to_string(),
                protocol_version: Some(3),
                state_version: Some(1),
                uptime_ms: Some(1_000),
            })
        }

        async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
            Ok(self.agents.clone())
        }

        async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(self.bindings.clone())
        }

        async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
            Ok(self.sessions.clone())
        }

        async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
            self.hints.clone()
        }
    }

    fn agent(id: &str, status: AgentStatus) -> AgentNode {
        AgentNode {
            id: id.to_string(),
            name: id.to_string(),
            is_default: id == "planner",
            heartbeat_enabled: true,
            heartbeat_schedule: "every 5m".to_string(),
            active_session_count: 0,
            latest_activity_age_ms: None,
            status,
        }
    }

    fn edge(source_id: &str, target_id: &str, kind: AgentEdgeKind) -> AgentEdge {
        AgentEdge {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            kind,
        }
    }

    #[test]
    fn connect_request_uses_backend_gateway_identity() {
        let request = connect_request("connect-test-1", "test-token");

        assert_eq!(request["method"], "connect");
        assert_eq!(request["params"]["client"]["id"], "gateway-client");
        assert_eq!(request["params"]["client"]["mode"], "backend");
        assert_eq!(request["params"]["auth"]["token"], "test-token");
    }

    #[test]
    fn graph_snapshot_endpoint_is_explicit() {
        assert_eq!(AGENT_GRAPH_SNAPSHOT_ENDPOINT, "graph/snapshot");
    }

    #[tokio::test]
    async fn graph_snapshot_loader_returns_full_snapshot_from_mock_adapter_data() {
        let snapshot = load_agent_graph_snapshot_with(
            &MockAdapter {
                agents: vec![
                    agent("planner", AgentStatus::Idle),
                    agent("calendar", AgentStatus::Unknown),
                ],
                bindings: vec![edge("planner", "calendar", AgentEdgeKind::RoutesTo)],
                sessions: vec![ActiveSessionRecord {
                    session_id: "session-1".to_string(),
                    agent_id: "planner".to_string(),
                    task: Some("work".to_string()),
                    age_ms: Some(250),
                }],
                hints: Ok(vec![edge(
                    "planner",
                    "calendar",
                    AgentEdgeKind::WorksWithHint,
                )]),
            },
            42,
        )
        .await
        .expect("load graph snapshot");

        assert_eq!(snapshot.snapshot_ts, 42);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["calendar", "planner"]
        );
        assert_eq!(
            snapshot.edges,
            vec![edge("planner", "calendar", AgentEdgeKind::RoutesTo)]
        );
        assert_eq!(snapshot.nodes[1].status, AgentStatus::Active);
    }

    #[tokio::test]
    async fn graph_snapshot_loader_returns_valid_empty_snapshot_when_adapter_data_is_empty() {
        let snapshot = load_agent_graph_snapshot_with(&MockAdapter::default(), 99)
            .await
            .expect("load empty graph snapshot");

        assert_eq!(snapshot.snapshot_ts, 99);
        assert!(snapshot.nodes.is_empty());
        assert!(snapshot.edges.is_empty());
    }

    #[tokio::test]
    async fn graph_snapshot_loader_succeeds_when_relationship_hints_are_unavailable() {
        let snapshot = load_agent_graph_snapshot_with(
            &MockAdapter {
                agents: vec![agent("planner", AgentStatus::Idle)],
                bindings: Vec::new(),
                sessions: Vec::new(),
                hints: Err("metadata unavailable".to_string()),
            },
            7,
        )
        .await;

        assert!(snapshot.is_ok());
    }

    #[test]
    fn snapshot_timestamp_is_nonzero() {
        assert!(snapshot_timestamp_ms() > 0);
    }

    #[tokio::test]
    #[ignore = "manual live verification against the developer's local OpenClaw gateway"]
    async fn live_gateway_status_fetch_reports_healthy() {
        let config = load_gateway_config().expect("load local OpenClaw gateway config");
        let snapshot = status::fetch_gateway_status_via_websocket(&config)
            .await
            .expect("fetch gateway status over websocket");

        assert!(snapshot.connected);
        assert!(matches!(snapshot.level, GatewayLevel::Healthy));
    }
}
