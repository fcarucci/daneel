# Daneel POC V1 Foundation Decisions

This note completes the remaining documentation-oriented T0 foundation tasks for POC V1.

It defines:

- the graph semantics for the first vertical slice
- the minimal adapter capability contract
- the rendering strategy for the first graph surface

## T0.1 POC Graph Semantics

### Node Definition

For POC V1, a graph node is a configured OpenClaw agent.

Each node should carry:

- stable identity: `agent_id`
- display name
- gateway-derived runtime status
- active-session summary
- optional presentation metadata needed for card rendering

### Node Status Semantics

Node status must be derived from data we can actually justify in the POC:

- `healthy` or `degraded` from gateway-exposed health/runtime state when present
- `active_recently` from active sessions inside the agreed recent-activity window
- `unknown` when the gateway does not expose enough runtime detail

The UI must not label an agent as "working now" unless the underlying data truly supports that claim.

### Edge Definition

For POC V1, an edge represents a relationship between two configured agents.

Edges must be sourced in this order:

1. gateway-native routing or binding relationships
2. optional local relationship hints from stable agent metadata such as `AGENTS.md`
3. future explicit delegation edges only if confirmed by real adapter data

### Edge Provenance

Every edge must carry provenance so the UI can distinguish stronger truth from weaker hints:

- `gateway_native`
- `metadata_hint`

POC V1 must not present metadata hints as if they were guaranteed gateway truth.

### Graph Snapshot Semantics

The first graph payload should be a derived snapshot that combines:

- configured agents
- active-session rollups
- gateway summary
- normalized edges

This snapshot is a request-response read model for the UI, not the transport protocol itself.

## T0.2 Minimal Adapter Capability Contract

POC V1 should depend on the smallest useful contract.

The concrete OpenClaw implementation can come later, but the contract for the first slice is:

```rust
pub trait GatewayAdapter {
    type Error;

    fn gateway_status(&self) -> impl Future<Output = Result<GatewayStatus, Self::Error>> + Send;
    fn list_agents(&self) -> impl Future<Output = Result<Vec<AgentNode>, Self::Error>> + Send;
    fn list_agent_bindings(
        &self,
    ) -> impl Future<Output = Result<Vec<AgentEdgeCandidate>, Self::Error>> + Send;
    fn list_active_sessions(
        &self,
    ) -> impl Future<Output = Result<Vec<ActiveSession>, Self::Error>> + Send;
    fn list_agent_relationship_hints(
        &self,
    ) -> impl Future<Output = Result<Vec<RelationshipHint>, Self::Error>> + Send;
}
```

### Contract Rules

- shared UI-facing models must stay OpenClaw-agnostic
- the adapter may normalize protocol payloads before returning them
- relationship hints are optional and may return an empty list
- active sessions are a derived operator signal, not proof of current execution
- the first UI slice should talk to service-layer snapshot builders, not directly to protocol frames

### POC Model Surface

The contract above implies the first shared models we should stabilize are:

- `GatewayStatus`
- `AgentNode`
- `AgentEdgeCandidate`
- `ActiveSession`
- `RelationshipHint`
- `AgentGraphSnapshot`

This is intentionally smaller than a full OpenClaw management API.

## T0.3 Graph Rendering Strategy

### Decision

POC V1 should use deterministic SVG rendering.

### Why SVG First

- deterministic output is easier to verify with screenshots
- edge styling and labels are straightforward
- static positioning is enough for the first operator story
- it avoids the cost and instability of force-directed layout

### Constraints

- no physics engine
- no drag interactions
- no dependency on undocumented graph metadata from OpenClaw
- mobile fallback should degrade into stacked tiles and summaries cleanly

### Layout Strategy

The first graph should use a fixed placement algorithm derived from stable inputs:

- default or primary agents closer to the center
- supporting agents grouped around them
- edge order normalized for stable render output

If no graph is available yet, the UI may render the summary cards and agent tiles alone without faking connections.

### Verification Expectations

The rendering choice must support:

- stable screenshot verification
- repeatable empty-state rendering
- repeatable degraded-state rendering
