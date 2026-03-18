# Daneel — Technical Design Document

## 1. Overview

Daneel is a Rust-based mission control UI for OpenClaw.

The system provides a browser-based dashboard for monitoring and operating OpenClaw through documented gateway capabilities such as sessions, agents, presence, devices, cron jobs, configuration, and live events.

Feature priorities should be guided by the most important supported operator workflows demonstrated by the OpenClaw Mission Control project, adapted to the subset of functionality that is currently supported by documented OpenClaw gateway capabilities.

Primary goals:

- simplicity
- single binary deployment
- maintainable Rust codebase
- adapter-based gateway integration
- polished operator experience

---

# 2. System Architecture

High-level architecture:

Browser (WASM UI)
    ↓
Daneel Server
    ↓
Gateway Adapter
    ↓
OpenClaw Gateway

The browser never communicates directly with the gateway.

The backend handles:

- adapter selection and configuration
- gateway communication
- device trust enforcement
- application state
- event streaming

The adapter layer isolates Daneel from OpenClaw-specific protocol details so the same core application can support other "claw" systems in the future.

---

# 3. Technology Stack

## Language

Rust

---

## Frontend

Framework: Dioxus

Version target:

the latest stable Dioxus release that cleanly supports the selected Tailwind workflow

Compiled to:

WASM

Responsibilities:

- UI rendering
- routing
- websocket communication with Daneel
- user interaction
- delivering a modern operator-focused experience with strong hierarchy and fast feedback
- applying theme data through a theming layer that is separate from component logic

---

## Backend

Dioxus fullstack server.

Responsibilities:

- API endpoints for the UI
- websocket server for live state updates
- gateway adapter orchestration
- device trust enforcement

---

## Styling

Tailwind CSS 4.x

Used for:

- layout
- component styling
- themes
- motion and visual polish

Theme architecture requirement:

- theme data such as colors, fonts, spacing tokens, radii, shadows, and motion values must be defined separately from component code
- components should consume semantic theme tokens rather than hard-coded visual values
- bright and dark themes should be swappable without requiring component rewrites
- the theme system should support adding future theme variants without changing core UI logic
- the theme system must fit cleanly into Dioxus's context and signal model
- the final visual values should be applied through CSS variables so Dioxus components can remain mostly semantic and declarative

The visual direction should be inspired by Sonars:

- crisp modern typography
- strong visual hierarchy
- premium-feeling panels and surfaces
- smooth, purposeful motion
- high signal density without looking cluttered

The frontend should feel modern and slick rather than purely utilitarian.

Version guidance:

- prefer the latest Tailwind CSS 4.x setup that is documented to work cleanly with the active Dioxus release
- avoid custom Tailwind integration workarounds if the standard Dioxus-supported workflow already covers the need

---

## Storage

Local persistence only.

Preferred:

SQLite (embedded via `libsqlite3-sys`, `rusqlite`, or `sqlx` to maintain single server binary and minimal infrastructure constraints)

Alternative:

JSON file

Stored data:

- Daneel configuration
- selected adapter configuration
- cached device metadata if needed by Daneel
- UI preferences

Authoritative operational state remains in the configured gateway whenever supported by that gateway.

---

# 4. Project Structure

 Example layout:

mission-control/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── router.rs
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── tokens.rs
│   │   ├── registry.rs
│   │   └── semantics.rs
│   ├── models/
│   │   ├── activity.rs
│   │   ├── agent.rs
│   │   ├── cron.rs
│   │   ├── device.rs
│   │   └── session.rs
│   ├── pages/
│   │   ├── activity.rs
│   │   ├── agents.rs
│   │   ├── cron.rs
│   │   ├── dashboard.rs
│   │   ├── devices.rs
│   │   ├── sessions.rs
│   │   └── settings.rs
│   ├── components/
│   │   ├── activity_feed.rs
│   │   ├── activity_filter.rs
│   │   ├── agent_list.rs
│   │   ├── cron_table.rs
│   │   ├── device_table.rs
│   │   ├── navbar.rs
│   │   ├── session_table.rs
│   │   └── sidebar.rs
│   ├── server/
│   │   ├── api.rs
│   │   ├── state.rs
│   │   └── websocket.rs
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   └── openclaw/
│   │       ├── client.rs
│   │       ├── mapper.rs
│   │       ├── events.rs
│   │       └── protocol.rs
│   └── pairing/
│       ├── guard.rs
│       └── trust.rs

---

# 5. Webapp Architecture

Daneel should use the Dioxus single-codebase fullstack model so the browser UI, server-rendered routes, and server communication all live in one Rust application.

Recommended architecture:

- one shared Rust application crate for UI components, routes, shared models, and server function definitions
- Dioxus fullstack server for server functions and live communication endpoints
- adapter implementations and operational services kept in server-focused modules behind stable internal interfaces
- browser UI delivered as a client-rendered Dioxus web application

The browser application should be structured around:

- a typed route enum using Dioxus router
- page components for dashboard, sessions, agents, activity, devices, cron, and settings
- shared presentational components for status cards, tables, timelines, filters, and detail drawers
- shared UI state provided via Dioxus context for layout, theme, and navigation concerns
- a theme layer that resolves semantic UI tokens into the active bright or dark theme values

## 5.1 Live Agent Tile Semantics

The agents view should distinguish between:

- server-derived runtime facts
- client-derived display time

Server-derived runtime facts:

- last observed activity timestamp
- active session counts
- heartbeat configuration or schedule presence

Client-derived display time:

- `time ago` ribbon text such as `4m ago` or `2h ago`
- active or inactive presentation as tiles cross the recent-activity threshold while the page remains open

Rules:

- the displayed recency must continue updating on the client after first render
- the active-state glow must update when the recency ages beyond the recent-activity threshold
- the recent-activity threshold should stay aligned with the current agents-page operator rule
- heartbeat presentation must reflect heartbeat configuration truthfully
- missing heartbeat schedule, `none`, or zero cadence must be treated as heartbeat disabled and render a gray heart

Testing guidance:

- unit tests should isolate recency formatting and active-threshold transitions from wall-clock time by injecting a controllable reference timestamp
- integration tests should verify that the live agents page updates recency text and active-state styling without a full page reload
- integration tests should verify that disabled-heartbeat agent data renders a gray heartbeat icon consistently

Theme separation guidance:

- keep theme definitions in dedicated theme modules or theme data files
- do not hard-code screen-specific colors or fonts inside page components
- express component styling in semantic terms such as `surface-primary`, `text-secondary`, or `state-success`
- let one theme provider or resolver control the active theme selection for the app shell
- use Dioxus context to expose the active theme and theme controls
- use CSS custom properties for actual color/font token application wherever possible

## 5.1 Theme System Design

The theme system should be structured so design data can change independently from UI implementation code.

Recommended responsibilities:

- `theme/tokens.rs`
  - raw theme token definitions
  - color palettes
  - typography families and weights
  - spacing scale
  - radius scale
  - shadow definitions
  - motion timing values

- `theme/semantics.rs`
  - semantic token names consumed by components
  - examples: `surface_primary`, `surface_secondary`, `text_primary`, `text_muted`, `accent_primary`, `state_success`
  - maps raw tokens into UI meaning

- `theme/registry.rs`
  - registration of available themes such as bright and dark
  - theme lookup and selection logic
  - default theme selection

- `theme/mod.rs`
  - public theme exports
  - shared theme types
  - theme provider integration helpers

- `assets/themes.css`
  - CSS custom property definitions for semantic tokens
  - theme selectors such as `[data-theme=\"dark\"]` and `[data-theme=\"bright\"]`
  - typography variable wiring
  - shared motion and surface variable bindings

Component rules:

- components should reference semantic theme values only
- components should not know which exact color, font, or shadow value is active
- reusable components should not branch on "dark theme" or "light theme" when semantic tokens can express the difference
- components should prefer stable class names and semantic CSS variables over assembling many inline style values in Rust

Dioxus implementation guidance:

- provide the active theme through `use_context_provider` using a dedicated theme context type
- keep theme selection reactive through signals stored inside that context
- expose small helpers such as `is_dark`, `theme_name`, or `set_theme(...)` rather than leaking raw token maps throughout the component tree
- avoid passing full theme objects down manually through props when context is more appropriate

App-shell rules:

- theme selection should happen once at the app-shell level
- the active theme should be made available through Dioxus context or an equivalent theme provider
- theme persistence should be handled separately from component rendering logic
- the app shell should set a root attribute such as `data-theme` so CSS variables switch seamlessly without component rewrites

Recommended implementation model in Dioxus:

1. Define semantic theme data in Rust.
2. Provide the active theme and theme controls through Dioxus context.
3. Reflect the active theme at the app root with a `data-theme` attribute.
4. Bind visual values through CSS custom properties loaded from a shared stylesheet.
5. Let components consume semantic classes and variables instead of raw visual constants.

This separation ensures that visual redesigns remain mostly theme-data changes rather than broad component rewrites.

## 5.2 App Shell Layout

The Daneel application shell implements the PRD's strict layout requirements:

- **Top bar:** Provided by `navbar.rs`, responsible for system status, connection state, and theme toggle.
- **Sidebar:** Provided by `sidebar.rs`, handling primary navigation.

This explicit mapping ensures the TDD aligns with the PRD's layout definitions (Section 4.3).

The server application should be structured around:

- server functions for request-response interactions
- an optional live-update endpoint using WebSockets or SSE when needed
- adapter-backed services that gather and normalize data from the configured gateway
- server-side route guards for trusted-device enforcement

---

# 6. UI/Server Communication Model

Daneel should use two communication patterns in the Dioxus fullstack model:

1. Server functions for request-response operations.
2. An optional live-update transport for operational events when polling is insufficient.

Server functions should be the default and preferred mechanism everywhere they are practical.

## 6.1 Server Functions

Dioxus server functions are the default mechanism for UI-to-server calls.

They should be used for:

- loading page data
- fetching detail views
- performing supported mutations such as approving devices or toggling cron jobs
- reading configuration and diagnostics data
- periodic refresh flows when simple polling is sufficient

Implementation guidance:

- define server functions next to the shared UI code that consumes them
- keep function signatures typed with shared request and response structs
- return normalized Daneel models rather than raw gateway payloads
- keep gateway access inside adapter-backed service layers called by the server functions
- prefer composing additional server functions over introducing custom transport-specific command protocols

For page data loading in a client-rendered app:

- use server functions as the primary way to fetch data from UI components
- trigger those calls from Dioxus async hooks and route-aware loading code
- keep responses typed with shared Rust structs

## 6.2 Live Updates

If Daneel needs push-based live updates, it should use a typed live-update channel.

The live-update channel may carry:

- gateway status changes
- activity feed entries
- session updates
- agent updates
- cron job updates
- device trust changes

Implementation guidance:

- prefer SSE or streams when updates are server-to-client only
- use WebSockets only if true bidirectional stateful communication is needed
- define shared message types in Rust for whichever live transport is selected
- connect from the UI with the appropriate Dioxus client utilities
- use reconnection-aware UI state so pages show degraded-but-clear status during disconnects

Request-response reads and writes should default to server functions rather than the live-update transport.

Live transport should be introduced only for cases where server functions plus periodic refresh are clearly insufficient for the desired operator experience.

## 6.3 Recommended Split Of Responsibilities

Use server functions for:

- initial page loads
- detail fetches
- filters and pagination requests
- explicit user actions
- recurring refresh of operational summaries when acceptable

Use a live transport for:

- incremental updates after first render
- connection status
- activity streaming
- push-style refresh triggers

This hybrid model fits Daneel well because it keeps most communication simple while still allowing real-time operator updates when they are worth the added complexity.

---

# 7. Client Rendering And Route Strategy

Use typed Dioxus routes for the main operator surfaces:

- `/`
- `/sessions`
- `/sessions/:id`
- `/agents`
- `/agents/:id`
- `/activity`
- `/devices`
- `/cron`
- `/cron/:id`
- `/devices/:id`
- `/settings`

Each route should:

- fetch its critical initial data through server functions
- subscribe to relevant live updates after initial render when needed
- keep local transient UI state in Dioxus signals or context

Routes should prefer server-function-driven refresh over persistent live subscriptions unless the page materially benefits from continuous updates.

---

# 8. Core State Model

The server maintains authoritative UI state derived from the configured gateway and broadcasts updates to connected clients.

Core models:

- Session
- Agent
- ActivityEvent
- TrustedDevice
- CronJob
- GatewayStatus

These are Daneel internal models, not direct gateway wire formats.

---

# 9. UX And Interaction Design

The UI should prioritize the most important operator workflows inspired by OpenClaw Mission Control, including:

- understanding current gateway state quickly
- monitoring live activity without losing context
- inspecting sessions, agents, and devices efficiently
- performing approval and operational actions with confidence

Frontend implementation guidance:

- emphasize dashboard summaries and detail drill-down flows
- keep key operational controls close to the state they affect
- use motion to reinforce change, loading, and live updates
- present dense operational data in a polished, readable layout
- avoid generic admin-dashboard styling
- keep aesthetic choices token-driven so visual updates can happen in theme data instead of component rewrites

The visual system should support a premium operator experience comparable in feel to Sonars, while staying appropriate for a mission-control interface.

---

# 10. Adapter Architecture

Gateway interactions are abstracted behind an adapter interface.

Adapter responsibilities:

- connection lifecycle management
- API request execution
- event subscription handling
- reconnection logic
- mapping gateway data into Daneel internal models
- mapping Daneel actions into gateway-specific operations

The adapter interface should expose capabilities for:

- sessions
- agents
- activity events
- trusted devices
- cron jobs
- configuration and diagnostics where supported

OpenClaw is the first required adapter target.

The core UI, routing, and application state must not depend on OpenClaw-specific payload formats.

---

# 11. OpenClaw Adapter

The OpenClaw adapter is responsible for integrating with currently documented OpenClaw capabilities.

Supported integration areas:

- session listing and inspection
- agent listing and inspection
- gateway event streaming
- device approval and revocation flows
- cron job visibility and supported management actions
- configuration and diagnostics surfaces where exposed by OpenClaw

The adapter must not assume unsupported OpenClaw features such as custom task management workflows.

## 11.1 Future Agent Relationship Discovery

In a future iteration, Daneel may enrich the agents view with runtime relationship hints derived from agent-to-agent chat or coordination messages.

This future capability should be treated as optional metadata enrichment rather than as authoritative gateway truth.

Potential future relationship questions include:

- which agents an agent is currently working with
- which agent delegated work to it
- which agent it is delegating work to

Design guidance:

- relationship discovery should use explicit structured prompts or message contracts rather than loose natural-language inference whenever possible
- chat-derived relationship hints should be time-scoped and labeled as runtime-reported hints
- chat-derived relationships must remain visually and semantically distinct from gateway-native bindings, routing relationships, and static local metadata
- the system should tolerate missing, stale, or contradictory self-reports without breaking the graph or presenting them as guaranteed fact
- if this capability is added, the adapter or service layer should normalize these reports into typed relationship models before the UI consumes them

This should not be treated as a current OpenClaw capability requirement for the initial implementation.

---

# 12. Device Pairing And Trust

Device trust follows the OpenClaw pairing model.

Trusted-device information may include:

- device_id
- public_key or credential identifier
- device_name
- first_seen
- last_seen
- revoked

Pairing flow:

1. The client is configured with the gateway URL and gateway auth token.
2. The client generates or loads a stable device identity based on a local keypair.
3. The client connects to the gateway and includes auth plus device identity during the connection handshake.
4. If the device is local, OpenClaw may auto-approve it.
5. If the device is remote and unknown, the gateway creates a pending device request and rejects protected access with a pairing-required response.
6. The operator approves the pending request through the OpenClaw CLI.
7. After approval, the gateway issues a device token for that device and role.
8. The client persists the device token and uses it on future connections.
9. The server refreshes trust state from the gateway and grants access according to OpenClaw trust rules.

Daneel must not introduce a parallel baseline authentication system for paired devices.

## 12.0 OpenClaw-Modeled Authorization Flow

The Daneel authorization flow should be modeled on the way OpenClaw works today for operator clients:

- gateway access is protected by gateway auth, typically a token
- device identity is part of the connection model
- new remote devices require explicit approval
- approved devices receive persisted device tokens
- revocation and rotation happen through gateway device management

Important behavioral rules from current OpenClaw:

- loopback or otherwise local connects may be auto-approved
- remote LAN or Tailnet connects require explicit approval unless insecure auth is enabled
- each browser profile effectively behaves like a distinct device identity, so clearing browser data or switching profiles may require re-pairing
- approval is by pending request id through `openclaw devices approve`
- revocation is by device id and role through `openclaw devices revoke --device <id> --role <role>`
- device tokens may be rotated if needed

## 12.1 Client Authorization States

Each client device should be treated as being in one of these authorization states:

- unknown
- pending_approval
- trusted
- revoked
- authorization_error

Expected behavior by state:

- `unknown`
  - the device is not yet recognized as trusted
  - if local auto-approval applies, this state may transition immediately to `trusted`
  - otherwise the UI should redirect to the pairing experience

- `pending_approval`
  - the gateway has created a pending pairing request
  - the UI should show a waiting state with clear instructions for the operator
  - the UI should surface that approval happens through the OpenClaw CLI

- `trusted`
  - the device has an approved trust relationship
  - a device token may already be persisted for future connects
  - full Daneel access is allowed according to the configured gateway trust model

- `revoked`
  - the device token or trust relationship is no longer valid
  - the device must lose access to protected routes and server functions
  - the UI should show a revoked-access explanation and recovery path if appropriate

- `authorization_error`
  - used when trust state cannot be determined reliably
  - protected operations should fail closed

## 12.2 Server Responsibilities In Authorization

The Daneel server is responsible for:

- determining trust state before serving protected data
- enforcing route and server-function guards for untrusted devices
- forwarding pairing requests through the configured adapter
- refreshing trust state after approval or revocation
- exposing a safe, human-readable authorization status to the UI
- validating the presence of gateway auth and device identity information before protected access is attempted
- handling persisted device-token reuse where supported by the adapter

The server should treat the gateway as the source of truth for device trust whenever supported by the configured adapter.

## 12.3 UI Responsibilities In Authorization

The browser UI should:

- show a lightweight pairing screen for unknown or pending devices
- avoid exposing protected operational screens before trust is established
- provide clear status messaging during approval wait states
- recover cleanly when trust changes while the app is open
- allow the operator to enter or confirm the gateway token during setup when needed
- explain when a new browser profile or cleared browser storage may require re-pairing

The client should never assume that previous trust remains valid without server confirmation.

## 12.4 Authorization Checks

Authorization should be enforced at multiple layers:

- route-level guards for protected screens
- server-function checks for protected backend operations
- live transport validation if a persistent connection is used

Protected data should fail closed when trust state is ambiguous.

---

# 13. Operator CLI Integration

OpenClaw already provides CLI-based operational and trust controls, and Daneel should integrate with that model rather than inventing a separate admin control plane for baseline operations.

CLI-related goals:

- support approval-driven operations
- keep the gateway as the source of truth
- ensure CLI actions are reflected cleanly in the UI

## 13.1 CLI Scope

The initial Daneel design assumes operator CLI flows for actions such as:

- listing trusted or pending devices
- approving devices
- rejecting devices
- revoking devices
- rotating device tokens
- checking gateway or system status

Relevant OpenClaw commands include:

- `openclaw devices list [--json]`
- `openclaw devices approve <requestId>`
- `openclaw devices reject <requestId>`
- `openclaw devices revoke --device <id> --role <role>`
- `openclaw devices rotate --device <id> --role <role>`

If additional operations are supported cleanly by the OpenClaw CLI, the adapter may expose them later.

## 13.2 Adapter Relationship To CLI Operations

The adapter layer should normalize CLI-backed operational concepts into Daneel models and actions.

That means:

- Daneel UI code should not be written against raw CLI output
- CLI-backed actions should be represented as typed adapter operations
- errors from CLI-mediated or CLI-equivalent flows should be mapped to user-readable states

The OpenClaw adapter may fulfill these actions through gateway APIs, CLI commands, or other documented control paths, but the rest of Daneel should not care which path is used.

## 13.3 UI Expectations Around CLI-Driven Changes

Because approvals and revocations may happen outside the webapp, Daneel must handle external state changes gracefully.

The UI should:

- refresh trust state after approval-sensitive actions
- show updated device state after revoke or approve events
- tolerate the operator using the CLI directly while the webapp is open
- tolerate device-token rotation without leaving the user in a permanently stale state

## 13.4 Future Daneel CLI

A small Daneel-specific CLI may be added later for local development, diagnostics, or automation support.

If added, it should focus on:

- local configuration inspection
- adapter diagnostics
- development-time fixture and mock tooling

It should not duplicate gateway-authoritative operations unless there is a strong operational reason.

---

# 14. Live Event Transport

If Daneel uses a persistent live-update channel, there should be a single connection per client.

Server-to-client events may include:

- SessionUpdated
- AgentUpdated
- ActivityAdded
- GatewayStatusChanged
- DeviceTrustChanged
- CronJobUpdated

The server normalizes gateway-specific events into Daneel event types before sending them to the browser.

If WebSockets are used, the endpoint should be implemented with shared typed message definitions so both client and server compile against the same protocol contract.
If SSE or streaming HTTP is sufficient, the same normalized event types should be reused for that transport.

---

# 15. API Surface

The Daneel server exposes UI-oriented APIs instead of forwarding raw gateway payloads directly.

API categories:

- dashboard summary
- sessions
- agents
- activity (with event type filtering)
- devices
- cron jobs
- settings and diagnostics, including server functions for reading and updating Daneel's local configuration (persisted in SQLite/JSON storage).

Most of this API surface should be implemented as Dioxus server functions rather than a separately maintained REST layer.

Mutating operations are only exposed when supported by the configured adapter and underlying gateway.

Design preference:

- use server functions first
- add polling second when a page benefits from regular refresh
- add live transport last, only where the operator experience requires push updates

---

# 16. Settings Architecture

The settings architecture covers how Daneel's own configuration and user preferences are managed.

## 16.1 Configuration Management

Daneel's specific configuration (e.g., gateway endpoint, adapter selection, listen port) should be managed via a structured configuration file.

- **Config File Format:** TOML is preferred for its human-readability and Rust ecosystem support.
- **Config File Location:**
  - Default: `daneel.toml` in the working directory.
  - User-specific: `~/.config/daneel/daneel.toml` for platform-standard config locations.
- **Required Keys:**
  - `gateway_url`: URL of the OpenClaw Gateway.
  - `gateway_auth_token`: Authentication token for the Gateway.
- **Optional Keys:**
  - `listen_port`: Port Daneel server listens on (default 8090).
  - `adapter_type`: Explicit adapter to use if multiple are available.
  - `theme_default`: Default theme preference (e.g., "dark", "bright").
  - `sqlite_path`: Path to the SQLite database file.
- **Runtime Reload:** Configuration is read once at startup. Hot-reload is not supported in V1 to reduce complexity.

## 16.2 Theme Preference Persistence

Theme preferences should be persisted to provide a consistent user experience.

- **Mechanism:** Stored in the local SQLite database.
- **Sync:** The preference is read from SQLite by the server, passed to the frontend, and can be updated via a server function. Changes are reflected immediately in the UI and persisted.

## 16.3 Diagnostics and Log Access

Daneel should provide mechanisms to access diagnostic information and logs to aid troubleshooting.

- **Log Access:** Links or an embedded viewer on the settings page to access Daneel's own logs (e.g., via a server function exposing recent log entries).
- **System Information:** A dedicated API endpoint (and corresponding UI section) to retrieve Daneel's version, build information, active configuration (with sensitive data masked), and adapter status.
- **Configuration Inspection:** The settings page will display the currently active configuration, indicating which values are from the config file vs. defaults.
- **Adapter-Exposed Diagnostics:** The `GatewayAdapter` trait should include optional methods for adapters to expose their own diagnostics or log links if the underlying gateway supports it.

## 16.4 Editable vs. Read-Only Settings

- **Editable:** Theme preference, potentially adapter-specific settings if the adapter supports runtime changes.
- **Read-Only (Restart Required):** Gateway URL, listen port, primary adapter selection (changes require Daneel server restart).

---

# 17. Testing Strategy

## Unit Tests

Test:

- adapter trait contracts
- OpenClaw mapper logic
- pairing and trust guards
- authorization-state transitions
- CLI-output or CLI-backed action mapping where applicable
- state transitions
- event normalization

---

## Integration Tests

Test:

- API endpoints
- websocket event delivery
- adapter-backed gateway simulation
- reconnect behavior
- access control for untrusted devices
- trust-state refresh after external approval or revocation
- CLI-mediated or equivalent device-approval flow

---

# 18. Deployment

Single binary server deployed on the OpenClaw host or another host on the same local network.

Example:

OpenClaw Gateway: 8080  
Daneel: 8090

Clients connect via LAN.

---

# 19. Risks

## Gateway API changes

Gateway updates may break compatibility.

Mitigation:

adapter abstraction layer and capability-based integration boundaries.

---

## Capability drift between claws

Future adapters may not support the same features as OpenClaw.

Mitigation:

capability checks in the adapter interface and graceful UI degradation.

---

## Live transport load

Large numbers of live events may stress clients.

Mitigation:

event batching, filtering, and bounded in-memory buffers.

---

## Pairing implementation flaws

Improper trust validation could allow unauthorized access.

Mitigation:

strict adapter-mediated verification and server-side route guards.

---

## Authorization drift

The UI may become stale if trust changes externally and the app does not refresh correctly.

Mitigation:

explicit trust-state refresh after protected operations and graceful handling of external approval changes.

---

# 20. Future Improvements

Possible enhancements:

- adapter support for additional "claw" backends
- richer diagnostics views
- advanced filtering (beyond event type)
- audit logging
- capability-aware UI modules
