// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::models::runtime::ActiveSessionRecord;
use crate::utils::time::ACTIVE_WINDOW_MS;

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

pub(super) fn snapshot_bindings(payload: &Value) -> Result<Vec<Value>, String> {
    Ok(snapshot_health(payload)?
        .get("bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
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
            let record = map_active_session_record(&session, Some(agent_id))?;
            if record.age_ms.is_none_or(|age_ms| age_ms < ACTIVE_WINDOW_MS) {
                records.push(record);
            }
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::snapshot_active_sessions;
    use crate::utils::time::ACTIVE_WINDOW_MS;

    /// Documents the gap fixed by reading `sessions.recent` in `map_agent_node`: fallback session
    /// synthesis intentionally ignores stale recent rows, so graph nodes cannot rely on merge alone.
    #[test]
    fn active_session_fallback_omits_recent_sessions_older_than_active_window() {
        let stale_age = ACTIVE_WINDOW_MS + 9_000;
        let payload = json!({
            "snapshot": {
                "health": {
                    "agents": [{
                        "agentId": "quiet-agent",
                        "sessions": {
                            "recent": [{
                                "key": "sess-stale",
                                "sessionId": "sess-stale",
                                "age": stale_age
                            }]
                        }
                    }],
                    "bindings": [],
                    "activeSessions": []
                }
            }
        });

        let sessions = snapshot_active_sessions(&payload).expect("snapshot active sessions");
        assert!(
            sessions.is_empty(),
            "stale recent session must not appear as an active-session row; graph needs node-level recency"
        );
    }
}
