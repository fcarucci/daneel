# Daneel — Project Memory

## What It Is
Daneel is Francesco's active coding project — a **Rust/Dioxus mission control UI for OpenClaw**.
Location: `~/dev/daneel/`

A browser-based operator dashboard for monitoring and controlling an OpenClaw gateway.

## Tech Stack
- **Language:** Rust 2024
- **Frontend:** Dioxus 0.7 (compiled to WASM)
- **Backend:** Dioxus fullstack server (server functions)
- **Styling:** Tailwind CSS 4.x
- **Storage:** SQLite (local persistence only)
- **Gateway:** OpenClaw over loopback WebSocket

## Dev Commands
```bash
npm start          # full dev server at http://127.0.0.1:4127
npm run dev        # lighter dev workflow

# Validation (must pass before MR):
npm run build:css
cargo fmt --all
cargo clippy -- -D warnings
cargo check
cargo check --features server
cargo test --bin daneel
cargo test --test e2e_mock_gateway

# Server build smoke test (must pass before MR):
# Verifies the full WASM + server compile succeeds end-to-end
dx build --features server 2>&1 | tail -5
# Expected: "Finished" with no errors
```

## Key Docs
- `~/dev/daneel/AGENTS_WORKFLOW.md` — task workflow (branches, MRs, refactoring)
- `~/dev/daneel/README.md` — project overview + setup
- `~/dev/daneel/docs/daneel_technical_design.md` — full TDD
- `~/dev/daneel/docs/daneel_requirements.md` — PRD
- `~/dev/daneel/docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md` — current milestone

---

## ⚠️ Daneel Task Execution Workflow (MANDATORY)

When Francesco asks to complete a task in Daneel, ALWAYS follow this exact process:

### Model selection
- **Implementation model: DeepSeek v3.1** (`nvidia/deepseek-ai/deepseek-v3.1`)
- **Code review model: DeepSeek V3.1** (`nvidia/deepseek-ai/deepseek-v3.1`) — DeepSeek is used for both implementation and review gate

1. **Spawn a DeepSeek subagent** (`nvidia/deepseek-ai/deepseek-v3.1`) to do the implementation
2. **Clone the repo to a temp location** (e.g. `/tmp/daneel-<task>`) — never work in `~/dev/daneel` directly
3. **Set GitHub issue → In Progress** before starting
4. **Follow `AGENTS_WORKFLOW.md` to the letter:**
   - Branch: `task/<tag>`
   - Implement → unit tests during development
   - Integration tests + visual verification
   - **Code review (BLOCKING GATE):** spawn a DeepSeek V3.1 subagent (`nvidia/deepseek-ai/deepseek-v3.1`) — give it the **exact task spec** from the breakdown doc AND the full diff (`git diff origin/main`). DeepSeek must output a **structured checklist** with an explicit PASS/FAIL verdict for each of the following:
     1. Every spec requirement is implemented (not just claimed)
     2. Every test actually validates the claimed behavior (not just exercises code paths)
     3. No compile errors in any build target (client WASM, server, tests)
     4. No new `unwrap()`/`expect()` without justification
     5. No dead code, debug artifacts, or `TODO`s left in
     6. For boundary tasks: the boundary is enforced by the type system, not just by convention
     The implementor must fix every FAIL, push the fix to the branch, and request re-review. **Push to origin is blocked until DeepSeek explicitly outputs "ALL CHECKS PASS".**
   - For **boundary/architectural tasks**: require at least one *failing compile test* that proves the boundary exists — a test module that tries to import transport-specific or OpenClaw-specific types from UI-facing code must fail to compile, confirming the boundary is enforced, not just claimed.
   - **Refactoring pass (mandatory for every implementation):** applies to **new features, bug fixes, and any follow-up code change** before you treat the slice as done. Run the `refactoring` skill on all files touched in that slice (see `skills/refactoring/` in repo). There is **no** exception for “small” or “just a bugfix” work.
   - Remove dead/debug/redundant code
   - **Mandatory final sequence (in this exact order):**
     1. reach a first green implementation run (tests / behavior prove the fix or feature)
     2. run the **dedicated** refactoring pass (`refactoring` skill)
     3. rerun fmt + unit + integration after the refactor
     4. run visual acceptance verification last
     5. only then commit or push
   - Opportunistic cleanup during implementation does **not** count as the refactoring pass
   - Visual acceptance verification must happen after the post-refactor automated re-check, not before
5. **Rebase onto main before pushing:**
   - `git fetch origin && git rebase origin/main`
   - Resolve all conflicts
   - Re-run full verification: fmt + unit + integration
   - Confirm everything still passes
6. **Push branch** → delete the temp clone
7. **Send a Telegram recap** here with: branch name, PR link, what changed, verbatim test result line

---

## ⚠️ Test Honesty Rule (NON-NEGOTIABLE)

**The subagent MUST report the EXACT `cargo test` output — never paraphrase, never assume, never fabricate.**

- Always paste the literal `test result: ok. X passed; Y failed` line in the summary
- Do NOT report "all tests pass" until the output literally says `0 failed`
- If tests time out or are killed: say so explicitly — do not report success
- If any test step is skipped for any reason: say so explicitly
- Fix every failing test before pushing — no exceptions
- **Run tests like this so output is captured and unambiguous:**
  ```bash
  npm install && npm run build:css   # MANDATORY — tests fail without CSS asset
  cargo test --bin daneel 2>&1 | tee /tmp/daneel-test-results.txt
  cat /tmp/daneel-test-results.txt | grep -E "test result|FAILED|error\[" | head -20
  ```
  Include that grep output verbatim in the final summary.

**The orchestrating agent (Gilfoyle) will independently verify by cloning the pushed branch and running `cargo test --bin daneel` before reporting to the user. If results don't match, the task is not done.**

---

## ⚠️ Test Quality Rule (NON-NEGOTIABLE)

A passing test suite is necessary but not sufficient. Tests must actually validate the claimed behavior:

- **For every new public function or trait**: there must be a test that would fail if the implementation were deleted or trivially stubbed
- **For boundary/architectural tasks**: there must be a compile test that proves the boundary is enforced by the type system — not just that the code compiles and runs
- **Test count increases must be justified**: if the test count jumps significantly, each new test must have a clear name and purpose. Do not pad test counts with trivial happy-path tests that add no signal.
- **The review checklist (DeepSeek) must explicitly confirm** that each test would catch a real regression, not just that tests exist

---

## ⚠️ Environment Verification Gate (MANDATORY — run FIRST before any work)

Before writing a single line of code, the subagent must confirm the environment is functional by running:

```bash
cd /tmp/daneel-<task>

# MANDATORY: use shared target dir — each clone's own target/ grows to 20GB+
export CARGO_TARGET_DIR=/root/.cargo/daneel-target

npm install && npm run build:css 2>&1 | tail -2
. $HOME/.cargo/env && cargo --version && dx --version
cargo check 2>&1 | tail -3
```

Expected output:
- `Done in Xs` from build:css
- `cargo X.Y.Z` and `dx X.Y.Z` from version checks
- `Finished` from cargo check

**If any of these fail (toolchain not found, cargo not available, dx missing): STOP. Report a roadblock immediately. Do NOT proceed and do NOT fabricate results.**

---

## ⚠️ Pre-Push Gate (MANDATORY — all five must pass before `git push`)

Before entering the pre-push gate, the implementation workflow must already have completed the dedicated refactoring pass and the post-refactor automated re-verification. Do not use the checks below as a substitute for that step.

Before pushing, run each check, pipe to file, grep the result, and confirm the required string is present:

```bash
# Ensure shared target dir is set (prevents 20GB+ accumulation per clone)
export CARGO_TARGET_DIR=/root/.cargo/daneel-target

# 1. Clippy — ccp compresses output dramatically
ccp cargo clippy -- -D warnings 2>&1 | tee /tmp/daneel-clippy.txt
grep "Finished" /tmp/daneel-clippy.txt || { echo "CLIPPY FAILED — DO NOT PUSH"; exit 1; }

# 2. Tests (CSS must be built first) — ccp: 283 lines → 2 lines
npm run build:css 2>/dev/null
ccp cargo test --bin daneel 2>&1 | tee /tmp/daneel-tests.txt
grep "0 failed" /tmp/daneel-tests.txt || { echo "TESTS FAILED — DO NOT PUSH"; exit 1; }

# 3. WASM compile check — fast correctness check (217 lines → 1 line via ccp)
# NOTE: dx build uses its own profile subdir and does NOT reuse cargo build artifacts
ccp cargo build --target wasm32-unknown-unknown 2>&1 | tee /tmp/daneel-wasm.txt
grep "ok" /tmp/daneel-wasm.txt || { echo "WASM BUILD FAILED — DO NOT PUSH"; exit 1; }

# 4. Server build — ccp: compiles clean, dx build --features server has a known Dioxus 0.7 framework
# bug where mio is compiled for WASM (pre-existing on all of main, not fixable in this repo)
ccp cargo build --features server 2>&1 | tee /tmp/daneel-server.txt
grep "ok" /tmp/daneel-server.txt || { echo "SERVER BUILD FAILED — DO NOT PUSH"; exit 1; }

# 5. Dev server smoke test — proves the app actually starts and is reachable at runtime
#    This catches broken server startup that cargo build misses (missing config, runtime panics, etc.)
fuser -k 4127/tcp 2>/dev/null || true
timeout 90 bash scripts/dev.sh 4127 127.0.0.1 &
DEV_PID=$!
SERVER_OK=0
for i in $(seq 1 45); do
  curl -sf http://127.0.0.1:4127/ > /dev/null 2>&1 && SERVER_OK=1 && break || sleep 2
done
kill $DEV_PID 2>/dev/null || true
wait $DEV_PID 2>/dev/null || true
[ $SERVER_OK -eq 1 ] || { echo "DEV SERVER FAILED TO START — DO NOT PUSH"; exit 1; }
echo "dev server smoke test: PASSED"

echo "ALL CHECKS PASSED — safe to push"
```

Only run `git push` after seeing `ALL CHECKS PASSED`. Paste the grep/echo output for all five checks verbatim in the final summary.
rm -rf /tmp/daneel-<task>
```

---

## ⚠️ Issue Resolution Rule (MANDATORY for subagent)

Fix issues as they arise — never stop or report failure. Specifically:
- Test fails → diagnose, fix, re-run. Repeat until `0 failed`.
- Compile error → fix and re-check
- Rebase conflict → resolve, re-verify
- Only stop as a **roadblock** if genuinely unresolvable without external input (undocumented API, architectural conflict, missing credentials)
- Never leave the branch in a broken state

---

## Architecture
```
Browser (WASM) → Daneel Server → Gateway Adapter → OpenClaw Gateway
```
- Browser never touches the gateway directly
- Server functions = primary communication pattern (request/response)
- WebSockets/SSE = only for live push updates where polling is insufficient

## Current Milestone: POC V1
Goal: webapp runs, connects to OpenClaw, fetches agents, renders polished agent graph.
- node = configured OpenClaw agent
- edge = routing/binding relationships + optional local metadata hints (AGENTS.md)
- Start with deterministic SVG layout (no force-directed physics in POC V1)

## Config (daneel.toml)
- Default: `daneel.toml` in working dir, or `~/.config/daneel/daneel.toml`
- Required: `gateway_url`, `gateway_auth_token`
- Optional: `listen_port` (default 8090), `adapter_type`, `theme_default`, `sqlite_path`
- Gateway config at `~/.openclaw/openclaw.json` → `gateway.port` + `gateway.auth.token`

## Theme System
- Semantic tokens only in components (never raw colors)
- `theme/tokens.rs` → raw palette
- `theme/semantics.rs` → semantic names (surface_primary, text_muted, accent_primary, state_success)
- `theme/registry.rs` → theme registration/selection
- `assets/themes.css` → CSS custom properties + `[data-theme="dark"]` selectors
- Active theme via Dioxus context provider, reflected as `data-theme` on root element

## Routes
`/`, `/sessions`, `/sessions/:id`, `/agents`, `/agents/:id`, `/activity`, `/devices`, `/cron`, `/cron/:id`, `/devices/:id`, `/settings`
