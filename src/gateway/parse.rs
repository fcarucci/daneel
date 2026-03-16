// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

pub(crate) fn find_string(value: &Value, candidates: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for candidate in candidates {
                if let Some(Value::String(found)) = map.get(*candidate) {
                    return Some(found.clone());
                }
            }

            for nested in map.values() {
                if let Some(found) = find_string(nested, candidates) {
                    return Some(found);
                }
            }

            None
        }
        Value::Array(values) => values
            .iter()
            .find_map(|nested| find_string(nested, candidates)),
        _ => None,
    }
}

pub(crate) fn find_u64(value: &Value, candidates: &[&str]) -> Option<u64> {
    match value {
        Value::Object(map) => {
            for candidate in candidates {
                if let Some(Value::Number(found)) = map.get(*candidate)
                    && let Some(parsed) = found.as_u64()
                {
                    return Some(parsed);
                }
            }

            for nested in map.values() {
                if let Some(found) = find_u64(nested, candidates) {
                    return Some(found);
                }
            }

            None
        }
        Value::Array(values) => values
            .iter()
            .find_map(|nested| find_u64(nested, candidates)),
        _ => None,
    }
}

pub(crate) fn require_object<'a>(
    value: &'a Value,
    key: &str,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{context} did not include {key}."))
}

pub(crate) fn require_array<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a [Value], String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.as_slice())
        .ok_or_else(|| format!("{context} did not include {key}."))
}
