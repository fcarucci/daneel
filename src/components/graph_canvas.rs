// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::agent_node_card::{AgentNodeCard, NODE_HEIGHT, NODE_WIDTH};
use crate::models::graph::{AgentEdgeKind, AgentGraphSnapshot, AgentNode};

const CANVAS_WIDTH: f32 = 1840.0;
const HORIZONTAL_MARGIN: f32 = 48.0;
const VERTICAL_MARGIN: f32 = 24.0;
const ROW_GAP: f32 = 56.0;

#[derive(Clone, Debug, PartialEq)]
struct PositionedNode {
    node: AgentNode,
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GraphLayoutMetrics {
    canvas_height: f32,
    column_count: usize,
    row_count: usize,
}

#[component]
pub fn GraphCanvas(snapshot: AgentGraphSnapshot) -> Element {
    if snapshot.nodes.is_empty() {
        return rsx! {
            div {
                class: "rounded-[1.5rem] border border-dashed border-white/10 bg-slate-950/25 px-5 py-8 text-center",
                p { class: "m-0 text-sm font-semibold uppercase tracking-[0.24em] text-slate-500", "Graph idle" }
                p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "No agents are available in the current graph snapshot yet." }
            }
        };
    }

    let layout = graph_layout_metrics(&snapshot);
    let positioned_nodes = layout_graph_nodes(&snapshot, layout);

    rsx! {
        div { class: "overflow-hidden rounded-[1.5rem] border border-white/8 bg-[radial-gradient(circle_at_top,rgba(34,197,94,0.08),transparent_35%),linear-gradient(180deg,rgba(15,23,42,0.92),rgba(2,6,23,0.98))]",
            svg {
                class: "block h-auto w-full",
                view_box: format!("0 0 {} {}", CANVAS_WIDTH, layout.canvas_height),
                role: "img",
                "aria-label": "Agent graph canvas",
                for edge in snapshot.edges.iter() {
                    if let Some((source, target)) = resolve_edge_nodes(&positioned_nodes, edge.source_id.as_str(), edge.target_id.as_str()) {
                        path {
                            "data-agent-edge": edge.kind.css_name(),
                            d: edge_path(source, target),
                            fill: "none",
                            stroke: edge.kind.stroke(),
                            stroke_width: edge.kind.stroke_width(),
                            stroke_dasharray: edge.kind.stroke_dasharray(),
                            stroke_linecap: "round",
                            opacity: "0.92",
                        }
                    }
                }
                for positioned in positioned_nodes.iter() {
                    AgentNodeCard { node: positioned.node.clone(), x: positioned.x, y: positioned.y }
                }
            }
        }
    }
}

fn graph_layout_metrics(snapshot: &AgentGraphSnapshot) -> GraphLayoutMetrics {
    let node_count = snapshot.nodes.len().max(1);
    let column_count = match node_count {
        0..=2 => node_count,
        3..=9 => 3,
        _ => 4,
    }
    .max(1);
    let row_count = node_count.div_ceil(column_count);

    let canvas_height = (VERTICAL_MARGIN * 2.0)
        + (row_count as f32 * NODE_HEIGHT)
        + ((row_count - 1) as f32 * ROW_GAP);

    GraphLayoutMetrics {
        canvas_height,
        column_count,
        row_count,
    }
}

fn layout_graph_nodes(
    snapshot: &AgentGraphSnapshot,
    layout: GraphLayoutMetrics,
) -> Vec<PositionedNode> {
    let horizontal_gap = if layout.column_count > 1 {
        (CANVAS_WIDTH - NODE_WIDTH - (HORIZONTAL_MARGIN * 2.0)) / (layout.column_count - 1) as f32
    } else {
        0.0
    };
    let vertical_gap = if layout.row_count > 1 { ROW_GAP } else { 0.0 };

    snapshot
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let column = index % layout.column_count;
            let row = index / layout.column_count;
            let x = if layout.column_count == 1 {
                (CANVAS_WIDTH - NODE_WIDTH) / 2.0
            } else {
                HORIZONTAL_MARGIN + column as f32 * horizontal_gap
            };
            let y = if layout.row_count == 1 {
                VERTICAL_MARGIN
            } else {
                VERTICAL_MARGIN + row as f32 * (NODE_HEIGHT + vertical_gap)
            };

            PositionedNode {
                node: node.clone(),
                x,
                y,
            }
        })
        .collect()
}

fn resolve_edge_nodes<'a>(
    positioned_nodes: &'a [PositionedNode],
    source_id: &str,
    target_id: &str,
) -> Option<(&'a PositionedNode, &'a PositionedNode)> {
    let source = positioned_nodes
        .iter()
        .find(|positioned| positioned.node.id == source_id)?;
    let target = positioned_nodes
        .iter()
        .find(|positioned| positioned.node.id == target_id)?;

    Some((source, target))
}

fn edge_path(source: &PositionedNode, target: &PositionedNode) -> String {
    let start_x = source.x + NODE_WIDTH;
    let start_y = source.y + (NODE_HEIGHT / 2.0);
    let end_x = target.x;
    let end_y = target.y + (NODE_HEIGHT / 2.0);
    let midpoint_x = (start_x + end_x) / 2.0;

    format!(
        "M {start_x:.1} {start_y:.1} C {midpoint_x:.1} {start_y:.1}, {midpoint_x:.1} {end_y:.1}, {end_x:.1} {end_y:.1}"
    )
}

impl AgentEdgeKind {
    fn css_name(&self) -> &'static str {
        match self {
            AgentEdgeKind::RoutesTo => "routes_to",
            AgentEdgeKind::WorksWithHint => "works_with_hint",
            AgentEdgeKind::DelegatesToHint => "delegates_to_hint",
        }
    }

    fn stroke(&self) -> &'static str {
        match self {
            AgentEdgeKind::RoutesTo => "#67e8f9",
            AgentEdgeKind::WorksWithHint => "#c084fc",
            AgentEdgeKind::DelegatesToHint => "#f59e0b",
        }
    }

    fn stroke_width(&self) -> &'static str {
        match self {
            AgentEdgeKind::RoutesTo => "3",
            AgentEdgeKind::WorksWithHint => "2.5",
            AgentEdgeKind::DelegatesToHint => "2.5",
        }
    }

    fn stroke_dasharray(&self) -> &'static str {
        match self {
            AgentEdgeKind::RoutesTo => "0",
            AgentEdgeKind::WorksWithHint => "7 9",
            AgentEdgeKind::DelegatesToHint => "10 7",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{AgentEdge, AgentGraphSnapshot, AgentStatus};

    #[component]
    fn GraphCanvasHarness(snapshot: AgentGraphSnapshot) -> Element {
        rsx! { GraphCanvas { snapshot } }
    }

    fn render_graph(snapshot: AgentGraphSnapshot) -> String {
        let mut dom =
            VirtualDom::new_with_props(GraphCanvasHarness, GraphCanvasHarnessProps { snapshot });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn node(id: &str, name: &str, status: AgentStatus) -> AgentNode {
        AgentNode {
            id: id.to_string(),
            name: name.to_string(),
            is_default: id == "planner",
            heartbeat_enabled: true,
            heartbeat_schedule: "every 5m".to_string(),
            active_session_count: if status == AgentStatus::Active { 1 } else { 0 },
            latest_activity_age_ms: Some(45_000),
            status,
        }
    }

    fn fixture_snapshot() -> AgentGraphSnapshot {
        AgentGraphSnapshot {
            nodes: vec![
                node("calendar", "calendar", AgentStatus::Idle),
                node("email", "email", AgentStatus::Active),
                node("planner", "planner", AgentStatus::Active),
            ],
            edges: vec![
                AgentEdge {
                    source_id: "planner".to_string(),
                    target_id: "email".to_string(),
                    kind: AgentEdgeKind::RoutesTo,
                },
                AgentEdge {
                    source_id: "planner".to_string(),
                    target_id: "calendar".to_string(),
                    kind: AgentEdgeKind::DelegatesToHint,
                },
            ],
            snapshot_ts: 1_640_995_200_000,
        }
    }

    #[test]
    fn graph_renders_the_expected_number_of_nodes_and_edges() {
        let html = render_graph(fixture_snapshot());

        assert_eq!(html.matches("data-agent-node=").count(), 3);
        assert_eq!(html.matches("data-agent-edge=").count(), 2);
        assert!(html.contains("Agent graph canvas"));
    }

    #[test]
    fn empty_graph_state_renders_gracefully() {
        let html = render_graph(AgentGraphSnapshot {
            nodes: Vec::new(),
            edges: Vec::new(),
            snapshot_ts: 1,
        });

        assert!(html.contains("Graph idle"));
        assert!(html.contains("No agents are available in the current graph snapshot yet."));
    }

    #[test]
    fn graph_layout_is_stable_for_the_same_snapshot() {
        let snapshot = fixture_snapshot();

        let layout = graph_layout_metrics(&snapshot);
        let first = layout_graph_nodes(&snapshot, layout);
        let second = layout_graph_nodes(&snapshot, layout);

        assert_eq!(first, second);
        assert_eq!(layout.canvas_height, 232.0);
        assert_eq!(first[0].x, 48.0);
        assert_eq!(first[0].y, 24.0);
        assert_eq!(first[1].x, 744.0);
        assert_eq!(first[2].x, 1440.0);
    }

    #[test]
    fn large_labels_are_truncated_for_the_canvas() {
        let html = render_graph(AgentGraphSnapshot {
            nodes: vec![node(
                "ops",
                "this-agent-name-is-way-too-long-for-the-first-slice",
                AgentStatus::Active,
            )],
            edges: Vec::new(),
            snapshot_ts: 1,
        });

        assert!(html.contains("this-agent-name-i…"));
        assert!(html.contains("aria-label=\"this-agent-name-is-way-too-long-for-the-first-slice agent card, status active\""));
    }

    #[test]
    fn default_badge_renders_as_a_separate_chip() {
        let html = render_graph(AgentGraphSnapshot {
            nodes: vec![node("planner", "planner", AgentStatus::Active)],
            edges: Vec::new(),
            snapshot_ts: 1,
        });

        assert!(html.contains("data-agent-default-badge=\"planner\""));
        assert!(html.contains(">DEFAULT<"));
    }
}
