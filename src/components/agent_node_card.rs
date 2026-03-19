// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::models::graph::{AgentNode, AgentStatus};

pub(crate) const NODE_WIDTH: f32 = 352.0;
pub(crate) const NODE_HEIGHT: f32 = 184.0;
const MAX_LABEL_CHARS: usize = 18;

#[component]
pub fn AgentNodeCard(node: AgentNode, x: f32, y: f32) -> Element {
    let title = truncated_graph_label(node.name.as_str());
    let monogram = node_monogram(node.name.as_str());
    let activity = activity_label(&node);
    let status_text = status_label(&node.status);
    let aria_label = format!(
        "{} agent card, status {}",
        node.name,
        status_text.to_lowercase()
    );
    let heartbeat = heartbeat_label(&node);

    rsx! {
        g {
            "data-agent-node": node.id.as_str(),
            "data-agent-node-status": status_text.to_lowercase(),
            transform: format!("translate({x} {y})"),
            tabindex: "0",
            "focusable": "true",
            role: "group",
            "aria-label": aria_label,
            style: "cursor: default;",
            rect {
                x: "-10",
                y: "-10",
                width: NODE_WIDTH + 20.0,
                height: NODE_HEIGHT + 20.0,
                rx: "34",
                fill: status_glow(&node.status),
                opacity: "0.32",
            }
            rect {
                width: NODE_WIDTH,
                height: NODE_HEIGHT,
                rx: "28",
                fill: node_fill(&node.status),
                stroke: node_stroke(&node.status),
                stroke_width: "1.5",
            }
            rect {
                x: "1",
                y: "1",
                width: NODE_WIDTH - 2.0,
                height: NODE_HEIGHT - 2.0,
                rx: "27",
                fill: "none",
                stroke: "rgba(255,255,255,0.06)",
                stroke_width: "1",
            }
            circle {
                cx: "38",
                cy: "38",
                r: "16",
                fill: node_signal(&node.status),
            }
            text {
                x: "38",
                y: "44",
                fill: "#052e2b",
                font_size: "13",
                font_weight: "800",
                letter_spacing: "0.12em",
                text_anchor: "middle",
                {monogram}
            }
            text {
                x: "78",
                y: "50",
                fill: "#f8fafc",
                font_size: "30",
                font_weight: "650",
                letter_spacing: "-0.02em",
                {title}
            }
            text {
                x: "34",
                y: "90",
                fill: "#7dd3fc",
                font_size: "15",
                font_weight: "700",
                letter_spacing: "0.12em",
                {heartbeat}
            }
            text {
                x: "34",
                y: "118",
                fill: "#94a3b8",
                font_size: "21",
                font_weight: "700",
                letter_spacing: "0.16em",
                {status_text}
            }
            text {
                x: "34",
                y: "154",
                fill: "#cbd5e1",
                font_size: "21",
                "Latest: "
                {activity}
            }
            if node.is_default {
                g { "data-agent-default-badge": node.id.as_str(),
                    rect {
                        x: NODE_WIDTH - 150.0,
                        y: "18",
                        width: "116",
                        height: "30",
                        rx: "17",
                        fill: "rgba(34,211,238,0.12)",
                        stroke: "rgba(103,232,249,0.3)",
                        stroke_width: "1.5",
                    }
                    text {
                        x: NODE_WIDTH - 92.0,
                        y: "38",
                        fill: "#67e8f9",
                        font_size: "15",
                        font_weight: "700",
                        letter_spacing: "0.18em",
                        text_anchor: "middle",
                        "DEFAULT"
                    }
                }
            }
        }
    }
}

pub(crate) fn activity_label(node: &AgentNode) -> String {
    match node.latest_activity_age_ms {
        Some(age_ms) if age_ms < 60_000 => format!("{}s ago", age_ms / 1_000),
        Some(age_ms) if age_ms < 3_600_000 => format!("{}m ago", age_ms / 60_000),
        Some(age_ms) => format!("{}h ago", age_ms / 3_600_000),
        None => "No recent activity".to_string(),
    }
}

pub(crate) fn node_fill(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "rgba(6, 78, 59, 0.82)",
        AgentStatus::Idle => "rgba(15, 23, 42, 0.96)",
        AgentStatus::Unknown => "rgba(30, 41, 59, 0.92)",
    }
}

pub(crate) fn node_stroke(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "rgba(110, 231, 183, 0.6)",
        AgentStatus::Idle => "rgba(148, 163, 184, 0.32)",
        AgentStatus::Unknown => "rgba(251, 191, 36, 0.34)",
    }
}

pub(crate) fn node_signal(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "#6ee7b7",
        AgentStatus::Idle => "#94a3b8",
        AgentStatus::Unknown => "#fbbf24",
    }
}

pub(crate) fn status_label(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "ACTIVE",
        AgentStatus::Idle => "IDLE",
        AgentStatus::Unknown => "UNKNOWN",
    }
}

pub(crate) fn truncated_graph_label(name: &str) -> String {
    let grapheme_count = name.chars().count();
    if grapheme_count <= MAX_LABEL_CHARS {
        return name.to_string();
    }

    let mut truncated = name.chars().take(MAX_LABEL_CHARS - 1).collect::<String>();
    truncated.push('…');
    truncated
}

fn heartbeat_label(node: &AgentNode) -> String {
    if node.heartbeat_enabled {
        format!(
            "HEARTBEAT {schedule}",
            schedule = node.heartbeat_schedule.to_uppercase()
        )
    } else {
        "HEARTBEAT OFF".to_string()
    }
}

fn node_monogram(name: &str) -> String {
    let mut letters = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    if letters.is_empty() {
        letters.push_str("AG");
    }

    letters
}

fn status_glow(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Active => "rgba(16, 185, 129, 0.28)",
        AgentStatus::Idle => "rgba(148, 163, 184, 0.14)",
        AgentStatus::Unknown => "rgba(245, 158, 11, 0.16)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::AgentStatus;

    #[component]
    fn AgentNodeCardHarness(node: AgentNode) -> Element {
        rsx! {
            svg {
                view_box: "0 0 500 300",
                AgentNodeCard { node, x: 24.0, y: 24.0 }
            }
        }
    }

    fn render_card(node: AgentNode) -> String {
        let mut dom =
            VirtualDom::new_with_props(AgentNodeCardHarness, AgentNodeCardHarnessProps { node });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn node(id: &str, name: &str, status: AgentStatus) -> AgentNode {
        AgentNode {
            id: id.to_string(),
            name: name.to_string(),
            is_default: id == "main",
            heartbeat_enabled: true,
            heartbeat_schedule: "every 5m".to_string(),
            active_session_count: if status == AgentStatus::Active { 1 } else { 0 },
            latest_activity_age_ms: Some(90_000),
            status,
        }
    }

    #[test]
    fn active_agent_card_renders_different_styling_from_idle() {
        let active = render_card(node("coder", "coder", AgentStatus::Active));
        let idle = render_card(node("calendar", "calendar", AgentStatus::Idle));

        assert!(active.contains("data-agent-node-status=\"active\""));
        assert!(active.contains("rgba(6, 78, 59, 0.82)"));
        assert!(idle.contains("data-agent-node-status=\"idle\""));
        assert!(idle.contains("rgba(15, 23, 42, 0.96)"));
    }

    #[test]
    fn card_content_is_readable_with_keyboard_focus() {
        let html = render_card(node("main", "main", AgentStatus::Active));

        assert!(html.contains("tabindex=\"0\""));
        assert!(html.contains("focusable=\"true\""));
        assert!(html.contains("aria-label=\"main agent card, status active\""));
        assert!(html.contains(">DEFAULT<"));
    }

    #[test]
    fn long_agent_names_and_missing_activity_render_gracefully() {
        let html = render_card(AgentNode {
            latest_activity_age_ms: None,
            ..node(
                "ops",
                "this-agent-name-is-way-too-long-for-the-first-slice",
                AgentStatus::Unknown,
            )
        });

        assert!(html.contains("this-agent-name-i…"));
        assert!(html.contains("No recent activity"));
    }
}
