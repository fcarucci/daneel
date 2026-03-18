// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
use crate::adapter::{GatewayAdapter, openclaw::OpenClawAdapter};
#[cfg(feature = "server")]
use crate::gateway::{LoadedGatewayConfig, load_gateway_config};

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct ServerAppState<A: GatewayAdapter> {
    gateway_config: LoadedGatewayConfig,
    adapter: A,
}

#[cfg(feature = "server")]
impl<A: GatewayAdapter> ServerAppState<A> {
    pub fn new(gateway_config: LoadedGatewayConfig, adapter: A) -> Self {
        Self {
            gateway_config,
            adapter,
        }
    }

    pub fn from_loader(
        load_config: impl FnOnce() -> Result<LoadedGatewayConfig, String>,
        adapter: A,
    ) -> Result<Self, String> {
        let gateway_config = load_config()?;
        Ok(Self::new(gateway_config, adapter))
    }

    pub fn gateway_config(&self) -> &LoadedGatewayConfig {
        &self.gateway_config
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

#[cfg(feature = "server")]
pub type DaneelAppState = ServerAppState<OpenClawAdapter>;

#[cfg(feature = "server")]
static APP_STATE: OnceLock<Result<DaneelAppState, String>> = OnceLock::new();

#[cfg(feature = "server")]
pub fn warm_server_app_state() -> Result<&'static DaneelAppState, String> {
    let state = server_app_state()?;
    let _ = state.adapter();
    Ok(state)
}

#[cfg(feature = "server")]
pub fn server_app_state() -> Result<&'static DaneelAppState, String> {
    cached_app_state(&APP_STATE, || {
        DaneelAppState::from_loader(load_gateway_config, OpenClawAdapter)
    })
}

#[cfg(feature = "server")]
fn cached_app_state<'a, A: GatewayAdapter>(
    slot: &'a OnceLock<Result<ServerAppState<A>, String>>,
    init: impl FnOnce() -> Result<ServerAppState<A>, String>,
) -> Result<&'a ServerAppState<A>, String> {
    let state = slot.get_or_init(init);

    match state {
        Ok(state) => Ok(state),
        Err(error) => Err(error.clone()),
    }
}

#[cfg(feature = "server")]
pub fn server_gateway_config() -> Result<&'static LoadedGatewayConfig, String> {
    Ok(server_app_state()?.gateway_config())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use std::sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    use async_trait::async_trait;

    use crate::adapter::GatewayAdapter;
    use crate::models::{
        gateway::{GatewayLevel, GatewayStatusSnapshot},
        graph::{AgentEdge, AgentNode},
        runtime::ActiveSessionRecord,
    };

    use super::ServerAppState;
    use crate::gateway::LoadedGatewayConfig;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockAdapter {
        name: &'static str,
    }

    #[async_trait]
    impl GatewayAdapter for MockAdapter {
        async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
            Ok(GatewayStatusSnapshot {
                connected: true,
                level: GatewayLevel::Healthy,
                summary: "healthy".to_string(),
                detail: self.name.to_string(),
                gateway_url: "ws://127.0.0.1:18789/".to_string(),
                protocol_version: Some(3),
                state_version: Some(1),
                uptime_ms: Some(1),
            })
        }

        async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
            Ok(Vec::new())
        }

        async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(Vec::new())
        }

        async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
            Ok(Vec::new())
        }

        async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn app_state_initializes_with_mock_adapter() {
        let adapter = MockAdapter {
            name: "mock-adapter",
        };
        let state = ServerAppState::from_loader(
            || LoadedGatewayConfig::new("test-token", "ws://127.0.0.1:18789/"),
            adapter.clone(),
        )
        .expect("build app state with mock adapter");

        assert_eq!(state.gateway_config().token, "test-token");
        assert_eq!(state.gateway_config().ws_url, "ws://127.0.0.1:18789/");
        assert_eq!(state.adapter(), &adapter);
    }

    #[test]
    fn app_state_initialization_fails_cleanly_with_invalid_config() {
        let error = ServerAppState::from_loader(
            || LoadedGatewayConfig::new("", "ws://127.0.0.1:18789/"),
            MockAdapter {
                name: "mock-adapter",
            },
        )
        .expect_err("reject invalid config");

        assert!(error.contains("No gateway auth token"));
    }

    #[test]
    fn cached_app_state_initializes_only_once() {
        let slot = OnceLock::new();
        let init_count = AtomicUsize::new(0);

        let first = super::cached_app_state(&slot, || {
            init_count.fetch_add(1, Ordering::SeqCst);
            ServerAppState::from_loader(
                || LoadedGatewayConfig::new("test-token", "ws://127.0.0.1:18789/"),
                MockAdapter {
                    name: "mock-adapter",
                },
            )
        })
        .expect("initialize cached app state");

        let second = super::cached_app_state(&slot, || {
            init_count.fetch_add(1, Ordering::SeqCst);
            ServerAppState::from_loader(
                || LoadedGatewayConfig::new("another-token", "ws://127.0.0.1:18790/"),
                MockAdapter {
                    name: "other-adapter",
                },
            )
        })
        .expect("reuse cached app state");

        assert_eq!(init_count.load(Ordering::SeqCst), 1);
        assert!(std::ptr::eq(first, second));
        assert_eq!(second.gateway_config().token, "test-token");
        assert_eq!(second.adapter().name, "mock-adapter");
    }

    #[test]
    fn cached_app_state_reuses_the_first_error() {
        let slot = OnceLock::new();
        let init_count = AtomicUsize::new(0);

        let first = super::cached_app_state(&slot, || {
            init_count.fetch_add(1, Ordering::SeqCst);
            ServerAppState::from_loader(
                || LoadedGatewayConfig::new("", "ws://127.0.0.1:18789/"),
                MockAdapter {
                    name: "mock-adapter",
                },
            )
        })
        .expect_err("cache the first initialization error");

        let second = super::cached_app_state(&slot, || {
            init_count.fetch_add(1, Ordering::SeqCst);
            ServerAppState::from_loader(
                || LoadedGatewayConfig::new("test-token", "ws://127.0.0.1:18789/"),
                MockAdapter {
                    name: "other-adapter",
                },
            )
        })
        .expect_err("reuse cached initialization error");

        assert_eq!(init_count.load(Ordering::SeqCst), 1);
        assert_eq!(first, second);
        assert!(second.contains("No gateway auth token"));
    }
}
