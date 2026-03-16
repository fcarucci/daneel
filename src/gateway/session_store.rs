// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

pub(crate) fn count_active_sessions(
    path: &Path,
    reference_timestamp_ms: u64,
    active_minutes: u64,
) -> Result<u64, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Could not read session store {}: {error}", path.display()))?;
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse session store {}: {error}", path.display()))?;
    let entries = parsed
        .as_object()
        .ok_or_else(|| format!("Session store {} is not a JSON object.", path.display()))?;

    let cutoff_ms = reference_timestamp_ms.saturating_sub(active_minutes * 60_000);
    Ok(entries
        .values()
        .filter_map(|entry| entry.get("updatedAt").and_then(Value::as_u64))
        .filter(|updated_at| *updated_at >= cutoff_ms)
        .count() as u64)
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
