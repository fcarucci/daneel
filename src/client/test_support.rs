// SPDX-License-Identifier: Apache-2.0

use super::AppClient;
use crate::models::{agents::AgentOverviewSnapshot, gateway::GatewayStatusSnapshot};
use dioxus::prelude::ServerFnError;

/// Mock AppClient for testing UI-facing code
pub struct MockAppClient {
    gateway_status: Result<GatewayStatusSnapshot, ServerFnError>,
    agent_overview: Result<AgentOverviewSnapshot, ServerFnError>,
}

impl MockAppClient {
    pub fn new(
        gateway_status: Result<GatewayStatusSnapshot, ServerFnError>,
        agent_overview: Result<AgentOverviewSnapshot, ServerFnError>,
    ) -> Self {
        Self {
            gateway_status,
            agent_overview,
        }
    }

    pub fn gateway_status(&self) -> &Result<GatewayStatusSnapshot, ServerFnError> {
        &self.gateway_status
    }

    pub fn agent_overview(&self) -> &Result<AgentOverviewSnapshot, ServerFnError> {
        &self.agent_overview
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
        )
    }
}

#[cfg_attr(feature = "server", async_trait::async_trait)]
impl AppClient for MockAppClient {
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError> {
        self.gateway_status.clone()
    }

    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError> {
        self.agent_overview.clone()
    }
}