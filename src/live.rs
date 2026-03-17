// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use std::time::Duration;

#[cfg(feature = "server")]
use crate::models::live_gateway::{LiveGatewayEvent, LiveGatewayLevel};

#[cfg(feature = "server")]
use {
    crate::gateway::{
        LoadedGatewayConfig, connect_request, load_gateway_config, wait_for_response,
    },
    dioxus_server::axum::{
        self, Router,
        response::sse::{Event, KeepAlive, Sse},
        routing::get,
    },
    futures_util::{SinkExt, StreamExt, stream},
    serde_json::{Value, json},
    std::convert::Infallible,
    std::sync::{Arc, RwLock},
    tokio::sync::broadcast,
    tokio_stream::wrappers::BroadcastStream,
    tokio_tungstenite::{connect_async, tungstenite::Message},
};

#[cfg(feature = "server")]
struct LiveBridgeConfig {
    connect_request_id: &'static str,
    health_request_id: &'static str,
    gateway_retry_delay_secs: u64,
    sse_keep_alive_secs: u64,
}

#[cfg(feature = "server")]
const LIVE_BRIDGE_CONFIG: LiveBridgeConfig = LiveBridgeConfig {
    connect_request_id: "connect-live-1",
    health_request_id: "health-live-1",
    gateway_retry_delay_secs: 5,
    sse_keep_alive_secs: 15,
};

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct LiveEventHub {
    tx: broadcast::Sender<LiveGatewayEvent>,
    latest: Arc<RwLock<Option<LiveGatewayEvent>>>,
}

#[cfg(feature = "server")]
impl LiveEventHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            tx,
            latest: Arc::new(RwLock::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LiveGatewayEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: LiveGatewayEvent) {
        *self.write_latest() = Some(event.clone());
        let _ = self.tx.send(event);
    }

    pub fn latest(&self) -> Option<LiveGatewayEvent> {
        self.read_latest().clone()
    }

    fn write_latest(&self) -> std::sync::RwLockWriteGuard<'_, Option<LiveGatewayEvent>> {
        self.latest
            .write()
            .expect("lock live gateway event snapshot")
    }

    fn read_latest(&self) -> std::sync::RwLockReadGuard<'_, Option<LiveGatewayEvent>> {
        self.latest
            .read()
            .expect("lock live gateway event snapshot")
    }
}

#[cfg(feature = "server")]
impl LiveGatewayEvent {
    fn new(level: LiveGatewayLevel, summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            level,
            summary: summary.into(),
            detail: detail.into(),
        }
    }

    fn disconnected(detail: String) -> Self {
        Self::connecting("Gateway event stream reconnecting", detail)
    }

    fn unavailable(detail: String) -> Self {
        Self::degraded("Gateway event stream unavailable", detail)
    }

    fn degraded(summary: &str, detail: String) -> Self {
        Self::new(LiveGatewayLevel::Degraded, summary, detail)
    }

    fn connecting(summary: &str, detail: String) -> Self {
        Self::new(LiveGatewayLevel::Connecting, summary, detail)
    }

    fn health_update(status: &str) -> Self {
        Self::new(
            if status.eq_ignore_ascii_case("healthy") {
                LiveGatewayLevel::Healthy
            } else {
                LiveGatewayLevel::Degraded
            },
            format!("Gateway health update: {status}."),
            "Live gateway event received.",
        )
    }
}

#[cfg(feature = "server")]
pub fn router() -> Router {
    axum::Router::new().route("/api/gateway/events", get(sse_gateway_events))
}

#[cfg(feature = "server")]
pub async fn run_gateway_event_bridge(hub: LiveEventHub) {
    let config = match load_gateway_config() {
        Ok(config) => config,
        Err(error) => {
            hub.publish(LiveGatewayEvent::unavailable(error));
            return;
        }
    };

    loop {
        hub.publish(LiveGatewayEvent::connecting(
            "Connecting to the gateway event stream",
            format!("Opening a live websocket to {}.", config.ws_url),
        ));
        if let Err(error) = stream_gateway_events(&config, &hub).await {
            hub.publish(LiveGatewayEvent::disconnected(error));
            tokio::time::sleep(Duration::from_secs(
                LIVE_BRIDGE_CONFIG.gateway_retry_delay_secs,
            ))
            .await;
        }
    }
}

#[cfg(feature = "server")]
async fn stream_gateway_events(
    config: &LoadedGatewayConfig,
    hub: &LiveEventHub,
) -> Result<(), String> {
    let (mut socket, _) = connect_async(config.ws_url.as_str())
        .await
        .map_err(|error| {
            format!(
                "Could not open gateway websocket {}: {error}",
                config.ws_url
            )
        })?;

    send_gateway_request(
        &mut socket,
        connect_live_request(config),
        "Could not send gateway connect request",
    )
    .await?;
    wait_for_response(&mut socket, LIVE_BRIDGE_CONFIG.connect_request_id).await?;

    send_gateway_request(
        &mut socket,
        health_live_request(),
        "Could not send gateway health request",
    )
    .await?;

    while let Some(message) = socket.next().await {
        let message = message.map_err(|error| format!("Gateway websocket error: {error}"))?;
        let Message::Text(text) = message else {
            continue;
        };
        let payload: Value = serde_json::from_str(&text)
            .map_err(|error| format!("Could not parse gateway event frame: {error}"))?;

        if let Some(event) = parse_gateway_event(&payload) {
            hub.publish(event);
        }
    }

    Err("Gateway websocket closed".to_string())
}

#[cfg(feature = "server")]
fn parse_gateway_event(payload: &Value) -> Option<LiveGatewayEvent> {
    let kind = payload.get("type").and_then(Value::as_str)?;
    if kind == "event" {
        return parse_health_event(payload);
    }
    if kind == "res" {
        return parse_health_response(payload);
    }
    None
}

#[cfg(feature = "server")]
fn parse_health_event(payload: &Value) -> Option<LiveGatewayEvent> {
    let event_name = payload.get("event").and_then(Value::as_str)?;
    if event_name != "health" {
        return None;
    }

    health_status_from_event(payload).map(|status| LiveGatewayEvent::health_update(&status))
}

#[cfg(feature = "server")]
fn parse_health_response(payload: &Value) -> Option<LiveGatewayEvent> {
    let response_id = payload.get("id").and_then(Value::as_str)?;
    if response_id != LIVE_BRIDGE_CONFIG.health_request_id {
        return None;
    }

    if payload.get("ok").and_then(Value::as_bool) == Some(false) {
        let detail = payload
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("The gateway rejected the live health request.")
            .to_string();
        return Some(LiveGatewayEvent::degraded(
            "Gateway health request failed",
            detail,
        ));
    }

    health_status_from_response(payload).map(|status| LiveGatewayEvent::health_update(&status))
}

#[cfg(feature = "server")]
fn health_status_from_event(payload: &Value) -> Option<String> {
    value_at_any_string_path(
        payload,
        &[
            &["payload", "status", "health", "state"],
            &["payload", "status"],
            &["payload", "health"],
            &["payload", "state"],
        ],
    )
}

#[cfg(feature = "server")]
fn health_status_from_response(payload: &Value) -> Option<String> {
    value_at_any_string_path(
        payload,
        &[
            &["payload", "status"],
            &["payload", "health"],
            &["payload", "state"],
        ],
    )
}

#[cfg(feature = "server")]
fn value_at_string_path(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

#[cfg(feature = "server")]
fn value_at_any_string_path(value: &Value, paths: &[&[&str]]) -> Option<String> {
    paths
        .iter()
        .find_map(|path| value_at_string_path(value, path))
}

#[cfg(feature = "server")]
fn event_to_sse(event: &LiveGatewayEvent) -> Option<Result<Event, Infallible>> {
    serde_json::to_string(event)
        .ok()
        .map(|data| Ok(Event::default().data(data)))
}

#[cfg(feature = "server")]
async fn sse_gateway_events() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let hub = require_live_hub();
    let initial_event = hub.latest().and_then(|event| event_to_sse(&event));
    let initial_stream = stream::iter(initial_event);
    let update_stream = BroadcastStream::new(hub.subscribe()).filter_map(|item| async move {
        match item {
            Ok(event) => event_to_sse(&event),
            Err(_) => None,
        }
    });
    let stream = initial_stream.chain(update_stream);

    Sse::new(stream).keep_alive(
        KeepAlive::new().interval(Duration::from_secs(LIVE_BRIDGE_CONFIG.sse_keep_alive_secs)),
    )
}

#[cfg(feature = "server")]
fn connect_live_request(config: &LoadedGatewayConfig) -> Value {
    connect_request(LIVE_BRIDGE_CONFIG.connect_request_id, &config.token)
}

#[cfg(feature = "server")]
fn health_live_request() -> Value {
    json!({
        "type": "req",
        "id": LIVE_BRIDGE_CONFIG.health_request_id,
        "method": "health",
        "params": {}
    })
}

#[cfg(feature = "server")]
async fn send_gateway_request(
    socket: &mut (
             impl futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin
         ),
    request: Value,
    context: &str,
) -> Result<(), String> {
    socket
        .send(Message::Text(request.to_string().into()))
        .await
        .map_err(|error| format!("{context}: {error}"))
}

#[cfg(feature = "server")]
static LIVE_HUB: std::sync::OnceLock<LiveEventHub> = std::sync::OnceLock::new();

#[cfg(feature = "server")]
pub fn init_live_hub() -> LiveEventHub {
    LIVE_HUB.get_or_init(LiveEventHub::new).clone()
}

#[cfg(feature = "server")]
fn require_live_hub() -> &'static LiveEventHub {
    LIVE_HUB
        .get()
        .expect("LiveEventHub must be initialized before serving SSE.")
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_gateway_event_ignores_non_health_events() {
        let payload = json!({
            "type": "event",
            "event": "agent",
            "payload": {
                "agentId": "email",
                "status": {
                    "health": {
                        "state": "degraded"
                    }
                }
            }
        });

        assert!(parse_gateway_event(&payload).is_none());
    }

    #[test]
    fn parse_gateway_event_reads_health_event_state() {
        let payload = json!({
            "type": "event",
            "event": "health",
            "payload": {
                "status": {
                    "health": {
                        "state": "healthy"
                    }
                }
            }
        });

        let event = parse_gateway_event(&payload).expect("parse health event");
        assert!(matches!(event.level, LiveGatewayLevel::Healthy));
        assert_eq!(event.summary, "Gateway health update: healthy.");
    }

    #[test]
    fn parse_gateway_event_reads_health_response_state() {
        let payload = json!({
            "type": "res",
            "id": "health-live-1",
            "ok": true,
            "payload": {
                "status": "degraded"
            }
        });

        let event = parse_gateway_event(&payload).expect("parse health response");
        assert!(matches!(event.level, LiveGatewayLevel::Degraded));
        assert_eq!(event.summary, "Gateway health update: degraded.");
    }

    #[test]
    fn parse_gateway_event_ignores_connect_response() {
        let payload = json!({
            "type": "res",
            "id": "connect-live-1",
            "ok": true,
            "payload": {
                "protocolVersion": 3
            }
        });

        assert!(parse_gateway_event(&payload).is_none());
    }
}
