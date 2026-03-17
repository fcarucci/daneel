// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveGatewayLevel {
    Healthy,
    Degraded,
    Connecting,
    Disconnected,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiveGatewayEvent {
    pub level: LiveGatewayLevel,
    pub summary: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorConnectionState {
    Connected,
    Connecting,
    Disconnected,
    Degraded,
}

pub fn resolve_operator_connection_state(
    backend_state: BackendConnectionState,
    gateway_level: Option<LiveGatewayLevel>,
) -> OperatorConnectionState {
    match backend_state {
        BackendConnectionState::Disconnected => OperatorConnectionState::Disconnected,
        BackendConnectionState::Connecting => match gateway_level {
            Some(LiveGatewayLevel::Healthy) => OperatorConnectionState::Connected,
            Some(LiveGatewayLevel::Degraded) => OperatorConnectionState::Degraded,
            Some(LiveGatewayLevel::Disconnected) => OperatorConnectionState::Disconnected,
            Some(LiveGatewayLevel::Connecting) | None => OperatorConnectionState::Connecting,
        },
        BackendConnectionState::Connected => match gateway_level {
            Some(LiveGatewayLevel::Healthy) => OperatorConnectionState::Connected,
            Some(LiveGatewayLevel::Degraded) => OperatorConnectionState::Degraded,
            Some(LiveGatewayLevel::Disconnected) => OperatorConnectionState::Disconnected,
            Some(LiveGatewayLevel::Connecting) | None => OperatorConnectionState::Connecting,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackendConnectionState, LiveGatewayLevel, OperatorConnectionState,
        resolve_operator_connection_state,
    };

    #[test]
    fn disconnected_backend_overrides_gateway_health() {
        let state = resolve_operator_connection_state(
            BackendConnectionState::Disconnected,
            Some(LiveGatewayLevel::Healthy),
        );

        assert_eq!(state, OperatorConnectionState::Disconnected);
    }

    #[test]
    fn connected_backend_with_degraded_gateway_stays_degraded() {
        let state = resolve_operator_connection_state(
            BackendConnectionState::Connected,
            Some(LiveGatewayLevel::Degraded),
        );

        assert_eq!(state, OperatorConnectionState::Degraded);
    }

    #[test]
    fn connecting_backend_without_gateway_state_is_connecting() {
        let state = resolve_operator_connection_state(BackendConnectionState::Connecting, None);

        assert_eq!(state, OperatorConnectionState::Connecting);
    }
}
