#![cfg_attr(not(feature = "server"), allow(dead_code))]

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use dioxus::prelude::{ServerFnError, dioxus_fullstack, server};
use serde::Deserialize;
use serde_json::Value;

#[cfg(feature = "server")]
use serde_json::json;

use crate::models::{
    agents::{AgentOverviewItem, AgentOverviewSnapshot},
    gateway::GatewayStatusSnapshot,
};

#[cfg(feature = "server")]
use crate::models::gateway::GatewayLevel;

#[derive(Clone, Debug, Deserialize)]
struct OpenClawConfig {
    #[serde(default)]
    gateway: GatewayConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GatewayConfig {
    #[serde(default = "default_gateway_port")]
    port: u16,
    #[serde(default)]
    auth: OpenClawAuth,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct OpenClawAuth {
    #[serde(default)]
    token: String,
}

#[derive(Debug)]
pub(crate) struct LoadedGatewayConfig {
    pub(crate) token: String,
    pub(crate) ws_url: String,
}

#[server]
pub async fn get_gateway_status() -> Result<GatewayStatusSnapshot, ServerFnError> {
    Ok(load_gateway_status().await)
}

#[server]
pub async fn get_agent_overview() -> Result<AgentOverviewSnapshot, ServerFnError> {
    load_agent_overview().await.map_err(ServerFnError::new)
}

async fn load_gateway_status() -> GatewayStatusSnapshot {
    let config = match load_gateway_config() {
        Ok(config) => config,
        Err(error) => {
            return GatewayStatusSnapshot::degraded(
                "ws://127.0.0.1:18789".to_string(),
                "Gateway configuration unavailable",
                error,
            );
        }
    };

    #[cfg(feature = "server")]
    {
        match fetch_gateway_status_via_websocket(&config).await {
            Ok(snapshot) => snapshot,
            Err(error) => GatewayStatusSnapshot::degraded(
                config.ws_url.clone(),
                "Gateway connection failed",
                error,
            ),
        }
    }

    #[cfg(not(feature = "server"))]
    {
        GatewayStatusSnapshot::degraded(
            config.ws_url,
            "Gateway status is only available from the server build",
            "Run Daneel in fullstack mode so server functions can connect to OpenClaw.".to_string(),
        )
    }
}

pub(crate) fn load_gateway_config() -> Result<LoadedGatewayConfig, String> {
    let path = openclaw_config_path()?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let parsed: OpenClawConfig = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))?;

    if parsed.gateway.auth.token.is_empty() {
        return Err(format!(
            "No gateway auth token was found in {}.",
            path.display()
        ));
    }

    Ok(LoadedGatewayConfig {
        token: parsed.gateway.auth.token,
        ws_url: format!("ws://127.0.0.1:{}/", parsed.gateway.port),
    })
}

fn openclaw_config_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("OPENCLAW_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = env::var("HOME").map_err(|_| "HOME is not set.".to_string())?;
    Ok(PathBuf::from(home).join(".openclaw").join("openclaw.json"))
}

fn default_gateway_port() -> u16 {
    18_789
}

#[cfg(feature = "server")]
pub(crate) fn connect_request(request_id: &str, token: &str) -> Value {
    json!({
        "type": "req",
        "id": request_id,
        "method": "connect",
        "params": {
            "minProtocol": 3,
            "maxProtocol": 3,
            "role": "operator",
            "scopes": ["operator.read"],
            "client": {
                "id": "gateway-client",
                "version": env!("CARGO_PKG_VERSION"),
                "platform": std::env::consts::OS,
                "mode": "backend"
            },
            "auth": {
                "token": token
            }
        }
    })
}

#[cfg(feature = "server")]
async fn fetch_gateway_status_via_websocket(
    config: &LoadedGatewayConfig,
) -> Result<GatewayStatusSnapshot, String> {
    use futures_util::SinkExt;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (mut socket, _) = connect_async(config.ws_url.as_str())
        .await
        .map_err(|error| {
            format!(
                "Could not open gateway websocket {}: {error}",
                config.ws_url
            )
        })?;

    socket
        .send(Message::Text(
            connect_request("connect-1", &config.token)
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| format!("Could not send gateway connect request: {error}"))?;

    let connect_frame = wait_for_response(&mut socket, "connect-1").await?;

    socket
        .send(Message::Text(
            json!({
                "type": "req",
                "id": "health-1",
                "method": "health",
                "params": {}
            })
            .to_string()
            .into(),
        ))
        .await
        .map_err(|error| format!("Could not send gateway health request: {error}"))?;

    let health_frame = wait_for_response(&mut socket, "health-1").await?;
    let _ = socket.close(None).await;

    let protocol_version = connect_frame
        .payload
        .as_ref()
        .and_then(|payload| find_u64(payload, &["protocolVersion"]))
        .map(|value| value as u32);

    let state_version = health_frame
        .payload
        .as_ref()
        .and_then(|payload| find_u64(payload, &["stateVersion"]))
        .or_else(|| {
            connect_frame
                .payload
                .as_ref()
                .and_then(|payload| find_u64(payload, &["stateVersion"]))
        });

    let uptime_ms = health_frame
        .payload
        .as_ref()
        .and_then(|payload| find_u64(payload, &["uptimeMs"]))
        .or_else(|| {
            connect_frame
                .payload
                .as_ref()
                .and_then(|payload| find_u64(payload, &["uptimeMs"]))
        });

    let health_label = health_frame
        .payload
        .as_ref()
        .and_then(|payload| find_string(payload, &["status", "health", "state"]))
        .unwrap_or_else(|| "healthy".to_string());
    let level = if health_label.eq_ignore_ascii_case("healthy") {
        GatewayLevel::Healthy
    } else {
        GatewayLevel::Degraded
    };

    Ok(GatewayStatusSnapshot {
        connected: true,
        level,
        summary: format!("Connected to the OpenClaw Gateway over WebSocket ({health_label})."),
        detail: format!(
            "Gateway status was fetched through the documented loopback WS connection at {}.",
            config.ws_url
        ),
        gateway_url: config.ws_url.clone(),
        protocol_version,
        state_version,
        uptime_ms,
    })
}

#[cfg(feature = "server")]
async fn load_agent_overview() -> Result<AgentOverviewSnapshot, String> {
    let config = load_gateway_config()?;
    fetch_agent_overview_via_websocket(&config).await
}

#[cfg(not(feature = "server"))]
async fn load_agent_overview() -> Result<AgentOverviewSnapshot, String> {
    Err("Agent data is only available from the server build.".to_string())
}

#[cfg(feature = "server")]
async fn fetch_agent_overview_via_websocket(
    config: &LoadedGatewayConfig,
) -> Result<AgentOverviewSnapshot, String> {
    use futures_util::SinkExt;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (mut socket, _) = connect_async(config.ws_url.as_str())
        .await
        .map_err(|error| {
            format!(
                "Could not open gateway websocket {}: {error}",
                config.ws_url
            )
        })?;

    socket
        .send(Message::Text(
            connect_request("connect-agents-1", &config.token)
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| format!("Could not send gateway connect request: {error}"))?;

    let connect_frame = wait_for_response(&mut socket, "connect-agents-1").await?;
    let _ = socket.close(None).await;

    let payload = connect_frame
        .payload
        .ok_or_else(|| "Gateway connect response did not include a payload.".to_string())?;

    map_agent_overview_snapshot(&payload)
}

#[cfg(feature = "server")]
pub(crate) async fn wait_for_response<
    S: futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
>(
    socket: &mut S,
    expected_id: &str,
) -> Result<Envelope, String> {
    use futures_util::StreamExt;
    use tokio::time::{Duration, timeout};
    use tokio_tungstenite::tungstenite::Message;

    loop {
        let frame = timeout(Duration::from_secs(10), socket.next())
            .await
            .map_err(|_| format!("Timed out waiting for gateway response {expected_id}."))?
            .ok_or_else(|| {
                format!("Gateway closed the socket before responding to {expected_id}.")
            })?
            .map_err(|error| {
                format!("Gateway websocket error while waiting for {expected_id}: {error}")
            })?;

        let Message::Text(text) = frame else {
            continue;
        };

        let envelope: Envelope = serde_json::from_str(&text)
            .map_err(|error| format!("Could not parse gateway response frame: {error}"))?;

        if envelope.kind == "res" && envelope.id.as_deref() == Some(expected_id) {
            if envelope.ok.unwrap_or(false) {
                return Ok(envelope);
            }

            let detail = envelope
                .error
                .as_ref()
                .and_then(|value| find_string(value, &["message", "code", "type"]))
                .unwrap_or_else(|| "Unknown gateway error".to_string());
            return Err(format!("Gateway request {expected_id} failed: {detail}"));
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::{
        GatewayLevel, connect_request, fetch_gateway_status_via_websocket, load_gateway_config,
    };

    #[test]
    fn connect_request_uses_backend_gateway_identity() {
        let request = connect_request("connect-test-1", "test-token");

        assert_eq!(request["method"], "connect");
        assert_eq!(request["params"]["client"]["id"], "gateway-client");
        assert_eq!(request["params"]["client"]["mode"], "backend");
        assert_eq!(request["params"]["auth"]["token"], "test-token");
    }

    #[tokio::test]
    #[ignore = "manual live verification against the developer's local OpenClaw gateway"]
    async fn live_gateway_status_fetch_reports_healthy() {
        let config = load_gateway_config().expect("load local OpenClaw gateway config");
        let snapshot = fetch_gateway_status_via_websocket(&config)
            .await
            .expect("fetch gateway status over websocket");

        assert!(snapshot.connected);
        assert!(matches!(snapshot.level, GatewayLevel::Healthy));
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    ok: Option<bool>,
    #[serde(default)]
    payload: Option<Value>,
    #[serde(default)]
    error: Option<Value>,
}

fn find_string(value: &Value, candidates: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for candidate in candidates {
                if let Some(Value::String(found)) = map.get(*candidate) {
                    return Some(found.clone());
                }
            }

            for nested in map.values() {
                if let Some(found) = find_string(nested, candidates) {
                    return Some(found);
                }
            }

            None
        }
        Value::Array(values) => values
            .iter()
            .find_map(|nested| find_string(nested, candidates)),
        _ => None,
    }
}

fn find_u64(value: &Value, candidates: &[&str]) -> Option<u64> {
    match value {
        Value::Object(map) => {
            for candidate in candidates {
                if let Some(Value::Number(found)) = map.get(*candidate)
                    && let Some(parsed) = found.as_u64()
                {
                    return Some(parsed);
                }
            }

            for nested in map.values() {
                if let Some(found) = find_u64(nested, candidates) {
                    return Some(found);
                }
            }

            None
        }
        Value::Array(values) => values
            .iter()
            .find_map(|nested| find_u64(nested, candidates)),
        _ => None,
    }
}

fn map_agent_overview_snapshot(payload: &Value) -> Result<AgentOverviewSnapshot, String> {
    let snapshot = payload
        .get("snapshot")
        .ok_or_else(|| "Gateway connect payload did not include a snapshot.".to_string())?;
    let health = snapshot
        .get("health")
        .ok_or_else(|| "Gateway snapshot did not include health data.".to_string())?;
    let agents = health
        .get("agents")
        .and_then(Value::as_array)
        .ok_or_else(|| "Gateway health snapshot did not include an agents list.".to_string())?;

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

        let sessions = agent.get("sessions").unwrap_or(&Value::Null);
        let session_store_path = sessions.get("path").and_then(Value::as_str);
        let active_session_count = session_store_path
            .map(|path| count_active_sessions(Path::new(path), snapshot_timestamp_ms, 10))
            .transpose()?
            .unwrap_or(0);
        total_active_sessions += active_session_count;

        let recent = sessions
            .get("recent")
            .and_then(Value::as_array)
            .and_then(|recent| recent.first());
        let latest_session_key = recent
            .and_then(|entry| entry.get("key"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let latest_activity_age_ms = recent
            .and_then(|entry| entry.get("age"))
            .and_then(Value::as_u64);

        items.push(AgentOverviewItem {
            id,
            name,
            is_default,
            heartbeat_enabled,
            heartbeat_schedule,
            heartbeat_model,
            active_session_count,
            latest_session_key,
            latest_activity_age_ms,
        });
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

fn count_active_sessions(
    path: &Path,
    reference_timestamp_ms: u64,
    active_minutes: u64,
) -> Result<u64, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Could not read session store {}: {error}", path.display()))?;
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse session store {}: {error}", path.display()))?;
    let entries = parsed
        .as_object()
        .ok_or_else(|| format!("Session store {} is not a JSON object.", path.display()))?;

    let cutoff_ms = reference_timestamp_ms.saturating_sub(active_minutes * 60_000);
    Ok(entries
        .values()
        .filter_map(|entry| entry.get("updatedAt").and_then(Value::as_u64))
        .filter(|updated_at| *updated_at >= cutoff_ms)
        .count() as u64)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
// SPDX-License-Identifier: Apache-2.0
