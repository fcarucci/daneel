// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::live_gateway::use_live_gateway_events;
use crate::gateway::get_gateway_status;
use crate::models::gateway::GatewayLevel;
use crate::models::live_gateway::{LiveGatewayEvent, LiveGatewayLevel};

#[component]
pub fn TopBar() -> Element {
    let gateway_status = use_resource(|| async move { get_gateway_status().await });
    let (live_status, live_seen) = use_live_gateway_events();

    let pill = status_pill(resolved_live_level(&gateway_status, live_status()));

    let live_attr = if live_seen() { "true" } else { "false" };

    rsx! {
        header { class: "flex flex-col gap-5 px-5 pt-6 sm:px-8 lg:flex-row lg:items-center lg:justify-between lg:px-10",
            div {
                p { class: "m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "Mission Control" }
            }
            div { class: "flex items-center gap-3",
                div { class: pill.class, "data-live": live_attr,
                    svg {
                        class: pill.dot_class,
                        view_box: "0 0 16 16",
                        width: "16",
                        height: "16",
                        circle { cx: "8", cy: "8", r: "6", fill: "currentColor" }
                    }
                    "{pill.label}"
                }
            }
        }
    }
}

fn resolved_live_level(
    gateway_status: &Resource<Result<crate::models::gateway::GatewayStatusSnapshot, ServerFnError>>,
    live_status: Option<LiveGatewayEvent>,
) -> Option<LiveGatewayLevel> {
    let gateway_level = gateway_status
        .read_unchecked()
        .as_ref()
        .and_then(|value| value.as_ref().ok())
        .map(|snapshot| match snapshot.level {
            GatewayLevel::Healthy => LiveGatewayLevel::Healthy,
            GatewayLevel::Degraded => LiveGatewayLevel::Degraded,
        });

    combine_gateway_levels(gateway_level, live_status.map(|event| event.level))
}

fn combine_gateway_levels(
    gateway_level: Option<LiveGatewayLevel>,
    live_level: Option<LiveGatewayLevel>,
) -> Option<LiveGatewayLevel> {
    match live_level {
        Some(LiveGatewayLevel::Healthy) => Some(LiveGatewayLevel::Healthy),
        Some(LiveGatewayLevel::Degraded) => Some(LiveGatewayLevel::Degraded),
        Some(LiveGatewayLevel::Connecting) => gateway_level.or(Some(LiveGatewayLevel::Connecting)),
        None => gateway_level,
    }
}

struct StatusPill {
    class: &'static str,
    dot_class: &'static str,
    label: &'static str,
}

fn status_pill(level: Option<LiveGatewayLevel>) -> StatusPill {
    match level {
        Some(LiveGatewayLevel::Healthy) => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-emerald-300/20 bg-emerald-300/10 px-4 py-3 text-sm font-medium text-emerald-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-emerald-300 drop-shadow-[0_0_8px_rgba(110,231,183,0.95)]",
            label: "Connected",
        },
        Some(LiveGatewayLevel::Degraded) => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-amber-400/20 bg-amber-400/10 px-4 py-3 text-sm font-medium text-amber-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Degraded",
        },
        Some(LiveGatewayLevel::Connecting) | None => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-white/12 bg-white/6 px-4 py-3 text-sm font-medium text-slate-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Connecting",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{LiveGatewayLevel, combine_gateway_levels, status_pill};

    #[test]
    fn connecting_live_state_falls_back_to_healthy_snapshot() {
        let resolved = combine_gateway_levels(
            Some(LiveGatewayLevel::Healthy),
            Some(LiveGatewayLevel::Connecting),
        );

        assert!(matches!(resolved, Some(LiveGatewayLevel::Healthy)));
    }

    #[test]
    fn degraded_live_state_overrides_healthy_snapshot() {
        let resolved = combine_gateway_levels(
            Some(LiveGatewayLevel::Healthy),
            Some(LiveGatewayLevel::Degraded),
        );

        assert!(matches!(resolved, Some(LiveGatewayLevel::Degraded)));
    }

    #[test]
    fn healthy_pill_uses_connected_label() {
        let pill = status_pill(Some(LiveGatewayLevel::Healthy));

        assert_eq!(pill.label, "Connected");
    }
}
