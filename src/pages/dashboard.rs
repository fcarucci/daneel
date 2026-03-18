// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::models::gateway::{GatewayLevel, GatewayStatusSnapshot};

#[component]
pub fn Dashboard() -> Element {
    let client = use_app_client();
    let gateway_status = use_resource(move || {
        let client = client.clone();
        async move { client.get_gateway_status().await }
    });

    rsx! {
        section { class: "flex flex-col gap-5",
            div { class: "rounded-[2rem] border border-white/10 bg-[var(--panel-bg)] px-6 py-7 shadow-[0_30px_80px_rgba(2,6,23,0.45)] backdrop-blur-2xl sm:px-8",
                p { class: "m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "POC V1" }
                h2 { class: "m-0 mt-3 max-w-3xl text-3xl font-semibold tracking-[-0.05em] text-white sm:text-4xl", "Gateway overview and graph surfaces start here." }
                p { class: "m-0 mt-3 max-w-2xl text-sm leading-7 text-slate-300 sm:text-base",
                    "This initial shell gives us a typed routing foundation, a shared layout, and room for the first adapter-backed dashboard queries."
                }
            }
            div { class: "grid grid-cols-1 gap-4 xl:grid-cols-3",
                article { class: "rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Gateway status" }
                    GatewayStatusCard { gateway_status }
                }
                article { class: "rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Agents graph" }
                    p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "The deterministic SVG graph will land in this route next." }
                }
                article { class: "rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Activity feed" }
                    p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Live event transport is intentionally deferred until after the first request-response slice." }
                }
            }
        }
    }
}

#[component]
fn GatewayStatusCard(
    gateway_status: Resource<Result<GatewayStatusSnapshot, ServerFnError>>,
) -> Element {
    match &*gateway_status.read_unchecked() {
        Some(Ok(snapshot)) => {
            let badge_class = match snapshot.level {
                GatewayLevel::Healthy => {
                    "mt-3 inline-flex rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-emerald-200"
                }
                GatewayLevel::Degraded => {
                    "mt-3 inline-flex rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-amber-400"
                }
            };

            rsx! {
                p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "{snapshot.summary}" }
                span { class: badge_class, "{snapshot.level:?}" }
                p { class: "m-0 mt-3 text-xs leading-6 text-slate-300", "{snapshot.detail}" }
                p { class: "m-0 mt-2 text-xs leading-6 text-slate-300", "Gateway URL: {snapshot.gateway_url}" }
                if let Some(protocol_version) = snapshot.protocol_version {
                    p { class: "m-0 text-xs leading-6 text-slate-300", "Protocol: v{protocol_version}" }
                }
                if let Some(state_version) = snapshot.state_version {
                    p { class: "m-0 text-xs leading-6 text-slate-300", "State version: {state_version}" }
                }
                if let Some(uptime_ms) = snapshot.uptime_ms {
                    p { class: "m-0 text-xs leading-6 text-slate-300", "Uptime: {uptime_ms} ms" }
                }
                button {
                    class: "mt-4 inline-flex items-center rounded-full border border-white/10 bg-white/6 px-4 py-2 text-sm font-medium text-slate-100 transition hover:border-white/20 hover:bg-white/8",
                    onclick: move |_| {
                        let mut gateway_status = gateway_status;
                        gateway_status.restart();
                    },
                    "Refresh status"
                }
            }
        }
        Some(Err(error)) => rsx! {
            p { class: "m-0 mt-3 text-sm leading-6 text-amber-400", "Gateway lookup failed: {error}" }
            button {
                class: "mt-4 inline-flex items-center rounded-full border border-white/10 bg-white/6 px-4 py-2 text-sm font-medium text-slate-100 transition hover:border-white/20 hover:bg-white/8",
                onclick: move |_| {
                    let mut gateway_status = gateway_status;
                    gateway_status.restart();
                },
                "Retry"
            }
        },
        None => rsx! {
            p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Contacting the OpenClaw Gateway through Daneel's server function..." }
        },
    }
}
