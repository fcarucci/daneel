# Daneel — Agent Chat Primitive Design

**Date:** 2026-03-15  
**Status:** Proposed  
**Scope:** First interactive primitive for sending a chat turn to an OpenClaw agent and receiving streamed responses in Daneel

---

## 1. Goal

Daneel needs a minimal but real operator-facing primitive for asking an OpenClaw agent a question and receiving its response.

This document defines the first version of that primitive:

- send one chat turn to a chosen agent
- stream assistant output back to the browser
- expose final success or failure state
- keep the browser isolated from direct gateway access
- align with documented and verified OpenClaw gateway behavior

This is intentionally narrower than a full chat product. It does not attempt to solve:

- rich transcript browsing
- attachments
- tool event visualization
- multi-session chat UX
- abort / retry controls
- persisted local Daneel chat state beyond what the gateway already stores

---

## 2. What We Learned

The design below is based on verified local behavior against the installed OpenClaw runtime on March 15, 2026.

### 2.1 Gateway methods

The relevant documented gateway methods are:

- `connect`
- `chat.send`
- `chat.history`
- `chat.abort`

The relevant live event is:

- `chat`

The `chat` event payload includes:

- `runId`
- `sessionKey`
- `seq`
- `state`: `delta | final | aborted | error`
- `message`
- `errorMessage`
- optional usage / stop metadata

### 2.2 `chat.send` shape

The verified required request shape for `chat.send` is:

- `sessionKey`
- `message`
- `idempotencyKey`

Useful optional fields exist in OpenClaw, but the first Daneel primitive should ignore them unless needed later.

### 2.3 Auth and scopes

This was the most important implementation finding.

OpenClaw chat writes are not available through Daneel's current read-only probe posture.

Verified behavior:

- a plain gateway token path was sufficient for read-only methods like `status`
- a raw websocket client using shared token auth hit `missing scope: operator.write` when calling `chat.send`
- a working chat websocket path required using OpenClaw's paired device identity flow, via the official JS gateway client

Implication:

Daneel cannot implement agent chat correctly by reusing its current `operator.read` probe connection shape from [src/gateway.rs](/root/dev/daneel/src/gateway.rs).

The server-side gateway client used for chat must authenticate with write-capable operator scopes, and likely needs to participate in the paired device identity model rather than only presenting the shared gateway token.

### 2.3.1 Rust feasibility

Nothing learned here requires JavaScript as a runtime dependency for Daneel.

The successful local verification used the official OpenClaw JS client because it already implemented the full handshake, but the underlying protocol details are simple enough to reproduce in Rust:

- websocket transport
- JSON request / response / event envelopes
- UUID request ids and idempotency keys
- Ed25519 signature generation for device auth payloads
- base64url encoding for public keys and signatures
- in-memory correlation of `chat.send` responses and later `chat` events

The Rust implementation path is realistic because Daneel already uses:

- `tokio`
- `tokio-tungstenite`
- `serde`
- `serde_json`

The remaining pieces are normal Rust building blocks, not JS-only behavior.

Recommended additional Rust crates when chat is implemented:

- `uuid` for request ids and idempotency keys
- `base64` for URL-safe base64 encoding
- `ed25519-dalek` for Ed25519 signing

Optional alternatives are also fine if the team prefers different crates, but the protocol itself is straightforward to implement in Rust.

### 2.3.2 Is Rust the right implementation language?

Yes.

This primitive can be implemented cleanly in Rust.

It only becomes messy if Daneel tries to extend the current ad hoc gateway probing code directly into a full interactive chat stack without first introducing a proper client / adapter layer.

Rust is a good fit for this work because the problem is mostly:

- websocket protocol handling
- typed JSON serialization
- async task coordination
- event fanout
- cryptographic signing

Those are all normal Rust backend concerns.

What would make the implementation messy:

- continuing to add one-off websocket logic to `src/gateway.rs`
- mixing read-only probe flows and write-capable chat flows in the same code path
- shelling out to the OpenClaw JS CLI from Rust instead of implementing the protocol natively
- leaking raw gateway frame shapes into Dioxus UI code

What keeps the implementation clean:

- a dedicated Rust OpenClaw gateway client module
- typed request / response / event models in Rust
- separate auth setup for read-only status and write-capable chat
- server-mediated browser updates through SSE
- a thin, explicit adapter boundary between UI-facing services and gateway protocol details

So the correct conclusion is:

- Rust is viable
- Rust is the preferred production implementation language for Daneel
- the risk is architectural mess, not language mismatch

### 2.4 Browser transport

The browser must not connect directly to the OpenClaw gateway.

This remains aligned with the existing technical design:

Browser -> Daneel server -> OpenClaw gateway

Because chat is bidirectional and stateful, this primitive should not use a server function alone. It needs:

- one request path to start the run
- one live stream path to deliver chat events

---

## 3. Product Shape

The first Daneel chat primitive should feel like an operator tool, not a consumer messenger.

Minimal operator story:

1. operator chooses an agent
2. operator enters a prompt
3. Daneel starts a gateway-backed run
4. Daneel shows streaming assistant output
5. Daneel shows final success or error state

First supported target example:

- ask `calendar` for this week's appointments

This is enough to prove that Daneel can move from passive monitoring to active gateway-backed operations.

---

## 4. Non-Goals

The first version should not include:

- direct browser-to-gateway websocket traffic
- local transcript persistence in Daneel
- custom session orchestration beyond choosing a session key
- markdown rendering complexity beyond plain text display
- upload flows
- approval UI for tool execution
- parallel multi-run management in one chat panel

Those can come later once the primitive is stable.

---

## 5. Proposed Architecture

### 5.1 High-level flow

```text
Browser
  -> POST / server function: start_agent_chat()
  <- returns accepted state, session key, run id placeholder if available

Browser
  -> subscribe to Daneel live stream for chat events

Daneel server
  -> establishes gateway client with write-capable operator auth
  -> sends chat.send
  -> listens for gateway chat events
  -> forwards normalized chat events to browser
```

### 5.2 Why not use only server functions

Server functions are good for:

- one-shot reads
- simple mutations with immediate final responses

They are not enough for this primitive because:

- `chat.send` is accepted first and completes asynchronously
- response text arrives as event frames
- the UI needs partial output and final state transitions

### 5.3 Why not let the browser talk to OpenClaw directly

That would break Daneel's intended trust boundary and duplicate gateway auth logic in the browser.

The server should remain responsible for:

- gateway credentials and device identity
- scope management
- event subscription
- translation from gateway protocol into UI-safe app events

---

## 6. Gateway Integration Design

### 6.1 New server-side gateway client layer

Daneel should stop embedding ad hoc websocket JSON logic directly inside feature functions once chat is introduced.

The current code in [src/gateway.rs](/root/dev/daneel/src/gateway.rs) is acceptable for the initial health probe, but chat needs a reusable server-side client abstraction.

Recommended new modules:

- `src/adapters/mod.rs`
- `src/adapters/traits.rs`
- `src/adapters/openclaw/mod.rs`
- `src/adapters/openclaw/client.rs`
- `src/adapters/openclaw/chat.rs`
- `src/models/chat.rs`

### 6.2 OpenClaw client responsibilities

The OpenClaw gateway client should:

- load Daneel/OpenClaw gateway configuration
- establish authenticated gateway connection
- request write-capable operator scopes for chat
- own websocket lifecycle
- send typed gateway requests
- parse typed gateway responses and events
- expose chat-specific subscriptions in Rust-native types

Rust-specific implementation note:

This client should be implemented as a native Rust service, not as a subprocess wrapper around the OpenClaw JS CLI. The JS test was useful for protocol verification, but the production Daneel primitive should keep all gateway logic inside the Rust server process.

### 6.3 Authentication requirement

This client must not copy Daneel's current probe connect shape.

Current Daneel status code uses:

- role `operator`
- scopes `["operator.read"]`
- probe mode

That is insufficient for chat.

For chat, the client should request at least:

- `operator.write`

In practice, the local verification strongly suggests that a robust implementation should use the same device-identity-backed operator authentication model that the official OpenClaw client uses, not just the shared gateway token.

Design implication:

- Daneel chat should initially be a server-only capability
- if write-capable device auth is unavailable, the UI should render a clear degraded state rather than silently failing

Rust-specific implementation note:

The paired device identity flow can be implemented in Rust by reading the same local OpenClaw identity files and reproducing the signed payload that the gateway verifies:

- `~/.openclaw/identity/device.json`
- optionally `~/.openclaw/identity/device-auth.json`

The verified payload ingredients are:

- device id
- client id
- client mode
- role
- scopes
- signed timestamp
- signature token when present
- nonce from `connect.challenge`
- platform
- device family

This is a protocol concern, not a JS runtime concern.

### 6.4 Session key strategy

OpenClaw `chat.send` operates on a `sessionKey`.

For the first primitive, Daneel should use a simple deterministic mapping:

- explicit session key if provided by the UI
- otherwise default to `agent:{agent_id}:main`

This is enough for:

- asking `calendar`
- asking `planner`
- asking `main`

Later versions can add:

- session discovery
- existing session pickers
- transient session creation strategies

---

## 7. Daneel Server API

### 7.1 Start chat server function

Add a new server function:

```rust
#[server]
pub async fn start_agent_chat(request: StartAgentChatRequest) -> Result<StartAgentChatAccepted, ServerFnError>
```

Suggested request:

```rust
pub struct StartAgentChatRequest {
    pub agent_id: String,
    pub session_key: Option<String>,
    pub message: String,
}
```

Suggested accepted response:

```rust
pub struct StartAgentChatAccepted {
    pub session_key: String,
    pub client_request_id: String,
}
```

Notes:

- this function should validate input and enqueue / start the run
- it should not wait for the final assistant response
- `client_request_id` is Daneel's own correlation id for UI state

### 7.2 Live event endpoint

Add a dedicated live endpoint for chat events.

Recommended first transport:

- SSE, following the current pattern in [src/live.rs](/root/dev/daneel/src/live.rs)

Recommended route:

- `/api/chat/events`

Recommended event scope:

- per connected browser session, filtered to only that browser's active chat runs

### 7.3 Why SSE first

For the first version, SSE is simpler than a Daneel browser websocket because:

- server-to-browser flow is one-way
- Daneel already has an SSE pattern
- the bidirectional stateful websocket is already between Daneel server and OpenClaw gateway

If Daneel later adds browser-originated aborts, presence, typing, or multi-run interaction, a dedicated browser websocket can be reconsidered.

---

## 8. Event Model

Introduce app-level chat models in `src/models/chat.rs`.

### 8.1 Request / accepted models

```rust
pub struct StartAgentChatRequest {
    pub agent_id: String,
    pub session_key: Option<String>,
    pub message: String,
}

pub struct StartAgentChatAccepted {
    pub client_request_id: String,
    pub session_key: String,
}
```

### 8.2 Stream event model

```rust
pub enum ChatStreamState {
    Started,
    Delta,
    Final,
    Error,
    Aborted,
}

pub struct ChatStreamEvent {
    pub client_request_id: String,
    pub run_id: Option<String>,
    pub agent_id: String,
    pub session_key: String,
    pub state: ChatStreamState,
    pub text: Option<String>,
    pub error: Option<String>,
}
```

### 8.3 Normalization rule

Daneel should normalize OpenClaw gateway `chat` events into these app-level events.

The browser should not need to know gateway frame details like:

- `seq`
- raw message content array shape
- gateway error envelopes

---

## 9. Server Runtime Design

### 9.1 Chat hub

Create a chat event hub similar to the current live gateway hub.

Suggested component:

- `ChatEventHub`

Responsibilities:

- publish normalized `ChatStreamEvent`
- let SSE clients subscribe
- keep minimal latest state for active runs if needed

### 9.2 Run registry

The server needs a small in-memory registry for active chat runs.

Suggested structure:

- map `client_request_id -> ActiveChatRun`

Where `ActiveChatRun` includes:

- `agent_id`
- `session_key`
- optional `run_id`
- accumulated text buffer
- created time
- terminal state flag

This registry is not meant to become durable storage. It only supports:

- correlation between start request and later gateway events
- streaming UI updates
- cleanup after terminal events

### 9.3 Concurrency model

The simplest first implementation is:

- one dedicated server task per started run

That task:

1. opens gateway client
2. starts `chat.send`
3. listens for matching `chat` events
4. publishes normalized updates
5. exits on `final | error | aborted | timeout`

This is operationally simple and appropriate for a primitive.

Later optimization:

- shared long-lived gateway chat multiplexer per Daneel process

That is not required for the first implementation.

---

## 10. Browser UI Design

### 10.1 First UI surface

The first UI should be very small and explicit.

Recommended location:

- a compact operator card on the agents page
- or a dedicated experimental section on the dashboard

Minimal controls:

- agent selector
- text area
- send button
- output panel

### 10.2 UI state machine

The browser should model:

- `idle`
- `sending`
- `streaming`
- `complete`
- `error`

### 10.3 Rendering behavior

On send:

1. call `start_agent_chat`
2. subscribe or attach to `/api/chat/events`
3. filter events by `client_request_id`
4. append / replace displayed text as events arrive
5. mark terminal state on `Final`, `Error`, or `Aborted`

### 10.4 Transcript handling

For the primitive, the UI should display only the current operator prompt and the latest assistant output.

Do not implement a full transcript viewer yet.

If transcript browsing is needed later, use gateway-backed `chat.history` rather than inventing a local transcript store in Daneel.

---

## 11. Error Handling

### 11.1 Expected failure modes

The first implementation should explicitly handle:

- gateway config missing
- gateway unreachable
- auth rejected
- write scope unavailable
- session key invalid
- agent runtime error
- stream timeout
- gateway socket close before terminal event

### 11.2 Operator-facing messaging

Errors should be translated into actionable messages.

Examples:

- "Daneel is connected to the gateway in read-only mode; agent chat requires write-capable operator auth."
- "The target session key could not be resolved for agent `calendar`."
- "The gateway accepted the chat request but no final response arrived before timeout."

### 11.3 Logging

Server logs should capture:

- client request id
- session key
- agent id
- gateway run id when known
- terminal outcome

Avoid logging full sensitive prompts by default.

---

## 12. Security Considerations

### 12.1 Browser isolation

The browser must never receive:

- raw gateway token
- device private key
- device token

### 12.2 Scope minimization

Daneel chat should request only the scopes needed for this feature.

If a write-capable operator identity is not available, the feature should be disabled.

### 12.3 Future trust boundary

If Daneel later supports multiple users or device-level trust, the chat primitive should remain server-mediated so authorization decisions stay centralized.

---

## 13. Testing Strategy

### 13.1 Unit tests

Add tests for:

- session key resolution
- gateway chat event normalization
- final / error / aborted state mapping
- SSE serialization of `ChatStreamEvent`

### 13.2 Mock gateway integration test

Extend the existing mock gateway harness in `tests/support/mod.rs` to support:

- `chat.send`
- emitted `chat` events with `delta` and `final`

Then add an integration test that:

1. starts Daneel with the mock gateway
2. invokes `start_agent_chat`
3. subscribes to `/api/chat/events`
4. verifies streamed delta and final payloads

Also add Rust unit tests for device-auth helpers:

- public-key normalization
- payload construction
- signature generation
- connect request serialization

### 13.3 Live verification

Manual verification should include:

- asking `calendar` for this week's appointments
- verifying final response text arrives in the browser
- verifying degraded auth message when write scope is unavailable

---

## 14. Implementation Plan

### Phase 1: Core server models and adapter

- add `src/models/chat.rs`
- add typed request / accepted / stream event structs
- add initial OpenClaw chat client abstraction

### Phase 2: Start request and event stream

- add `start_agent_chat` server function
- add `ChatEventHub`
- add `/api/chat/events` SSE endpoint

### Phase 3: Mock-gateway coverage

- extend mock gateway test server with `chat.send` and `chat` event emission
- add integration test for end-to-end streaming

### Phase 4: Minimal UI

- add small operator prompt form
- add streaming output panel
- add clear loading and error states

### Phase 5: Hardening

- add cleanup for stale runs
- add timeout policy
- improve operator-facing degraded/auth states

---

## 15. Open Questions

These questions do not block the primitive, but should be resolved during implementation:

1. What is the clean Rust-native way for Daneel to participate in OpenClaw's paired device identity model?
2. Should Daneel own its own dedicated paired device identity, separate from the local CLI identity?
3. Should the first browser stream be global SSE with client-side filtering, or per-run scoped SSE subscriptions?
4. What timeout is acceptable for an operator chat turn before surfacing a terminal timeout?
5. Should the first primitive expose only final text, or should it render partial deltas live?

Recommended default answers for V1:

- use a dedicated server-side Rust OpenClaw client abstraction
- initially allow reuse of the host's trusted operator device identity if that is the only practical path
- use SSE
- render deltas live
- use a conservative timeout with explicit operator messaging

The important boundary here is:

- reuse of the host identity files is acceptable for the first Rust implementation
- dependence on the OpenClaw JS client at runtime is not required and should not be the target design

---

## 16. Recommendation

Daneel should implement agent chat as:

- a server-mediated OpenClaw gateway client with write-capable auth
- a `start_agent_chat` server function
- an SSE stream of normalized chat events
- a minimal operator UI for one active run

This approach fits the current architecture, aligns with the verified OpenClaw behavior, avoids exposing gateway auth to the browser, and can be implemented entirely in Rust using normal websocket, serde, UUID, base64url, and Ed25519 building blocks. It also creates a strong foundation for later features like transcript history, abort, and richer session management.
