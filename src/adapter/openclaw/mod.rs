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
mod fetch;
#[cfg(feature = "server")]
mod mapping;
#[cfg(feature = "server")]
mod snapshot;

#[cfg(all(test, feature = "server"))]
mod tests;

#[cfg(feature = "server")]
use fetch::fetch_connect_payload;
#[cfg(feature = "server")]
use mapping::{map_agent_node, normalize_binding_edges};
#[cfg(feature = "server")]
use snapshot::{snapshot_active_sessions, snapshot_agents, snapshot_bindings};

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
        let payload = fetch_connect_payload("connect-list-agents-1").await?;
        let agents = snapshot_agents(&payload)?;

        agents.iter().map(map_agent_node).collect()
    }

    async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
        let payload = fetch_connect_payload("connect-list-bindings-1").await?;
        let bindings = snapshot_bindings(&payload)?;

        normalize_binding_edges(bindings)
    }

    async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
        let payload = fetch_connect_payload("connect-list-active-sessions-1").await?;
        snapshot_active_sessions(&payload)
    }

    async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
        not_implemented("list_agent_relationship_hints")
    }
}
