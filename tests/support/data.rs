// SPDX-License-Identifier: Apache-2.0

use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn pick_unused_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind an ephemeral port")
        .local_addr()
        .expect("read ephemeral port")
        .port()
}

pub fn write_session_store(path: &Path, updated_times: &[u64]) -> Result<(), String> {
    let payload = updated_times
        .iter()
        .enumerate()
        .map(|(index, updated_at)| {
            (
                format!("session-{index}"),
                json!({
                    "updatedAt": updated_at
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();

    fs::write(
        path,
        serde_json::to_vec_pretty(&Value::Object(payload)).unwrap(),
    )
    .map_err(|error| format!("Could not write session store {}: {error}", path.display()))
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis() as u64
}
