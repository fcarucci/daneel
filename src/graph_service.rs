// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), allow(dead_code))]

use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "server")]
use crate::adapter::GatewayAdapter;
use crate::models::{
    graph::{AgentEdge, AgentEdgeKind, AgentGraphSnapshot, AgentNode, AgentStatus},
    runtime::ActiveSessionRecord,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphAssemblySummary {
    pub agent_count: usize,
    pub active_agent_count: usize,
    pub edge_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphAssemblyInputs {
    pub agents: Vec<AgentNode>,
    pub gateway_edges: Vec<AgentEdge>,
    pub active_sessions: Vec<ActiveSessionRecord>,
    pub hint_edges: Vec<AgentEdge>,
}

pub fn assemble_graph_snapshot(
    inputs: GraphAssemblyInputs,
    snapshot_ts: u64,
) -> AgentGraphSnapshot {
    let mut nodes_by_id = inputs
        .agents
        .into_iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    apply_active_sessions(&mut nodes_by_id, inputs.active_sessions);

    let known_ids = nodes_by_id.keys().cloned().collect::<BTreeSet<_>>();
    let edges = normalize_edges(&known_ids, inputs.gateway_edges, inputs.hint_edges);

    AgentGraphSnapshot {
        nodes: nodes_by_id.into_values().collect(),
        edges,
        snapshot_ts,
    }
}

pub fn summarize_graph_snapshot(snapshot: &AgentGraphSnapshot) -> GraphAssemblySummary {
    GraphAssemblySummary {
        agent_count: snapshot.nodes.len(),
        active_agent_count: snapshot
            .nodes
            .iter()
            .filter(|node| node.status == AgentStatus::Active)
            .count(),
        edge_count: snapshot.edges.len(),
    }
}

#[cfg(feature = "server")]
pub async fn load_graph_snapshot(
    adapter: &impl GatewayAdapter,
    snapshot_ts: u64,
) -> Result<AgentGraphSnapshot, String> {
    let (agents, gateway_edges, active_sessions, hint_edges) = tokio::try_join!(
        adapter.list_agents(),
        adapter.list_agent_bindings(),
        adapter.list_active_sessions(),
        adapter.list_agent_relationship_hints(),
    )?;

    Ok(assemble_graph_snapshot(
        GraphAssemblyInputs {
            agents,
            gateway_edges,
            active_sessions,
            hint_edges,
        },
        snapshot_ts,
    ))
}

fn apply_active_sessions(
    nodes_by_id: &mut BTreeMap<String, AgentNode>,
    active_sessions: Vec<ActiveSessionRecord>,
) {
    let mut sessions_by_agent = BTreeMap::<String, (u64, Option<u64>)>::new();

    for session in active_sessions {
        let Some(node) = nodes_by_id.get(&session.agent_id) else {
            continue;
        };

        let entry = sessions_by_agent
            .entry(node.id.clone())
            .or_insert((0, None));
        entry.0 += 1;
        entry.1 = min_optional_age(entry.1, session.age_ms);
    }

    for node in nodes_by_id.values_mut() {
        let (session_count, youngest_age) = sessions_by_agent
            .get(&node.id)
            .copied()
            .unwrap_or((0, None));

        node.active_session_count = session_count;
        node.latest_activity_age_ms = min_optional_age(node.latest_activity_age_ms, youngest_age);
        node.status = derive_status(node.latest_activity_age_ms, session_count);
    }
}

fn normalize_edges(
    known_ids: &BTreeSet<String>,
    gateway_edges: Vec<AgentEdge>,
    hint_edges: Vec<AgentEdge>,
) -> Vec<AgentEdge> {
    let mut normalized = Vec::new();
    let mut native_pairs = BTreeSet::new();

    for edge in gateway_edges {
        if !is_valid_edge(&edge, known_ids) {
            continue;
        }

        let key = edge_pair_key(&edge);
        if native_pairs.insert(key) {
            normalized.push(AgentEdge {
                source_id: edge.source_id,
                target_id: edge.target_id,
                kind: AgentEdgeKind::GatewayRouting,
            });
        }
    }

    let mut hint_pairs = BTreeSet::new();
    for edge in hint_edges {
        if !is_valid_edge(&edge, known_ids) {
            continue;
        }

        let key = edge_pair_key(&edge);
        if native_pairs.contains(&key) || !hint_pairs.insert(key) {
            continue;
        }

        normalized.push(AgentEdge {
            source_id: edge.source_id,
            target_id: edge.target_id,
            kind: AgentEdgeKind::MetadataHint,
        });
    }

    normalized.sort_by(|left, right| edge_sort_key(left).cmp(&edge_sort_key(right)));
    normalized
}

fn edge_sort_key(edge: &AgentEdge) -> (&str, &str, u8) {
    (
        &edge.source_id,
        &edge.target_id,
        match edge.kind {
            AgentEdgeKind::GatewayRouting => 0,
            AgentEdgeKind::MetadataHint => 1,
        },
    )
}

fn edge_pair_key(edge: &AgentEdge) -> (String, String) {
    (edge.source_id.clone(), edge.target_id.clone())
}

fn is_valid_edge(edge: &AgentEdge, known_ids: &BTreeSet<String>) -> bool {
    edge.source_id != edge.target_id
        && known_ids.contains(&edge.source_id)
        && known_ids.contains(&edge.target_id)
}

fn derive_status(latest_activity_age_ms: Option<u64>, active_session_count: u64) -> AgentStatus {
    if active_session_count > 0 {
        AgentStatus::Active
    } else if latest_activity_age_ms.is_some() {
        AgentStatus::Idle
    } else {
        AgentStatus::Unknown
    }
}

fn min_optional_age(existing: Option<u64>, candidate: Option<u64>) -> Option<u64> {
    match (existing, candidate) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "server")]
    use async_trait::async_trait;

    #[cfg(feature = "server")]
    use super::load_graph_snapshot;
    use super::{GraphAssemblyInputs, assemble_graph_snapshot, summarize_graph_snapshot};
    use crate::models::{
        graph::{AgentEdge, AgentEdgeKind, AgentNode, AgentStatus},
        runtime::ActiveSessionRecord,
    };
    #[cfg(feature = "server")]
    use crate::{
        adapter::GatewayAdapter,
        models::gateway::{GatewayLevel, GatewayStatusSnapshot},
    };

    fn agent(
        id: &str,
        latest_activity_age_ms: Option<u64>,
        active_session_count: u64,
        status: AgentStatus,
    ) -> AgentNode {
        AgentNode {
            id: id.to_string(),
            name: id.to_string(),
            is_default: id == "main",
            heartbeat_enabled: true,
            heartbeat_schedule: "every 5m".to_string(),
            active_session_count,
            latest_activity_age_ms,
            status,
        }
    }

    fn edge(source_id: &str, target_id: &str, kind: AgentEdgeKind) -> AgentEdge {
        AgentEdge {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            kind,
        }
    }

    fn active_session(
        session_id: &str,
        agent_id: &str,
        age_ms: Option<u64>,
    ) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            task: Some("work".to_string()),
            age_ms,
        }
    }

    fn assembly_inputs(
        agents: Vec<AgentNode>,
        gateway_edges: Vec<AgentEdge>,
        active_sessions: Vec<ActiveSessionRecord>,
        hint_edges: Vec<AgentEdge>,
    ) -> GraphAssemblyInputs {
        GraphAssemblyInputs {
            agents,
            gateway_edges,
            active_sessions,
            hint_edges,
        }
    }

    #[test]
    fn empty_adapter_data_produces_valid_empty_snapshot_without_panicking() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            42,
        );

        assert!(snapshot.nodes.is_empty());
        assert!(snapshot.edges.is_empty());
        assert_eq!(snapshot.snapshot_ts, 42);
    }

    #[test]
    fn agents_and_bindings_create_expected_node_and_edge_counts() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", None, 0, AgentStatus::Unknown),
                ],
                vec![edge("planner", "coder", AgentEdgeKind::GatewayRouting)],
                Vec::new(),
                Vec::new(),
            ),
            10,
        );

        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.edges.len(), 1);
    }

    #[test]
    fn active_sessions_decorate_node_status_correctly() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(50_000), 0, AgentStatus::Idle),
                    agent("coder", None, 0, AgentStatus::Unknown),
                ],
                Vec::new(),
                vec![
                    active_session("session-1", "planner", Some(500)),
                    active_session("session-2", "planner", Some(1_000)),
                ],
                Vec::new(),
            ),
            11,
        );

        assert_eq!(snapshot.nodes[0].id, "coder");
        assert_eq!(snapshot.nodes[0].status, AgentStatus::Unknown);
        assert_eq!(snapshot.nodes[1].id, "planner");
        assert_eq!(snapshot.nodes[1].status, AgentStatus::Active);
        assert_eq!(snapshot.nodes[1].active_session_count, 2);
        assert_eq!(snapshot.nodes[1].latest_activity_age_ms, Some(500));
    }

    #[test]
    fn local_relationship_hints_merge_without_duplicating_gateway_edges() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", Some(20_000), 0, AgentStatus::Idle),
                    agent("calendar", Some(30_000), 0, AgentStatus::Idle),
                ],
                vec![edge("planner", "coder", AgentEdgeKind::GatewayRouting)],
                Vec::new(),
                vec![
                    edge("planner", "coder", AgentEdgeKind::MetadataHint),
                    edge("planner", "calendar", AgentEdgeKind::MetadataHint),
                ],
            ),
            12,
        );

        assert_eq!(snapshot.edges.len(), 2);
        assert_eq!(
            snapshot.edges[0],
            edge("planner", "calendar", AgentEdgeKind::MetadataHint)
        );
        assert_eq!(
            snapshot.edges[1],
            edge("planner", "coder", AgentEdgeKind::GatewayRouting)
        );
    }

    #[test]
    fn edge_ordering_is_deterministic_for_stable_snapshots() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("calendar", Some(30_000), 0, AgentStatus::Idle),
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", Some(20_000), 0, AgentStatus::Idle),
                ],
                vec![
                    edge("planner", "coder", AgentEdgeKind::GatewayRouting),
                    edge("coder", "calendar", AgentEdgeKind::GatewayRouting),
                ],
                Vec::new(),
                vec![edge("planner", "calendar", AgentEdgeKind::MetadataHint)],
            ),
            13,
        );

        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["calendar", "coder", "planner"]
        );
        assert_eq!(
            snapshot.edges,
            vec![
                edge("coder", "calendar", AgentEdgeKind::GatewayRouting),
                edge("planner", "calendar", AgentEdgeKind::MetadataHint),
                edge("planner", "coder", AgentEdgeKind::GatewayRouting),
            ]
        );
    }

    #[test]
    fn orphan_edges_are_dropped_safely() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", Some(20_000), 0, AgentStatus::Idle),
                ],
                vec![
                    edge("planner", "ghost", AgentEdgeKind::GatewayRouting),
                    edge("planner", "coder", AgentEdgeKind::GatewayRouting),
                    edge("planner", "planner", AgentEdgeKind::GatewayRouting),
                ],
                Vec::new(),
                vec![edge("ghost", "coder", AgentEdgeKind::MetadataHint)],
            ),
            14,
        );

        assert_eq!(
            snapshot.edges,
            vec![edge("planner", "coder", AgentEdgeKind::GatewayRouting)]
        );
    }

    #[test]
    fn unknown_session_agent_references_do_not_create_phantom_nodes_or_crash_graph_assembly() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![agent("planner", Some(10_000), 0, AgentStatus::Idle)],
                Vec::new(),
                vec![active_session("session-1", "ghost", Some(100))],
                Vec::new(),
            ),
            15,
        );

        assert_eq!(snapshot.nodes.len(), 1);
        assert_eq!(snapshot.nodes[0].id, "planner");
        assert_eq!(snapshot.nodes[0].active_session_count, 0);
        assert_eq!(snapshot.nodes[0].status, AgentStatus::Idle);
    }

    #[test]
    fn node_ordering_is_deterministic_even_when_adapter_inputs_are_shuffled() {
        let left = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", Some(20_000), 0, AgentStatus::Idle),
                    agent("calendar", Some(30_000), 0, AgentStatus::Idle),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            16,
        );
        let right = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("calendar", Some(30_000), 0, AgentStatus::Idle),
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", Some(20_000), 0, AgentStatus::Idle),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            16,
        );

        assert_eq!(left.nodes, right.nodes);
    }

    #[test]
    fn graph_summary_values_match_the_assembled_snapshot() {
        let snapshot = assemble_graph_snapshot(
            assembly_inputs(
                vec![
                    agent("planner", Some(10_000), 0, AgentStatus::Idle),
                    agent("coder", None, 0, AgentStatus::Unknown),
                ],
                vec![edge("planner", "coder", AgentEdgeKind::GatewayRouting)],
                vec![active_session("session-1", "planner", Some(100))],
                Vec::new(),
            ),
            17,
        );

        let summary = summarize_graph_snapshot(&snapshot);

        assert_eq!(summary.agent_count, 2);
        assert_eq!(summary.active_agent_count, 1);
        assert_eq!(summary.edge_count, 1);
    }

    #[cfg(feature = "server")]
    #[derive(Clone, Debug, Default)]
    struct MockAdapter {
        agents: Vec<AgentNode>,
        bindings: Vec<AgentEdge>,
        sessions: Vec<ActiveSessionRecord>,
        hints: Vec<AgentEdge>,
    }

    #[cfg(feature = "server")]
    #[async_trait]
    impl GatewayAdapter for MockAdapter {
        async fn gateway_status(&self) -> Result<GatewayStatusSnapshot, String> {
            Ok(GatewayStatusSnapshot {
                connected: true,
                level: GatewayLevel::Healthy,
                summary: "healthy".to_string(),
                detail: "mock".to_string(),
                gateway_url: "ws://127.0.0.1:18789/".to_string(),
                protocol_version: Some(3),
                state_version: Some(1),
                uptime_ms: Some(1_000),
            })
        }

        async fn list_agents(&self) -> Result<Vec<AgentNode>, String> {
            Ok(self.agents.clone())
        }

        async fn list_agent_bindings(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(self.bindings.clone())
        }

        async fn list_active_sessions(&self) -> Result<Vec<ActiveSessionRecord>, String> {
            Ok(self.sessions.clone())
        }

        async fn list_agent_relationship_hints(&self) -> Result<Vec<AgentEdge>, String> {
            Ok(self.hints.clone())
        }
    }

    #[cfg(feature = "server")]
    #[tokio::test]
    async fn graph_assembly_against_a_mock_adapter_combines_inputs_into_one_stable_snapshot() {
        let adapter = MockAdapter {
            agents: vec![
                agent("planner", Some(10_000), 0, AgentStatus::Idle),
                agent("coder", None, 0, AgentStatus::Unknown),
                agent("calendar", Some(60_000), 0, AgentStatus::Idle),
            ],
            bindings: vec![edge("planner", "coder", AgentEdgeKind::GatewayRouting)],
            sessions: vec![active_session("session-1", "planner", Some(250))],
            hints: vec![
                edge("planner", "coder", AgentEdgeKind::MetadataHint),
                edge("planner", "calendar", AgentEdgeKind::MetadataHint),
            ],
        };

        let snapshot = load_graph_snapshot(&adapter, 18)
            .await
            .expect("assemble graph snapshot from adapter");

        assert_eq!(snapshot.snapshot_ts, 18);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["calendar", "coder", "planner"]
        );
        assert_eq!(snapshot.nodes[2].status, AgentStatus::Active);
        assert_eq!(
            snapshot.edges,
            vec![
                edge("planner", "calendar", AgentEdgeKind::MetadataHint),
                edge("planner", "coder", AgentEdgeKind::GatewayRouting),
            ]
        );
    }
}
