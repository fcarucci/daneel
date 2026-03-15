# Daneel — Requirements Document

## 1. Purpose

Daneel is a lightweight Mission Control interface for OpenClaw.

The system provides a browser-based control plane for monitoring and operating an OpenClaw gateway using capabilities that are supported by OpenClaw as documented on March 14, 2026.

Goals:

- simple architecture
- single Rust codebase
- WASM frontend using Dioxus
- integration with documented OpenClaw gateway capabilities
- local-network deployment
- explicit device pairing using OpenClaw's trust model
- clean UI using the latest Tailwind CSS 4.x workflow cleanly supported by the current Dioxus release

Daneel must not require OpenClaw features that are not currently documented by OpenClaw.

Feature prioritization should be informed by the most important supported operational patterns from the OpenClaw Mission Control project, especially:

- operational visibility across sessions, agents, devices, and gateway state
- approval-driven control flows where supported by OpenClaw
- gateway-aware operations and diagnostics
- live activity visibility for operators

These inspirations must be adapted to the subset of functionality that can be implemented on top of documented OpenClaw capabilities.

---

# 2. Design Principles

1. **Single Codebase**
   - Rust is used for frontend and backend.

2. **OpenClaw-Compatible Scope**
   - Daneel only depends on documented OpenClaw gateway features.

3. **Gateway Adapter Abstraction**
   - All OpenClaw data and control flows through a gateway adapter layer.
   - The adapter layer must isolate Daneel from OpenClaw-specific protocol details.
   - The adapter layer must be reusable so Daneel can support other "claw" backends in the future without rewriting the core UI and application logic.

4. **Gateway-Centric Architecture**
   - All runtime data and control flows through the configured gateway integration.

5. **Local Network Deployment**
   - Application is intended for LAN usage.

6. **Explicit Device Authorization**
   - Devices must be approved through OpenClaw's pairing and trust flow.

7. **Simplicity**
   - Single server process.
   - Minimal infrastructure.

---

# 3. Functional Requirements

## 3.1 Dashboard

Displays a system overview sourced from OpenClaw-supported state.

Includes:

- gateway connection status
- configured agents
- active sessions
- recent activity and events
- channel or service health indicators when available from the gateway

Features:

- live updates
- connection indicator
- reconnect state visibility

---

## 3.2 Sessions

Session monitoring and control interface.

Capabilities:

- list sessions exposed by OpenClaw
- inspect session details
- view session status and timestamps
- navigate to the related agent or conversation context when available

Session detail should display, when available from OpenClaw:

- session id
- associated agent
- current state
- timestamps
- recent related activity

Daneel must not invent a separate task lifecycle on top of sessions in the baseline requirements.

---

## 3.3 Agents

Displays agents known to OpenClaw.

Information shown:

- agent name
- agent status or availability
- capabilities or metadata exposed by OpenClaw
- current session or current work context when available
- last heartbeat or last-seen information when available

Capabilities:

- view agent metadata
- inspect agent status

Agent assignment or mutation actions are only required if directly supported by the OpenClaw gateway.

---

## 3.4 Activity Feed

Real-time event stream showing gateway-supported events such as:

- session updates
- agent updates
- presence updates
- gateway events
- cron events
- system messages

Must support:

- live updates
- filtering by event type
- clear ordering by event time

---

## 3.5 Device Pairing

Unrecognized devices must request authorization through the OpenClaw trust flow.

Untrusted devices may only access the pairing screen and status needed to complete pairing.

Pairing workflow:

1. Device connects to the application
2. Device participates in the OpenClaw pairing flow
3. Pairing request is sent to the server or gateway
4. Request is approved via CLI
5. Device becomes trusted

Trusted devices gain access consistent with the device trust granted by OpenClaw.

---

## 3.6 Trusted Devices

System must display and manage authorized devices known to OpenClaw.

Tracked data should include, when available:

- device id
- public key or device credential identifier
- device label
- first seen timestamp
- last seen timestamp
- revoked status

Capabilities:

- list trusted devices
- approve pending devices
- revoke trusted devices

---

## 3.7 Cron Jobs

Cron job management interface for OpenClaw-supported automation.

Capabilities:

- list cron jobs
- inspect cron job details
- enable or disable jobs if supported
- create, edit, or remove jobs only if directly supported by OpenClaw

Cron detail should display, when available:

- job id
- name
- schedule
- enabled state
- target agent or execution context
- recent run information

---

## 3.8 Settings

Settings page must allow access to OpenClaw-supported operational configuration, such as:

- gateway endpoint or connection settings for Daneel
- theme preference
- device management
- system information
- configuration inspection
- log access or diagnostics links when available

Configuration editing is only required where supported by the OpenClaw gateway.

---

# 4. UI Requirements

## 4.1 Styling

Tailwind CSS 4.x must be used for styling, using the latest setup that is cleanly supported by the chosen Dioxus release.

No component framework beyond the chosen Rust UI stack is required.

The UI must feel modern, slick, and polished.

Visual direction should be inspired by the Sonars product aesthetic:

- crisp modern typography
- strong visual hierarchy
- smooth, purposeful motion
- high-quality spacing and panel composition
- premium-feeling status and activity surfaces

The final UI should feel contemporary and operator-focused rather than utilitarian or generic.

---

## 4.2 Themes

Support:

- dark theme
- light theme

Theme toggle available globally.

---

## 4.3 Layout

Application layout:

Top bar:
- system status
- connection state
- theme toggle

Sidebar navigation:
- Dashboard
- Sessions
- Agents
- Activity
- Devices
- Cron
- Settings

Main panel:
- selected page content

---

# 5. Networking

Application runs on the same host as OpenClaw or on the same local network with access to the OpenClaw gateway.

Example:

OpenClaw Gateway: port 8080
Daneel: port 8090

Clients connect via LAN.

---

# 6. Security Model

Security approach:

- local network access
- OpenClaw device pairing required
- trusted-device verification
- unpaired devices cannot access protected APIs or pages

Daneel must align with OpenClaw's existing trust and device approval model rather than inventing a parallel authentication system in the baseline scope.

---

# 7. Deployment

Deployment characteristics:

- single server binary
- static asset hosting
- WebSocket support
- no external services required

---

# 8. Integration Architecture

Gateway interactions must be abstracted behind an adapter interface.

Requirements for the adapter layer:

- map gateway-specific data into Daneel's internal models
- expose a stable interface for sessions, agents, activity, devices, cron jobs, and configuration where supported
- isolate transport and protocol details from the UI layer
- allow additional backend implementations for other "claw" systems in the future

OpenClaw is the first required adapter target, but Daneel's core application logic must not be tightly coupled to OpenClaw-specific message formats.

---

# 9. Non-Functional Requirements

## Performance

UI must update in near real time using WebSockets or equivalent gateway-supported live event transport.

## Maintainability

Code should emphasize:

- readability
- low complexity
- minimal dependencies

## Reliability

System must handle:

- gateway disconnect
- websocket reconnect
- agent disconnect
- revoked device access

---

# 10. Out of Scope

The following are intentionally excluded from the baseline requirements:

- custom task management or kanban workflows
- custom task status models such as planning, inbox, review, or done
- multi-user role-based access control
- external authentication providers
- cloud infrastructure
- plugin system
- distributed services
- features not documented as supported by OpenClaw
