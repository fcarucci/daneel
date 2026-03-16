// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(feature = "server"), allow(dead_code))]

use dioxus::prelude::{ServerFnError, dioxus_fullstack, server};

use crate::models::{agents::AgentOverviewSnapshot, gateway::GatewayStatusSnapshot};

mod agents;
mod config;
mod parse;
mod session_store;
mod status;
mod ws;

pub(crate) use config::{DEFAULT_GATEWAY_URL, load_gateway_config};
#[cfg(feature = "server")]
pub(crate) use config::LoadedGatewayConfig;

#[cfg(feature = "server")]
pub(crate) use ws::{connect_request, wait_for_response};

#[server]
pub async fn get_gateway_status() -> Result<GatewayStatusSnapshot, ServerFnError> {
    Ok(load_gateway_status().await)
}

#[server]
pub async fn get_agent_overview() -> Result<AgentOverviewSnapshot, ServerFnError> {
    agents::load_agent_overview()
        .await
        .map_err(ServerFnError::new)
}

async fn load_gateway_status() -> GatewayStatusSnapshot {
    let config = match load_gateway_config() {
        Ok(config) => config,
        Err(error) => {
            return degraded_gateway_status(
                DEFAULT_GATEWAY_URL.to_string(),
                "Gateway configuration unavailable",
                error,
            );
        }
    };

    #[cfg(feature = "server")]
    {
        match status::fetch_gateway_status_via_websocket(&config).await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                degraded_gateway_status(config.ws_url.clone(), "Gateway connection failed", error)
            }
        }
    }

    #[cfg(not(feature = "server"))]
    {
        degraded_gateway_status(
            config.ws_url,
            "Gateway status is only available from the server build",
            "Run Daneel in fullstack mode so server functions can connect to OpenClaw.".to_string(),
        )
    }
}

fn degraded_gateway_status(
    gateway_url: String,
    summary: impl Into<String>,
    detail: impl Into<String>,
) -> GatewayStatusSnapshot {
    GatewayStatusSnapshot::degraded(gateway_url, summary, detail)
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use crate::models::gateway::GatewayLevel;

    use super::{connect_request, load_gateway_config};

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
        let snapshot = status::fetch_gateway_status_via_websocket(&config)
            .await
            .expect("fetch gateway status over websocket");

        assert!(snapshot.connected);
        assert!(matches!(snapshot.level, GatewayLevel::Healthy));
    }
}
