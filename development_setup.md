# Daneel Development Setup

## Overview

Daneel is intended to be a single Rust codebase with:

- a Dioxus fullstack server
- a WASM frontend target
- Tailwind CSS 4.0 for styling
- Playwright-based route verification driven by system `google-chrome`

This document captures the local development setup prepared for this repository on March 14, 2026.

## Installed Toolchain

The following Rust tooling is installed on this machine:

- `rustup 1.29.0`
- `rustc 1.94.0`
- `cargo 1.94.0`
- `clippy`
- `rustfmt`
- `wasm32-unknown-unknown` target

The Dioxus CLI is also being installed for local app scaffolding, development, and production builds.

## Shell Setup

Rust was installed with `rustup`, which places binaries under `$HOME/.cargo/bin`.

If a shell session was opened before the install, reload the environment with:

```bash
. "$HOME/.cargo/env"
```

To make sure the Rust binaries are available:

```bash
rustc --version
cargo --version
```

## Current Installed Targets

Installed Rust compilation targets:

- `x86_64-unknown-linux-gnu`
- `wasm32-unknown-unknown`

The WASM target is required for the browser frontend described in the project requirements.

## Recommended Project Bootstrap

When development begins, a good baseline layout for this repo is:

```text
Cargo.toml
src/
src/main.rs
src/router.rs
src/pages/
src/components/
src/server/
src/gateway/
src/pairing/
```

## Useful Commands

After reloading the shell environment, these are the main commands to verify or extend the setup:

```bash
rustup show
rustup target list --installed
cargo fmt --all
cargo clippy --all-targets --all-features
```

If Dioxus CLI is present:

```bash
dx --version
dx serve
npm install
npm run verify:route -- --help
```

## Notes For Daneel

- The backend and frontend are both expected to live in Rust.
- The frontend will compile to WebAssembly.
- The local setup is aligned with the existing design docs in `daneel_requirements.md` and `docs/daneel_technical_design.md`.
- Tailwind CSS and the Playwright verifier are part of the checked-in frontend workflow now.
- The preferred browser automation setup in this environment is Playwright driving the system `google-chrome` binary.
