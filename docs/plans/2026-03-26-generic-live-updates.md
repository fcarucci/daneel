# Generic Live Gateway Updates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` (recommended) or `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a generic, extensible live update system that normalizes arbitrary gateway push events into typed Daneel events and delivers them to the browser UI via SSE, starting with agent state updates and generalizing to any gateway object.

**Architecture:** Extend the existing live bridge WebSocket loop to parse all gateway event types (not just `health`) into a discriminated `GatewayEvent` enum. Replace the single-type `LiveEventHub` with a multi-event hub that broadcasts typed normalized events. The browser subscribes to a single SSE stream carrying all event types and dispatches them into the appropriate Dioxus signal/context. The system is designed so adding a new gateway object type requires: (1) a new variant on the event enum, (2) a parser function, (3) a UI consumer — no plumbing changes.

**Tech Stack:** Rust, Dioxus 0.7.3 fullstack, Axum SSE, tokio broadcast channels, serde for JSON serialization, web-sys EventSource on the browser side.

---

## Architecture Overview

```
  OpenClaw Gateway (WebSocket)
         │
         │  { type: "event", event: "health"|"agent"|..., payload: {...} }
         │  { type: "res", id: "...", ok: true, payload: {...} }
         ▼
  ┌─────────────────────────────────────┐
  │  Live Bridge (persistent WS conn)   │
  │  stream_gateway_events()            │
  │                                     │
  │  parse_gateway_frame() ─────────┐   │
  │    ├── parse_health_push()      │   │
  │    ├── parse_agent_push()   NEW │   │
  │    └── (future event types)     │   │
  │                                 ▼   │
  │  LiveEventHub::publish(GatewayEvent)│
  └──────────────┬──────────────────────┘
                 │
                 │  broadcast::Sender<GatewayEvent>
                 ▼
  ┌──────────────────────────────────┐
  │  SSE Endpoint /api/gateway/events│
  │  Serializes GatewayEvent as JSON │
  │  (unnamed messages, type in JSON)│
  └──────────────┬───────────────────┘
                 │
                 │  SSE stream (plain JSON)
                 ▼
  ┌──────────────────────────────────────┐
  │  Browser (EventSource.onmessage)     │
  │                                      │
  │  LiveGatewayProvider (expanded)      │
  │    ├── live_status: Signal (health)  │
  │    ├── agent_updates: Signal<Map>    │
  │    ├── backend_state: Signal         │
  │    └── (future: sessions, cron, ...) │
  │                                      │
  │  Components read from context        │
  └──────────────────────────────────────┘
```

## File Structure

### New files

| File | Responsibility |
|------|---------------|
| `src/models/live_events.rs` | `GatewayEvent` enum, per-variant payload structs, serialization |
| `src/live/event_parser.rs` | Gateway JSON frame -> `GatewayEvent` parsing (all event types) |
| `src/live/hub.rs` | `LiveEventHub` (extracted from current `live.rs`, generalized for `GatewayEvent`) |
| `src/live/bridge.rs` | WebSocket bridge loop (extracted from current `live.rs`) |
| `src/live/sse.rs` | SSE endpoint and Axum router (extracted from current `live.rs`) |
| `src/live/mod.rs` | Module wiring, re-exports of `init_live_hub`, `router`, `run_gateway_event_bridge` |

### Modified files

| File | Change |
|------|--------|
| `src/models/mod.rs` | Add `pub mod live_events;` |
| `src/live.rs` | Replace with `src/live/mod.rs` (split into submodules) |
| `src/components/live_gateway.rs` | Expand to dispatch `GatewayEvent`, add `agent_updates` signal |
| `src/pages/agents.rs` | Read agent updates from `LiveGatewayState` for live tile health indicators |
| `src/main.rs` | Update `mod live` to point at the new module directory |
| `tests/support/gateway.rs` | Add mock agent event pushes |

### Unchanged files

| File | Why unchanged |
|------|--------------|
| `src/adapter/mod.rs` | The adapter trait is for request-response; live events flow through the bridge, not the adapter |
| `src/gateway/ws.rs` | Envelope parsing stays the same |
| `src/gateway/agents.rs` | One-shot agent overview fetch remains for initial data load |
| `src/models/live_gateway.rs` | `LiveGatewayEvent`, `LiveGatewayLevel`, connection state types all stay as-is |
| `src/pages/dashboard.rs` | Continues using `live_status` from `LiveGatewayState` (unchanged signal) |

---

## Task 1: Server-Side Event System

**Files:**
- Create: `src/models/live_events.rs`
- Modify: `src/models/mod.rs`
- Create: `src/live/mod.rs`, `src/live/hub.rs`, `src/live/event_parser.rs`, `src/live/bridge.rs`, `src/live/sse.rs`
- Delete: `src/live.rs` (replaced by `src/live/` directory)

This task builds the entire server-side pipeline: event types, hub, parser, bridge, and SSE. All server-only code with no browser-side changes.

### Step 1: Create the event model

- [ ] **Create `src/models/live_events.rs`**

```rust
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Normalized gateway event envelope. Every event flowing through the live
/// bridge, the SSE stream, and into browser state is one of these variants.
///
/// To add a new gateway object type:
/// 1. Add a variant here with its payload struct.
/// 2. Add a parser in `src/live/event_parser.rs`.
/// 3. Add a consumer in the appropriate UI component.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayEvent {
    Health(HealthUpdate),
    Agent(AgentUpdate),
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthUpdate {
    pub level: HealthLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Connecting,
    Disconnected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentUpdate {
    pub agent_id: String,
    pub health_state: String,
}
```

Tests at the bottom of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_event_serializes_with_type_tag() {
        let event = GatewayEvent::Health(HealthUpdate {
            level: HealthLevel::Healthy,
            summary: "Gateway healthy.".to_string(),
        });
        let json = serde_json::to_value(&event).expect("serialize");
        assert_eq!(json["type"], "health");
        assert_eq!(json["level"], "healthy");
    }

    #[test]
    fn agent_event_serializes_with_type_tag() {
        let event = GatewayEvent::Agent(AgentUpdate {
            agent_id: "email".to_string(),
            health_state: "degraded".to_string(),
        });
        let json = serde_json::to_value(&event).expect("serialize");
        assert_eq!(json["type"], "agent");
        assert_eq!(json["agent_id"], "email");
    }

    #[test]
    fn gateway_event_round_trips_through_json() {
        let json = r#"{"type":"health","level":"degraded","summary":"Gateway degraded."}"#;
        let event: GatewayEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, GatewayEvent::Health(h) if h.level == HealthLevel::Degraded));
    }
}
```

- [ ] **Wire the module** in `src/models/mod.rs`:

```rust
pub mod live_events;
```

### Step 2: Create the generalized hub

- [ ] **Create `src/live/hub.rs`**

```rust
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use crate::models::live_events::GatewayEvent;

#[cfg(feature = "server")]
use std::sync::{Arc, RwLock};
#[cfg(feature = "server")]
use tokio::sync::broadcast;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct LiveEventHub {
    tx: broadcast::Sender<GatewayEvent>,
    latest: Arc<RwLock<Option<GatewayEvent>>>,
}

#[cfg(feature = "server")]
impl LiveEventHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            tx,
            latest: Arc::new(RwLock::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: GatewayEvent) {
        *self.latest.write().expect("lock live event hub") = Some(event.clone());
        let _ = self.tx.send(event);
    }

    pub fn latest(&self) -> Option<GatewayEvent> {
        self.latest.read().expect("lock live event hub").clone()
    }
}

#[cfg(feature = "server")]
static LIVE_HUB: std::sync::OnceLock<LiveEventHub> = std::sync::OnceLock::new();

#[cfg(feature = "server")]
pub fn init_live_hub() -> LiveEventHub {
    LIVE_HUB.get_or_init(LiveEventHub::new).clone()
}

#[cfg(feature = "server")]
pub fn require_live_hub() -> &'static LiveEventHub {
    LIVE_HUB
        .get()
        .expect("LiveEventHub must be initialized before serving SSE.")
}
```

Tests:

```rust
#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::models::live_events::{GatewayEvent, HealthLevel, HealthUpdate};

    #[test]
    fn publish_stores_latest_event() {
        let hub = LiveEventHub::new();
        let event = GatewayEvent::Health(HealthUpdate {
            level: HealthLevel::Healthy,
            summary: "ok".to_string(),
        });
        hub.publish(event);
        let latest = hub.latest().expect("should have latest");
        assert!(matches!(latest, GatewayEvent::Health(_)));
    }

    #[tokio::test]
    async fn subscriber_receives_published_event() {
        let hub = LiveEventHub::new();
        let mut rx = hub.subscribe();
        let event = GatewayEvent::Health(HealthUpdate {
            level: HealthLevel::Degraded,
            summary: "degraded".to_string(),
        });
        hub.publish(event);
        let received = rx.recv().await.expect("receive event");
        assert!(matches!(received, GatewayEvent::Health(h) if h.level == HealthLevel::Degraded));
    }
}
```

### Step 3: Create the event parser

- [ ] **Create `src/live/event_parser.rs`**

```rust
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "server")]
use crate::models::live_events::{
    AgentUpdate, GatewayEvent, HealthLevel, HealthUpdate,
};
#[cfg(feature = "server")]
use serde_json::Value;

#[cfg(feature = "server")]
const HEALTH_LIVE_REQUEST_ID: &str = "health-live-1";

#[cfg(feature = "server")]
pub fn parse_gateway_frame(frame: &Value) -> Option<GatewayEvent> {
    let kind = frame.get("type").and_then(Value::as_str)?;
    match kind {
        "event" => parse_push_event(frame),
        "res" => parse_response_event(frame),
        _ => None,
    }
}

#[cfg(feature = "server")]
fn parse_push_event(frame: &Value) -> Option<GatewayEvent> {
    let event_name = frame.get("event").and_then(Value::as_str)?;
    match event_name {
        "health" => parse_health_push(frame),
        "agent" => parse_agent_push(frame),
        _ => None,
    }
}

#[cfg(feature = "server")]
fn parse_response_event(frame: &Value) -> Option<GatewayEvent> {
    let id = frame.get("id").and_then(Value::as_str)?;
    if id != HEALTH_LIVE_REQUEST_ID {
        return None;
    }
    if frame.get("ok").and_then(Value::as_bool) == Some(false) {
        return Some(GatewayEvent::Health(HealthUpdate {
            level: HealthLevel::Degraded,
            summary: frame
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Gateway health request failed.")
                .to_string(),
        }));
    }
    extract_health_status_string(frame, &HEALTH_RESPONSE_PATHS)
        .map(|status| health_update_from_status(&status))
}

#[cfg(feature = "server")]
fn parse_health_push(frame: &Value) -> Option<GatewayEvent> {
    extract_health_status_string(frame, &HEALTH_EVENT_PATHS)
        .map(|status| health_update_from_status(&status))
}

#[cfg(feature = "server")]
fn parse_agent_push(frame: &Value) -> Option<GatewayEvent> {
    let payload = frame.get("payload")?;
    let agent_id = payload.get("agentId").and_then(Value::as_str)?;

    let health_state = payload
        .pointer("/status/health/state")
        .or_else(|| payload.pointer("/status"))
        .and_then(Value::as_str)?;

    Some(GatewayEvent::Agent(AgentUpdate {
        agent_id: agent_id.to_string(),
        health_state: health_state.to_string(),
    }))
}

#[cfg(feature = "server")]
fn health_update_from_status(status: &str) -> GatewayEvent {
    GatewayEvent::Health(HealthUpdate {
        level: if status.eq_ignore_ascii_case("healthy") {
            HealthLevel::Healthy
        } else {
            HealthLevel::Degraded
        },
        summary: format!("Gateway health update: {status}."),
    })
}

#[cfg(feature = "server")]
const HEALTH_EVENT_PATHS: [&[&str]; 4] = [
    &["payload", "status", "health", "state"],
    &["payload", "status"],
    &["payload", "health"],
    &["payload", "state"],
];

#[cfg(feature = "server")]
const HEALTH_RESPONSE_PATHS: [&[&str]; 3] = [
    &["payload", "status"],
    &["payload", "health"],
    &["payload", "state"],
];

#[cfg(feature = "server")]
fn extract_health_status_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    paths.iter().find_map(|path| {
        let mut current = value;
        for segment in *path {
            current = current.get(*segment)?;
        }
        current.as_str().map(ToOwned::to_owned)
    })
}
```

Tests:

```rust
#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_health_event() {
        let frame = json!({
            "type": "event",
            "event": "health",
            "payload": {
                "status": { "health": { "state": "healthy" } }
            }
        });
        let event = parse_gateway_frame(&frame).expect("parse health");
        assert!(matches!(event, GatewayEvent::Health(h) if h.level == HealthLevel::Healthy));
    }

    #[test]
    fn parses_agent_event_with_health_state() {
        let frame = json!({
            "type": "event",
            "event": "agent",
            "payload": {
                "agentId": "email",
                "status": { "health": { "state": "degraded" } }
            }
        });
        let event = parse_gateway_frame(&frame).expect("parse agent");
        match event {
            GatewayEvent::Agent(update) => {
                assert_eq!(update.agent_id, "email");
                assert_eq!(update.health_state, "degraded");
            }
            _ => panic!("expected Agent event"),
        }
    }

    #[test]
    fn parses_health_response_matching_live_id() {
        let frame = json!({
            "type": "res",
            "id": "health-live-1",
            "ok": true,
            "payload": { "status": "healthy" }
        });
        let event = parse_gateway_frame(&frame).expect("parse health response");
        assert!(matches!(event, GatewayEvent::Health(_)));
    }

    #[test]
    fn ignores_connect_response() {
        let frame = json!({
            "type": "res",
            "id": "connect-live-1",
            "ok": true,
            "payload": { "protocolVersion": 3 }
        });
        assert!(parse_gateway_frame(&frame).is_none());
    }

    #[test]
    fn ignores_unknown_event_type() {
        let frame = json!({
            "type": "event",
            "event": "device_trust",
            "payload": { "deviceId": "abc" }
        });
        assert!(parse_gateway_frame(&frame).is_none());
    }

    #[test]
    fn agent_event_without_agent_id_returns_none() {
        let frame = json!({
            "type": "event",
            "event": "agent",
            "payload": {
                "status": { "health": { "state": "degraded" } }
            }
        });
        assert!(parse_gateway_frame(&frame).is_none());
    }
}
```

### Step 4: Extract bridge and SSE, replace `src/live.rs`

- [ ] **Create `src/live/bridge.rs`**

Move `stream_gateway_events`, `run_gateway_event_bridge`, `connect_live_request`, `health_live_request`, and `send_gateway_request` from the current `src/live.rs`. Replace the `parse_gateway_event` call with `event_parser::parse_gateway_frame`. Replace `LiveGatewayEvent` publishing with `GatewayEvent` publishing.

The key change in the event loop:

```rust
// Before (in old live.rs):
if let Some(event) = parse_gateway_event(&payload) {
    hub.publish(event);  // LiveGatewayEvent
}

// After (in bridge.rs):
if let Some(event) = event_parser::parse_gateway_frame(&payload) {
    hub.publish(event);  // GatewayEvent
}
```

Update the connecting/disconnected/unavailable helper events to produce `GatewayEvent::Health(HealthUpdate { ... })` instead of `LiveGatewayEvent`:

```rust
fn connecting_event(summary: &str, detail: String) -> GatewayEvent {
    GatewayEvent::Health(HealthUpdate {
        level: HealthLevel::Connecting,
        summary: summary.to_string(),
    })
}

fn disconnected_event(detail: String) -> GatewayEvent {
    GatewayEvent::Health(HealthUpdate {
        level: HealthLevel::Connecting,
        summary: "Gateway event stream reconnecting".to_string(),
    })
}

fn unavailable_event(detail: String) -> GatewayEvent {
    GatewayEvent::Health(HealthUpdate {
        level: HealthLevel::Degraded,
        summary: "Gateway event stream unavailable".to_string(),
    })
}
```

- [ ] **Create `src/live/sse.rs`**

Move the SSE endpoint and Axum router. Update it to serialize `GatewayEvent` (which derives `Serialize`). Keep SSE messages unnamed — the `"type"` tag in the JSON payload is the discriminator:

```rust
fn event_to_sse(event: &GatewayEvent) -> Option<Result<Event, Infallible>> {
    if matches!(event, GatewayEvent::Unknown) {
        return None;
    }
    serde_json::to_string(event)
        .ok()
        .map(|data| Ok(Event::default().data(data)))
}
```

The SSE catch-up for newly connecting subscribers uses `hub.latest()`:

```rust
async fn sse_gateway_events() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
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
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
```

- [ ] **Create `src/live/mod.rs`**

```rust
// SPDX-License-Identifier: Apache-2.0

pub mod event_parser;
pub mod hub;

#[cfg(feature = "server")]
pub mod bridge;
#[cfg(feature = "server")]
pub mod sse;

#[cfg(feature = "server")]
pub use hub::{init_live_hub, require_live_hub};
#[cfg(feature = "server")]
pub use bridge::run_gateway_event_bridge;
#[cfg(feature = "server")]
pub use sse::router;
```

- [ ] **Delete `src/live.rs`** and update imports in `src/main.rs`. The public API stays the same: `init_live_hub()`, `run_gateway_event_bridge()`, `router()`.

### Step 5: Verify and commit

- [ ] **Run full verification**

```bash
cargo fmt --all
cargo check
cargo check --features server
cargo test --features server
cargo test
```

All existing tests must pass. The old `live::tests` module tests (which tested health parsing) should be migrated to `live::event_parser::tests`.

- [ ] **Commit**

```bash
git add -A src/models/live_events.rs src/models/mod.rs src/live/
git rm src/live.rs
git commit -m "feat(live): generic event system with GatewayEvent enum, parser, and split live modules"
```

---

## Task 2: Browser Integration

**Files:**
- Modify: `src/components/live_gateway.rs`

Expand the existing `LiveGatewayProvider` to handle `GatewayEvent` dispatch. Add an `agent_updates` signal alongside the existing health signals. No new provider, no new EventSource, no migration.

- [ ] **Step 1: Add `agent_updates` to `LiveGatewayState`**

In `src/components/live_gateway.rs`, expand the struct:

```rust
use std::collections::HashMap;
use crate::models::live_events::{AgentUpdate, GatewayEvent};

#[derive(Clone, Copy)]
pub(crate) struct LiveGatewayState {
    pub live_status: Signal<Option<LiveGatewayEvent>>,
    pub backend_state: Signal<BackendConnectionState>,
    pub agent_updates: Signal<HashMap<String, AgentUpdate>>,
}
```

Add an accessor:

```rust
impl LiveGatewayState {
    // ... existing methods ...

    pub fn agent_health_state(&self, agent_id: &str) -> Option<String> {
        (self.agent_updates)()
            .get(agent_id)
            .map(|update| update.health_state.clone())
    }
}
```

- [ ] **Step 2: Update `use_live_gateway_state` to initialize the new signal**

```rust
fn use_live_gateway_state() -> LiveGatewayState {
    let live_status = use_signal(|| None::<LiveGatewayEvent>);
    let backend_state = use_signal(initial_backend_connection_state);
    let agent_updates = use_signal(HashMap::new);
    // ... existing EventSource wiring ...

    LiveGatewayState {
        live_status,
        backend_state,
        agent_updates,
    }
}
```

- [ ] **Step 3: Update the WASM `onmessage` handler to dispatch `GatewayEvent`**

In the `attach_live_gateway_listener` function, change the `onmessage` closure to deserialize as `GatewayEvent` and dispatch by variant:

```rust
let onmessage = Closure::<dyn FnMut(web_sys::MessageEvent)>::new({
    let mut live_status = live_status.clone();
    let mut backend_state = backend_state.clone();
    let mut agent_updates = agent_updates.clone();
    move |event: web_sys::MessageEvent| {
        let Some(text) = event.data().as_string() else { return };
        let Ok(gateway_event) = serde_json::from_str::<GatewayEvent>(&text) else { return };

        backend_state.set(BackendConnectionState::Connected);

        match gateway_event {
            GatewayEvent::Health(update) => {
                let level = match update.level {
                    HealthLevel::Healthy => LiveGatewayLevel::Healthy,
                    HealthLevel::Degraded => LiveGatewayLevel::Degraded,
                    HealthLevel::Connecting => LiveGatewayLevel::Connecting,
                    HealthLevel::Disconnected => LiveGatewayLevel::Disconnected,
                };
                live_status.set(Some(LiveGatewayEvent {
                    level,
                    summary: update.summary.clone(),
                    detail: update.summary,
                }));
            }
            GatewayEvent::Agent(update) => {
                agent_updates.write().insert(update.agent_id.clone(), update);
            }
            GatewayEvent::Unknown => {}
        }
    }
});
```

The `onopen` and `onerror` handlers stay exactly as they are — they continue setting `backend_state` and `live_status` directly.

- [ ] **Step 4: Verify and commit**

```bash
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

```bash
git add src/components/live_gateway.rs
git commit -m "feat(live): expand LiveGatewayProvider to dispatch agent updates from GatewayEvent"
```

---

## Task 3: Agent Updates End-to-End

**Files:**
- Modify: `tests/support/gateway.rs`
- Modify: `src/pages/agents.rs`

Add agent event pushes to the mock gateway, then wire live health indicators into the agents page.

- [ ] **Step 1: Add agent event push to the mock gateway**

In `tests/support/gateway.rs`, add:

```rust
fn agent_event(agent_id: &str, health_state: &str) -> Value {
    json!({
        "type": "event",
        "event": "agent",
        "payload": {
            "agentId": agent_id,
            "status": {
                "health": {
                    "state": health_state
                }
            }
        }
    })
}
```

In the `handle_gateway_client` connect handler, after pushing the health event, also push an agent event for each agent:

```rust
Some("connect") => {
    let response = connect_response(&request, payload);
    let _ = socket.send(Message::Text(health_event(payload).to_string().into()));
    for agent in &payload.agents {
        let _ = socket.send(Message::Text(
            agent_event(&agent.id, "healthy").to_string().into()
        ));
    }
    response
}
```

- [ ] **Step 2: Wire live health into the agents page**

In `src/pages/agents.rs`, add a helper function:

```rust
fn effective_health_indicator(
    agent: &AgentOverviewItem,
    live_state: &LiveGatewayState,
) -> Option<String> {
    live_state.agent_health_state(&agent.id)
}
```

In the `AgentCard` component, read from `use_live_gateway()` and render a health indicator when a live update is available:

```rust
let live_state = use_live_gateway();
let live_health = effective_health_indicator(&agent, &live_state);
// Render a health indicator dot/badge when live_health is Some
```

- [ ] **Step 3: Run full verification and commit**

```bash
cargo fmt --all
cargo check
cargo check --features server
cargo test
cargo test --test e2e_mock_gateway
```

```bash
git add tests/support/gateway.rs src/pages/agents.rs
git commit -m "feat(agents): live agent health updates from gateway events"
```

---

## Task 4: Refactoring Pass

**Files:** All files touched in Tasks 1-3.

Run the mandatory refactoring skill pass on all touched files.

- [ ] **Step 1: Read and follow `skills/refactoring/SKILL.md`**
- [ ] **Step 2: Apply refactorings to touched files**
- [ ] **Step 3: Run full verification**

```bash
cargo fmt --all
cargo check
cargo check --features server
cargo test
cargo test --test e2e_mock_gateway
```

- [ ] **Step 4: Commit any refactoring changes**

```bash
git add -A
git commit -m "refactor: post-implementation cleanup for generic live updates"
```

---

## Extension Notes

To add a new gateway object type (e.g., sessions): add a variant to `GatewayEvent`, a parser function in `event_parser.rs`, and a signal + match arm in `LiveGatewayState`. No plumbing changes — the hub, bridge, SSE, and EventSource handle new types automatically.
