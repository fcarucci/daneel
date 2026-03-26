// SPDX-License-Identifier: Apache-2.0

//! Agent overview fetch lifted above route `Outlet` so navigating away from `/agents`
//! does not drop the in-flight or last successful snapshot (issue #61).
//! Naming mirrors `dashboard_data` (`*DataContext`, `*DataProvider`, `use_*_data`).

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::components::shell_provider_utils::sync_last_ok_snapshot;
use crate::models::agents::AgentOverviewSnapshot;

#[derive(Clone)]
pub struct AgentOverviewDataContext {
    pub overview: Resource<Result<AgentOverviewSnapshot, ServerFnError>>,
    pub last_ok: Signal<Option<AgentOverviewSnapshot>>,
}

pub fn use_agent_overview_data() -> AgentOverviewDataContext {
    use_context::<AgentOverviewDataContext>()
}

#[component]
pub fn AgentOverviewDataProvider(children: Element) -> Element {
    let client = use_app_client();
    let last_ok = use_signal(|| None::<AgentOverviewSnapshot>);
    let overview_resource = use_resource(move || {
        let client = client.clone();
        async move { client.get_agent_overview().await }
    });

    use_effect({
        let overview_resource = overview_resource.clone();
        let cache = last_ok;
        move || {
            sync_last_ok_snapshot(&overview_resource, cache);
        }
    });

    let ctx = AgentOverviewDataContext {
        overview: overview_resource,
        last_ok,
    };
    use_context_provider(|| ctx.clone());

    rsx! {
        {children}
    }
}
