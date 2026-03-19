// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_history::{History, MemoryHistory};
use dioxus_router::components::HistoryProvider;

use crate::client::AppClientHandle;
use crate::router::Route;

#[component]
fn RouteHarness(path: String) -> Element {
    use_context_provider(AppClientHandle::default);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        HistoryProvider {
            history: move |_| Rc::new(MemoryHistory::with_initial_path(path.clone())) as Rc<dyn History>,
            Router::<Route> {}
        }
    }
}

pub(crate) fn render_route(path: &str) -> String {
    let mut dom = VirtualDom::new_with_props(
        RouteHarness,
        RouteHarnessProps {
            path: path.to_string(),
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[cfg(test)]
mod tests {
    use super::render_route;

    #[test]
    fn dashboard_route_renders_through_the_shared_harness() {
        let html = render_route("/");

        assert!(html.contains("Gateway overview and graph surfaces start here."));
        assert!(html.contains("Gateway status"));
        assert!(html.contains("Agents graph"));
        assert!(html.contains("Loading the latest graph snapshot from Daneel"));
    }

    #[test]
    fn navigation_renders_expected_route_links() {
        let html = render_route("/");

        assert!(html.contains("Dashboard"));
        assert!(html.contains("Agents"));
        assert!(html.contains("Settings"));
    }

    #[test]
    fn unknown_route_renders_the_not_found_fallback() {
        let html = render_route("/missing/route");

        assert!(html.contains("Route not found"));
        assert!(html.contains("/missing/route"));
    }

    #[test]
    fn agents_route_renders_through_the_shared_harness() {
        let html = render_route("/agents");

        assert!(html.contains("Graph View"));
        assert!(html.contains("Loading agents"));
    }
}
