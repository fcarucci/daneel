// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for layout-level data providers (`dashboard_data`, `agent_overview_data`).

use dioxus::prelude::*;

/// When `resource` holds `Ok(value)`, store a clone in `cache` for stale-while-revalidate UI.
pub(crate) fn sync_last_ok_snapshot<T: Clone + 'static>(
    resource: &Resource<Result<T, ServerFnError>>,
    mut cache: Signal<Option<T>>,
) {
    if let Some(Ok(value)) = resource.read().as_ref() {
        cache.set(Some(value.clone()));
    }
}
