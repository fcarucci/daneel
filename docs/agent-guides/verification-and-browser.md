# Browser and verification guide

Companion to [`AGENTS.md`](../../AGENTS.md). Use this for hydration checks, Playwright verification, gateway UI signals, and manual screenshots. **Task order, refactor gates, pre-push checks, and memory sweep** stay in [`docs/agent-workflows/DANEEL_WORKFLOW.md`](../agent-workflows/DANEEL_WORKFLOW.md).

## Context

Validation mixes compile checks, `cargo test`, runtime smoke tests, and browser verification. There is a small frontend SSR harness in `src/test_support.rs`.

## Minimum validation (local)

```bash
npm run build:css
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

## Mock-gateway integration test (`T1.5`)

End-to-end test via `cargo test`:

- Starts a mock OpenClaw-style WebSocket gateway
- Writes a temporary OpenClaw config for Daneel
- Launches the real Dioxus fullstack app
- Verifies route DOM/HTML over HTTP
- Verifies the live gateway bridge through the SSE endpoint

```bash
cargo test --test e2e_mock_gateway
```

Notes:

- Does not depend on personal OpenClaw data
- Requires `dx` and `npm` on `PATH`
- Keep screenshot capture out of this path; stay deterministic and fail fast

## Runtime smoke test

With dev server on `127.0.0.1:4127`:

```bash
npm run build:css
dx serve --web --fullstack --addr 127.0.0.1 --port 4127 --open false
```

```bash
curl -I http://127.0.0.1:4127
curl -I http://127.0.0.1:4127/wasm/daneel.js
```

## Headless browser verification

Prefer the repo verifier (`scripts/verify-route.mjs`) over raw `google-chrome --dump-dom` when you need trustworthy hydration results. It:

- Uses Playwright with the **system** `google-chrome` binary (not Playwright’s bundled Chromium)
- Waits for hydration, hashed `/assets/main-*.css`, and background styling
- Accepts required and forbidden text; writes screenshot + hydrated DOM

**Pattern** (adjust `--url`, `--wait-text`, `--forbid-text`, and output paths per route):

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/<path> \
  --screenshot /tmp/daneel-<route>.png \
  --dom /tmp/daneel-<route>.html \
  --wait-text "<first signal>" \
  --wait-text "<second signal>" \
  --forbid-text "<optional bad state>"
```

**Route-specific wait/forbid hints (live OpenClaw data):**

| Route   | URL    | Typical `--wait-text` | Forbid (agents) |
|---------|--------|------------------------|-----------------|
| Home    | `/`    | `Mission Control`, `Gateway status` | — |
| Dashboard gateway | `/` | `Gateway status`, `Connected to the OpenClaw Gateway over WebSocket` | — |
| Agents  | `/agents` | `Agent tiles`, real agent id (e.g. `email`) | `Loading agents`, `Gateway lookup failed` |

Concrete examples:

```bash
# Agents
npm run verify:route -- \
  --url http://127.0.0.1:4127/agents \
  --screenshot /tmp/daneel-agents.png \
  --dom /tmp/daneel-agents.html \
  --wait-text "Agent tiles" \
  --wait-text "email" \
  --forbid-text "Loading agents" \
  --forbid-text "Gateway lookup failed"

# Home
npm run verify:route -- \
  --url http://127.0.0.1:4127/ \
  --screenshot /tmp/daneel-home.png \
  --dom /tmp/daneel-home.html \
  --wait-text "Mission Control" \
  --wait-text "Gateway status"

# Dashboard — live gateway line
npm run verify:route -- \
  --url http://127.0.0.1:4127/ \
  --screenshot /tmp/daneel-dashboard-gateway.png \
  --dom /tmp/daneel-dashboard-gateway.html \
  --wait-text "Gateway status" \
  --wait-text "Connected to the OpenClaw Gateway over WebSocket"
```

Raw Chrome (quick secondary signal only):

```bash
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --dump-dom http://127.0.0.1:4127
timeout 25s google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=15000 --dump-dom http://127.0.0.1:4127/agents
```

### Expected success signals

- HTML title `Daneel`
- App content under `#main`
- Dashboard: `Mission Control`, `Dashboard`
- Agents route: `Agents`
- Hydrated DOM: hashed stylesheet `/assets/main-*.css`
- Stylesheet begins with Tailwind banner for `tailwindcss v4`
- Dashboard gateway card shows a real Dioxus server-function result (not stuck loading)

### Lessons

- `curl` and raw SSR HTML do not prove hydration
- Screenshots before hydration are misleading
- Verifier timeout → inspect saved DOM and dev-server logs
- Route stuck loading → confirm `/wasm/daneel.js` serves before blaming UI

## Gateway status card (manual)

Run the app (`npm run dev` or `npm start`), open `/`, and check the **Gateway status** card:

- Healthy: `Connected to the OpenClaw Gateway over WebSocket.` once in the summary row (not duplicated in the detail panel)
- Otherwise: degraded with a specific gateway/config message

Healthy signals:

- Summary: large `Healthy` plus the connection line
- Detail: fetch path, `Gateway URL`, uptime (no duplicate healthy badge/headline)
- Example URL line: `Gateway URL: ws://127.0.0.1:18789/`
- Live uptime in milliseconds

Depends on local OpenClaw config `~/.openclaw/openclaw.json`: `gateway.port`, `gateway.auth.token`.

## Manual screenshots (live data)

Use screenshots before committing UI work when validating against real OpenClaw. **Do not** fold screenshot capture into the mock-gateway automated suite.

**Reliable sequence:**

1. Fresh app on fixed port: `npm start` (or `npm run build:css` + `dx serve` as in [`AGENTS.md`](../../AGENTS.md))
2. `curl -I` on `/` and `/wasm/daneel.js`
3. Capture with `scripts/verify-route.mjs` (not raw `--screenshot` before hydration)
4. Review screenshot and saved DOM; treat as proof only if the verifier succeeded

If UI disagrees with backend:

```bash
cargo test --features server live_gateway_status_fetch_reports_healthy -- --ignored --nocapture
journalctl -u openclaw-gateway.service --since '5 minutes ago' --no-pager
```

A stale dev process can keep a bad handshake and degrade the ribbon; restart the app if needed.

### Screenshot / DOM checklist

- Not blank; sidebar, nav highlight, top bar, status pill
- Ribbon `Connected` (green) when gateway healthy
- Page content and Tailwind styling visible
- DOM: required wait text present, loading placeholders gone, no forbidden failure strings

**Anti-pattern:** raw `google-chrome --screenshot` before hydration — often captures SSR placeholders.

**Environment:** keep system `google-chrome` installed; Playwright should use that path, not bundled Chromium.

Known-good artifact names from validation runs: `/tmp/daneel-home.png`, `/tmp/daneel-agents.png`, `/tmp/daneel-dashboard-gateway.png`.
