// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), allow(dead_code))]

#[cfg(feature = "server")]
use async_trait::async_trait;

#[cfg(feature = "server")]
use crate::models::{
    gateway::GatewayStatusSnapshot,
    graph::{AgentEdge, AgentNode},
    runtime::ActiveSessionRecord,
};

#[cfg(feature = "server")]
pub mod openclaw;

#[cfg(feature = "server")]
#[async_trait]
pub trait GatewayAdapter: Clone + Send + Sync + 'static {
    async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String>;

    async fn list_agents(&self) -> Result<Vec<AgentNode>, String>;

    async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String>;

    async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String>;

    async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String>;
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use async_trait::async_trait;

    use super::GatewayAdapter;
    use crate::models::{
        gateway::{GatewayLevel, GatewayStatusSnapshot},
        graph::{AgentEdge, AgentEdgeKind, AgentNode, AgentStatus},
        runtime::ActiveSessionRecord,
    };

    #[derive(Clone, Debug, Default)]
    struct MockAdapter;

    fn sample_agent_node() -> AgentNode {
        AgentNode {
            id: "main".to_string(),
            name: "main".to_string(),
            is_default: true,
            heartbeat_enabled: true,
            heartbeat_schedule: "every 5m".to_string(),
            active_session_count: 1,
            latest_activity_age_ms: Some(1_000),
            status: AgentStatus::Active,
        }
    }

    fn sample_gateway_binding() -> AgentEdge {
        AgentEdge {
            source_id: "main".to_string(),
            target_id: "planner".to_string(),
            kind: AgentEdgeKind::GatewayRouting,
        }
    }

    fn sample_relationship_hint() -> AgentEdge {
        AgentEdge {
            source_id: "main".to_string(),
            target_id: "planner".to_string(),
            kind: AgentEdgeKind::MetadataHint,
        }
    }

    fn sample_active_session() -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: "session-1".to_string(),
            agent_id: "main".to_string(),
            task: Some("status".to_string()),
            age_ms: Some(1_000),
        }
    }

    #[async_trait]
    impl GatewayAdapter for MockAdapter {
        async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
            Ok(GatewayStatusSnapshot {
                connected: true,
                level: GatewayLevel::Healthy,
                summary: "healthy".to_string(),
                detail: "mock".to_string(),
                gateway_url: "ws://127.0.0.1:18789/".to_string(),
                protocol_version: Some(3),
                state_version: Some(1),
                uptime_ms: Some(1000),
            })
        }

        async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
            Ok(vec![sample_agent_node()])
        }

        async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(vec![sample_gateway_binding()])
        }

        async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
            Ok(vec![sample_active_session()])
        }

        async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(vec![sample_relationship_hint()])
        }
    }

    fn assert_trait_usage_is_shared_model_only(
        adapter: &impl GatewayAdapter,
    ) -> impl std::future::Future<
        Output = (
            Result<GatewayStatusSnapshot, String>,
            Result<Vec<AgentNode>, String>,
            Result<Vec<AgentEdge>, String>,
            Result<Vec<ActiveSessionRecord>, String>,
            Result<Vec<AgentEdge>, String>,
        ),
    > + '_ {
        async move {
            (
                adapter.gateway_status().await,
                adapter.list_agents().await,
                adapter.list_agent_bindings().await,
                adapter.list_active_sessions().await,
                adapter.list_agent_relationship_hints().await,
            )
        }
    }

    #[tokio::test]
    async fn mock_adapter_can_satisfy_the_trait() {
        let adapter = MockAdapter;
        let (status, agents, bindings, sessions, hints) =
            assert_trait_usage_is_shared_model_only(&adapter).await;

        assert!(status.expect("status").connected);
        assert_eq!(agents.expect("agents").len(), 1);
        assert_eq!(bindings.expect("bindings").len(), 1);
        assert_eq!(sessions.expect("sessions").len(), 1);
        assert_eq!(hints.expect("hints").len(), 1);
    }
}
