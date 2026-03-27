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

## Mandatory Workflow

For any implementation task in this repo, the detailed execution workflow in
`docs/agent-workflows/DANEEL_WORKFLOW.md` is mandatory and must be followed to the letter.

`AGENTS.md` is the repo orientation and policy guide.
`docs/agent-workflows/DANEEL_WORKFLOW.md` is the mandatory task-execution workflow.

If there is any conflict or overlap about task execution order, verification order,
refactoring, review gates, or push/commit readiness, follow
`docs/agent-workflows/DANEEL_WORKFLOW.md`.

**Refactoring (non-negotiable):** At the end of **every** implementation slice—**features and bug fixes** alike—run a dedicated **`refactoring` skill** pass on the touched files (see `skills/refactoring/`), re-run the relevant verification, then proceed to the next gate or commit. Ad-hoc cleanup while coding does not replace this pass. Full ordering and gates are in `docs/agent-workflows/DANEEL_WORKFLOW.md`.

**Proactive:** Do this **without waiting** for the user to say “refactor,” ask for cleanup, or confirm. It is part of the standard completion sequence, not a separate optional step you offer or postpone.

## Current Tech Stack

- Rust edition `2024`
- Dioxus `0.7.3`
- Dioxus Router via `dioxus` feature `router`
- Default app platform feature: `web`
- Dioxus app metadata in `Dioxus.toml`
- Tailwind CSS `4.2.1` via `@tailwindcss/cli`
- Playwright route verification using the system `google-chrome`
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
- `skills/github-admin/`: **Github Admin** skill — use [`skills/github-admin/SKILL.md`](skills/github-admin/SKILL.md) for all GitHub CLI automation (labels, milestones, project board, PRs, issues, releases); Cursor symlink: `.cursor/skills/github-admin`
- `skills/project-management/`: **Project Management** skill — use [`skills/project-management/SKILL.md`](skills/project-management/SKILL.md) to synchronise GitHub Project board status and post issue comments at each workflow lifecycle checkpoint (`started`, `blocked`, `ready-for-merge`, `done`); Cursor symlink: `.cursor/skills/project-management`
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
- `google-chrome`
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

Run the Rust test suite:

```bash
cargo test
```

## GitHub administration

**Prefer the [`github-admin` skill](skills/github-admin/SKILL.md)** (same content at `.cursor/skills/github-admin`) for any GitHub automation this repo already supports: labels, milestones, project board, issues, pull requests, releases, verification comments, and related maintenance. Read that skill before running commands; it holds invocation, auth, typical examples, PR conventions, approval-prefix guidance, and links to per-command reference files.

**For task lifecycle transitions** (starting work, blocking, opening a PR, closing), use the **[`project-management` skill](skills/project-management/SKILL.md)** instead of calling `set-issue-status` and `comment-issue` by hand. The skill documents the exact command sequence for each event and delegates to `github-admin` for execution.

Do not duplicate those instructions in this guide, and do not replace the skill with ad hoc `curl` or one-off scripts when the CLI covers the work.

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

The repository now includes a small frontend SSR test harness for route rendering in `src/test_support.rs`.

### Minimum validation

```bash
npm run build:css
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

### Mock-gateway integration test

The repo now includes an end-to-end integration test that runs through `cargo test`.

It:

- starts a mock OpenClaw-style WebSocket gateway
- writes a temporary OpenClaw config for Daneel
- launches the real Dioxus fullstack app
- verifies route DOM/HTML over HTTP
- verifies the live gateway bridge through the SSE endpoint

Use:

```bash
cargo test --test e2e_mock_gateway
```

Notes:

- this is the preferred `T1.5` end-to-end browser test entrypoint
- it does not depend on the developer's personal OpenClaw data
- it requires `dx` and `npm` on `PATH`
- keep screenshot capture out of this automated test path; the mock-gateway suite should stay deterministic and fail fast

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

Prefer the repo verifier over raw `google-chrome --dump-dom` when you need a trustworthy answer about hydration.

Use the verifier in `scripts/verify-route.mjs` because it:

- uses Playwright to drive the page through normal browser APIs
- uses the system `google-chrome` binary instead of Playwright's bundled browser
- waits for the page to finish hydrating
- waits for the hashed `/assets/main-*.css` stylesheet to be present
- waits for the page background styling to be applied
- lets you declare required text and forbidden text
- writes both a screenshot and the final hydrated DOM

Example for the agents route:

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/agents \
  --screenshot /tmp/daneel-agents-live.png \
  --dom /tmp/daneel-agents-live.html \
  --wait-text "Agent tiles" \
  --wait-text "email" \
  --forbid-text "Loading agents" \
  --forbid-text "Gateway lookup failed"
```

Use raw `google-chrome --dump-dom` only as a quick secondary signal.

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

Important lessons:

- `curl` and raw SSR HTML do not prove hydration; they can still show `Loading agents` or `Connecting`
- a screenshot taken too early can be misleading even when the app is fine
- if the verifier times out, inspect the saved DOM and the dev-server logs before trusting the screenshot result
- if the live route never leaves a loading state, verify that `/wasm/daneel.js` is serving correctly before blaming the UI

### Gateway status verification

The first live backend slice is the dashboard gateway status card.

Run the app:

```bash
npm run dev
```

Then open `/` and verify the card labeled `Gateway status` renders one of:

- `Connected to the OpenClaw Gateway over WebSocket.` once in the top gateway summary row (not repeated in the detail panel when healthy)
- a degraded state with a specific gateway/config error message

Known-good healthy render signals:

- summary row: large `Healthy` plus the connection line above
- detail panel: fetch path, `Gateway URL`, and uptime (no second healthy badge or duplicate headline sentence)
- `Gateway URL: ws://127.0.0.1:18789/`
- a live uptime value in milliseconds

This flow depends on the local OpenClaw config at `~/.openclaw/openclaw.json`, specifically:

- `gateway.port`
- `gateway.auth.token`

### Screenshot-based verification

Use screenshots as a manual visual verification step before committing UI work, especially when checking the live app against real OpenClaw data.

Do not treat screenshot capture as part of the automated mock-gateway integration suite. The automated path should stay focused on DOM/HTML and SSE assertions.

### Reliable live screenshot workflow

Use this order for efficient and trustworthy manual verification:

1. Start a fresh app instance on the fixed port:

```bash
npm start
```

2. Verify the app and browser assets are reachable:

```bash
curl -I http://127.0.0.1:4127
curl -I http://127.0.0.1:4127/wasm/daneel.js
```

3. Capture the route through `scripts/verify-route.mjs`, not raw `--screenshot`, so the capture waits for hydration.

4. Inspect both the screenshot and the saved DOM.

5. Only if the verifier succeeds should the screenshot be treated as the final visual proof.

When checking the real live app, use route-specific wait text that proves real data loaded:

- dashboard: require `Gateway status`
- dashboard: require `Connected to the OpenClaw Gateway over WebSocket`
- agents: require `Agent tiles`
- agents: require one or more real agent ids like `email` or `calendar`
- agents: forbid `Loading agents`
- agents: forbid `Gateway lookup failed`

If the screenshot disagrees with the backend:

- verify the backend directly with the manual live test:

```bash
cargo test --features server live_gateway_status_fetch_reports_healthy -- --ignored --nocapture
```

- inspect the gateway journal for live bridge handshake problems:

```bash
journalctl -u openclaw-gateway.service --since '5 minutes ago' --no-pager
```

This is especially important for the top-right ribbon. A stale dev process can keep sending an old invalid handshake and make the ribbon look degraded even after the source code is fixed.

To verify the rendered UI visually, run the app and capture screenshots with headless Chrome.

Start the app with the full dev workflow:

```bash
npm start
```

Preferred screenshot capture for the home route:

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/ \
  --screenshot /tmp/daneel-home.png \
  --dom /tmp/daneel-home.html \
  --wait-text "Mission Control" \
  --wait-text "Gateway status"
```

Preferred screenshot capture for the agents route:

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/agents \
  --screenshot /tmp/daneel-agents.png \
  --dom /tmp/daneel-agents.html \
  --wait-text "Agent tiles" \
  --wait-text "email" \
  --forbid-text "Loading agents" \
  --forbid-text "Gateway lookup failed"
```

Preferred screenshot capture for the dashboard with live gateway status:

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/ \
  --screenshot /tmp/daneel-dashboard-gateway.png \
  --dom /tmp/daneel-dashboard-gateway.html \
  --wait-text "Gateway status" \
  --wait-text "Connected to the OpenClaw Gateway over WebSocket"
```

What to verify from the screenshots:

- the page is not blank
- the sidebar renders with Daneel branding and navigation
- the active route is visually highlighted
- the top bar and status pill are visible
- the top-right ribbon says `Connected` in green when the live gateway is healthy
- the page-specific content is present
- Tailwind styling is clearly applied to spacing, typography, cards, borders, and colors

What to verify from the saved DOM:

- the required wait text is present in hydrated markup
- loading placeholders are gone
- route-specific failure text is absent
- the status pill text matches the expected state

Avoid this anti-pattern:

- capturing `/tmp/*.png` with raw `google-chrome --screenshot` before hydration and then trusting the image

Environment note:

- keep `google-chrome` installed on the machine and let Playwright drive that binary
- do not rely on Playwright's bundled Chromium in this environment; the system Chrome path is the stable option here

That approach is fast, but it frequently captures SSR-only placeholders and leads to false conclusions.

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
- there is now a mock-gateway cargo integration test for route DOM/HTML and live SSE behavior
- there is no OpenClaw adapter yet
- there is not yet a full Rust unit/integration suite beyond the SSR harness and the mock-gateway browser path

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

## Test Cadence

Prefer a two-speed testing workflow while developing:

- run the fast Rust tests often during implementation
- run the heavier browser integration path before committing

During normal development, use:

```bash
cargo test
```

Before committing code, make sure all formatting and warnings are cleaned up first, then run:

```bash
npm run build:css
cargo fmt --all
cargo check --features server
cargo test --test e2e_mock_gateway
dx serve --web --fullstack --addr 127.0.0.1 --port 4127 --open false
```

If any test is failing or hanging, stop and repair the test or the implementation before committing. Do not commit code while knowingly leaving the suite broken.

If the change affects UI presentation, do a manual live-data visual pass after the automated checks by capturing and inspecting screenshots yourself.

Expectations before commit:

- code is formatted
- warnings are removed, not ignored
- all relevant tests pass before commit; repair broken tests instead of working around them
- the mock-gateway integration test passes
- the app is verified to start successfully with `dx serve --web --fullstack`
- UI changes get a manual screenshot-based visual check against the live app before commit

For the required end-of-task execution order, including the mandatory refactoring pass and
the requirement to run visual acceptance verification last, follow
`docs/agent-workflows/DANEEL_WORKFLOW.md`.
