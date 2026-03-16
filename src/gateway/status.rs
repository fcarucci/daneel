// SPDX-License-Identifier: Apache-2.0

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

pub(crate) fn map_gateway_level(label: &str) -> GatewayLevel {
    if label.eq_ignore_ascii_case("healthy") {
        GatewayLevel::Healthy
    } else {
        GatewayLevel::Degraded
    }
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

    let health_label = health_frame
        .payload
        .as_ref()
        .and_then(|payload| find_string(payload, &["status", "health", "state"]))
        .unwrap_or_else(|| "healthy".to_string());

    GatewayStatusSnapshot {
        connected: true,
        level: map_gateway_level(&health_label),
        summary: format!("Connected to the OpenClaw Gateway over WebSocket ({health_label})."),
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
