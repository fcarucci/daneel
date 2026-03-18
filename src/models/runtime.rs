// SPDX-License-Identifier: Apache-2.0

// This contract is introduced ahead of its first server-function consumer so the adapter
// boundary can stay neutral in T3.1.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ActiveSessionRecord {
    pub session_id: String,
    pub agent_id: String,
    pub task: Option<String>,
    pub age_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::ActiveSessionRecord;

    #[test]
    fn active_session_record_json_round_trip() {
        let record = ActiveSessionRecord {
            session_id: "session-1".to_string(),
            agent_id: "planner".to_string(),
            task: Some("plan".to_string()),
            age_ms: Some(500),
        };

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: ActiveSessionRecord = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(record, deserialized);
    }
}
