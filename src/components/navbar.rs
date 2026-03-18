// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::components::live_gateway::use_live_gateway;
use crate::models::gateway::GatewayLevel;
use crate::models::live_gateway::{
    LiveGatewayLevel, OperatorConnectionState, resolve_operator_connection_state,
};

#[component]
pub fn TopBar() -> Element {
    let client = use_app_client();
    let gateway_status = use_resource(move || {
        let client = client.clone();
        async move { client.get_gateway_status().await }
    });
    let live_gateway = use_live_gateway();

    let pill = status_pill(resolved_live_level(&gateway_status, &live_gateway));

    let live_attr = if live_gateway.operator_state() == OperatorConnectionState::Connected {
        "true"
    } else {
        "false"
    };

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
    live_gateway: &crate::components::live_gateway::LiveGatewayState,
) -> OperatorConnectionState {
    let gateway_level = gateway_level_from_status(gateway_status);
    let live_level = (live_gateway.live_status)().map(|event| event.level);
    let preferred_level = preferred_gateway_level(live_level, gateway_level);

    resolve_operator_connection_state((live_gateway.backend_state)(), preferred_level)
}

fn gateway_level_from_status(
    gateway_status: &Resource<Result<crate::models::gateway::GatewayStatusSnapshot, ServerFnError>>,
) -> Option<LiveGatewayLevel> {
    gateway_status
        .read_unchecked()
        .as_ref()
        .and_then(|value| value.as_ref().ok())
        .map(|snapshot| match snapshot.level {
            GatewayLevel::Healthy => LiveGatewayLevel::Healthy,
            GatewayLevel::Degraded => LiveGatewayLevel::Degraded,
        })
}

fn preferred_gateway_level(
    live_level: Option<LiveGatewayLevel>,
    gateway_level: Option<LiveGatewayLevel>,
) -> Option<LiveGatewayLevel> {
    match live_level {
        Some(LiveGatewayLevel::Connecting) => gateway_level.or(Some(LiveGatewayLevel::Connecting)),
        Some(level) => Some(level),
        None => gateway_level,
    }
}

struct StatusPill {
    class: &'static str,
    dot_class: &'static str,
    label: &'static str,
}

fn status_pill(level: OperatorConnectionState) -> StatusPill {
    match level {
        OperatorConnectionState::Connected => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-emerald-300/20 bg-emerald-300/10 px-4 py-3 text-sm font-medium text-emerald-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-emerald-300 drop-shadow-[0_0_8px_rgba(110,231,183,0.95)]",
            label: "Connected",
        },
        OperatorConnectionState::Degraded => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-amber-400/20 bg-amber-400/10 px-4 py-3 text-sm font-medium text-amber-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Degraded",
        },
        OperatorConnectionState::Disconnected => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-rose-300/20 bg-rose-300/10 px-4 py-3 text-sm font-medium text-rose-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-rose-300 drop-shadow-[0_0_8px_rgba(253,164,175,0.9)]",
            label: "Disconnected",
        },
        OperatorConnectionState::Connecting => StatusPill {
            class: "inline-flex items-center gap-3 rounded-full border border-white/12 bg-white/6 px-4 py-3 text-sm font-medium text-slate-100 shadow-[0_12px_32px_rgba(2,6,23,0.28)] backdrop-blur-xl",
            dot_class: "inline-block shrink-0 text-[1rem] leading-none text-amber-300 drop-shadow-[0_0_8px_rgba(252,211,77,0.9)]",
            label: "Connecting",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{LiveGatewayLevel, OperatorConnectionState, preferred_gateway_level, status_pill};

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
    fn connecting_live_level_yields_to_healthy_gateway_snapshot() {
        let level = preferred_gateway_level(
            Some(LiveGatewayLevel::Connecting),
            Some(LiveGatewayLevel::Healthy),
        );

        assert!(matches!(level, Some(LiveGatewayLevel::Healthy)));
    }

    #[test]
    fn disconnected_live_level_overrides_healthy_gateway_snapshot() {
        let level = preferred_gateway_level(
            Some(LiveGatewayLevel::Disconnected),
            Some(LiveGatewayLevel::Healthy),
        );

        assert!(matches!(level, Some(LiveGatewayLevel::Disconnected)));
    }
}
