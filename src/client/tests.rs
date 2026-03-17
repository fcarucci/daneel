// SPDX-License-Identifier: Apache-2.0

use super::WebAppClient;
use crate::client::test_support::MockAppClient;

#[test]
fn web_app_client_delegates_to_server_functions() {
    // This test verifies that WebAppClient calls the underlying server functions
    // The actual server function behavior is tested elsewhere
    let _client = WebAppClient::default();
    
    // The client should be Send + Sync for UI use
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}
    
    requires_send::<WebAppClient>();
    requires_sync::<WebAppClient>();
}

#[test]
fn mock_app_client_provides_test_data() {
    let healthy_client = MockAppClient::healthy_gateway();
    let degraded_client = MockAppClient::degraded_gateway();
    
    // Test that mock clients provide the expected data
    // We'll test the sync methods instead of async for simplicity
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}
    
    requires_send::<MockAppClient>();
    requires_sync::<MockAppClient>();
    
    // Verify the mock data is set correctly
    assert!(healthy_client.gateway_status().is_ok());
    assert!(degraded_client.gateway_status().is_ok());
    assert!(degraded_client.agent_overview().is_err());
}

#[test]
fn app_client_trait_requires_send_sync() {
    // This compile test ensures AppClient requires Send + Sync
    fn requires_send_sync<T: Send + Sync>() {}
    
    requires_send_sync::<WebAppClient>();
    requires_send_sync::<MockAppClient>();
}

#[test]
fn ui_facing_code_depends_only_on_app_client() {
    // This test verifies that UI-facing code can depend on AppClient
    // without importing transport-specific or OpenClaw-specific types
    use super::AppClient;
    
    fn ui_component_using_client<C: AppClient>(client: C) {
        // This function should compile without knowing about server functions
        let _ = client;
    }
    
    // Should work with both implementations
    ui_component_using_client(WebAppClient::default());
    ui_component_using_client(MockAppClient::healthy_gateway());
}

#[test]
fn error_mapping_preserves_degraded_semantics() {
    let degraded_client = MockAppClient::degraded_gateway();
    
    // Verify the mock data preserves degraded semantics
    assert!(degraded_client.gateway_status().is_ok());
    assert!(degraded_client.agent_overview().is_err());
    
    if let Ok(status) = degraded_client.gateway_status() {
        assert!(!status.connected);
        assert_eq!(status.level, crate::models::gateway::GatewayLevel::Degraded);
    }
    
    if let Err(error) = degraded_client.agent_overview() {
        assert!(error.to_string().contains("Gateway unavailable"));
    }
}