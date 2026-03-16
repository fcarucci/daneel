// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use serde_json::Value;

use crate::models::agents::{AgentOverviewItem, AgentOverviewSnapshot};

use super::parse::{require_array, require_object};
use super::session_store::{count_active_sessions, now_ms};

#[cfg(feature = "server")]
use super::config::LoadedGatewayConfig;
#[cfg(feature = "server")]
use super::ws::connect_gateway;

const ACTIVE_SESSION_WINDOW_MINUTES: u64 = 10;

#[cfg(feature = "server")]
pub(crate) async fn load_agent_overview() -> Result<AgentOverviewSnapshot, String> {
    let config = super::config::load_gateway_config()?;
    fetch_agent_overview_via_websocket(&config).await
}

#[cfg(not(feature = "server"))]
pub(crate) async fn load_agent_overview() -> Result<AgentOverviewSnapshot, String> {
    Err("Agent data is only available from the server build.".to_string())
}

#[cfg(feature = "server")]
async fn fetch_agent_overview_via_websocket(
    config: &LoadedGatewayConfig,
) -> Result<AgentOverviewSnapshot, String> {
    let (mut socket, connect_frame) = connect_gateway(config, "connect-agents-1").await?;
    let _ = socket.close(None).await;

    let payload = connect_frame
        .payload
        .ok_or_else(|| "Gateway connect response did not include a payload.".to_string())?;

    map_agent_overview_snapshot(&payload)
}

fn map_agent_overview_item(
    agent: &Value,
    snapshot_timestamp_ms: u64,
) -> Result<AgentOverviewItem, String> {
    let (id, name, is_default) = map_agent_identity(agent)?;
    let (heartbeat_enabled, heartbeat_schedule, heartbeat_model) = map_heartbeat(agent);
    let (active_session_count, latest_session_key, latest_activity_age_ms) =
        map_session_recency(agent, snapshot_timestamp_ms)?;

    Ok(AgentOverviewItem {
        id,
        name,
        is_default,
        heartbeat_enabled,
        heartbeat_schedule,
        heartbeat_model,
        active_session_count,
        latest_session_key,
        latest_activity_age_ms,
    })
}

fn map_agent_overview_snapshot(payload: &Value) -> Result<AgentOverviewSnapshot, String> {
    let snapshot = payload
        .get("snapshot")
        .ok_or_else(|| "Gateway connect payload did not include a snapshot.".to_string())?;
    let health = require_object(snapshot, "health", "Gateway snapshot")?;
    let agents = require_array(health, "agents", "Gateway health snapshot")?;

    let default_agent_id = health
        .get("defaultAgentId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let snapshot_timestamp_ms = health
        .get("ts")
        .and_then(Value::as_u64)
        .unwrap_or_else(now_ms);

    let mut total_active_sessions = 0_u64;
    let mut items = Vec::with_capacity(agents.len());

    for agent in agents {
        let item = map_agent_overview_item(agent, snapshot_timestamp_ms)?;
        total_active_sessions += item.active_session_count;
        items.push(item);
    }

    let active_recent_agents = items
        .iter()
        .filter(|agent| agent.active_session_count > 0)
        .count();

    Ok(AgentOverviewSnapshot {
        total_agents: items.len(),
        default_agent_id,
        total_active_sessions,
        active_recent_agents,
        agents: items,
    })
}

fn map_agent_identity(agent: &Value) -> Result<(String, String, bool), String> {
    let id = agent
        .get("agentId")
        .and_then(Value::as_str)
        .ok_or_else(|| "Gateway agent snapshot is missing agentId.".to_string())?
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

    Ok((id, name, is_default))
}

fn map_heartbeat(agent: &Value) -> (bool, String, Option<String>) {
    let heartbeat = agent.get("heartbeat").unwrap_or(&Value::Null);
    let heartbeat_enabled = heartbeat
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let heartbeat_schedule = if heartbeat_enabled {
        heartbeat
            .get("every")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "Scheduled".to_string())
    } else {
        "Disabled".to_string()
    };
    let heartbeat_model = heartbeat
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    (heartbeat_enabled, heartbeat_schedule, heartbeat_model)
}

fn map_session_recency(
    agent: &Value,
    snapshot_timestamp_ms: u64,
) -> Result<(u64, Option<String>, Option<u64>), String> {
    let sessions = agent.get("sessions").unwrap_or(&Value::Null);
    let session_store_path = sessions.get("path").and_then(Value::as_str);
    let active_session_count = session_store_path
        .map(|path| {
            count_active_sessions(
                Path::new(path),
                snapshot_timestamp_ms,
                ACTIVE_SESSION_WINDOW_MINUTES,
            )
        })
        .transpose()?
        .unwrap_or(0);

    let recent = sessions
        .get("recent")
        .and_then(Value::as_array)
        .and_then(|entries| entries.first());
    let latest_session_key = recent
        .and_then(|entry| entry.get("key"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let latest_activity_age_ms = recent
        .and_then(|entry| entry.get("age"))
        .and_then(Value::as_u64);

    Ok((
        active_session_count,
        latest_session_key,
        latest_activity_age_ms,
    ))
}
