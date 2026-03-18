// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use async_trait::async_trait;

#[cfg(feature = "server")]
use crate::{
    adapter::GatewayAdapter,
    models::{
        gateway::GatewayStatusSnapshot,
        graph::{AgentEdge, AgentNode},
        runtime::ActiveSessionRecord,
    },
};

#[cfg(feature = "server")]
#[derive(Clone, Debug, Default)]
pub struct OpenClawAdapter;

#[cfg(feature = "server")]
fn not_implemented<T>(method: &str) -> Result<T, String> {
    Err(format!(
        "OpenClawAdapter::{method}() is not implemented yet."
    ))
}

#[cfg(feature = "server")]
#[async_trait]
impl GatewayAdapter for OpenClawAdapter {
    async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
        not_implemented("gateway_status")
    }

    async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
        not_implemented("list_agents")
    }

    async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
        not_implemented("list_agent_bindings")
    }

    async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
        not_implemented("list_active_sessions")
    }

    async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
        not_implemented("list_agent_relationship_hints")
    }
}
