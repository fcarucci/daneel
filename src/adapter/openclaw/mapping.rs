// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::models::{
    graph::{AgentEdge, AgentEdgeKind, AgentNode, AgentStatus},
    runtime::ActiveSessionRecord,
};

pub(super) fn map_heartbeat(agent: &Value) -> (bool, String) {
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

pub(super) fn map_agent_node(agent: &Value) -> Result<AgentNode, String> {
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

pub(super) fn map_binding_edge(binding: &Value) -> Result<AgentEdge, String> {
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
        kind: AgentEdgeKind::RoutesTo,
    })
}

pub(super) fn normalize_binding_edges(bindings: &[Value]) -> Result<Vec<AgentEdge>, String> {
    let mut edges: Vec<_> = bindings
        .iter()
        .map(map_binding_edge)
        .collect::<Result<Vec<_>, _>>()?;
    edges.sort_by(|left, right| {
        (&left.source_id, &left.target_id, edge_kind_rank(&left.kind)).cmp(&(
            &right.source_id,
            &right.target_id,
            edge_kind_rank(&right.kind),
        ))
    });
    edges.dedup_by(|left, right| {
        left.source_id == right.source_id && left.target_id == right.target_id
    });
    Ok(edges)
}

fn edge_kind_rank(kind: &AgentEdgeKind) -> u8 {
    match kind {
        AgentEdgeKind::RoutesTo => 0,
        AgentEdgeKind::DelegatesToHint => 1,
        AgentEdgeKind::WorksWithHint => 2,
    }
}

pub(super) fn map_active_session_record(
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

pub(super) fn normalize_active_sessions(
    sessions: &[Value],
) -> Result<Vec<ActiveSessionRecord>, String> {
    let records: Vec<_> = sessions
        .iter()
        .map(|session| map_active_session_record(session, None))
        .collect::<Result<Vec<_>, _>>()?;
    normalize_active_session_records(records)
}

pub(super) fn normalize_active_session_records(
    mut records: Vec<ActiveSessionRecord>,
) -> Result<Vec<ActiveSessionRecord>, String> {
    records.sort_by(|left, right| left.session_id.cmp(&right.session_id));
    records.dedup_by(|left, right| left.session_id == right.session_id);
    Ok(records)
}
