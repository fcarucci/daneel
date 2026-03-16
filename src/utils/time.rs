// SPDX-License-Identifier: Apache-2.0

const SECOND_MS: u64 = 1_000;
const MINUTE_MS: u64 = 60_000;
const HOUR_MS: u64 = 3_600_000;
const DAY_MS: u64 = 86_400_000;

pub(crate) const ACTIVE_WINDOW_MS: u64 = 600_000;

pub(crate) fn format_age_badge(age_ms: u64) -> String {
    if age_ms < MINUTE_MS {
        format!("{}s ago", age_ms / SECOND_MS)
    } else if age_ms < HOUR_MS {
        format!("{}m ago", age_ms / MINUTE_MS)
    } else if age_ms < DAY_MS {
        format!("{}h ago", age_ms / HOUR_MS)
    } else {
        format!("{}d ago", age_ms / DAY_MS)
    }
}
