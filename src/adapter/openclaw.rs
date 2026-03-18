// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use serde_json::Value;

#[cfg(feature = "server")]
use crate::{
    adapter::GatewayAdapter,
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

    let heartbeat = agent.get("heartbeat").unwrap_or(&Value::Null);
    let heartbeat_enabled = heartbeat
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let heartbeat_schedule = heartbeat
        .get("every")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

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
#[async_trait]
impl GatewayAdapter for OpenClawAdapter {
    async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
        not_implemented("gateway_status")
    }

    async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
        not_implemented("list_agents")
    }

    async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
        not_implemented("list_agent_bindings")
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
    use serde_json::json;

    use crate::models::graph::AgentStatus;

    use super::map_agent_node;

    #[test]
    fn openclaw_agent_json_maps_to_agent_node() {
        let node = map_agent_node(
            &json!({
                "agentId": "planner",
                "name": "Planner",
                "isDefault": true,
                "heartbeat": {
                    "enabled": true,
                    "every": "15m"
                }
            }),
        )
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
        let node = map_agent_node(
            &json!({
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
            }),
        )
        .expect("map noisy agent node");

        assert_eq!(node.id, "calendar");
        assert_eq!(node.name, "Calendar");
        assert_eq!(node.heartbeat_schedule, "30m");
    }

    #[test]
    fn missing_optional_fields_fall_back_safely() {
        let node = map_agent_node(&json!({ "agentId": "health-coach" }))
            .expect("map sparse agent node");

        assert_eq!(node.id, "health-coach");
        assert_eq!(node.name, "health-coach");
        assert!(!node.is_default);
        assert!(!node.heartbeat_enabled);
        assert_eq!(node.heartbeat_schedule, "");
        assert_eq!(node.active_session_count, 0);
        assert_eq!(node.latest_activity_age_ms, None);
        assert_eq!(node.status, AgentStatus::Unknown);
    }
}
