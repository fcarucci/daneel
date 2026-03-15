# Daneel Agent Guide

## Project Summary

Daneel is a Rust-based Dioxus web application intended to become a mission control UI for OpenClaw.

Today, the repository contains the initial app shell:

- a Dioxus app entrypoint
- typed client-side routes
- a shared layout with sidebar and top bar
- dashboard, agents, settings, and not-found pages
- a first Dioxus server-function slice for gateway status
- a custom CSS theme in `assets/main.css`

The current implementation is an early proof-of-concept scaffold, not a finished product.

## Product Direction

The source of truth for product and architecture intent is in:

- `docs/daneel_requirements.md`
- `docs/daneel_technical_design.md`
- `docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md`

High-level goals:

- single Rust codebase
- Dioxus-based browser UI
- OpenClaw gateway integration through an adapter layer
- polished operator-focused UX
- expanding server-backed data loading and live operational views

## Current Tech Stack

- Rust edition `2024`
- Dioxus `0.7.3`
- Dioxus Router via `dioxus` feature `router`
- Default app platform feature: `web`
- Dioxus app metadata in `Dioxus.toml`
- Tailwind CSS `4.2.1` via `@tailwindcss/cli`
- Tailwind source stylesheet in `styles/app.css`
- Generated application stylesheet in `assets/main.css`
- Dioxus fullstack server functions for backend calls
- OpenClaw loopback WebSocket gateway status fetch

There is currently no full adapter implementation and no automated Rust test suite beyond build and runtime verification.

## Repository Layout

Important files and folders:

- `Cargo.toml`: crate manifest and Dioxus platform features
- `Dioxus.toml`: Dioxus app metadata for web serving
- `src/main.rs`: app entrypoint, stylesheet link, router mount
- `src/gateway.rs`: Dioxus server function and OpenClaw gateway WebSocket status fetch
- `src/models/`: shared Rust models for server/UI state
- `src/router.rs`: typed routes
- `src/components/`: shared shell UI pieces
- `src/pages/`: route-level page components
- `assets/main.css`: app styling
- `scripts/github-admin.mjs`: GitHub labels, milestones, assignments, and project admin helper
- `docs/`: requirements, design, and milestone planning
- `development_setup.md`: local toolchain notes

Current page routes:

- `/`
- `/agents`
- `/settings`
- fallback not-found route

## Local Prerequisites

Expected tools:

- `rustc`
- `cargo`
- `rustfmt`
- `dx` (Dioxus CLI)
- `wasm32-unknown-unknown` Rust target

Useful checks:

```bash
rustc --version
cargo --version
dx --version
rustup target list --installed
```

If Rust binaries are not on `PATH` yet:

```bash
. "$HOME/.cargo/env"
```

## Build Commands

Build the Tailwind CSS bundle:

```bash
npm run build:css
```

Watch and rebuild the Tailwind CSS bundle during UI work:

```bash
npm run watch:css
```

Run the preferred hot-reload development workflow:

```bash
npm run dev
```

This starts both:

- the Tailwind 4 watcher
- the Dioxus web dev server

Format the code:

```bash
cargo fmt --all
```

Verify the crate compiles:

```bash
cargo check
```

Verify the server-enabled fullstack build too:

```bash
cargo check --features server
```

## GitHub Administration

Use the dedicated repo admin script for GitHub maintenance instead of ad hoc shell snippets:

```bash
node scripts/github-admin.mjs help
```

The script reads auth from:

- `GITHUB_TOKEN`
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `~/.env` as a fallback for `GITHUB_PERSONAL_ACCESS_TOKEN`

Typical commands:

```bash
node scripts/github-admin.mjs sync-labels
node scripts/github-admin.mjs remap-poc-milestones
node scripts/github-admin.mjs set-project-status-workflow
node scripts/github-admin.mjs assign-poc --assignee fcarucci
node scripts/github-admin.mjs close-implemented --commit 062d615
node scripts/github-admin.mjs report
```

There is also an npm alias:

```bash
npm run github:admin -- help
```

For Codex approval hygiene, prefer the narrow reusable command prefix:

```text
node scripts/github-admin.mjs
```

That keeps future GitHub admin actions under a single predictable command instead of repeated approvals for generic scripting commands.

## Run Commands

Preferred day-to-day command:

```bash
npm run dev
```

This starts both:

- the Tailwind watcher
- the Dioxus fullstack dev server

Use a fixed local port when you want deterministic browser testing:

```bash
npm run build:css
dx serve --web --fullstack --addr 127.0.0.1 --port 4127 --open false
```

If you are using VS Code port forwarding from a remote session, this variant is usually safest:

```bash
npm run build:css
dx serve --web --fullstack --addr 0.0.0.0 --port 4127 --open false
```

Notes:

- The first web build can take a while because Dioxus may install or compile web tooling.
- `assets/main.css` is generated from `styles/app.css` through Tailwind 4.
- `npm run dev` is the intended hot-reload workflow because it runs both the Tailwind watcher and the Dioxus fullstack dev server together.
- Running only `dx serve --web --fullstack` is not enough for CSS hot reload, because Tailwind output must also be regenerated from `styles/app.css`.
- The app now uses Dioxus server functions, so the fullstack serve path is required for backend-backed UI features.
- `Dioxus.toml` is required so `dx serve` recognizes and serves the web app correctly.

## Test And Verification Workflow

There is not yet a formal Rust test suite, so validation is currently a mix of compile checks and runtime smoke testing.

### Minimum validation

```bash
npm run build:css
cargo fmt --all
cargo check
cargo check --features server
```

### Runtime smoke test

Run the dev server:

```bash
npm run build:css
dx serve --web --fullstack --addr 127.0.0.1 --port 4127 --open false
```

Then verify the server responds:

```bash
curl -I http://127.0.0.1:4127
curl -I http://127.0.0.1:4127/wasm/daneel.js
```

### Headless browser verification

To confirm the page hydrates and renders actual app DOM:

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --dump-dom http://127.0.0.1:4127
```

For route verification:

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --dump-dom http://127.0.0.1:4127/agents
```

Expected signals of success:

- the HTML title is `Daneel`
- the DOM contains app content under `#main`
- the dashboard route includes `Mission Control` and `Dashboard`
- the agents route includes `Agents`
- the hydrated DOM includes a hashed stylesheet under `/assets/main-*.css`
- the served stylesheet begins with the Tailwind banner for `tailwindcss v4`
- the dashboard gateway card renders a real status result from the Dioxus server function

### Gateway status verification

The first live backend slice is the dashboard gateway status card.

Run the app:

```bash
npm run dev
```

Then open `/` and verify the card labeled `Gateway status` renders one of:

- `Connected to the OpenClaw Gateway over WebSocket (healthy).`
- a degraded state with a specific gateway/config error message

Known-good healthy render signals:

- badge text `Healthy`
- `Gateway URL: ws://127.0.0.1:18789/`
- a live uptime value in milliseconds

This flow depends on the local OpenClaw config at `~/.openclaw/openclaw.json`, specifically:

- `gateway.port`
- `gateway.auth.token`

### Screenshot-based verification

To verify the rendered UI visually instead of relying only on DOM output, run the app and capture screenshots with headless Chrome.

Start the app with the full dev workflow:

```bash
npm run dev
```

Capture the home route:

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --window-size=1440,1200 --screenshot=/tmp/daneel-home.png http://127.0.0.1:4127
```

Capture the agents route:

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --window-size=1440,1200 --screenshot=/tmp/daneel-agents.png http://127.0.0.1:4127/agents
```

Capture the dashboard with live gateway status:

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --window-size=1440,1200 --screenshot=/tmp/daneel-dashboard-gateway.png http://127.0.0.1:4127
```

What to verify from the screenshots:

- the page is not blank
- the sidebar renders with Daneel branding and navigation
- the active route is visually highlighted
- the top bar and status pill are visible
- the page-specific content is present
- Tailwind styling is clearly applied to spacing, typography, cards, borders, and colors

Known-good screenshot outputs used during validation:

- `/tmp/daneel-home.png`
- `/tmp/daneel-agents.png`
- `/tmp/daneel-dashboard-gateway.png`

## Known Current State

As of the current scaffold:

- the app shell renders successfully in the browser
- the dashboard and agents routes hydrate correctly
- the dashboard fetches gateway status through a Dioxus server function
- the gateway status request reaches OpenClaw over loopback WebSocket
- styling is loaded through the Dioxus asset pipeline
- there is no OpenClaw adapter yet
- there are no snapshot, unit, or integration tests yet

## Development Guidance

When extending the project:

- keep the app as a Rust-first Dioxus application
- prefer typed routes and shared Rust models
- preserve the separation between route components and shared shell components
- keep visual changes aligned with the existing custom mission-control styling direction
- use the design docs before inventing new architecture
- do not assume backend or OpenClaw behavior exists unless it is implemented in this repo

Planned next major areas from the design docs:

- minimal adapter capability contract
- broader server-function backbone
- graph-oriented agents view
- deterministic testing support

## File-Level Orientation

Quick map of the current UI code:

- `src/main.rs`: launches the app and mounts the router
- `src/gateway.rs`: gateway config loading, WS handshake, and `get_gateway_status()`
- `src/router.rs`: defines `Route` and route labels
- `src/components/layout.rs`: wraps all pages with sidebar and top bar
- `src/components/sidebar.rs`: primary navigation
- `src/components/navbar.rs`: current page title and status pill
- `src/pages/dashboard.rs`: gateway status server-function card and dashboard content
- `src/pages/agents.rs`: placeholder agents page
- `src/pages/settings.rs`: placeholder settings page
- `src/pages/not_found.rs`: fallback route UI
- `assets/main.css`: all current theme and layout styles
- `styles/app.css`: Tailwind 4 source file and theme tokens

## Git Hygiene

Ignored by `.gitignore`:

- `target/`
- editor swap files
- local env files
- common IDE folders
- log files

Do not commit generated build output.

## If You Are Continuing Implementation

Recommended first checks before changing code:

```bash
git status --short
npm run build:css
cargo check
```

Recommended final checks after changes:

```bash
npm run build:css
cargo fmt --all
cargo check
```

If you change routing, rendering, or asset loading, also do a browser smoke test with `dx serve --web`.
