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

/// Returns true when the heartbeat indicator should glow (active/rose).
/// Gray when disabled OR schedule is empty, "none", "0", or "disabled".
pub(crate) fn heartbeat_is_active(enabled: bool, schedule: &str) -> bool {
    if !enabled {
        return false;
    }
    let s = schedule.trim().to_ascii_lowercase();
    !matches!(s.as_str(), "" | "none" | "0" | "disabled")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_disabled_flag_is_gray() {
        assert!(!heartbeat_is_active(false, "*/5 * * * *"));
    }
    #[test]
    fn heartbeat_empty_schedule_is_gray() {
        assert!(!heartbeat_is_active(true, ""));
    }
    #[test]
    fn heartbeat_none_schedule_is_gray() {
        assert!(!heartbeat_is_active(true, "none"));
    }
    #[test]
    fn heartbeat_none_mixed_case_is_gray() {
        assert!(!heartbeat_is_active(true, "None"));
    }
    #[test]
    fn heartbeat_zero_schedule_is_gray() {
        assert!(!heartbeat_is_active(true, "0"));
    }
    #[test]
    fn heartbeat_disabled_string_is_gray() {
        assert!(!heartbeat_is_active(true, "disabled"));
    }
    #[test]
    fn heartbeat_valid_cron_is_active() {
        assert!(heartbeat_is_active(true, "*/5 * * * *"));
    }
    #[test]
    fn heartbeat_valid_interval_is_active() {
        assert!(heartbeat_is_active(true, "30m"));
    }
}
