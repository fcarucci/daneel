// SPDX-License-Identifier: Apache-2.0

//! Agent overview fetch lifted above route `Outlet` so navigating away from `/agents`
//! does not drop the in-flight or last successful snapshot (issue #61).

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::models::agents::AgentOverviewSnapshot;

#[derive(Clone)]
pub struct AgentOverviewContext {
    pub overview: Resource<Result<AgentOverviewSnapshot, ServerFnError>>,
    pub last_ok: Signal<Option<AgentOverviewSnapshot>>,
}

pub fn use_agent_overview() -> AgentOverviewContext {
    use_context::<AgentOverviewContext>()
}

#[component]
pub fn AgentOverviewProvider(children: Element) -> Element {
    let client = use_app_client();
    let mut last_ok = use_signal(|| None::<AgentOverviewSnapshot>);
    let agent_overview = use_resource(move || {
        let client = client.clone();
        async move { client.get_agent_overview().await }
    });

    use_effect({
        let agent_overview = agent_overview.clone();
        move || {
            if let Some(Ok(snapshot)) = agent_overview.read().as_ref() {
                last_ok.set(Some(snapshot.clone()));
            }
        }
    });

    let ctx = AgentOverviewContext {
        overview: agent_overview,
        last_ok,
    };
    use_context_provider(|| ctx.clone());

    rsx! {
        {children}
    }
}
