// SPDX-License-Identifier: Apache-2.0

use super::AppClient;
use crate::models::{
    agents::AgentOverviewSnapshot,
    gateway::GatewayStatusSnapshot,
    graph::{AgentEdge, AgentEdgeKind, AgentGraphSnapshot, AgentNode, AgentStatus},
};
use async_trait::async_trait;
use dioxus::prelude::ServerFnError;

/// Mock AppClient for testing UI-facing code
pub struct MockAppClient {
    gateway_status: Result<GatewayStatusSnapshot, ServerFnError>,
    agent_overview: Result<AgentOverviewSnapshot, ServerFnError>,
    graph_snapshot: Result<AgentGraphSnapshot, ServerFnError>,
}

impl MockAppClient {
    pub fn new(
        gateway_status: Result<GatewayStatusSnapshot, ServerFnError>,
        agent_overview: Result<AgentOverviewSnapshot, ServerFnError>,
        graph_snapshot: Result<AgentGraphSnapshot, ServerFnError>,
    ) -> Self {
        Self {
            gateway_status,
            agent_overview,
            graph_snapshot,
        }
    }

    pub fn healthy_gateway() -> Self {
        Self::new(
            Ok(GatewayStatusSnapshot {
                connected: true,
                level: crate::models::gateway::GatewayLevel::Healthy,
                summary: "Gateway is healthy".to_string(),
                detail: "All systems operational".to_string(),
                gateway_url: "ws://localhost:8080".to_string(),
                protocol_version: Some(1),
                state_version: Some(123),
                uptime_ms: Some(1000),
            }),
            Ok(AgentOverviewSnapshot {
                total_agents: 3,
                default_agent_id: Some("planner".to_string()),
                total_active_sessions: 5,
                active_recent_agents: 2,
                agents: vec![],
            }),
            Ok(AgentGraphSnapshot {
                nodes: vec![AgentNode {
                    id: "planner".to_string(),
                    name: "planner".to_string(),
                    is_default: true,
                    heartbeat_enabled: true,
                    heartbeat_schedule: "every 5m".to_string(),
                    active_session_count: 1,
                    latest_activity_age_ms: Some(250),
                    status: AgentStatus::Active,
                }],
                edges: vec![AgentEdge {
                    source_id: "planner".to_string(),
                    target_id: "calendar".to_string(),
                    kind: AgentEdgeKind::RoutesTo,
                }],
                snapshot_ts: 1_640_995_200_000,
            }),
        )
    }

    pub fn degraded_gateway() -> Self {
        Self::new(
            Ok(GatewayStatusSnapshot::degraded(
                "ws://localhost:8080".to_string(),
                "Gateway connection failed",
                "Connection timeout",
            )),
            Err(ServerFnError::new("Gateway unavailable")),
            Err(ServerFnError::new("Graph unavailable")),
        )
    }
}

#[async_trait(?Send)]
impl AppClient for MockAppClient {
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError> {
        self.gateway_status.clone()
    }

    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError> {
        self.agent_overview.clone()
    }

    async fn get_agent_graph_snapshot(&self) -> Result<AgentGraphSnapshot, ServerFnError> {
        self.graph_snapshot.clone()
    }
}
