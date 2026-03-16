// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::models::gateway::{GatewayLevel, GatewayStatusSnapshot};

use super::config::LoadedGatewayConfig;
use super::parse::{find_string, find_u64};
#[cfg(feature = "server")]
use super::ws::{connect_gateway, request_gateway};

#[cfg(feature = "server")]
use serde_json::json;

#[cfg(feature = "server")]
pub(crate) async fn fetch_gateway_status_via_websocket(
    config: &LoadedGatewayConfig,
) -> Result<GatewayStatusSnapshot, String> {
    let (mut socket, connect_frame) = connect_gateway(config, "connect-1").await?;
    let health_frame = request_gateway(&mut socket, "health-1", "health", json!({})).await?;
    let _ = socket.close(None).await;
    Ok(map_gateway_status_snapshot(
        config,
        &connect_frame,
        &health_frame,
    ))
}

pub(crate) fn map_gateway_status_snapshot(
    config: &LoadedGatewayConfig,
    connect_frame: &super::ws::Envelope,
    health_frame: &super::ws::Envelope,
) -> GatewayStatusSnapshot {
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

    let health = health_status_from_payload(health_frame.payload.as_ref());

    GatewayStatusSnapshot {
        connected: true,
        level: health.level(),
        summary: format!(
            "Connected to the OpenClaw Gateway over WebSocket ({}).",
            health.label
        ),
        detail: format!(
            "Gateway status was fetched through the documented loopback WS connection at {}.",
            config.ws_url
        ),
        gateway_url: config.ws_url.clone(),
        protocol_version,
        state_version,
        uptime_ms,
    }
}

#[derive(Clone, Copy, Debug)]
enum HealthState {
    Healthy,
    Degraded,
}

impl HealthState {
    fn from_label(label: &str) -> Self {
        if label.eq_ignore_ascii_case("healthy") {
            HealthState::Healthy
        } else {
            HealthState::Degraded
        }
    }

    fn level(self) -> GatewayLevel {
        match self {
            HealthState::Healthy => GatewayLevel::Healthy,
            HealthState::Degraded => GatewayLevel::Degraded,
        }
    }
}

struct HealthStatus {
    label: String,
    state: HealthState,
}

impl HealthStatus {
    fn level(&self) -> GatewayLevel {
        self.state.level()
    }
}

fn health_status_from_payload(payload: Option<&Value>) -> HealthStatus {
    let label = payload
        .and_then(|payload| find_string(payload, &["status", "health", "state"]))
        .unwrap_or_else(|| "healthy".to_string());
    let state = HealthState::from_label(&label);
    HealthStatus { label, state }
}
