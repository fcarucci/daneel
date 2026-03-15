use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentOverviewSnapshot {
    pub total_agents: usize,
    pub default_agent_id: Option<String>,
    pub total_active_sessions: u64,
    pub active_recent_agents: usize,
    pub agents: Vec<AgentOverviewItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentOverviewItem {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub heartbeat_enabled: bool,
    pub heartbeat_schedule: String,
    pub heartbeat_model: Option<String>,
    pub active_session_count: u64,
    pub latest_session_key: Option<String>,
    pub latest_activity_age_ms: Option<u64>,
}
// SPDX-License-Identifier: Apache-2.0
