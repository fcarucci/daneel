// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::models::runtime::ActiveSessionRecord;

use super::mapping::{
    map_active_session_record, normalize_active_session_records, normalize_active_sessions,
};

pub(super) fn snapshot_health(payload: &Value) -> Result<&serde_json::Map<String, Value>, String> {
    payload
        .get("snapshot")
        .ok_or_else(|| "Gateway connect payload did not include a snapshot.".to_string())?
        .get("health")
        .and_then(Value::as_object)
        .ok_or_else(|| "Gateway snapshot did not include health.".to_string())
}

pub(super) fn snapshot_agents(payload: &Value) -> Result<&Vec<Value>, String> {
    snapshot_health(payload)?
        .get("agents")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include agents.".to_string())
}

pub(super) fn snapshot_bindings(payload: &Value) -> Result<&Vec<Value>, String> {
    snapshot_health(payload)?
        .get("bindings")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include bindings.".to_string())
}

fn explicit_active_sessions(health: &serde_json::Map<String, Value>) -> Option<&Vec<Value>> {
    health
        .get("activeSessions")
        .and_then(Value::as_array)
        .filter(|sessions| !sessions.is_empty())
}

fn fallback_recent_active_sessions(
    health: &serde_json::Map<String, Value>,
) -> Result<Vec<ActiveSessionRecord>, String> {
    let agents = health
        .get("agents")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include agents.".to_string())?;
    let mut records = Vec::new();

    for agent in agents {
        let agent_id = agent
            .get("agentId")
            .and_then(Value::as_str)
            .ok_or_else(|| "OpenClaw agent payload is missing agentId.".to_string())?;
        let recent_sessions = agent
            .get("sessions")
            .and_then(|sessions| sessions.get("recent"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for session in recent_sessions {
            records.push(map_active_session_record(&session, Some(agent_id))?);
        }
    }

    normalize_active_session_records(records)
}

pub(super) fn snapshot_active_sessions(
    payload: &Value,
) -> Result<Vec<ActiveSessionRecord>, String> {
    let health = snapshot_health(payload)?;

    if let Some(sessions) = explicit_active_sessions(health) {
        return normalize_active_sessions(sessions);
    }

    fallback_recent_active_sessions(health)
}
