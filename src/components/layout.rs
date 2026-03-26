// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::{
    agent_overview::AgentOverviewProvider,
    dashboard_data::DashboardDataProvider,
    live_gateway::{LiveGatewayProvider, use_live_gateway},
    navbar::TopBar,
    sidebar::Sidebar,
};
use crate::router::Route;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        LiveGatewayProvider {
            DashboardDataProvider {
                AgentOverviewProvider {
                    div {
                        class: "mission-shell min-h-screen bg-[var(--app-bg)] text-[var(--ink-0)]",
                        "data-visual-shell": "mission-control",
                        div { class: "grid min-h-screen grid-cols-1 lg:grid-cols-[15.5rem_minmax(0,1fr)]",
                            Sidebar {}
                            LayoutContent {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LayoutContent() -> Element {
    let live_gateway = use_live_gateway();
    let main_class = main_content_class(live_gateway.is_frozen());

    rsx! {
        div { class: "min-w-0",
            TopBar {}
            main { class: format!("{main_class} route-stage"), "data-route-stage": "true",
                Outlet::<Route> {}
            }
        }
    }
}

fn main_content_class(is_frozen: bool) -> &'static str {
    if is_frozen {
        "px-5 pb-8 pt-6 opacity-75 grayscale-[0.18] saturate-[0.8] transition-[opacity,filter] duration-300 sm:px-8 lg:px-10"
    } else {
        "px-5 pb-8 pt-6 transition-[opacity,filter] duration-300 sm:px-8 lg:px-10"
    }
}

#[cfg(test)]
mod tests {
    use super::main_content_class;

    #[test]
    fn frozen_layout_uses_subdued_class() {
        let class = main_content_class(true);

        assert!(class.contains("opacity-75"));
        assert!(class.contains("grayscale"));
    }

    #[test]
    fn live_layout_keeps_normal_class() {
        let class = main_content_class(false);

        assert!(!class.contains("opacity-75"));
        assert!(!class.contains("grayscale"));
    }
}
