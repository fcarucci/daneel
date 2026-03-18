// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::models::{agents::AgentOverviewSnapshot, gateway::GatewayStatusSnapshot};
use async_trait::async_trait;
use dioxus::prelude::{ServerFnError, use_context};

#[cfg(test)]
pub mod test_support;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests;

/// UI-facing client interface for Daneel request-response operations.
/// This boundary separates UI code from transport-specific details
/// and prepares the backend contract for future native clients.
#[async_trait(?Send)]
pub trait AppClient: Send + Sync + 'static {
    /// Fetch gateway status snapshot
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError>;

    /// Fetch agent overview snapshot
    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError>;
}

#[derive(Clone)]
pub struct AppClientHandle(Arc<dyn AppClient>);

impl PartialEq for AppClientHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for AppClientHandle {}

impl AppClientHandle {
    pub fn new(client: impl AppClient) -> Self {
        Self(Arc::new(client))
    }

    pub async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError> {
        self.0.get_gateway_status().await
    }

    pub async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError> {
        self.0.get_agent_overview().await
    }
}

impl Default for AppClientHandle {
    fn default() -> Self {
        Self::new(WebAppClient)
    }
}

/// Web-specific implementation that delegates to Dioxus server functions
#[derive(Default, Clone)]
pub struct WebAppClient;

#[async_trait(?Send)]
impl AppClient for WebAppClient {
    async fn get_gateway_status(&self) -> Result<GatewayStatusSnapshot, ServerFnError> {
        crate::gateway::get_gateway_status().await
    }

    async fn get_agent_overview(&self) -> Result<AgentOverviewSnapshot, ServerFnError> {
        crate::gateway::get_agent_overview().await
    }
}

pub fn use_app_client() -> AppClientHandle {
    use_context::<AppClientHandle>()
}
