# Daneel POC V1 Task Breakdown

## Goal

Reach the first vertical proof of concept where we can:

- run the webapp
- connect to an OpenClaw gateway
- extract agents from the gateway
- show the list of running or active agents in a polished visual graph
- show connections between agents based on documented OpenClaw relationships that are available for the POC

## Important Scope Notes

This task plan is intentionally constrained to documented OpenClaw capabilities.

Based on the current OpenClaw docs:

- `openclaw agents list --json --bindings` is supported
- agent bindings and routing relationships are supported
- active sessions are supported
- gateway health and status are supported
- a native "agent delegates to agent" relationship is not clearly documented

From the local `.openclaw` installation:

- `/root/.openclaw/agents/planner/agent/AGENTS.md` contains explicit `Works With` relationship hints
- some agent config files include delegation guidance in stable configuration fields
- these local files can be used as optional relationship inputs for the POC adapter when present

For this POC, graph edges should therefore be implemented using documented relationship sources in this order:

1. agent bindings and routing relationships exposed by OpenClaw
2. optional adapter-derived edges from local agent metadata such as `AGENTS.md` or stable agent config fields
3. explicit delegation edges only if later confirmed by the OpenClaw adapter contract

## Definition Of Done For POC V1

The POC is complete when:

- the Daneel webapp starts locally
- the webapp can call the server through Dioxus server functions
- the server can connect to a configured OpenClaw gateway
- the OpenClaw adapter can fetch agents and bindings
- the UI renders a graph of agents and relationships
- the graph uses polished cards, icons, layout, and connection styling
- at least one integration path is covered by automated tests

---

# Feasibility Review

This breakdown is feasible for a first vertical slice, but the following delivery rules should be enforced to keep risk low:

- graph rendering should start with deterministic SVG layout, not a force-directed system
- local relationship metadata must be treated as optional hints, not authoritative gateway truth
- the first live experience should rely on server functions and manual refresh before adding any persistent live transport

Implementation guardrails:

- prefer one route, one server function, one graph snapshot, and one adapter implementation for the first slice
- do not build generalized graph editing, drag behavior, or physics simulation in POC V1
- do not block the POC on undocumented delegation semantics

---

# Phase 0: Foundation Decisions

## T0.1 Define the POC graph semantics

Purpose:

- avoid building the wrong graph model

Output:

- short design note defining what a node is and what an edge is for POC V1

Decisions:

- node = configured OpenClaw agent
- node status = derived from active sessions, health, or adapter-exposed runtime state
- edge = routing, binding, or locally declared collaboration relationship until true delegation data exists

Tests:

- review test: confirm the note does not claim undocumented delegation support
- review test: confirm the note distinguishes gateway-native edges from metadata-derived hints

---

## T0.2 Define the minimal adapter capability contract for the POC

Purpose:

- keep the POC focused on the smallest useful interface

Output:

- adapter trait with only the methods needed for the first vertical slice

Required methods:

- `gateway_status()`
- `list_agents()`
- `list_agent_bindings()`
- `list_active_sessions()`
- `list_agent_relationship_hints()`

Tests:

- unit test: mock adapter implements the trait
- compile test: UI-facing service layer depends only on the trait, not OpenClaw-specific types
- compile test: shared graph models do not import adapter implementation modules

---

## T0.3 Choose the graph rendering strategy

Purpose:

- avoid overbuilding the diagram layer

Output:

- short decision note selecting SVG-based rendering with deterministic layout for POC V1

Decision constraints:

- no physics engine
- no drag interactions
- deterministic node placement for stable tests
- straightforward mobile fallback

Tests:

- review test: selected rendering approach supports deterministic screenshot testing
- review test: layout strategy does not require undocumented runtime graph metadata

---

# Phase 1: App Bootstrap

## T1.1 Create the Rust/Dioxus application skeleton

Purpose:

- make the project runnable

Output:

- `Cargo.toml`
- `src/main.rs`
- `src/router.rs`
- initial route shell

Tests:

- `cargo check`
- smoke test: app binary starts without panicking
- smoke test: root route returns HTML successfully in local development

---

## T1.2 Add the base route structure

Purpose:

- create the first navigation frame

Output:

- typed routes for `/`, `/agents`, and `/settings`
- shared app layout with sidebar and top bar

Tests:

- component test: router renders the dashboard route
- component test: navigation renders expected route links
- component test: unknown route renders a not-found or safe fallback state

---

## T1.3 Add design tokens and global styling

Purpose:

- establish the visual system before feature work

Output:

- CSS variables for color, spacing, radius, shadows, and motion
- Tailwind setup for the design system
- typography and panel styles aligned with the Sonars-inspired direction

Tests:

- visual smoke test: base layout renders without unstyled content
- manual QA: mobile and desktop layouts load without overflow regressions
- visual regression test: root dashboard shell remains stable

---

## T1.4 Add the frontend test harness

Purpose:

- make UI work testable from the first component

Output:

- test helpers for rendering components with router, theme, and fixture state

Tests:

- smoke test: a simple component renders through the shared harness
- smoke test: dashboard route render works in test mode

---

## T1.5 Add browser-driven frontend testing against a mock gateway

Purpose:

- verify the real browser UX without depending on a live local OpenClaw instance

Output:

- automated browser-driven test suite that runs Daneel against a mock gateway process with deterministic test data

Tests:

- start Daneel in test mode against the mock gateway
- open `/` and verify the gateway status card and navbar status pill render expected mock state
- open `/agents` and verify expected agent tiles, active indicators, and counts render from mock data
- verify degraded gateway data produces the expected operator-facing error state
- verify the suite runs headlessly and independently of the developer's personal OpenClaw data

---

# Phase 2: Server Function Backbone

## T2.1 Add the first live gateway connectivity slice

Purpose:

- prove the webapp can reach the server and the server can reach the gateway through the documented WebSocket path before building broader abstractions

Output:

- one minimal Dioxus server function
- one minimal server-side gateway connection
- one small typed response rendered in the UI

Preferred scope:

- establish a local WebSocket connection from Daneel to the OpenClaw Gateway
- fetch one small typed status snapshot over that connection
- render the result on an existing route such as `/`

Fallback scope:

- `list_agents()` returning a minimal count or list if gateway status is not the cleanest first call over the gateway connection

Delivery rules:

- keep the implementation intentionally narrow
- do not block this step on the full adapter trait or graph model
- do not introduce browser-to-gateway direct communication
- optimize for proving connectivity, typed responses, local loopback security, and error handling
- use the documented gateway WebSocket protocol as the primary integration path
- only fall back to polling if the documented event path is insufficient for the specific POC need

Tests:

- integration test: UI can call the server function and receive a typed response
- integration test: server-side gateway success path renders a visible UI state
- integration test: unreachable or invalid gateway config renders a user-displayable error or degraded status
- integration test: Daneel keeps the gateway token server-side and does not expose it to the browser

---

## T2.2 Create shared UI/server model types

Purpose:

- define the cross-boundary contract once

Output:

- shared structs for `AgentNode`, `AgentEdge`, `GatewayStatus`, and `AgentGraphSnapshot`

Tests:

- unit test: JSON serialization round-trip for all shared models
- unit test: graph snapshot supports empty nodes and empty edges cleanly
- unit test: metadata-derived edge variants serialize distinctly from gateway-native edge variants

---

## T2.3 Create the server-side app state container

Purpose:

- provide a clean place for adapter configuration and shared services

Output:

- app state struct holding config and adapter instance

Tests:

- unit test: app state initializes with a mock adapter
- unit test: app state initialization fails cleanly with invalid config

---

## T2.4 Add the first Dioxus server function

Purpose:

- prove end-to-end UI-to-server communication

Output:

- `get_gateway_status()` server function

Tests:

- integration test: server function returns a typed response from a mock adapter
- component test: dashboard can call and render the returned gateway status
- integration test: server function error path maps to a user-displayable status

---

## T2.5 Add the first live gateway event bridge

Purpose:

- prove Daneel can receive and relay live gateway state changes without introducing polling as the default path

Output:

- one server-side gateway event subscription over WebSocket
- one server-to-browser live update path for gateway health or presence state
- one UI element that updates from live gateway events

Preferred transport split:

- OpenClaw Gateway to Daneel: WebSocket
- Daneel to browser: SSE or WebSocket, whichever is simplest in the Dioxus stack for server-to-client updates

Delivery rules:

- keep this limited to one or two event types
- prefer health, heartbeat, or presence signals before broader activity streaming
- do not block this step on the full graph implementation
- avoid periodic polling as the primary steady-state mechanism

Tests:

- integration test: gateway event reaches Daneel and updates internal state
- integration test: browser receives a live state update after first render
- integration test: disconnect and reconnect paths degrade clearly and recover cleanly

---

## T2.6 Add full event-driven webapp state sync

Purpose:

- keep operator-facing data fresh without relying on repeated request-response polling

Output:

- one shared server-side state synchronization layer that consumes gateway events and updates the web app's live data surfaces

Initial scope:

- gateway health and connectivity
- agent healthy or unhealthy status
- active session counts
- recent agent activity state

Delivery rules:

- treat the gateway event stream as the source of truth for live runtime state
- reuse the same synchronization backbone across dashboard, agents, and future graph surfaces
- avoid duplicating separate live-update logic per route
- preserve a clean degraded state when the event stream is unavailable

Tests:

- integration test: a gateway health event updates both the dashboard card and navbar status pill
- integration test: an agent-status event updates the agents view without a full page reload
- integration test: an active-session event updates aggregate and per-agent counts consistently
- integration test: disconnect and reconnect behavior preserves a truthful degraded or recovered UI state

---

## T2.7 Make agent recency and heartbeat state update live

Purpose:

- keep the agents view truthful after the initial live event bridge is in place

Output:

- agent tile `time ago` ribbons that continue advancing on the client without requiring a full page refresh
- active tile styling that updates as agents cross the recent-activity threshold
- heartbeat indicator logic that treats disabled, missing, `none`, or zero heartbeat cadence as inactive and gray
- unit and integration coverage for the recency ticker and heartbeat normalization rules

Delivery rules:

- derive displayed recency from the last-known event timestamp plus client-side elapsed time
- keep the active-state threshold consistent with the current 10-minute recent-activity window
- do not show a live heartbeat heart when the gateway data indicates no heartbeat schedule
- preserve deterministic behavior in tests by controlling the clock or recency formatter input

Tests:

- unit test: recency formatter advances from minutes to hours correctly as time passes
- unit test: crossing the recent-activity threshold flips an agent from active to inactive styling
- unit test: heartbeat values of missing, `none`, or zero produce the gray heart state
- integration test: the agents view updates `time ago` ribbons live without a page reload
- integration test: an agent tile loses active glow when it ages out of the recent-activity window
- integration test: planner-style disabled heartbeat data renders a gray heart consistently

---

## T2.8 Add live backend connection and reconnection state management

Purpose:

- make browser-to-backend and backend-to-gateway connectivity explicit, resilient, and operator-friendly

Output:

- a first-class connection-state model covering browser, backend, and gateway reachability
- a top-right status pill that can truthfully show `Connected`, `Connecting`, and `Disconnected`
- frozen views when the backend is disconnected, keeping the last known good UI visible while subtly grayed out
- background reconnect behavior from browser to backend and from backend to gateway

Required behavior:

- when the web app loses the backend event stream or backend HTTP connectivity, the status pill must show `Disconnected`
- when disconnected, the current views should remain visible and frozen rather than clearing or thrashing
- frozen views should be slightly grayed out or otherwise visually subdued without becoming unreadable
- the browser should keep trying to reconnect to the backend in the background
- after backend recovery, the backend should continue reconnecting to the gateway until live state is restored
- once the full path recovers, the UI should return to the normal live-connected state without a full page reload

Delivery rules:

- distinguish browser-to-backend disconnects from backend-to-gateway disconnects
- do not discard the last known good route data just because the live transport is down
- keep the frozen state informative and calm rather than alarming by default
- centralize connection-state logic so routes do not invent their own reconnect behavior

Tests:

- unit test: connection-state resolution maps browser/backend/gateway combinations to the correct operator-facing state
- unit test: disconnected state preserves last-known-good data while marking the UI frozen
- unit test: reconnecting state does not incorrectly downgrade healthy cached data to a hard failure state
- integration test: mock backend disconnect changes the pill to `Disconnected` while the current views remain rendered
- integration test: reconnect restores the live pill and unfreezes the UI without a reload
- integration test: backend-up but gateway-down still shows the correct degraded or reconnecting semantics
- acceptance test (automated): end-to-end test toggles backend availability and asserts pill transitions plus frozen/unfrozen UI behavior
- acceptance test (automated): end-to-end test validates cached data remains visible while disconnected
- acceptance test (visual): live screenshot confirms the disconnected frozen state is readable and the reconnect path returns to `Connected`

---

## T2.9 Introduce the shared `AppClient` boundary

Purpose:

- introduce one shared UI-facing `AppClient` boundary so the current web app stops depending directly on server-function details and the backend contract is prepared cleanly for future native clients

Output:

- one shared `AppClient` interface for request-response reads
- stable explicit endpoint paths for the current Dioxus server functions
- a clear separation between UI-facing client code, server-function transport, and backend app-service logic

Initial scope:

- cover `get_gateway_status()`
- cover `get_agent_overview()` only if it follows the same shape cleanly
- keep live transport out of scope except where stable endpoint naming or client setup affects future work

Delivery rules:

- keep the web app calling through Dioxus server functions underneath `AppClient`
- keep UI components unaware of gateway protocol and transport details
- keep `AppClient` small and focused on the current read paths
- do not collapse `AppClient` into the deeper `GatewayAdapter`; they solve different layers of the architecture
- prefer explicit stable endpoint annotations for the relevant server functions so future native clients are not coupled to unstable generated paths

Tests:

- unit test: the web `AppClient` delegates to the existing server-function-backed path for gateway status
- unit test: shared UI-facing code can depend on `AppClient` without importing transport-specific or OpenClaw-specific types
- unit test: error mapping at the `AppClient` boundary preserves the current operator-facing degraded semantics
- compile test: UI-facing code depends only on shared models and the `AppClient` interface, not on OpenClaw-specific transport code
- integration test: existing server functions still return the same typed payloads for the web `AppClient`
- integration test: dashboard route still renders the expected gateway status states through the web `AppClient`
- integration test: agents route still renders the expected overview state if `get_agent_overview()` is included in this task
- integration test: the relevant server functions expose explicit stable endpoint paths suitable for both web and desktop callers
- live verification: confirm the running web app still loads dashboard and agents through the web `AppClient`, and verify the stable endpoint design is ready for future native-client use without changing the UI-facing boundary

---

# Phase 3: OpenClaw Adapter Minimum Slice

## T3.1 Create the adapter trait and OpenClaw adapter module

Purpose:

- establish the abstraction boundary early

Output:

- `GatewayAdapter` trait
- `OpenClawAdapter` struct

Tests:

- unit test: mock adapter can satisfy the trait
- compile test: no OpenClaw protocol types leak into shared UI models

---

## T3.2 Implement gateway configuration loading

Purpose:

- make the adapter connectable in local development

Output:

- config loader for gateway base URL, auth/token settings, and optional local metadata root

Tests:

- unit test: valid config parses correctly
- unit test: missing required config returns an actionable error
- unit test: optional local metadata root can be disabled
- unit test: invalid metadata root is rejected or ignored safely

---

## T3.3 Implement `gateway_status()`

Purpose:

- verify connectivity before agent graph work

Output:

- OpenClaw adapter health call mapped into `GatewayStatus`

Tests:

- unit test: maps healthy response correctly
- unit test: maps unreachable gateway to degraded status
- integration test: adapter timeout maps to a recoverable gateway status

---

## T3.4 Implement `list_agents()`

Purpose:

- extract agent nodes from OpenClaw

Output:

- adapter method to fetch configured agents from OpenClaw

Tests:

- unit test: OpenClaw agent JSON maps to internal `AgentNode`
- fixture test: unknown fields do not break parsing
- unit test: missing optional presentation fields fall back safely

---

## T3.5 Implement `list_agent_bindings()`

Purpose:

- extract graph edges from documented routing relationships

Output:

- adapter method to fetch bindings and normalize them into edge candidates

Tests:

- unit test: binding payload maps to expected edge metadata
- unit test: empty bindings produce zero edges without error
- unit test: duplicate bindings are normalized deterministically

---

## T3.6 Implement `list_active_sessions()`

Purpose:

- derive which agents are active or running for the POC view

Output:

- adapter method to fetch sessions and map them to agent activity information

Tests:

- unit test: OpenClaw session JSON maps to internal `ActiveSessionRecord`
- unit test: multiple sessions for different agents map correctly
- unit test: missing optional session detail fields fall back safely
- fixture test: unknown fields do not break session parsing
- unit test: malformed required session identity fails with a clear error
- unit test: duplicate session ids are normalized deterministically if duplicate records are returned
- unit test: unknown session agent references still map safely and do not crash graph derivation
- integration test: `OpenClawAdapter::list_active_sessions()` reads session data end to end through the mock gateway path

---

## T3.7 Implement `list_agent_relationship_hints()`

Purpose:

- enrich the graph with optional local relationship metadata

Output:

- adapter method that inspects local agent files such as `AGENTS.md` and stable config fields for collaboration or delegation hints

Allowed sources:

- `AGENTS.md` sections such as `Works With`
- explicit config constraints that instruct delegation to another agent

Tests:

- unit test: planner `Works With` content maps to collaboration edges
- unit test: `Works With` content with multiple entries maps to deterministic, deduplicated relationship edges
- unit test: health-coach config delegation hint maps to a relationship edge
- unit test: unknown referenced agent names are ignored safely and do not create dangling hints
- unit test: missing local metadata returns an empty set without error
- unit test: malformed markdown or malformed config for one agent does not fail the full hint load for other agents
- unit test: metadata parsing can be disabled by config
- unit test: mixed markdown and config hints merge without duplicate edges
- integration test: `OpenClawAdapter::list_agent_relationship_hints()` reads temp local agent metadata end to end and returns shared-model edges only

---

# Phase 4: Graph Assembly Service

## T4.1 Build the graph assembly service

Purpose:

- keep graph derivation out of UI components

Output:

- service that combines agents, bindings, sessions, and optional metadata hints into `AgentGraphSnapshot`

Tests:

- unit test: agents + bindings create the expected node and edge counts
- unit test: active sessions decorate node status correctly
- unit test: local relationship hints merge into the graph without duplicating binding edges
- unit test: edge ordering is deterministic for stable rendering and snapshots
- unit test: orphan edges are dropped or marked safely
- unit test: empty adapter data produces a valid empty snapshot without panicking
- unit test: unknown session agent references do not create phantom nodes and do not crash graph assembly
- unit test: metadata-derived edges do not duplicate gateway-native edges when source and target match
- unit test: node ordering is deterministic even when adapter results arrive in different input orders
- unit test: graph summary values match the assembled nodes, edges, and active-agent count
- integration test: graph assembly against a mock adapter combines agents, bindings, sessions, and hints into one stable snapshot

---

## T4.2 Define edge semantics for the POC

Purpose:

- ensure the UI labels relationships correctly

Output:

- a small, justified edge-kind set for the POC
- one gateway-native kind: `routes_to`
- two metadata-derived hint kinds: `works_with_hint` and `delegates_to_hint`

Tests:

- unit test: gateway-native bindings map to `routes_to` for the POC
- review test: no edge is labeled `delegates_to_hint` unless backed by real delegation metadata
- review test: metadata-derived hints are visually distinguished from gateway-native relationships
- unit test: edge priority rules prefer gateway-native edges over metadata hints when both exist

---

## T4.3 Add `get_agent_graph_snapshot()` server function

Purpose:

- provide one typed backend read for the first graph-driven UI slice

Output:

- explicit server function `get_agent_graph_snapshot()`
- stable endpoint path for graph snapshot reads
- `AppClient` support for fetching the graph snapshot
- typed payload returning nodes, edges, and snapshot timestamp from graph assembly

Tests:

- unit test: web `AppClient` delegates graph snapshot reads through the new server-function path
- integration test: `get_agent_graph_snapshot()` returns a full snapshot from mock adapter data
- integration test: empty adapter data returns a valid empty graph snapshot instead of an error
- integration test: server function preserves deterministic node and edge ordering from graph assembly
- integration test: graph snapshot generation still succeeds when relationship hints are unavailable
- integration test: explicit stable endpoint path is exposed for the graph snapshot server function
- integration test: graph-consuming UI code can depend on the graph snapshot payload without additional fetch hops or transport-specific imports
- live verification: existing live routes still hydrate correctly after the new graph snapshot server function is wired into the backend contract

---

# Phase 5: Vertical UI Slice

## T5.1 Build the dashboard shell

Purpose:

- create the first operator surface

Output:

- dashboard page with hero status area and graph section

Tests:

- component test: dashboard renders loading, error, and success states
- component test: dashboard renders a safe empty state when zero agents are returned

---

## T5.2 Build the graph canvas component

Purpose:

- render nodes and connections clearly

Output:

- graph component with positioned nodes and edges

Implementation preference:

- start with a simple deterministic layout
- avoid physics or drag behavior in the first slice
- implement with SVG first unless a simpler deterministic renderer proves better

Tests:

- component test: graph renders the correct number of nodes and edges
- component test: empty graph state renders gracefully
- snapshot test: the same fixture produces the same node positions across test runs
- component test: large labels truncate or wrap without overlapping the entire graph

---

## T5.3 Build the agent node card

Purpose:

- make each agent visually rich and easy to scan

Output:

- reusable card showing name, status, activity, and relationship affordances

Visual requirements:

- distinctive iconography per agent
- status ring or glow
- strong typography
- subtle motion on hover or focus

Tests:

- component test: active agent card renders different styling from idle agent card
- accessibility test: card content is readable with keyboard focus
- component test: long agent names and missing descriptions render gracefully

---

## T5.4 Build the connection rendering

Purpose:

- communicate relationships between agents clearly

Output:

- styled lines or curves between agent cards with optional labels

Tests:

- component test: edge labels render for known edge kinds
- visual regression test: overlapping edges remain legible for the fixture graph
- component test: metadata-derived edges use a distinct visual treatment from gateway-native edges

---

## T5.5 Build gateway and graph summary cards

Purpose:

- make the dashboard useful even before graph inspection

Output:

- summary cards for gateway status, agent count, active agent count, and edge count

Tests:

- component test: summary values match the snapshot fixture
- component test: degraded gateway state is reflected in the status summary card

---

# Phase 6: Error Handling And Polish

## T6.1 Add connection-state UX

Purpose:

- make failures legible during demos and local development

Output:

- clear loading, empty, degraded, and disconnected states
- route-level graph and summary views that keep the last known good data visible when live connectivity drops
- operator-facing recovery copy that distinguishes "still loading", "no graph data yet", "gateway degraded", and "backend disconnected"
- a visually subdued frozen mode for stale-but-still-useful dashboard content while reconnect is in progress

Required behavior:

- initial page load must show an explicit loading state before the first graph snapshot resolves
- an empty graph snapshot must render as a truthful empty state rather than an error
- malformed or partial graph data must not crash the page; available summaries and valid nodes should still render
- when the backend remains reachable but graph loading fails, the dashboard should show a degraded state with retry affordance while preserving any independently available gateway status data
- when the live backend connection is disconnected after a successful load, the dashboard should keep the last known good graph and summaries visible in a frozen state instead of clearing the page
- disconnected and degraded states must use distinct copy and styling so operators can tell whether the problem is backend reachability or gateway/data quality

Visual acceptance:

- loading state reads as intentional and in-progress, not like a blank card or layout failure
- empty state clearly explains that Daneel is healthy enough to load the dashboard even though no graph nodes are currently available
- degraded state uses warning styling and explanatory copy without hiding any still-valid gateway or graph data
- disconnected state keeps the last known graph visible, adds a readable frozen treatment, and surfaces reconnect messaging without overwhelming the rest of the UI
- top-right connection pill, connection banner, summary cards, and graph panel remain semantically aligned for each state
- screenshot verification covers loading, empty, degraded, disconnected-frozen, and recovered-connected renders on the dashboard route

Tests:

- unit test: graph snapshot view resolves `Loading`, `Empty`, `Error`, and `Ready` consistently from the resource state
- unit test: malformed snapshot rows or partial node metadata degrade gracefully without panicking and still render valid summaries
- unit test: disconnected frozen mode keeps the last known good graph payload instead of downgrading to an empty state
- unit test: degraded and disconnected copy stay distinct so operator messaging does not regress into a generic error
- component test: disconnected gateway renders a recovery message
- component test: malformed snapshot does not crash the page
- component test: partial data still renders available nodes and summaries
- component test: empty snapshot renders the dedicated empty-state copy and does not show graph-canvas primitives
- component test: loading state and retry affordance do not appear at the same time
- integration test: backend-up but gateway-degraded dashboard keeps summary cards visible while the graph panel enters a degraded state
- integration test: after one successful load, a mocked backend disconnect preserves the last graph render and applies the frozen layout treatment
- integration test: reconnect after disconnect returns the pill/banner/dashboard to the normal connected state without a full reload
- integration test: retrying a failed graph snapshot replaces the degraded panel with the recovered graph once the backend responds successfully
- visual acceptance test: route verifier captures and saves hydrated DOM plus screenshots for loading, empty, degraded, disconnected-frozen, and recovered dashboard states

---

## T6.2 Add manual refresh interaction

Purpose:

- keep the POC aligned with server-functions-first architecture

Output:

- refresh button that re-runs `get_agent_graph_snapshot()`

Tests:

- component test: clicking refresh triggers a refetch
- integration test: refreshed data replaces the old snapshot
- integration test: refresh failure preserves the last good graph while showing an error state

---

## T6.3 Add motion and polish pass

Purpose:

- hit the "beautiful diagram with nice graphics" bar

Output:

- motion tuning, hover states, empty-state art direction, polished gradients and shadows
- coherent motion and polish across the dashboard and agents routes without weakening the operator-state semantics added in `T6.1`
- visual treatments that still read clearly during loading, empty, degraded, disconnected, and recovered states

Visual acceptance:

- dashboard entrance, summary cards, graph panel, and agents tiles feel intentional rather than static or boilerplate
- hover, focus, and active states are polished on both desktop and mobile without making the UI feel noisy
- loading, empty, degraded, disconnected, and recovered states preserve clear readability after the motion pass
- motion remains subtle enough that status changes still read as operational state, not decorative animation
- screenshot and video verification cover the primary dashboard route plus the agents route after the polish pass
- visual verification captures at least one healthy state and one degraded or empty state after hydration

Tests:

- manual QA checklist: desktop and mobile both feel polished
- component test: polished dashboard state markers still distinguish loading, empty, degraded, disconnected, and ready renders
- integration test: mock-gateway healthy dashboard and agents routes render the polished shell without regressing required hydrated text
- integration test: mock-gateway degraded or empty route still renders the polished state treatment without collapsing into an error page
- visual regression snapshots for the main dashboard state
- visual regression snapshots for loading, empty, degraded, disconnected, and success states
- visual acceptance test: hydrated screenshot capture for the polished dashboard and agents routes plus uploaded verification media

---

# Phase 7: End-To-End Proof

## T7.1 Add adapter integration test against a mock OpenClaw endpoint

Purpose:

- prove the OpenClaw adapter works against realistic payloads

Output:

- integration test server with canned OpenClaw responses

Tests:

- integration test: gateway health fetch
- integration test: agents list fetch
- integration test: bindings fetch
- integration test: sessions fetch
- integration test: optional local metadata hint loading
- integration test: gateway data still renders when local metadata files are absent

---

## T7.2 Add end-to-end POC smoke test

Purpose:

- verify the full vertical slice from UI to adapter

Output:

- one automated smoke test covering app load through graph render

Tests:

- start app with mock adapter
- open dashboard
- verify gateway status is visible
- verify expected agent nodes are visible
- verify expected edges are visible
- verify a metadata-derived edge uses the expected visual distinction

---

## T7.3 Add a manual demo checklist

Purpose:

- ensure the polished demo path is repeatable outside automated tests

Output:

- short runbook for starting Daneel and demonstrating the first graph

Tests:

- manual check: local startup instructions are accurate
- manual check: expected gateway state produces the documented dashboard behavior

---

# Suggested Execution Order

1. T0.1
2. T0.2
3. T0.3
4. T1.1
5. T1.4
6. T1.5
7. T2.1
8. T2.2
9. T2.3
10. T2.5
11. T2.6
12. T2.7
13. T2.8
14. T3.1
15. T3.2
16. T3.3
17. T3.4
18. T3.5
19. T3.6
20. T3.7
21. T4.1
22. T4.2
23. T4.3
24. T1.2
25. T1.3
26. T5.1
27. T5.2
28. T5.3
29. T5.4
30. T5.5
31. T6.1
32. T6.2
33. T6.3
34. T7.1
35. T7.2
36. T7.3

## Smallest Useful Vertical Slice

If we want the absolute minimum path before the full polish pass:

1. T0.1
2. T0.2
3. T0.3
4. T1.1
5. T1.4
6. T1.5
7. T2.1
8. T2.2
9. T2.5
10. T2.6
11. T2.7
12. T2.8
13. T3.1
14. T3.2
15. T3.3
16. T3.4
17. T3.5
18. T3.6
19. T4.1
20. T4.3
21. T5.1
22. T5.2
23. T5.3
24. T5.4
25. T6.2
26. T7.3

## POC Success Demo Script

At the end of the first vertical slice we should be able to demo:

1. Start Daneel locally.
2. Open the dashboard in the browser.
3. Show that gateway status loads from OpenClaw.
4. Show that agents are fetched through the adapter.
5. Show the graph of agents and relationships.
6. Trigger a manual refresh and show updated state.
