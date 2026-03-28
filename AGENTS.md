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

## Agent role (Rust + frontend)

When implementing in this repo, work as a **strong Rust developer** with a **frontend specialty**: idiomatic Rust (clear types, error handling, and tests where they add signal), Dioxus **0.7** (components, state, routing, fullstack server functions and their boundaries), **CSS**, and **Tailwind 4** (`styles/app.css` as the source of truth; regenerate `assets/main.css` via `npm run build:css`—do not hand-edit generated CSS). Prefer existing theme tokens and layout patterns; ship UI that is cohesive, accessible (focus, contrast, semantics), and aligned with the mission-control visual direction in the design docs.

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

Rust 2024, Dioxus `0.7.3` (feature `router`, default platform `web`), Dioxus fullstack server functions, Tailwind CSS `4.2.1` via `@tailwindcss/cli` (`styles/app.css` → `assets/main.css`), Playwright route verification with **system** `google-chrome`, OpenClaw loopback WebSocket gateway status fetch. Metadata in `Dioxus.toml`.

There is no full adapter yet; automated coverage is build checks, `cargo test`, mock-gateway E2E, and the procedures in [`docs/agent-guides/verification-and-browser.md`](docs/agent-guides/verification-and-browser.md).

## Repository layout

| Path | Role |
|------|------|
| `Cargo.toml`, `Dioxus.toml` | Crate manifest, Dioxus web metadata |
| `src/main.rs` | Entrypoint, stylesheet link, router mount |
| `src/router.rs` | `Route` and labels |
| `src/gateway.rs` | Server function, gateway config, WS handshake, `get_gateway_status()` |
| `src/models/` | Shared Rust models for server/UI state |
| `src/components/` | Shell: `layout.rs`, `sidebar.rs`, `navbar.rs` |
| `src/pages/` | `dashboard.rs`, `agents.rs`, `settings.rs`, `not_found.rs` |
| `assets/main.css`, `styles/app.css` | Built theme; Tailwind 4 source and tokens |
| `src/test_support.rs` | SSR route test harness |
| `docs/agent-guides/` | Detailed agent procedures ([`verification-and-browser.md`](docs/agent-guides/verification-and-browser.md)) |
| `docs/` | Requirements, design, milestones |
| `development_setup.md` | Local toolchain notes |
| [`skills/github-admin/SKILL.md`](skills/github-admin/SKILL.md) | GitHub CLI automation |
| [`skills/project-management/SKILL.md`](skills/project-management/SKILL.md) | Issue/PR lifecycle (`started`, `blocked`, `ready-for-merge`, `done`) |
| [`skills/memory/SKILL.md`](skills/memory/SKILL.md) | Memory store/query/maintain (see **Agent memory**) |
| `MEMORY.md` | Curated previews; full entries in `memory/*.md` (memory skill) |

**Routes:** `/`, `/agents`, `/settings`, fallback not-found.

## Local prerequisites

`rustc`, `cargo`, `rustfmt`, `dx`, `google-chrome`, `wasm32-unknown-unknown`. Reload Rust on `PATH` if needed: `. "$HOME/.cargo/env"`. Version checks: `rustc --version`, `cargo --version`, `dx --version`, `rustup target list --installed`. More detail: `development_setup.md`.

## Development commands

**Daily dev** (Tailwind watcher + fullstack dev server):

```bash
npm run dev
```

**CSS:** `npm run build:css` — one-off build; `npm run watch:css` — watch during UI work.

**Rust:**

```bash
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

**Fixed port** (deterministic browser testing; build CSS first if not using `npm run dev`):

```bash
npm run build:css
dx serve --web --fullstack --addr 127.0.0.1 --port 4127 --open false
```

**Remote / port forwarding** (bind all interfaces):

```bash
npm run build:css
dx serve --web --fullstack --addr 0.0.0.0 --port 4127 --open false
```

**Notes:** First web build can be slow. `npm run dev` is preferred for hot reload (Tailwind + Dioxus). `dx serve` alone does not rebuild CSS from `styles/app.css`. Server functions require **fullstack** serve. `Dioxus.toml` is required for `dx serve` to serve the app correctly.

## GitHub administration

**Prefer the [`github-admin` skill](skills/github-admin/SKILL.md)** for any GitHub automation this repo already supports: labels, milestones, project board, issues, pull requests, releases, verification comments, and related maintenance. Read that skill before running commands; it holds invocation, auth, typical examples, PR conventions, approval-prefix guidance, and links to per-command reference files.

**For task lifecycle transitions** (starting work, blocking, opening a PR, closing), **spawn a subagent** that reads and follows the **[`project-management` skill](skills/project-management/SKILL.md)**. Pass the event name (`started`, `blocked`, `ready-for-merge`, or `done`) together with the relevant context (issue number, branch, PR number, summary). Never call `set-issue-status` or `comment-issue` directly for lifecycle transitions — delegate to the skill subagent.

Do not duplicate those instructions in this guide, and do not replace the skill with ad hoc `curl` or one-off scripts when the CLI covers the work.

## Agent memory

- Use **only** [`skills/memory/SKILL.md`](skills/memory/SKILL.md) (and its `ref/` docs, including [`skills/memory/ref/config.md`](skills/memory/ref/config.md) for host routing such as `MEMORY_SKILL_HOST` and `memory-skill.config.json`) for remember/show/recall/reflect/maintain/promote. Do not copy memory CLI or workflows into this file.
- **Store or maintain:** spawn a subagent per the skill. **Read-only** questions about memories: use `show` / `recall` per the skill. Do not edit `MEMORY.md` or `memory/*.md` by hand for routine writes.
- Post-task memory sweep and ordering: [`docs/agent-workflows/DANEEL_WORKFLOW.md`](docs/agent-workflows/DANEEL_WORKFLOW.md).

## Test and verification

**Deep procedures** (mock-gateway E2E, hydration, `verify-route`, gateway card, screenshots): [`docs/agent-guides/verification-and-browser.md`](docs/agent-guides/verification-and-browser.md).

**Quick local loop** (not a substitute for `DANEEL_WORKFLOW.md` pre-push gates):

```bash
npm run build:css
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

**Mock-gateway E2E:** `cargo test --test e2e_mock_gateway` (needs `dx` and `npm` on `PATH`).

**Before commit:** follow `docs/agent-workflows/DANEEL_WORKFLOW.md` for the full sequence (refactoring pass, re-verification, visual acceptance last, memory sweep). For heavier checks before pushing, that workflow’s pre-push section is authoritative.

## Known current state

- App shell, dashboard, and agents hydrate; gateway status via server function and loopback WebSocket
- Styling via Dioxus asset pipeline; mock-gateway integration test for DOM/HTML and SSE
- No OpenClaw adapter yet; no large Rust integration suite beyond SSR harness and mock-gateway path

## Development guidance

Rust-first Dioxus; typed routes and shared models; keep route vs shell separation; align visuals with mission-control styling; follow design docs before new architecture; do not assume OpenClaw behavior unless implemented here.

Planned (from design docs): adapter contract, server-function backbone, graph-oriented agents view, richer deterministic testing.

## Git hygiene

`.gitignore` includes `target/`, editor swaps, local env files, common IDE dirs, logs. Do not commit generated build output.

## If you are continuing implementation

Before changes: `git status --short`, `npm run build:css`, `cargo check`. After changes: `npm run build:css`, `cargo fmt --all`, `cargo check`. Routing/rendering/asset changes: browser smoke with `dx serve --web` (or fullstack as above).

## Test cadence

Run `cargo test` often while coding; run `cargo test --test e2e_mock_gateway` and workflow-defined gates before push. UI changes: manual hydrated verification and screenshots per [`docs/agent-guides/verification-and-browser.md`](docs/agent-guides/verification-and-browser.md).
