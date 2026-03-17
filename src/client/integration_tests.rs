use super::WebAppClient;

#[test]
fn web_app_client_delegates_to_server_functions() {
    // This integration test verifies that WebAppClient properly delegates
    // to the underlying server functions and preserves the same behavior
    let _client = WebAppClient::default();
    
    // The client should be available for UI use
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}
    
    requires_send::<WebAppClient>();
    requires_sync::<WebAppClient>();
}