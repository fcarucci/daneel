# Contributing

Thanks for contributing to Daneel.

## Ground Rules

- Keep the project Rust-first and Dioxus-first.
- Prefer typed models and server-function flows over ad hoc JSON plumbing in the UI.
- Preserve the existing operator-focused visual direction unless a change is intentional and cohesive.
- Keep gateway credentials server-side only.

## Setup

Install the core prerequisites:

- `rustc`
- `cargo`
- `dx`
- `npm`
- `google-chrome`

This repository is expected to be developed the same way it is used here:

- directly inside an OpenClaw machine or instance
- with the OpenClaw gateway available on the same host over loopback
- with Daneel reading gateway config from `~/.openclaw/openclaw.json`

Verify the OpenClaw config is present:

```bash
test -f "$HOME/.openclaw/openclaw.json"
```

Verify the gateway config values Daneel depends on:

```bash
jq '.gateway.port, .gateway.auth.token' "$HOME/.openclaw/openclaw.json"
```

Verify the local toolchain:

```bash
. "$HOME/.cargo/env"
rustc --version
cargo --version
dx --version
rustup target list --installed
```

Make sure the WASM target is installed:

```bash
rustup target add wasm32-unknown-unknown
```

Install frontend dependencies:

```bash
npm install
```

This installs the local Playwright package used by the repo verifier. In this environment, the verifier should drive the system `google-chrome` binary rather than Playwright's bundled browser.

Recommended first checks:

```bash
npm run build:css
cargo check
cargo check --features server
cargo test
```

## Development Workflow

Run the app the same way this project is normally run in development:

```bash
npm start
```

This starts the full local workflow on:

```text
http://127.0.0.1:4127
```

It is the expected contributor path because it:

- builds Tailwind CSS
- starts the Tailwind watcher
- starts the Dioxus fullstack dev server
- uses the same fixed local port we use for browser testing and verification

`npm run dev` is currently equivalent and uses the same fixed-port workflow:

```bash
npm run dev
```

If you are developing through VS Code Remote or another remote shell environment, forward port `4127` and open the forwarded URL locally.

Notes for this runtime setup:

- Daneel talks to OpenClaw through Dioxus server functions.
- The browser should not connect to the OpenClaw gateway directly.
- If the gateway is unavailable or misconfigured, backend-backed routes will render degraded or failed states instead of live data.

Before sending changes, run:

```bash
npm run build:css
cargo fmt --all
cargo check
cargo check --features server
cargo test
```

If you change rendering or route behavior, include the browser integration pass too:

```bash
cargo test --test e2e_mock_gateway
```

For manual visual verification, use the route verifier:

```bash
npm run verify:route -- \
  --url http://127.0.0.1:4127/agents \
  --screenshot /tmp/daneel-agents.png \
  --dom /tmp/daneel-agents.html \
  --wait-text "Agent tiles" \
  --wait-text "email" \
  --forbid-text "Loading agents"
```

## Pull Request Expectations

- Keep changes focused.
- Explain user-visible behavior changes clearly.
- Mention any known limitations or follow-up work.
- Include screenshots for meaningful UI changes when practical.
- Avoid bundling unrelated refactors into the same PR.

## Code Style

- Prefer clear, typed Rust models.
- Keep route-level code in `src/pages/`.
- Keep shared shell UI in `src/components/`.
- Add comments sparingly and only when they reduce real confusion.

## License

By contributing to this repository, you agree that your contributions are provided under the Apache License, Version 2.0.
