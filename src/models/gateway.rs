// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(feature = "server"), allow(dead_code))]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GatewayStatusSnapshot {
    pub connected: bool,
    pub level: GatewayLevel,
    pub summary: String,
    pub detail: String,
    pub gateway_url: String,
    pub protocol_version: Option<u32>,
    pub state_version: Option<u64>,
    pub uptime_ms: Option<u64>,
}

impl GatewayStatusSnapshot {
    pub fn degraded(
        gateway_url: String,
        summary: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            connected: false,
            level: GatewayLevel::Degraded,
            summary: summary.into(),
            detail: detail.into(),
            gateway_url,
            protocol_version: None,
            state_version: None,
            uptime_ms: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayLevel {
    Healthy,
    Degraded,
}
