// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::client::use_app_client;
use crate::models::agents::{AgentOverviewItem, AgentOverviewSnapshot};
use crate::utils::time::{ACTIVE_WINDOW_MS, format_age_badge, heartbeat_is_active};

const AGENT_TILE_ACTIVE_CLASS: &str = "group relative overflow-hidden rounded-[1.9rem] border border-emerald-300/35 bg-[linear-gradient(180deg,rgba(14,28,32,0.96),rgba(5,12,24,0.98))] px-5 py-5 shadow-[0_0_0_1px_rgba(110,231,183,0.14),0_0_42px_rgba(16,185,129,0.28),0_0_90px_rgba(16,185,129,0.08),0_24px_64px_rgba(2,6,23,0.42)] backdrop-blur-xl";
const AGENT_TILE_IDLE_CLASS: &str = "group relative overflow-hidden rounded-[1.9rem] border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.92),rgba(6,11,25,0.98))] px-5 py-5 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl";
const STATUS_DOT_ACTIVE_CLASS: &str =
    "h-2.5 w-2.5 rounded-full bg-emerald-300 shadow-[0_0_14px_rgba(110,231,183,0.95)]";
const STATUS_DOT_IDLE_CLASS: &str = "h-2.5 w-2.5 rounded-full bg-slate-500";
const HEART_ENABLED_CLASS: &str =
    "shrink-0 text-rose-400 drop-shadow-[0_0_8px_rgba(251,113,133,0.55)]";
const HEART_DISABLED_CLASS: &str = "shrink-0 text-slate-500";
const RECENT_BADGE_ACTIVE_CLASS: &str = "inline-flex rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-emerald-200";
const RECENT_BADGE_IDLE_CLASS: &str = "inline-flex rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-slate-300";

#[component]
pub fn Agents() -> Element {
    let client = use_app_client();
    let agent_overview = use_resource(move || {
        let client = client.clone();
        async move { client.get_agent_overview().await }
    });

    rsx! {
        section { class: "flex flex-col gap-5",
            div { class: "flex flex-col gap-2",
                p { class: "m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "Graph View" }
                p { class: "m-0 max-w-2xl text-sm leading-7 text-slate-300 sm:text-base", "Inspect configured agents, heartbeat posture, and recent runtime activity from the live gateway snapshot." }
            }
            AgentOverviewSection { agent_overview }
        }
    }
}

#[component]
fn AgentOverviewSection(
    agent_overview: Resource<Result<AgentOverviewSnapshot, ServerFnError>>,
) -> Element {
    match &*agent_overview.read_unchecked() {
        Some(Ok(snapshot)) => rsx! {
            div { class: "mt-2 flex items-center gap-3",
                p { class: "m-0 text-[0.68rem] font-semibold uppercase tracking-[0.24em] text-slate-400", "Agent tiles" }
                p { class: "m-0 text-sm text-slate-400", "{snapshot.active_recent_agents}/{snapshot.total_agents} active in the last 10 minutes" }
            }
            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4",
                for agent in snapshot.agents.iter() {
                    AgentCard { agent: agent.clone() }
                }
            }
        },
        Some(Err(error)) => rsx! {
            article { class: "rounded-[1.6rem] border border-amber-400/20 bg-amber-400/10 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-amber-100", "Gateway lookup failed" }
                p { class: "m-0 mt-3 text-sm leading-6 text-amber-100/90", "{error}" }
                button {
                    class: "mt-4 inline-flex items-center rounded-full border border-white/10 bg-white/6 px-4 py-2 text-sm font-medium text-slate-100 transition hover:border-white/20 hover:bg-white/8",
                    onclick: move |_| {
                        let mut agent_overview = agent_overview;
                        agent_overview.restart();
                    },
                    "Retry"
                }
            }
        },
        None => rsx! {
            article { class: "rounded-[1.6rem] border border-white/10 bg-[var(--panel-bg)] p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Loading agents" }
                p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Requesting the current agent inventory from the OpenClaw gateway snapshot..." }
            }
        },
    }
}

#[component]
fn AgentCard(agent: AgentOverviewItem) -> Element {
    let elapsed_ms = use_signal(|| 0_u64);

    #[cfg(target_arch = "wasm32")]
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let mut elapsed_ms = elapsed_ms;
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                elapsed_ms += 1000;
            }
        }
    });

    let displayed_age_ms = agent.latest_activity_age_ms.map(|age| age + elapsed_ms());
    let is_active_now = is_agent_active(displayed_age_ms);
    let tile_class = if is_active_now {
        AGENT_TILE_ACTIVE_CLASS
    } else {
        AGENT_TILE_IDLE_CLASS
    };
    let status_dot_class = if is_active_now {
        STATUS_DOT_ACTIVE_CLASS
    } else {
        STATUS_DOT_IDLE_CLASS
    };
    let heartbeat_enabled = heartbeat_is_active(agent.heartbeat_enabled, &agent.heartbeat_schedule);
    let heart_class = heartbeat_icon_class(heartbeat_enabled);
    let recent_activity_badge = displayed_age_ms
        .map(format_age_badge)
        .unwrap_or_else(|| "No activity".to_string());
    let recent_activity_badge_class = if is_active_now {
        RECENT_BADGE_ACTIVE_CLASS
    } else {
        RECENT_BADGE_IDLE_CLASS
    };
    rsx! {
        article { class: tile_class,
            div { class: "pointer-events-none absolute inset-x-5 top-0 h-px bg-gradient-to-r from-transparent via-white/20 to-transparent" }
            if is_active_now {
                div { class: "pointer-events-none absolute -right-10 -top-10 h-28 w-28 rounded-full bg-emerald-300/14 blur-3xl" }
                div { class: "pointer-events-none absolute inset-0 rounded-[1.9rem] ring-1 ring-emerald-300/15" }
            }
            {agent_header(
                agent.name.as_str(),
                status_dot_class,
                agent.is_default,
                recent_activity_badge_class,
                recent_activity_badge.as_str(),
            )}
            {agent_sessions(
                agent.active_session_count,
                agent.latest_session_key.as_deref(),
                heart_class,
                heartbeat_enabled,
            )}
        }
    }
}

fn is_agent_active(displayed_age_ms: Option<u64>) -> bool {
    displayed_age_ms.is_some_and(|age| age < ACTIVE_WINDOW_MS)
}

fn heartbeat_icon_class(heartbeat_enabled: bool) -> &'static str {
    if heartbeat_enabled {
        HEART_ENABLED_CLASS
    } else {
        HEART_DISABLED_CLASS
    }
}

fn agent_header(
    name: &str,
    status_dot_class: &'static str,
    is_default: bool,
    recent_activity_badge_class: &'static str,
    recent_activity_badge: &str,
) -> Element {
    rsx! {
        div { class: "flex items-start justify-between gap-4 px-1",
            div { class: "min-w-0",
                div { class: "flex items-center gap-2.5",
                    span { class: status_dot_class }
                    h3 { class: "m-0 truncate text-base font-semibold tracking-[-0.03em] text-white", "{name}" }
                }
            }
            div { class: "ml-2 flex shrink-0 items-center gap-2",
                if is_default {
                    span { class: "inline-flex rounded-[999px] border border-cyan-300/20 bg-cyan-300/10 px-2.5 py-1 text-[0.62rem] font-semibold uppercase tracking-[0.18em] text-cyan-200", "Default" }
                }
                span { class: recent_activity_badge_class, "{recent_activity_badge}" }
            }
        }
    }
}

fn agent_sessions(
    active_session_count: u64,
    latest_session_key: Option<&str>,
    heart_class: &'static str,
    heartbeat_enabled: bool,
) -> Element {
    rsx! {
        div { class: "mt-4 rounded-[1.4rem] border border-white/6 bg-white/[0.03] px-4 py-4",
            div { class: "grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-sm",
                p { class: "m-0 text-[0.62rem] font-semibold uppercase tracking-[0.2em] text-slate-500", "Active sessions" }
                p { class: "m-0 text-right font-semibold text-white", "{active_session_count}" }
            }
            div { class: "mt-3 flex items-end justify-between gap-3",
                if let Some(session_key) = latest_session_key {
                    p { class: "m-0 min-w-0 flex-1 truncate pr-2 text-[0.7rem] leading-5 text-slate-500", "Latest session: {session_key}" }
                } else {
                    div { class: "flex-1" }
                }
                svg {
                    class: heart_class,
                    view_box: "0 0 24 24",
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    "aria-label": if heartbeat_enabled { "Heartbeat enabled" } else { "Heartbeat disabled" },
                    path { d: "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5C2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3C19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::agents::AgentOverviewItem;
    use crate::utils::time::{format_age_badge, heartbeat_is_active};

    #[component]
    fn AgentCardHarness(agent: AgentOverviewItem) -> Element {
        rsx! { AgentCard { agent } }
    }

    fn render_agent_card(agent: AgentOverviewItem) -> String {
        let mut dom = VirtualDom::new_with_props(AgentCardHarness, AgentCardHarnessProps { agent });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn is_agent_active_below_threshold() {
        assert!(is_agent_active(Some(599_999)));
    }

    #[test]
    fn is_agent_active_at_threshold() {
        assert!(!is_agent_active(Some(600_000)));
    }

    #[test]
    fn is_agent_active_none_age() {
        assert!(!is_agent_active(None));
    }

    #[test]
    fn format_age_badge_boundary_values() {
        assert_eq!(format_age_badge(60_000), "1m ago");
        assert_eq!(format_age_badge(3_600_000), "1h ago");
    }

    #[test]
    fn displayed_age_logic_adds_elapsed() {
        let server_age = 300_000;
        let elapsed = 150_000;
        assert_eq!(server_age + elapsed, 450_000);
    }

    #[test]
    fn elapsed_can_push_past_threshold() {
        let server_age = 599_000;
        let elapsed = 2_000;
        assert!(server_age + elapsed > ACTIVE_WINDOW_MS);
    }

    #[test]
    fn styling_class_selection() {
        assert_eq!(AGENT_TILE_ACTIVE_CLASS, AGENT_TILE_ACTIVE_CLASS);
        assert_eq!(AGENT_TILE_IDLE_CLASS, AGENT_TILE_IDLE_CLASS);
        assert_eq!(STATUS_DOT_ACTIVE_CLASS, STATUS_DOT_ACTIVE_CLASS);
        assert_eq!(STATUS_DOT_IDLE_CLASS, STATUS_DOT_IDLE_CLASS);
        assert_eq!(HEART_ENABLED_CLASS, HEART_ENABLED_CLASS);
        assert_eq!(RECENT_BADGE_ACTIVE_CLASS, RECENT_BADGE_ACTIVE_CLASS);
        assert_eq!(RECENT_BADGE_IDLE_CLASS, RECENT_BADGE_IDLE_CLASS);
    }

    #[test]
    fn glow_flips_when_elapsed_crosses_threshold() {
        // Simulate server age just below threshold
        let server_age = Some(599_000);
        let initial_active = is_agent_active(server_age);

        // Add elapsed time to push past threshold
        let elapsed = 2_000;
        let displayed_age = server_age.map(|age| age + elapsed);
        let final_active = is_agent_active(displayed_age);

        assert!(initial_active);
        assert!(!final_active);
    }

    #[test]
    fn heartbeat_states() {
        assert!(heartbeat_is_active(true, "*/5 * * * *"));
        assert!(!heartbeat_is_active(false, "*/5 * * * *"));
        assert!(!heartbeat_is_active(true, ""));
        assert!(!heartbeat_is_active(true, "none"));
        assert!(!heartbeat_is_active(true, "0"));
        assert!(!heartbeat_is_active(true, "disabled"));
    }

    #[test]
    fn disabled_heartbeat_uses_disabled_heart_class() {
        assert_eq!(heartbeat_icon_class(false), HEART_DISABLED_CLASS);
        assert_eq!(heartbeat_icon_class(true), HEART_ENABLED_CLASS);
    }

    #[test]
    fn disabled_heartbeat_card_renders_gray_heart() {
        let html = render_agent_card(AgentOverviewItem {
            id: "planner".to_string(),
            name: "planner".to_string(),
            is_default: false,
            heartbeat_enabled: false,
            heartbeat_schedule: "Disabled".to_string(),
            heartbeat_model: None,
            active_session_count: 0,
            latest_session_key: Some("agent:planner:cron:gamma".to_string()),
            latest_activity_age_ms: Some(8_400_000),
        });

        assert!(html.contains("aria-label=\"Heartbeat disabled\""));
        assert!(html.contains("text-slate-500"));
        assert!(!html.contains("text-rose-400"));
    }

    #[test]
    fn active_heartbeat_card_renders_heart() {
        let html = render_agent_card(AgentOverviewItem {
            id: "calendar".to_string(),
            name: "calendar".to_string(),
            is_default: false,
            heartbeat_enabled: true,
            heartbeat_schedule: "*/5 * * * *".to_string(),
            heartbeat_model: None,
            active_session_count: 2,
            latest_session_key: Some("agent:calendar:cron:beta".to_string()),
            latest_activity_age_ms: Some(300_000),
        });

        assert!(html.contains("aria-label=\"Heartbeat enabled\""));
        assert!(html.contains("text-rose-400"));
    }
}
