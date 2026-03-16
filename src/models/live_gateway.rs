// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveGatewayLevel {
    Healthy,
    Degraded,
    Connecting,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiveGatewayEvent {
    pub level: LiveGatewayLevel,
    pub summary: String,
    pub detail: String,
}
