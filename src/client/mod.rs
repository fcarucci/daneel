// SPDX-License-Identifier: Apache-2.0

use crate::models::{agents::AgentOverviewSnapshot, gateway::GatewayStatusSnapshot};
use dioxus::prelude::ServerFnError;

#[cfg(test)]
pub mod test_support;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests;

/// UI-facing client interface for Daneel request-response operations.
/// This boundary separates UI code from transport-specific details
/// and prepares the backend contract for future native clients.
#[cfg_attr(feature = "server", async_trait::async_trait)]
pub trait AppClient: Send + Sync + 'static {
    /// Fetch gateway status snapshot
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError>;

    /// Fetch agent overview snapshot
    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError>;
}

/// Web-specific implementation that delegates to Dioxus server functions
#[derive(Default, Clone)]
pub struct WebAppClient;

#[cfg_attr(feature = "server", async_trait::async_trait)]
impl AppClient for WebAppClient {
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError> {
        crate::gateway::get_gateway_status().await
    }

    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError> {
        crate::gateway::get_agent_overview().await
    }
}