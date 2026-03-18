// SPDX-License-Identifier: Apache-2.0

use super::WebAppClient;
use crate::gateway::{
    AGENT_GRAPH_SNAPSHOT_ENDPOINT, AGENT_OVERVIEW_ENDPOINT, GATEWAY_STATUS_ENDPOINT,
};

#[test]
fn web_app_client_is_available_for_ui_use() {
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}

    requires_send::<WebAppClient>();
    requires_sync::<WebAppClient>();
}

#[test]
fn stable_server_function_endpoints_are_explicit() {
    assert_eq!(GATEWAY_STATUS_ENDPOINT, "gateway/status");
    assert_eq!(AGENT_OVERVIEW_ENDPOINT, "agents/overview");
    assert_eq!(AGENT_GRAPH_SNAPSHOT_ENDPOINT, "graph/snapshot");
}
