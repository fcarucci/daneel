# Daneel UI Visual Design Reference

## Purpose

This document defines the intended look and feel of the Daneel web UI for the first milestone and beyond.

It draws from two reference products:

- OpenClaw Mission Control, for operational structure, control-surface priorities, approvals, gateway management, and activity visibility
- Sonars, for visual polish, hierarchy, density, motion, and premium-feeling operator UX

This is a design reference, not a pixel-perfect copy brief.

Important note:

- Mission Control provides the operational UI reference
- Sonars provides the aesthetic and interaction-quality reference
- font and color recommendations below are design inferences based on these references, not claims about the exact implementation choices used by those products

## Reference Sources

- OpenClaw Mission Control README: https://github.com/abhi1693/openclaw-mission-control
- Sonars homepage: https://sonars.dev/
- Sonars product blog: https://sonars.dev/blog/introducing-sonars
- Sonars agent chat blog: https://sonars.dev/blog/agent-chat-feature

## What To Borrow From Each Reference

### From Mission Control

Use Mission Control as the reference for:

- dashboard-as-operations-surface framing
- gateway-aware management
- agent operations and inspection
- approvals and governance posture
- activity and audit visibility
- an interface that feels built for operators, not consumers

Reference basis:

- Mission Control describes itself as a centralized operations and governance platform with unified visibility, approval controls, gateway-aware orchestration, agent management, and activity visibility.

### From Sonars

Use Sonars as the reference for:

- crisp modern layout composition
- premium panel treatment
- strong typography and hierarchy
- smooth, understated motion
- high information density without visual clutter
- prominent escalation or attention states

Reference basis:

- Sonars emphasizes native performance, polished UX, parallel workspaces, diff review, strong status surfaces, and prominent escalations.

## Core Visual Principles

1. Daneel should feel like an operator console, not a generic admin dashboard.
2. High-signal data should be visible at a glance.
3. Complexity should be hidden as much as possible behind clear defaults, progressive disclosure, and simple interactions.
4. The product should feel approachable in the way the iPhone feels approachable: powerful underneath, calm and obvious on the surface.
5. Daneel should aim for AI for the masses rather than AI for geeks only.
6. Panels should feel layered, sharp, and intentional.
7. Motion should communicate state changes, not decorate the screen.
8. Graph and status surfaces should feel premium enough to demo confidently.
9. Metadata-derived relationship hints must be visible but visually secondary to gateway-native relationships.

## Simplicity Philosophy

Daneel should present advanced system capability through a simple, welcoming interface.

Design intent:

- hide unnecessary operational complexity by default
- expose advanced detail only when the user asks for it or clearly needs it
- use strong defaults so the primary experience feels obvious and low-friction
- reduce jargon where possible
- make important actions feel safe, clear, and guided

The product should feel closer to consumer-grade product design than to a developer tool full of exposed internal mechanics.

Reference framing:

- iPhone-like simplicity on the surface
- powerful system behavior underneath
- AI for the masses rather than AI for geeks only

## Layout System

### Global App Frame

All primary screens should use the same frame:

- left sidebar for persistent navigation
- compact top bar for environment and gateway status
- large primary canvas for the active operator view
- optional right-side detail drawer for focused inspection

Simplicity rules:

- the main canvas should emphasize one primary job per screen
- secondary detail should move into drawers, expandable sections, or follow-up views
- avoid showing every possible system knob in the default layout

Mission Control reference:

- centralized operations surface and multi-area navigation model

Sonars reference:

- workspace-oriented shell with clear hierarchy and prominent content region

### Sidebar

Purpose:

- stable navigation
- immediate location awareness
- room for gateway/environment context

Behavior:

- visually quiet, darker than content panels in both themes
- active item gets strong contrast, accent edge, and subtle glow
- optional section labels should be small caps or compact uppercase

### Top Bar

Purpose:

- lightweight operational chrome
- always-visible gateway status
- refresh and environment actions

Behavior:

- thin bar, not a heavy app header
- status pills for connection state and environment
- compact action cluster on the right

## Screen-by-Screen Design

### 1. Dashboard

Purpose:

- the main mission-control landing surface

Primary content:

- gateway status
- active agent count
- relationship graph summary
- activity preview
- connection and refresh controls

Recommended composition:

- top summary strip with 3-5 stat cards
- main hero graph occupying most of the viewport
- smaller activity rail or lower-band event panel

Mission Control UI references:

- unified visibility
- gateway-aware orchestration
- activity visibility

Sonars UI references:

- high-impact hero surface
- premium stats panels
- strong hierarchy between primary canvas and secondary details

Visual notes:

- this screen should do the most visual work
- graph area should feel like a centerpiece, not an afterthought
- the screen should feel understandable within a few seconds, even to a non-expert

### 2. Agents Graph

Purpose:

- show active agents and how they relate

Primary content:

- agent nodes
- relationship edges
- active/running status
- hover/focus inspection

Recommended composition:

- large SVG graph canvas with generous breathing room
- deterministic layout
- optional bottom legend or side legend

Mission Control UI references:

- agent operations
- unified control surface

Sonars UI references:

- visual clarity in dense work surfaces
- polished status presentation

Node treatment:

- each node is a premium card, not a plain dot
- include icon, name, state, and one secondary metadata line
- active nodes should feel more luminous and energetic

Edge treatment:

- gateway-native edges are solid and more prominent
- metadata-derived edges are lighter, dashed, or otherwise secondary
- edge labels should only appear where they add clarity

### 3. Sessions Screen

Purpose:

- inspect live or recent execution contexts

Primary content:

- session list
- current state
- owning agent
- timestamps
- quick jump into related graph node or detail view

Mission Control UI references:

- work orchestration and activity review surfaces

Sonars UI references:

- clear work-item lists
- compact but readable detail stacks

Visual notes:

- use dense tabular cards rather than spreadsheet styling
- prioritize scanability and row state contrast
- advanced session detail should stay behind expansion or drill-in, not in the default row presentation

### 4. Activity Screen

Purpose:

- provide a clean system timeline for debugging and operator confidence

Primary content:

- event timeline
- event type filters
- timestamps
- gateway and agent context

Mission Control UI references:

- activity visibility
- audit and incident review orientation

Sonars UI references:

- prominent escalations and high-attention event presentation

Visual notes:

- timeline should read like a premium incident log
- escalated or failed items should stand out immediately

### 5. Devices Screen

Purpose:

- manage trust and pairing

Primary content:

- trusted devices
- pending devices
- revoke and approve actions
- trust metadata

Mission Control UI references:

- governance and approvals

Sonars UI references:

- “escalations when it matters” style visual priority for decision-required items

Visual notes:

- pending approvals should be impossible to miss
- high-risk actions should be visually clear but not alarmist
- trust-management wording should stay simple and human-readable

### 6. Cron Screen

Purpose:

- inspect and operate scheduled automation

Primary content:

- cron list
- enabled state
- target agent
- schedule
- recent run state

Mission Control UI references:

- operational control surface

Sonars UI references:

- compact information blocks with strong primary/secondary text separation

### 7. Settings Screen

Purpose:

- system configuration and diagnostics access

Primary content:

- gateway config
- adapter config
- theme controls
- diagnostics entry points

Mission Control UI references:

- API-backed operations and gateway-aware control

Sonars UI references:

- structured settings surfaces that still feel polished

Visual notes:

- use grouped settings panels
- favor concise labels and clear help text over long forms

## Component-Level Style Guidance

### Status Cards

Should feel:

- dense
- elegant
- immediately legible

Use:

- bold numeric hierarchy
- compact labels
- subtle border and depth separation
- one accent color per state family

Avoid:

- exposing low-level implementation terms in the primary stat label

### Agent Cards

Each card should include:

- distinct icon or monogram
- display name
- primary runtime status
- secondary metadata line
- optional status glow or ring

### Activity Items

Each item should include:

- event icon
- event title
- timestamp
- contextual entity labels
- severity treatment when needed

### Detail Drawer

Use for:

- agent details
- relationship detail
- gateway detail
- device detail

Visual direction:

- blurred or layered separation from main canvas
- strong title area
- compact metadata blocks

Behavior:

- use drawers to hide complexity until needed
- start with a human-readable summary before raw technical details

## Typography

## Recommendation

Use a two-font system:

- `Space Grotesk` for headlines, key stat values, and graph labels
- `Inter` for body text, tables, forms, and supporting UI

Fallback option:

- `Manrope` for headlines if `Space Grotesk` feels too expressive in testing

Why this fits the references:

- Sonars presents as modern, high-performance, and polished; a geometric grotesk plus a clean UI sans aligns well with that tone
- Mission Control’s operational nature benefits from a highly readable UI body face

Usage rules:

- headlines: `Space Grotesk` 600-700
- stat numerals: `Space Grotesk` 600
- tables and settings: `Inter` 400-500
- tiny meta labels: `Inter` 500 with slightly increased tracking

Reference links:

- Space Grotesk: https://fonts.google.com/specimen/Space+Grotesk
- Inter: https://fonts.google.com/specimen/Inter
- Manrope: https://fonts.google.com/specimen/Manrope

## Color System

## Bright Theme

Intent:

- bright, sharp, and clean
- more “studio control room” than “SaaS white page”

Recommended palette:

- background base: `#F5F7FB`
- elevated panel: `#FFFFFF`
- panel border: `#D8E0EA`
- primary text: `#0F1728`
- secondary text: `#526074`
- muted text: `#7A8699`
- accent blue: `#2F6BFF`
- accent cyan: `#17B8D9`
- accent green: `#19A874`
- accent amber: `#E6A700`
- accent red: `#D84C4C`

Graph-specific accents:

- gateway-native edge: `#2F6BFF`
- metadata-derived hint edge: `#8FA8D8`
- active node glow: `rgba(47, 107, 255, 0.18)`

Reference rationale:

- Mission Control suggests an operations-first palette where status matters
- Sonars suggests cleaner premium contrast rather than loud rainbow accents

## Dark Theme

Intent:

- premium control-room atmosphere
- deep surfaces with restrained glow

Recommended palette:

- background base: `#09111F`
- elevated panel: `#101A2C`
- secondary panel: `#132038`
- panel border: `#22314D`
- primary text: `#EDF3FF`
- secondary text: `#A9B7CC`
- muted text: `#7C8AA3`
- accent blue: `#5B8CFF`
- accent cyan: `#35D0E6`
- accent green: `#34C98B`
- accent amber: `#F2B84B`
- accent red: `#FF6B6B`

Graph-specific accents:

- gateway-native edge: `#5B8CFF`
- metadata-derived hint edge: `#60779E`
- active node glow: `rgba(91, 140, 255, 0.22)`

Reference rationale:

- Sonars’ product positioning strongly suggests a dark, premium, native-tool aesthetic
- Mission Control’s governance and gateway focus benefits from strong state color contrast in dark surfaces

## State Colors

Use consistent state mapping across both themes:

- healthy / connected: green
- active / running: blue
- warning / degraded: amber
- error / disconnected: red
- hint / optional metadata: desaturated blue-gray

## Motion

Motion should be inspired by Sonars’ polished “native performance” feel.

Use motion for:

- page entrance
- graph node appearance
- hover/focus emphasis
- refresh transitions
- status changes

Rules:

- durations should stay mostly in the 120ms-220ms range
- no bouncing animations
- use opacity, translation, glow, and scale sparingly
- live status changes should feel crisp, not playful

## Screen-Specific Notes For POC V1

For the first proof of concept, prioritize these screens:

1. Dashboard
2. Agents Graph
3. Lightweight Settings

POC visual bar:

- the dashboard must already feel demo-worthy
- the graph must look intentional and polished even before advanced interactions
- one beautiful graph screen is more important than many average screens

## Explicit Non-Goals

Do not copy the Mission Control UI literally.

Do not copy the Sonars UI literally.

Do not use:

- generic purple gradients on white
- default admin template styling
- crowded KPI dashboards with weak hierarchy
- neon overload in dark mode
- force-directed graph motion in the first slice

## Implementation Notes

Recommended design tokens:

- `--bg-base`
- `--bg-panel`
- `--bg-panel-2`
- `--border-subtle`
- `--text-primary`
- `--text-secondary`
- `--accent-primary`
- `--accent-cyan`
- `--state-success`
- `--state-warning`
- `--state-danger`
- `--graph-edge-primary`
- `--graph-edge-secondary`

Recommended reusable primitives:

- app shell
- sidebar item
- status pill
- stat card
- graph node card
- edge label
- activity item
- detail drawer

## References Summary

Mission Control references used for this document:

- centralized operations and governance framing
- agent operations
- gateway management
- approvals and governance
- activity visibility

Source:

- https://github.com/abhi1693/openclaw-mission-control

Sonars references used for this document:

- premium, modern operator aesthetic
- parallel-workspace hierarchy
- polished panels and strong content hierarchy
- prominent escalations
- fast, native-feeling motion

Sources:

- https://sonars.dev/
- https://sonars.dev/blog/introducing-sonars
- https://sonars.dev/blog/agent-chat-feature
