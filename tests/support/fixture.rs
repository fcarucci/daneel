// SPDX-License-Identifier: Apache-2.0

use super::util::{now_ms, write_session_store};
use serde_json::{Value, json};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tempfile::{TempDir, tempdir};

const AUTH_TOKEN: &str = "test-token";

#[derive(Clone)]
pub(crate) struct GatewayPayload {
    pub(crate) state_version: u64,
    pub(crate) uptime_ms: u64,
    pub(crate) snapshot_ts: u64,
    pub(crate) agents: Vec<MockAgent>,
}

#[derive(Clone)]
pub(crate) struct MockAgent {
    id: String,
    is_default: bool,
    heartbeat_enabled: bool,
    heartbeat_every: String,
    session_store_path: PathBuf,
    latest_session_key: String,
    latest_activity_age_ms: u64,
}

impl MockAgent {
    pub(crate) fn new(
        id: &str,
        is_default: bool,
        heartbeat_enabled: bool,
        heartbeat_every: &str,
        session_store_path: &Path,
        latest_session_key: &str,
        latest_activity_age_ms: u64,
    ) -> Self {
        Self {
            id: id.to_string(),
            is_default,
            heartbeat_enabled,
            heartbeat_every: heartbeat_every.to_string(),
            session_store_path: session_store_path.to_path_buf(),
            latest_session_key: latest_session_key.to_string(),
            latest_activity_age_ms,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        json!({
            "agentId": self.id,
            "name": self.id,
            "isDefault": self.is_default,
            "heartbeat": {
                "enabled": self.heartbeat_enabled,
                "every": self.heartbeat_every,
                "model": if self.is_default {
                    Value::String("default".to_string())
                } else {
                    Value::Null
                }
            },
            "sessions": {
                "path": self.session_store_path.display().to_string(),
                "recent": [
                    {
                        "key": self.latest_session_key,
                        "age": self.latest_activity_age_ms
                    }
                ]
            }
        })
    }
}

pub(crate) struct TestFixture {
    _tempdir: TempDir,
    pub(crate) config_path: PathBuf,
    pub(crate) gateway_payload: GatewayPayload,
}

impl TestFixture {
    pub(crate) fn healthy() -> Result<Self, String> {
        let tempdir = tempdir().map_err(|error| format!("Could not create tempdir: {error}"))?;
        let config_path = tempdir.path().join("openclaw.json");
        let session_root = tempdir.path().join("sessions");
        fs::create_dir_all(&session_root)
            .map_err(|error| format!("Could not create session root: {error}"))?;

        let now_ms = now_ms();
        let main_path = session_root.join("main.json");
        let calendar_path = session_root.join("calendar.json");
        let planner_path = session_root.join("planner.json");

        write_session_store(
            &main_path,
            &[now_ms - 120_000, now_ms - 240_000, now_ms - 7_200_000],
        )?;
        write_session_store(&calendar_path, &[now_ms - 300_000, now_ms - 8_400_000])?;
        write_session_store(&planner_path, &[now_ms - 8_400_000])?;

        Ok(Self {
            _tempdir: tempdir,
            config_path,
            gateway_payload: GatewayPayload {
                state_version: 42,
                uptime_ms: 123_456,
                snapshot_ts: now_ms,
                agents: vec![
                    MockAgent::new(
                        "main",
                        true,
                        true,
                        "120m",
                        &main_path,
                        "agent:main:cron:alpha",
                        120_000,
                    ),
                    MockAgent::new(
                        "calendar",
                        false,
                        true,
                        "120m",
                        &calendar_path,
                        "agent:calendar:cron:beta",
                        300_000,
                    ),
                    MockAgent::new(
                        "planner",
                        false,
                        true,
                        "0m",
                        &planner_path,
                        "agent:planner:cron:gamma",
                        8_400_000,
                    ),
                ],
            },
        })
    }

    pub(crate) fn degraded() -> Result<Self, String> {
        let tempdir = tempdir().map_err(|error| format!("Could not create tempdir: {error}"))?;
        let config_path = tempdir.path().join("openclaw.json");
        Ok(Self {
            _tempdir: tempdir,
            config_path,
            gateway_payload: GatewayPayload {
                state_version: 0,
                uptime_ms: 0,
                snapshot_ts: now_ms(),
                agents: vec![],
            },
        })
    }

    pub(crate) fn write_openclaw_config(&self, gateway_addr: SocketAddr) -> Result<(), String> {
        let payload = json!({
            "gateway": {
                "port": gateway_addr.port(),
                "auth": {
                    "token": AUTH_TOKEN
                }
            }
        });
        fs::write(
            &self.config_path,
            serde_json::to_vec_pretty(&payload).unwrap(),
        )
        .map_err(|error| format!("Could not write test OpenClaw config: {error}"))
    }
}
