# Daneel TDD Review — Feedback & Recommendations

**Reviewer:** Mirko (subagent)  
**Date:** 2026-03-14  
**Scope:** Cross-referencing the Technical Design Document (TDD) against the Requirements Document (REQ), plus the UI Visual Design Reference and POC V1 Task Breakdown for consistency.

---

## Executive Summary

The TDD is well-structured and substantially aligned with the requirements. The adapter abstraction, theme system, server-function-first communication model, and device pairing flow are all thorough and well-thought-out. However, there are several gaps where the TDD either under-specifies a requirement, introduces concepts not in the requirements, or misses areas that the requirements explicitly call for. Below are the findings organized by severity.

---

## 1. GAPS — Requirements Not Adequately Covered by TDD

### 1.1 Settings Page — Incomplete Specification (REQ §3.8)

**Requirement says:**
> Settings page must allow access to: gateway endpoint or connection settings for Daneel, theme preference, device management, system information, configuration inspection, log access or diagnostics links when available.

**TDD says (§15 API Surface):**
> "settings and diagnostics" is listed as an API category, but there is no dedicated settings section in the TDD comparable to Sections 5, 8, 9, 10, 12, etc.

**Gap:** The TDD has no section defining the Settings architecture. It mentions settings in the route list (§7: `/settings`) and in API categories (§15), but never specifies:
- How Daneel's own configuration (gateway endpoint, port, adapter selection) is read/edited
- How theme preference persistence works (SQLite? localStorage? server-side?)
- How log access or diagnostics links are surfaced
- How "system information" is defined and fetched

**Recommendation:** Add a dedicated **§X Settings Architecture** section covering:
- Which settings are read-only vs. editable
- Storage location for Daneel-specific config (SQLite vs. config file)
- Theme preference persistence mechanism and sync between browser/server
- Diagnostics/log access: what the adapter exposes vs. what Daneel provides locally
- How configuration inspection maps to the adapter interface

---

### 1.2 Reconnect / Disconnect Handling — Sparse (REQ §9 Non-Functional)

**Requirement says:**
> System must handle: gateway disconnect, websocket reconnect, agent disconnect, revoked device access.

**TDD says (§6.2):**
> "use reconnection-aware UI state so pages show degraded-but-clear status during disconnects"

**Gap:** The TDD mentions reconnection awareness in the live-update section but doesn't define:
- Reconnection strategy (exponential backoff? immediate retry? configurable?)
- State reconciliation after reconnect (full refresh vs. delta?)
- What "degraded-but-clear status" means concretely for each page
- How agent disconnect (vs. gateway disconnect) is surfaced differently
- How revoked device access triggers mid-session (does the WebSocket close? does a server function return 403? does the UI poll trust state?)

**Recommendation:** Add a dedicated **§X Resilience & Reconnection** section, or expand §6.2, to cover:
- Reconnection algorithm and timing
- Post-reconnect state reconciliation strategy
- Per-page degraded state behavior
- Revocation detection during an active session (push vs. poll)

---

### 1.3 Channel / Service Health Indicators — Missing (REQ §3.1)

**Requirement says:**
> Dashboard includes: "channel or service health indicators when available from the gateway"

**TDD mentions:** Gateway status extensively, but never mentions channel health or service health indicators as a data model, adapter method, or dashboard element.

**Gap:** If the OpenClaw gateway exposes channel health (e.g., Telegram connected, Discord degraded), the TDD provides no path for surfacing it.

**Recommendation:** 
- Add a `channel_health` or `service_health` optional field to the `GatewayStatus` model
- Add an optional `list_channel_health()` or similar to the adapter trait
- Reference it in the Dashboard section as an optional data source
- Mark it as capability-gated (only shown when the adapter supports it)

---

### 1.4 Session Navigation — Under-specified (REQ §3.2)

**Requirement says:**
> "navigate to the related agent or conversation context when available"

**TDD §7 route list** includes `/sessions/:id` but the TDD never describes:
- How the session detail view links to the related agent
- What "conversation context" means in the UI (is it a link to the agent page? a separate view?)
- Cross-navigation between session ↔ agent detail views

**Recommendation:** In the route strategy (§7) or a new UX section, describe the cross-navigation model between sessions and agents. Even a brief note like "Session detail includes a link to `/agents/:id` for the owning agent" would close this gap.

---

### 1.5 Activity Feed Filtering — Under-specified (REQ §3.4)

**Requirement says:**
> Must support: filtering by event type, clear ordering by event time

**TDD §14** defines event types but doesn't describe:
- How filtering works (client-side on the buffered events? server-function with filter params? URL query params?)
- How ordering is guaranteed (server-side sort? client sort?)
- Whether filters are persisted across navigation

**Recommendation:** Add filtering architecture to §14 or §15. Specify whether filtering is client-side (simpler, fine for small event counts) or server-side (needed at scale), and how event ordering is enforced.

---

### 1.6 Cron Job CRUD — Conditional but Unaddressed (REQ §3.7)

**Requirement says:**
> "create, edit, or remove jobs only if directly supported by OpenClaw"

**TDD §11 (OpenClaw Adapter):**
> "cron job visibility and supported management actions"

**Gap:** The TDD never defines what "management actions" means for cron jobs. The adapter trait (§10) mentions cron jobs as a capability area but doesn't specify:
- Whether enable/disable is a mutation the adapter supports
- Whether create/edit/delete are capability-gated methods on the trait
- How the UI gracefully disables unavailable cron actions

**Recommendation:** In the adapter trait definition or §11, explicitly list cron mutation methods as optional/capability-gated:
- `toggle_cron_job(id, enabled)` — optional
- `create_cron_job(...)` — optional  
- `delete_cron_job(id)` — optional
- UI should check adapter capabilities and disable controls for unsupported operations

---

## 2. INCONSISTENCIES Between TDD and Requirements

### 2.1 Theme Naming: "Bright" vs. "Light"

**REQ §4.2:** "dark theme, light theme"  
**TDD §5.1 and throughout:** Uses "bright" and "dark"  
**UI Visual Design Reference:** Consistently uses "Bright Theme" and "Dark Theme"

**Issue:** The requirements say "light"; the TDD and visual reference say "bright." This is a minor naming inconsistency, but it will propagate into code (enum variants, CSS selectors, config values).

**Recommendation:** Pick one and align all documents. "Bright" is more distinctive and already used in the TDD/visual reference — update the REQ to match, or vice versa. Either way, make it consistent.

---

### 2.2 WebSocket vs. SSE — Ambiguity

**REQ §7 (Deployment):** "WebSocket support" listed as a deployment characteristic  
**REQ §9:** "UI must update in near real time using WebSockets or equivalent gateway-supported live event transport"  
**TDD §6.2:** "prefer SSE or streams when updates are server-to-client only; use WebSockets only if true bidirectional stateful communication is needed"

**Issue:** The requirements list WebSocket support as a deployment requirement, but the TDD recommends SSE as the default and WebSockets only as a fallback. This isn't necessarily contradictory, but it could confuse implementers about which transport to implement first.

**Recommendation:** Clarify in the TDD that WebSocket support is available as a deployment capability (per REQ), but the *preferred* initial transport for live updates is SSE unless bidirectional communication is needed. Explicitly state which POC milestone introduces which transport.

---

### 2.3 Device Pairing Flow — Steps Don't Fully Match

**REQ §3.5 Pairing workflow:**
1. Device connects
2. Device participates in the OpenClaw pairing flow
3. Pairing request sent to server or gateway
4. Request approved via CLI
5. Device becomes trusted

**TDD §12 Pairing flow:**
1. Client configured with gateway URL and auth token
2. Client generates/loads device identity keypair
3. Client connects with auth + device identity
4. Local devices may auto-approve
5. Remote unknown devices get pending request
6. Operator approves via CLI
7. Gateway issues device token
8. Client persists device token
9. Server refreshes trust state

**Issue:** The TDD flow is much more detailed (which is good), but step 1 introduces a prerequisite ("configured with gateway URL and auth token") that the requirements don't mention. The requirements imply a simpler "connect and get prompted" flow. The TDD also introduces auto-approval for local devices, which the requirements don't mention.

**Assessment:** The TDD flow is more realistic and technically correct. The requirements are too simplified.

**Recommendation:** 
- Keep the TDD flow as-is (it's better)
- Flag in the TDD that the requirements describe a simplified version
- Consider updating the requirements to mention the initial configuration step and local auto-approval as a future REQ update

---

## 3. TDD INTRODUCES CONCEPTS NOT IN REQUIREMENTS

### 3.1 Detail Drawer Component

**TDD §9 and UI Visual Design Reference** describe an "optional right-side detail drawer" for focused inspection. The requirements don't mention drawers.

**Assessment:** This is fine — it's a UX pattern choice that implements the requirements' "inspect" capabilities. No action needed, but the TDD should note this is a design decision, not a requirement.

---

### 3.2 Fixture-Driven Development

**TDD (via POC task breakdown)** heavily emphasizes fixture-driven development and deterministic testing. The requirements don't mention testing strategy.

**Assessment:** This is purely implementation guidance and appropriate for the TDD. No issue.

---

### 3.3 SQLite Storage

**TDD §3 (Storage):** Specifies SQLite as preferred local storage.  
**REQ:** Never mentions storage technology.

**Assessment:** Appropriate for TDD to specify. No issue, but the TDD should clarify what exactly goes into SQLite vs. what stays in the gateway. Currently §3 says "Daneel configuration, selected adapter configuration, cached device metadata if needed, UI preferences" — this is good but could be more explicit about what is *not* stored locally (e.g., sessions, agents, activity events are always fetched from the gateway).

**Recommendation:** Add a brief "What is NOT stored locally" note to §3 to prevent scope creep where someone might cache operational data in SQLite.

---

## 4. CLARITY & COMPLETENESS IMPROVEMENTS

### 4.1 Adapter Trait — Needs Concrete Method Signatures

**TDD §10** describes adapter responsibilities in prose but never shows a concrete trait definition. The POC task breakdown (T0.2) defines a minimal POC trait, but the full TDD should provide the complete baseline trait.

**Recommendation:** Add a code-level trait definition (even as pseudocode) to §10 showing all required and optional methods:
```rust
trait GatewayAdapter {
    // Required
    async fn gateway_status(&self) -> Result<GatewayStatus>;
    async fn list_sessions(&self) -> Result<Vec<Session>>;
    async fn list_agents(&self) -> Result<Vec<Agent>>;
    async fn list_activity(&self, filter: ActivityFilter) -> Result<Vec<ActivityEvent>>;
    async fn list_devices(&self) -> Result<Vec<TrustedDevice>>;
    async fn list_cron_jobs(&self) -> Result<Vec<CronJob>>;
    
    // Optional / capability-gated
    async fn approve_device(&self, request_id: &str) -> Result<()>;
    async fn revoke_device(&self, device_id: &str, role: &str) -> Result<()>;
    async fn toggle_cron_job(&self, id: &str, enabled: bool) -> Result<()>;
    async fn subscribe_events(&self) -> Result<EventStream>;
    
    // Capability introspection
    fn capabilities(&self) -> AdapterCapabilities;
}
```

This would make the adapter contract unambiguous for implementers.

---

### 4.2 Core State Model (§8) — Needs Field Definitions

**TDD §8** lists model names but no fields. The requirements specify fields for several models (e.g., REQ §3.2 lists session fields, §3.3 lists agent fields, §3.6 lists device fields, §3.7 lists cron fields).

**Recommendation:** Expand §8 with field-level definitions for each model, mapping directly to what the requirements specify. For example:

**Session model should include:** session_id, associated_agent, current_state, timestamps (created, updated), recent_activity  
**Agent model should include:** name, status/availability, capabilities_metadata, current_session, last_heartbeat  
**Device model should include:** device_id, public_key/credential_id, label, first_seen, last_seen, revoked  
**CronJob model should include:** job_id, name, schedule, enabled, target_agent, recent_runs

---

### 4.3 Error Handling Strategy — Missing

The TDD discusses error states in specific contexts (adapter timeout, trust failure, reconnection) but has no centralized error handling strategy.

**Recommendation:** Add a section covering:
- Error categorization (network, auth, adapter, validation)
- Error propagation from adapter → server function → UI
- User-facing error message guidelines (human-readable, actionable)
- Error recovery patterns (retry, redirect to pairing, show degraded state)

---

### 4.4 Configuration Management — Under-specified

The TDD mentions "adapter selection and configuration" (§2) and "config loader" (POC T3.2) but doesn't define:
- Config file format (TOML? JSON? env vars?)
- Config file location
- Required vs. optional config keys
- Runtime config reload support

**Recommendation:** Add a **§X Configuration** section specifying:
- Config format and location (e.g., `daneel.toml` in working directory or `~/.config/daneel/`)
- Required keys: gateway URL, gateway auth token
- Optional keys: listen port, adapter type, theme default, SQLite path
- Whether config is read once at startup or supports hot-reload

---

### 4.5 Logging & Observability — Missing

**REQ §3.8** mentions "log access or diagnostics links when available."  
**REQ §9** lists maintainability and reliability requirements.

The TDD has no logging strategy. For a mission-control tool, this is a notable gap.

**Recommendation:** Add a brief logging section:
- Structured logging (e.g., `tracing` crate)
- Log levels and what each covers
- Whether logs are accessible via the Settings page
- Adapter-level logging for gateway communication debugging

---

### 4.6 CORS / Security Headers — Missing

**REQ §6** defines a local-network security model, but the TDD doesn't address:
- CORS policy (does the Dioxus fullstack server need one?)
- Security headers (CSP, X-Frame-Options, etc.)
- Whether the server binds to localhost only or all interfaces

**Recommendation:** Add a brief note in §2 or a new security section about:
- Default bind address (localhost vs. 0.0.0.0)
- CORS configuration for LAN access
- Basic security headers

---

## 5. ALIGNMENT WITH POC V1 TASK BREAKDOWN

The POC task breakdown is well-structured but has a few alignment issues with the TDD:

### 5.1 Graph Focus vs. TDD's Broader Scope

The POC focuses heavily on the agent relationship graph, which is a great first vertical slice. However, the TDD doesn't have a dedicated section on graph rendering, graph layout strategy, or graph data model. The POC introduces `AgentGraphSnapshot`, `AgentNode`, `AgentEdge` concepts that aren't defined in the TDD's core state model (§8).

**Recommendation:** Add `AgentGraphSnapshot` (or equivalent) to §8 as a derived/composed model that combines Agent, Session, and binding data for the dashboard graph view.

### 5.2 Local Metadata Hints — Not in TDD

The POC introduces `list_agent_relationship_hints()` which reads local agent files (`AGENTS.md`, config files) for relationship metadata. This is a clever approach but is not mentioned anywhere in the main TDD.

**Recommendation:** Add a note in §11 (OpenClaw Adapter) about optional local metadata enrichment as a data source for relationship hints, with clear marking that these are secondary to gateway-native data.

---

## 6. MINOR ISSUES

### 6.1 Section Numbering

The TDD uses `#` for section 1 and `#` for all subsequent sections (inconsistent heading levels in some places). The opening is `## 1. Overview` but later sections use `# 2. System Architecture`. This is cosmetic but affects readability.

### 6.2 "Daneel" Name Reference

The TDD consistently uses "Daneel" which aligns with the requirements. Good.

### 6.3 Future Improvements (§19)

The TDD lists "audit logging" as a future improvement. Given that this is a mission-control tool, consider whether basic audit logging should be in the baseline requirements rather than deferred.

---

## Summary of Recommended TDD Changes (Priority Order)

| # | Change | Severity |
|---|--------|----------|
| 1 | Add Settings Architecture section | High |
| 2 | Add Resilience & Reconnection section | High |
| 3 | Add concrete adapter trait definition to §10 | High |
| 4 | Expand Core State Model §8 with field definitions | High |
| 5 | Add Configuration Management section | Medium |
| 6 | Add channel/service health to GatewayStatus model | Medium |
| 7 | Add Error Handling Strategy section | Medium |
| 8 | Clarify activity feed filtering architecture | Medium |
| 9 | Define cron mutation methods as capability-gated | Medium |
| 10 | Add session↔agent cross-navigation description | Medium |
| 11 | Add Logging & Observability section | Medium |
| 12 | Add AgentGraphSnapshot to core state model | Medium |
| 13 | Add local metadata hints to §11 | Medium |
| 14 | Align "bright" vs. "light" theme naming | Low |
| 15 | Clarify WebSocket vs. SSE recommendation | Low |
| 16 | Add "what is NOT stored locally" to §3 | Low |
| 17 | Add CORS/security headers note | Low |
| 18 | Fix section heading inconsistencies | Low |

---

*End of review.*
