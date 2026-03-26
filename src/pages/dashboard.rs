// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::dashboard_data::use_dashboard_data;
use crate::components::graph_canvas::GraphCanvas;
use crate::components::live_gateway::use_live_gateway;
use crate::graph_service::{GraphAssemblySummary, summarize_graph_snapshot};
use crate::models::{
    gateway::{GatewayLevel, GatewayStatusSnapshot},
    graph::AgentGraphSnapshot,
    live_gateway::OperatorConnectionState,
};

#[cfg(test)]
use crate::models::graph::{AgentNode, AgentStatus};

#[derive(Clone, Debug, PartialEq)]
struct SummaryCardModel {
    title: &'static str,
    value: String,
    detail: String,
    accent_class: &'static str,
    stale: bool,
}

#[component]
pub fn Dashboard() -> Element {
    let ctx = use_dashboard_data();
    let live_gateway = use_live_gateway();
    let operator_state = live_gateway.operator_state();
    let gateway_status = ctx.gateway_status.clone();
    let graph_snapshot = ctx.graph_snapshot.clone();
    let cached_gateway_status = ctx.cached_gateway_status;
    let cached_graph_snapshot = ctx.cached_graph_snapshot;

    rsx! {
        section { class: "flex flex-col gap-5",
            div { class: "polish-hero rounded-[2rem] border border-white/10 bg-[var(--panel-bg)] px-6 py-7 shadow-[0_30px_80px_rgba(2,6,23,0.45)] backdrop-blur-2xl sm:px-8", "data-polish-hero": "dashboard",
                p { class: "page-kicker m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "POC V1" }
                h2 { class: "m-0 mt-3 max-w-3xl text-3xl font-semibold tracking-[-0.05em] text-white sm:text-4xl", "Gateway overview and graph surfaces start here." }
                p { class: "m-0 mt-3 max-w-2xl text-sm leading-7 text-slate-300 sm:text-base",
                    "This initial shell gives us a typed routing foundation, a shared layout, and room for the first adapter-backed dashboard queries."
                }
            }
            DashboardSummaryRow {
                operator_state,
                gateway_status: gateway_status.clone(),
                graph_snapshot: graph_snapshot.clone(),
                cached_gateway_status: cached_gateway_status(),
                cached_graph_snapshot: cached_graph_snapshot(),
            }
            div { class: "grid grid-cols-1 gap-4 xl:grid-cols-[minmax(19rem,0.85fr)_minmax(0,1.15fr)_minmax(0,1.15fr)]",
                div { class: "flex flex-col gap-4 xl:col-span-1",
                    article { class: "polish-panel polish-panel--interactive rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl", "data-polish-tone": "gateway", "data-dashboard-panel": "gateway",
                        h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Gateway status" }
                        GatewayStatusCard { gateway_status }
                    }
                    article { class: "polish-panel polish-panel--interactive rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl", "data-polish-tone": "activity", "data-dashboard-panel": "activity",
                        h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Activity feed" }
                        p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Live event transport is intentionally deferred until after the first request-response slice." }
                    }
                }
                article { class: "polish-panel polish-panel--interactive rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl xl:col-span-2", "data-polish-tone": "graph", "data-dashboard-panel": "graph",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Agents graph" }
                    GraphSnapshotCard {
                        operator_state,
                        graph_snapshot,
                        cached_graph_snapshot: cached_graph_snapshot(),
                    }
                }
            }
        }
    }
}

#[component]
fn DashboardSummaryRow(
    operator_state: OperatorConnectionState,
    gateway_status: Resource<Result<GatewayStatusSnapshot, ServerFnError>>,
    graph_snapshot: Resource<Result<AgentGraphSnapshot, ServerFnError>>,
    cached_gateway_status: Option<GatewayStatusSnapshot>,
    cached_graph_snapshot: Option<AgentGraphSnapshot>,
) -> Element {
    let cards = build_summary_cards(
        operator_state,
        gateway_status.read_unchecked().as_ref(),
        graph_snapshot.read_unchecked().as_ref(),
        cached_gateway_status.as_ref(),
        cached_graph_snapshot.as_ref(),
    );
    let [
        gateway_card,
        agents_card,
        active_agents_card,
        connections_card,
    ] = cards;

    rsx! {
        div { class: "grid grid-cols-1 items-start gap-4 xl:grid-cols-[minmax(19rem,0.85fr)_minmax(0,1.15fr)_minmax(0,1.15fr)]",
            div { class: "xl:col-span-1",
                SummaryCard { card: gateway_card }
            }
            div { class: "min-w-0 grid items-start gap-4 sm:grid-cols-2 xl:col-span-2 xl:grid-cols-3",
                SummaryCard { card: agents_card }
                SummaryCard { card: active_agents_card }
                SummaryCard { card: connections_card }
            }
        }
    }
}

#[component]
fn SummaryCard(card: SummaryCardModel) -> Element {
    rsx! {
        article {
            class: "polish-card polish-card--interactive min-w-0 h-[14rem] rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
            "data-summary-card": card.title,
            "data-summary-stale": if card.stale { "true" } else { "false" },
            "data-summary-polish": "enhanced",
            p { class: "m-0 text-[0.68rem] font-semibold uppercase tracking-[0.22em] text-slate-400", "{card.title}" }
            p { class: format!("m-0 mt-4 text-3xl font-semibold tracking-[-0.05em] {}", card.accent_class), "{card.value}" }
            p { class: "m-0 mt-2 text-sm leading-6 text-slate-300", "{card.detail}" }
        }
    }
}

#[component]
fn GraphSnapshotCard(
    operator_state: OperatorConnectionState,
    graph_snapshot: Resource<Result<AgentGraphSnapshot, ServerFnError>>,
    cached_graph_snapshot: Option<AgentGraphSnapshot>,
) -> Element {
    match graph_snapshot_view(
        operator_state,
        graph_snapshot.read_unchecked().as_ref(),
        cached_graph_snapshot.as_ref(),
    ) {
        GraphSnapshotView::Ready { snapshot, stale } => rsx! {
            div {
                "data-graph-view": "ready",
                "data-stale-view": if stale { "true" } else { "false" },
                "data-graph-polish": "enhanced",
                p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Deterministic first-slice graph layout from the latest assembled gateway snapshot." }
                if stale {
                    p { class: "m-0 mt-2 text-sm leading-6 text-rose-200", "Showing the last known graph snapshot while Daneel reconnects to the backend." }
                }
                div { class: "mt-4",
                    GraphCanvas { snapshot }
                }
            }
        },
        GraphSnapshotView::Empty { stale } => rsx! {
            div {
                "data-graph-view": "empty",
                "data-stale-view": if stale { "true" } else { "false" },
                "data-graph-polish": "enhanced",
                class: "mt-3 rounded-[1.3rem] border border-white/10 bg-white/[0.04] px-5 py-5 text-slate-300",
                p { class: "m-0 text-sm font-medium text-white", "Nothing to show yet" }
                p { class: "m-0 mt-2 text-sm leading-6 text-slate-300", "The graph snapshot loaded, but there are no assembled nodes yet. Gateway summaries remain available above." }
                if stale {
                    p { class: "m-0 mt-2 text-sm leading-6 text-rose-200", "This empty graph is the last known dashboard state while the backend reconnects." }
                }
            }
        },
        GraphSnapshotView::Error(error) => rsx! {
            div {
                "data-graph-view": "error",
                "data-stale-view": "false",
                "data-graph-polish": "enhanced",
                p { class: "m-0 mt-3 text-sm leading-6 text-amber-400", "Graph snapshot unavailable: {error}" }
                p { class: "m-0 mt-2 text-sm leading-6 text-slate-300", "The dashboard will keep any available gateway and summary data visible while graph loading recovers." }
                button {
                    class: "mt-4 inline-flex items-center rounded-full border border-white/10 bg-white/6 px-4 py-2 text-sm font-medium text-slate-100 transition hover:border-white/20 hover:bg-white/8",
                    onclick: move |_| {
                        let mut graph_snapshot = graph_snapshot;
                        graph_snapshot.restart();
                    },
                    "Retry graph"
                }
            }
        },
        GraphSnapshotView::Disconnected => rsx! {
            div {
                "data-graph-view": "disconnected",
                "data-stale-view": "false",
                "data-graph-polish": "enhanced",
                class: "mt-3 rounded-[1.3rem] border border-rose-300/20 bg-rose-300/10 px-5 py-5 text-rose-100",
                p { class: "m-0 text-sm font-medium text-white", "Graph snapshot paused" }
                p { class: "m-0 mt-2 text-sm leading-6 text-rose-100", "Daneel is retrying the backend connection. The graph will repopulate as soon as the stream recovers." }
            }
        },
        GraphSnapshotView::Loading => rsx! {
            div {
                "data-graph-view": "loading",
                "data-stale-view": "false",
                "data-graph-polish": "enhanced",
                p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "Loading the latest graph snapshot from Daneel's graph assembly service..." }
            }
        },
    }
}

enum GraphSnapshotView {
    Loading,
    Empty {
        stale: bool,
    },
    Error(String),
    Disconnected,
    Ready {
        snapshot: AgentGraphSnapshot,
        stale: bool,
    },
}

fn graph_snapshot_view(
    operator_state: OperatorConnectionState,
    graph_snapshot: Option<&Result<AgentGraphSnapshot, ServerFnError>>,
    cached_graph_snapshot: Option<&AgentGraphSnapshot>,
) -> GraphSnapshotView {
    if operator_state == OperatorConnectionState::Disconnected {
        if let Some(snapshot) = cached_graph_snapshot {
            return if snapshot.nodes.is_empty() {
                GraphSnapshotView::Empty { stale: true }
            } else {
                GraphSnapshotView::Ready {
                    snapshot: snapshot.clone(),
                    stale: true,
                }
            };
        }

        return GraphSnapshotView::Disconnected;
    }

    match graph_snapshot {
        Some(Ok(snapshot)) if snapshot.nodes.is_empty() => {
            GraphSnapshotView::Empty { stale: false }
        }
        Some(Ok(snapshot)) => GraphSnapshotView::Ready {
            snapshot: snapshot.clone(),
            stale: false,
        },
        Some(Err(error)) => GraphSnapshotView::Error(error.to_string()),
        None => GraphSnapshotView::Loading,
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

fn gateway_summary_card(
    operator_state: OperatorConnectionState,
    gateway_status: Option<&Result<GatewayStatusSnapshot, ServerFnError>>,
    cached_gateway_status: Option<&GatewayStatusSnapshot>,
) -> SummaryCardModel {
    if operator_state == OperatorConnectionState::Disconnected {
        if let Some(snapshot) = cached_gateway_status {
            return SummaryCardModel {
                title: "Gateway",
                value: match snapshot.level {
                    GatewayLevel::Healthy => "Healthy".to_string(),
                    GatewayLevel::Degraded => "Degraded".to_string(),
                },
                detail: format!(
                    "Last known gateway status: {} Daneel is retrying the backend connection.",
                    snapshot.summary
                ),
                accent_class: match snapshot.level {
                    GatewayLevel::Healthy => "text-emerald-200",
                    GatewayLevel::Degraded => "text-amber-300",
                },
                stale: true,
            };
        }

        return SummaryCardModel {
            title: "Gateway",
            value: "Disconnected".to_string(),
            detail: "Waiting for the backend connection to recover before loading a fresh gateway snapshot."
                .to_string(),
            accent_class: "text-rose-200",
            stale: false,
        };
    }

    match gateway_status {
        Some(Ok(snapshot)) => SummaryCardModel {
            title: "Gateway",
            value: match snapshot.level {
                GatewayLevel::Healthy => "Healthy".to_string(),
                GatewayLevel::Degraded => "Degraded".to_string(),
            },
            detail: snapshot.summary.clone(),
            accent_class: match snapshot.level {
                GatewayLevel::Healthy => "text-emerald-200",
                GatewayLevel::Degraded => "text-amber-300",
            },
            stale: false,
        },
        Some(Err(error)) => SummaryCardModel {
            title: "Gateway",
            value: "Unavailable".to_string(),
            detail: format!("Gateway lookup failed: {error}"),
            accent_class: "text-amber-300",
            stale: false,
        },
        None => SummaryCardModel {
            title: "Gateway",
            value: "Loading".to_string(),
            detail: "Fetching the latest gateway health snapshot.".to_string(),
            accent_class: "text-slate-200",
            stale: false,
        },
    }
}

fn build_summary_cards(
    operator_state: OperatorConnectionState,
    gateway_status: Option<&Result<GatewayStatusSnapshot, ServerFnError>>,
    graph_snapshot: Option<&Result<AgentGraphSnapshot, ServerFnError>>,
    cached_gateway_status: Option<&GatewayStatusSnapshot>,
    cached_graph_snapshot: Option<&AgentGraphSnapshot>,
) -> [SummaryCardModel; 4] {
    let gateway = gateway_summary_card(operator_state, gateway_status, cached_gateway_status);
    let graph_summary = graph_summary_model(operator_state, graph_snapshot, cached_graph_snapshot);
    let graph_stale =
        operator_state == OperatorConnectionState::Disconnected && cached_graph_snapshot.is_some();

    [
        gateway,
        SummaryCardModel {
            title: "Agents",
            value: graph_summary.agent_count.to_string(),
            detail: "Known nodes in the assembled snapshot.".to_string(),
            accent_class: "text-sky-200",
            stale: graph_stale,
        },
        SummaryCardModel {
            title: "Active agents",
            value: graph_summary.active_agent_count.to_string(),
            detail: "Nodes currently marked active by session state.".to_string(),
            accent_class: "text-emerald-200",
            stale: graph_stale,
        },
        SummaryCardModel {
            title: "Connections",
            value: graph_summary.edge_count.to_string(),
            detail: "Rendered relationships across routes and hints.".to_string(),
            accent_class: "text-violet-200",
            stale: graph_stale,
        },
    ]
}

fn graph_summary_model(
    operator_state: OperatorConnectionState,
    graph_snapshot: Option<&Result<AgentGraphSnapshot, ServerFnError>>,
    cached_graph_snapshot: Option<&AgentGraphSnapshot>,
) -> GraphAssemblySummary {
    if operator_state == OperatorConnectionState::Disconnected {
        return cached_graph_snapshot
            .map(summarize_graph_snapshot)
            .unwrap_or_default();
    }

    match graph_snapshot {
        Some(Ok(snapshot)) => summarize_graph_snapshot(snapshot),
        Some(Err(_)) | None => GraphAssemblySummary::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn SummaryRowHarness(
        operator_state: OperatorConnectionState,
        gateway_status: Option<Result<GatewayStatusSnapshot, ServerFnError>>,
        graph_snapshot: Option<Result<AgentGraphSnapshot, ServerFnError>>,
        cached_gateway_status: Option<GatewayStatusSnapshot>,
        cached_graph_snapshot: Option<AgentGraphSnapshot>,
    ) -> Element {
        let cards = build_summary_cards(
            operator_state,
            gateway_status.as_ref(),
            graph_snapshot.as_ref(),
            cached_gateway_status.as_ref(),
            cached_graph_snapshot.as_ref(),
        );

        rsx! {
            div {
                for card in cards {
                    SummaryCard { card }
                }
            }
        }
    }

    fn render_summary_row(
        operator_state: OperatorConnectionState,
        gateway_status: Option<Result<GatewayStatusSnapshot, ServerFnError>>,
        graph_snapshot: Option<Result<AgentGraphSnapshot, ServerFnError>>,
        cached_gateway_status: Option<GatewayStatusSnapshot>,
        cached_graph_snapshot: Option<AgentGraphSnapshot>,
    ) -> String {
        let mut dom = VirtualDom::new_with_props(
            SummaryRowHarness,
            SummaryRowHarnessProps {
                operator_state,
                gateway_status,
                graph_snapshot,
                cached_gateway_status,
                cached_graph_snapshot,
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[component]
    fn GraphCardHarness(
        operator_state: OperatorConnectionState,
        graph_snapshot: Option<Result<AgentGraphSnapshot, ServerFnError>>,
        cached_graph_snapshot: Option<AgentGraphSnapshot>,
    ) -> Element {
        match graph_snapshot_view(
            operator_state,
            graph_snapshot.as_ref(),
            cached_graph_snapshot.as_ref(),
        ) {
            GraphSnapshotView::Ready { snapshot, stale } => rsx! {
                div {
                    "data-stale-view": if stale { "true" } else { "false" },
                    "data-graph-polish": "enhanced",
                    if stale {
                        p { "Showing the last known graph snapshot while Daneel reconnects to the backend." }
                    }
                    GraphCanvas { snapshot }
                }
            },
            GraphSnapshotView::Empty { stale } => rsx! {
                div {
                    "data-stale-view": if stale { "true" } else { "false" },
                    "data-graph-polish": "enhanced",
                    "Nothing to show yet"
                }
            },
            GraphSnapshotView::Error(error) => {
                rsx! {
                    div {
                        "data-stale-view": "false",
                        "data-graph-polish": "enhanced",
                        "Graph snapshot unavailable: {error}"
                        button { "Retry graph" }
                    }
                }
            }
            GraphSnapshotView::Disconnected => {
                rsx! { div { "data-graph-polish": "enhanced", "Graph snapshot paused" } }
            }
            GraphSnapshotView::Loading => {
                rsx! {
                    div {
                        "data-stale-view": "false",
                        "data-graph-polish": "enhanced",
                        "Loading the latest graph snapshot from Daneel's graph assembly service..."
                    }
                }
            }
        }
    }

    fn render_graph_card(
        operator_state: OperatorConnectionState,
        graph_snapshot: Option<Result<AgentGraphSnapshot, ServerFnError>>,
        cached_graph_snapshot: Option<AgentGraphSnapshot>,
    ) -> String {
        let mut dom = VirtualDom::new_with_props(
            GraphCardHarness,
            GraphCardHarnessProps {
                operator_state,
                graph_snapshot,
                cached_graph_snapshot,
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn summary_values_match_the_snapshot_fixture() {
        let html = render_summary_row(
            OperatorConnectionState::Connected,
            Some(Ok(GatewayStatusSnapshot {
                connected: true,
                level: GatewayLevel::Healthy,
                summary: "Gateway connected".to_string(),
                detail: "healthy detail".to_string(),
                gateway_url: "ws://127.0.0.1:18789/".to_string(),
                protocol_version: Some(1),
                state_version: Some(7),
                uptime_ms: Some(12_000),
            })),
            Some(Ok(AgentGraphSnapshot {
                nodes: vec![
                    graph_node("calendar", AgentStatus::Active),
                    graph_node("planner", AgentStatus::Idle),
                ],
                edges: vec![crate::models::graph::AgentEdge {
                    source_id: "calendar".to_string(),
                    target_id: "planner".to_string(),
                    kind: crate::models::graph::AgentEdgeKind::RoutesTo,
                }],
                snapshot_ts: 1,
            })),
            None,
            None,
        );

        assert!(html.contains("data-summary-card=\"Gateway\""));
        assert!(html.contains("data-summary-polish=\"enhanced\""));
        assert!(html.contains(">Healthy<"));
        assert!(html.contains("data-summary-card=\"Agents\""));
        assert!(html.contains(">2<"));
        assert!(html.contains("data-summary-card=\"Active agents\""));
        assert!(html.contains(">1<"));
        assert!(html.contains("data-summary-card=\"Connections\""));
    }

    #[test]
    fn degraded_gateway_state_is_reflected_in_the_status_summary_card() {
        let html = render_summary_row(
            OperatorConnectionState::Degraded,
            Some(Ok(GatewayStatusSnapshot::degraded(
                "ws://127.0.0.1:18789/".to_string(),
                "Gateway degraded",
                "detail",
            ))),
            None,
            None,
            None,
        );

        assert!(html.contains("data-summary-card=\"Gateway\""));
        assert!(html.contains(">Degraded<"));
        assert!(html.contains("Gateway degraded"));
        assert!(html.contains("text-amber-300"));
    }

    #[test]
    fn malformed_snapshot_does_not_crash_the_page() {
        let html = render_graph_card(
            OperatorConnectionState::Connected,
            Some(Err(ServerFnError::new("Malformed snapshot payload"))),
            None,
        );

        assert!(html.contains("Graph snapshot unavailable"));
        assert!(html.contains("Malformed snapshot payload"));
    }

    #[test]
    fn partial_data_still_renders_available_nodes_and_summaries() {
        let snapshot = AgentGraphSnapshot {
            nodes: vec![graph_node("calendar", AgentStatus::Active)],
            edges: vec![],
            snapshot_ts: 1,
        };

        let summary_html = render_summary_row(
            OperatorConnectionState::Connected,
            None,
            Some(Ok(snapshot.clone())),
            None,
            None,
        );
        let graph_html =
            render_graph_card(OperatorConnectionState::Connected, Some(Ok(snapshot)), None);

        assert!(summary_html.contains("data-summary-card=\"Agents\""));
        assert!(summary_html.contains(">1<"));
        assert!(summary_html.contains("data-summary-card=\"Connections\""));
        assert!(summary_html.contains(">0<"));
        assert!(graph_html.contains("calendar"));
    }

    #[test]
    fn empty_snapshot_renders_the_dedicated_empty_state() {
        let html = render_graph_card(
            OperatorConnectionState::Connected,
            Some(Ok(AgentGraphSnapshot {
                nodes: vec![],
                edges: vec![],
                snapshot_ts: 1,
            })),
            None,
        );

        assert!(html.contains("Nothing to show yet"));
        assert!(html.contains("data-stale-view=\"false\""));
        assert!(html.contains("data-graph-polish=\"enhanced\""));
    }

    #[test]
    fn loading_state_does_not_render_retry_affordance() {
        let html = render_graph_card(OperatorConnectionState::Connecting, None, None);

        assert!(html.contains("Loading the latest graph snapshot"));
        assert!(!html.contains("Retry graph"));
    }

    #[test]
    fn disconnected_state_uses_cached_graph_snapshot_when_available() {
        let cached_snapshot = AgentGraphSnapshot {
            nodes: vec![graph_node("calendar", AgentStatus::Active)],
            edges: vec![],
            snapshot_ts: 1,
        };

        let html = render_graph_card(
            OperatorConnectionState::Disconnected,
            Some(Err(ServerFnError::new("backend down"))),
            Some(cached_snapshot),
        );

        assert!(html.contains("calendar"));
        assert!(html.contains("Showing the last known graph snapshot"));
        assert!(html.contains("data-stale-view=\"true\""));
    }

    #[test]
    fn disconnected_without_cached_graph_renders_paused_state() {
        let html = render_graph_card(
            OperatorConnectionState::Disconnected,
            Some(Err(ServerFnError::new("backend down"))),
            None,
        );

        assert!(html.contains("Graph snapshot paused"));
    }

    #[test]
    fn disconnected_summary_cards_preserve_cached_counts_and_gateway_state() {
        let cached_gateway = GatewayStatusSnapshot {
            connected: true,
            level: GatewayLevel::Healthy,
            summary: "Gateway connected".to_string(),
            detail: "healthy detail".to_string(),
            gateway_url: "ws://127.0.0.1:18789/".to_string(),
            protocol_version: Some(1),
            state_version: Some(7),
            uptime_ms: Some(12_000),
        };
        let cached_graph = AgentGraphSnapshot {
            nodes: vec![
                graph_node("calendar", AgentStatus::Active),
                graph_node("planner", AgentStatus::Idle),
            ],
            edges: vec![],
            snapshot_ts: 1,
        };
        let html = render_summary_row(
            OperatorConnectionState::Disconnected,
            Some(Err(ServerFnError::new("backend down"))),
            Some(Err(ServerFnError::new("backend down"))),
            Some(cached_gateway),
            Some(cached_graph),
        );

        assert!(html.contains(">Healthy<"));
        assert!(html.contains("Last known gateway status: Gateway connected"));
        assert!(html.contains("data-summary-stale=\"true\""));
        assert!(html.contains("data-summary-card=\"Agents\""));
        assert!(html.contains(">2<"));
    }

    #[test]
    fn degraded_and_disconnected_gateway_copy_stay_distinct() {
        let degraded_html = render_summary_row(
            OperatorConnectionState::Degraded,
            Some(Ok(GatewayStatusSnapshot::degraded(
                "ws://127.0.0.1:18789/".to_string(),
                "Gateway degraded",
                "detail",
            ))),
            None,
            None,
            None,
        );
        let disconnected_html = render_summary_row(
            OperatorConnectionState::Disconnected,
            Some(Err(ServerFnError::new("backend down"))),
            None,
            None,
            None,
        );

        assert!(degraded_html.contains("Gateway degraded"));
        assert!(!degraded_html.contains("Waiting for the backend connection to recover"));
        assert!(disconnected_html.contains("Waiting for the backend connection to recover"));
        assert!(disconnected_html.contains(">Disconnected<"));
    }

    #[test]
    fn polished_graph_state_markers_remain_distinct() {
        let loading_html = render_graph_card(OperatorConnectionState::Connecting, None, None);
        let empty_html = render_graph_card(
            OperatorConnectionState::Connected,
            Some(Ok(AgentGraphSnapshot {
                nodes: vec![],
                edges: vec![],
                snapshot_ts: 1,
            })),
            None,
        );
        let error_html = render_graph_card(
            OperatorConnectionState::Connected,
            Some(Err(ServerFnError::new("malformed"))),
            None,
        );

        assert!(loading_html.contains("Loading the latest graph snapshot"));
        assert!(empty_html.contains("Nothing to show yet"));
        assert!(error_html.contains("Graph snapshot unavailable"));
        assert!(error_html.contains("malformed"));
        assert!(empty_html.contains("data-graph-polish=\"enhanced\""));
    }

    fn graph_node(id: &str, status: AgentStatus) -> AgentNode {
        AgentNode {
            id: id.to_string(),
            name: id.to_string(),
            is_default: false,
            heartbeat_enabled: true,
            heartbeat_schedule: "*/5 * * * *".to_string(),
            active_session_count: matches!(status, AgentStatus::Active) as u64,
            latest_activity_age_ms: Some(60_000),
            status,
        }
    }
}
