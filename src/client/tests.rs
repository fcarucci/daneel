// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use super::{AppClientHandle, WebAppClient, use_app_client};
use crate::client::test_support::MockAppClient;

#[component]
fn ClientProviderHarness() -> Element {
    let _client = use_app_client();
    rsx! { div { "client available" } }
}

#[component]
fn ClientProviderRoot(client: AppClientHandle) -> Element {
    use_context_provider(|| client.clone());
    rsx! {
        ClientProviderHarness {}
    }
}

fn render_with_client(client: AppClientHandle) -> String {
    let mut dom =
        VirtualDom::new_with_props(ClientProviderRoot, ClientProviderRootProps { client });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn web_app_client_is_send_and_sync() {
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}

    requires_send::<WebAppClient>();
    requires_sync::<WebAppClient>();
}

#[test]
fn app_client_handle_is_send_sync_and_cloneable() {
    fn requires_send_sync<T: Send + Sync + Clone>() {}

    requires_send_sync::<AppClientHandle>();
}

#[test]
fn app_client_handle_uses_injected_mock_gateway_data() {
    let client = AppClientHandle::new(MockAppClient::healthy_gateway());

    let gateway_status = pollster::block_on(client.get_gateway_status()).unwrap();
    let agent_overview = pollster::block_on(client.get_agent_overview()).unwrap();
    let graph_snapshot = pollster::block_on(client.get_agent_graph_snapshot()).unwrap();

    assert!(gateway_status.connected);
    assert_eq!(agent_overview.total_agents, 3);
    assert_eq!(graph_snapshot.nodes.len(), 1);
    assert_eq!(graph_snapshot.edges.len(), 1);
}

#[test]
fn error_mapping_preserves_degraded_semantics() {
    let client = AppClientHandle::new(MockAppClient::degraded_gateway());

    let gateway_status = pollster::block_on(client.get_gateway_status()).unwrap();
    let agent_overview = pollster::block_on(client.get_agent_overview());

    assert!(!gateway_status.connected);
    assert_eq!(
        gateway_status.level,
        crate::models::gateway::GatewayLevel::Degraded
    );
    assert!(agent_overview.is_err());
    assert!(
        agent_overview
            .unwrap_err()
            .to_string()
            .contains("Gateway unavailable")
    );
    assert!(
        pollster::block_on(client.get_agent_graph_snapshot())
            .unwrap_err()
            .to_string()
            .contains("Graph unavailable")
    );
}

#[test]
fn app_client_provider_supplies_shared_client_context() {
    let html = render_with_client(AppClientHandle::new(MockAppClient::healthy_gateway()));
    assert!(html.contains("client available"));
}
