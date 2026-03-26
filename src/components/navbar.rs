// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::dashboard_data::use_dashboard_data;
use crate::components::live_gateway::{
    gateway_snapshot_level, resolve_operator_state_with_gateway_snapshot, use_live_gateway,
};
use crate::models::live_gateway::OperatorConnectionState;

#[component]
pub fn TopBar() -> Element {
    let ctx = use_dashboard_data();
    let gateway_status = ctx.gateway_status.clone();
    let live_gateway = use_live_gateway();
    let operator_state = resolved_live_level(&gateway_status, &live_gateway);

    let pill = status_pill(operator_state);

    let live_attr = if operator_state == OperatorConnectionState::Connected {
        "true"
    } else {
        "false"
    };

    rsx! {
        header { class: "flex flex-col gap-5 px-5 pt-6 sm:px-8 lg:flex-row lg:items-center lg:justify-between lg:px-10",
            div {
                p { class: "page-kicker m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "Mission Control" }
            }
            div { class: "flex items-center gap-3",
                div { class: format!("state-chip {}", pill.class), "data-live": live_attr, "data-topbar-polish": "enhanced",
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
    live_gateway: &crate::components::live_gateway::LiveGatewayState,
) -> OperatorConnectionState {
    resolve_operator_state_with_gateway_snapshot(
        (live_gateway.backend_state)(),
        (live_gateway.live_status)().map(|event| event.level),
        gateway_snapshot_level(gateway_status),
    )
}

struct StatusPill {
    class: &'static str,
    dot_class: &'static str,
    label: &'static str,
}

fn status_pill(level: OperatorConnectionState) -> StatusPill {
    match level {
        OperatorConnectionState::Connected => StatusPill {
            class: "status-pill inline-flex items-center gap-3 rounded-full border border-emerald-300/20 bg-emerald-300/10 px-4 py-3 text-sm font-medium text-emerald-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-emerald-300 drop-shadow-[0_0_8px_rgba(110,231,183,0.95)]",
            label: "Connected",
        },
        OperatorConnectionState::Degraded => StatusPill {
            class: "status-pill inline-flex items-center gap-3 rounded-full border border-amber-400/20 bg-amber-400/10 px-4 py-3 text-sm font-medium text-amber-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Degraded",
        },
        OperatorConnectionState::Disconnected => StatusPill {
            class: "status-pill inline-flex items-center gap-3 rounded-full border border-rose-300/20 bg-rose-300/10 px-4 py-3 text-sm font-medium text-rose-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-rose-300 drop-shadow-[0_0_8px_rgba(253,164,175,0.9)]",
            label: "Disconnected",
        },
        OperatorConnectionState::Connecting => StatusPill {
            class: "status-pill status-pill--connecting inline-flex items-center gap-3 rounded-full border border-white/12 bg-white/6 px-4 py-3 text-sm font-medium text-slate-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Connecting",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatorConnectionState, status_pill};

    #[test]
    fn disconnected_pill_uses_disconnected_label() {
        let pill = status_pill(OperatorConnectionState::Disconnected);

        assert_eq!(pill.label, "Disconnected");
    }

    #[test]
    fn healthy_pill_uses_connected_label() {
        let pill = status_pill(OperatorConnectionState::Connected);

        assert_eq!(pill.label, "Connected");
    }

    #[test]
    fn connecting_pill_uses_connecting_animation_class() {
        let pill = status_pill(OperatorConnectionState::Connecting);

        assert!(pill.class.contains("status-pill--connecting"));
    }

    #[test]
    fn connected_pill_keeps_surface_chip_spacing() {
        let pill = status_pill(OperatorConnectionState::Connected);

        assert!(pill.class.contains("rounded-full"));
        assert!(pill.class.contains("gap-3"));
    }
}
