// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Active,
    Idle,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEdgeKind {
    GatewayRouting,
    MetadataHint,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentNode {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub heartbeat_enabled: bool,
    pub heartbeat_schedule: String,
    pub active_session_count: u64,
    pub latest_activity_age_ms: Option<u64>,
    pub status: AgentStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: AgentEdgeKind,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentGraphSnapshot {
    pub nodes: Vec<AgentNode>,
    pub edges: Vec<AgentEdge>,
    pub snapshot_ts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_node_json_round_trip() {
        let node = AgentNode {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            is_default: false,
            heartbeat_enabled: true,
            heartbeat_schedule: "* * * * *".to_string(),
            active_session_count: 3,
            latest_activity_age_ms: Some(1000),
            status: AgentStatus::Active,
        };

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: AgentNode = serde_json::from_str(&json).unwrap();

        assert_eq!(node, deserialized);
    }

    #[test]
    fn agent_edge_json_round_trip() {
        let edge = AgentEdge {
            source_id: "source-agent".to_string(),
            target_id: "target-agent".to_string(),
            kind: AgentEdgeKind::GatewayRouting,
        };

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: AgentEdge = serde_json::from_str(&json).unwrap();

        assert_eq!(edge, deserialized);
    }

    #[test]
    fn graph_snapshot_json_round_trip() {
        let snapshot = AgentGraphSnapshot {
            nodes: vec![AgentNode {
                id: "test-agent".to_string(),
                name: "Test Agent".to_string(),
                is_default: false,
                heartbeat_enabled: true,
                heartbeat_schedule: "* * * * *".to_string(),
                active_session_count: 3,
                latest_activity_age_ms: Some(1000),
                status: AgentStatus::Active,
            }],
            edges: vec![AgentEdge {
                source_id: "test-agent".to_string(),
                target_id: "other-agent".to_string(),
                kind: AgentEdgeKind::MetadataHint,
            }],
            snapshot_ts: 1640995200000,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: AgentGraphSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn empty_snapshot_round_trips_cleanly() {
        let snapshot = AgentGraphSnapshot {
            nodes: Vec::new(),
            edges: Vec::new(),
            snapshot_ts: 1640995200000,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: AgentGraphSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot, deserialized);
        assert!(deserialized.nodes.is_empty());
        assert!(deserialized.edges.is_empty());
    }

    #[test]
    fn gateway_routing_edge_serializes_distinctly() {
        let edge = AgentEdge {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            kind: AgentEdgeKind::GatewayRouting,
        };

        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("gateway_routing"));
        assert!(!json.contains("metadata_hint"));
    }

    #[test]
    fn metadata_hint_edge_serializes_distinctly() {
        let edge = AgentEdge {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            kind: AgentEdgeKind::MetadataHint,
        };

        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("metadata_hint"));
        assert!(!json.contains("gateway_routing"));
    }

    #[test]
    fn agent_status_active_serializes_correctly() {
        let status = AgentStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn agent_status_idle_serializes_correctly() {
        let status = AgentStatus::Idle;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"idle\"");
    }
}
