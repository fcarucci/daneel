// SPDX-License-Identifier: Apache-2.0

//! Gateway status and graph snapshot resources lifted above the route `Outlet`
//! so navigating away from `/` does not reset loading state (same pattern as
//! agent overview, issue #61).

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::components::shell_provider_utils::sync_last_ok_snapshot;
use crate::models::{gateway::GatewayStatusSnapshot, graph::AgentGraphSnapshot};

#[derive(Clone)]
pub struct DashboardDataContext {
    pub gateway_status: Resource<Result<GatewayStatusSnapshot, ServerFnError>>,
    pub graph_snapshot: Resource<Result<AgentGraphSnapshot, ServerFnError>>,
    pub cached_gateway_status: Signal<Option<GatewayStatusSnapshot>>,
    pub cached_graph_snapshot: Signal<Option<AgentGraphSnapshot>>,
}

pub fn use_dashboard_data() -> DashboardDataContext {
    use_context::<DashboardDataContext>()
}

#[component]
pub fn DashboardDataProvider(children: Element) -> Element {
    let client = use_app_client();
    let gateway_client = client.clone();
    let graph_client = client.clone();

    let gateway_status = use_resource(move || {
        let client = gateway_client.clone();
        async move { client.get_gateway_status().await }
    });
    let graph_snapshot = use_resource(move || {
        let client = graph_client.clone();
        async move { client.get_agent_graph_snapshot().await }
    });

    let cached_gateway_status = use_signal(|| None::<GatewayStatusSnapshot>);
    let cached_graph_snapshot = use_signal(|| None::<AgentGraphSnapshot>);

    use_effect({
        let gateway_status = gateway_status.clone();
        let cache = cached_gateway_status;
        move || {
            sync_last_ok_snapshot(&gateway_status, cache);
        }
    });

    use_effect({
        let graph_snapshot = graph_snapshot.clone();
        let cache = cached_graph_snapshot;
        move || {
            sync_last_ok_snapshot(&graph_snapshot, cache);
        }
    });

    let ctx = DashboardDataContext {
        gateway_status,
        graph_snapshot,
        cached_gateway_status,
        cached_graph_snapshot,
    };
    use_context_provider(|| ctx.clone());

    rsx! {
        {children}
    }
}
