// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::{
    live_gateway::{LiveGatewayProvider, use_live_gateway},
    navbar::TopBar,
    sidebar::Sidebar,
};
use crate::router::Route;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        LiveGatewayProvider {
            div { class: "min-h-screen bg-[var(--app-bg)] text-[var(--ink-0)]",
                div { class: "grid min-h-screen grid-cols-1 lg:grid-cols-[15.5rem_minmax(0,1fr)]",
                    Sidebar {}
                    LayoutContent {}
                }
            }
        }
    }
}

#[component]
fn LayoutContent() -> Element {
    let live_gateway = use_live_gateway();
    let operator_state = live_gateway.operator_state();
    let main_class = main_content_class(live_gateway.is_frozen());

    rsx! {
        div { class: "min-w-0",
            TopBar {}
            ConnectionStateBanner { state: operator_state }
            main { class: main_class,
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn ConnectionStateBanner(state: crate::models::live_gateway::OperatorConnectionState) -> Element {
    let Some(banner) = connection_banner(state) else {
        return rsx! {};
    };

    rsx! {
        div { class: "px-5 pt-2 sm:px-8 lg:px-10",
            article { class: banner.class,
                p { class: "m-0 text-[0.68rem] font-semibold uppercase tracking-[0.22em] text-inherit/80", "{banner.title}" }
                p { class: "m-0 mt-2 text-sm leading-6 text-inherit", "{banner.detail}" }
            }
        }
    }
}

struct ConnectionBannerCopy {
    title: &'static str,
    detail: &'static str,
    class: &'static str,
}

fn connection_banner(
    state: crate::models::live_gateway::OperatorConnectionState,
) -> Option<ConnectionBannerCopy> {
    match state {
        crate::models::live_gateway::OperatorConnectionState::Connected => None,
        crate::models::live_gateway::OperatorConnectionState::Connecting => {
            Some(ConnectionBannerCopy {
                title: "Connecting",
                detail: "Loading the latest live gateway state from the backend.",
                class: "rounded-[1.3rem] border border-white/10 bg-white/6 px-5 py-4 text-slate-100 shadow-[0_18px_44px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            })
        }
        crate::models::live_gateway::OperatorConnectionState::Degraded => {
            Some(ConnectionBannerCopy {
                title: "Gateway degraded",
                detail: "Showing the latest available dashboard data while the gateway recovers.",
                class: "rounded-[1.3rem] border border-amber-400/20 bg-amber-400/10 px-5 py-4 text-amber-100 shadow-[0_18px_44px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            })
        }
        crate::models::live_gateway::OperatorConnectionState::Disconnected => {
            Some(ConnectionBannerCopy {
                title: "Live updates paused",
                detail: "Daneel is retrying the backend connection automatically. The current view will stay visible until the stream recovers.",
                class: "rounded-[1.3rem] border border-rose-300/20 bg-rose-300/10 px-5 py-4 text-rose-100 shadow-[0_18px_44px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            })
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
    use super::{connection_banner, main_content_class};
    use crate::models::live_gateway::OperatorConnectionState;

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

    #[test]
    fn disconnected_gateway_renders_a_recovery_message() {
        let banner = connection_banner(OperatorConnectionState::Disconnected)
            .expect("disconnected state should render a banner");

        assert_eq!(banner.title, "Live updates paused");
        assert!(banner.detail.contains("retrying the backend connection"));
    }
}
