---
name: github-admin
description: Runs the Daneel GitHub admin CLI for labels, milestones, project board sync, issues, pull requests, releases, and review threads against the GitHub REST and GraphQL APIs. Use when the user or workflow needs GitHub repository or project maintenance, creating or updating PRs and issues (including bodies from files), syncing labels or milestones, posting verification comments, or any task matching the commands in this skill's reference.
---

# Github Admin

## Prefer this skill first

Repository guides point here on purpose: **load this file before** running GitHub maintenance for Daneel. Use [reference.md](reference.md) and only the [references/commands/](references/commands/) pages you need so context stays small. Do not improvise ad hoc `curl`, throwaway `/tmp` API scripts, or **new domain-specific subcommands** in `github-admin.mjs` (e.g. per-plan or per-project issue generators). Prefer **generic** commands (`create-issue`, `create-project`, `update-issue`, `delete-issue`, `list-prs`, `project`, etc.) and drive GitHub **directly** when you need one-off edits—do not wire repo markdown plans into the admin script for that.

Use this skill when an agent should perform **repository automation** or **project management** for the Daneel project via the checked-in Node script instead of manual UI steps or raw API experiments.

## Authentication and configuration

Details: [references/environment.md](references/environment.md).

- **Token:** `GITHUB_TOKEN` or `GITHUB_PERSONAL_ACCESS_TOKEN`; optional fallback from `~/.env` (`export GITHUB_PERSONAL_ACCESS_TOKEN=…`) per script `envFileToken()`.
- **`GITHUB_REPOSITORY`:** default `fcarucci/daneel`.
- **`GITHUB_PROJECT_NUMBER`:** default `1` (GraphQL project helpers).

For large issue bodies, use **`create-issue`** / **`update-issue`** with `--body-file <path>` instead of inlining markdown in the shell (see [references/commands/create-issue.md](references/commands/create-issue.md) and [update-issue.md](references/commands/update-issue.md)).

## How to run the CLI

From the repository root:

```bash
node skills/github-admin/scripts/github-admin.mjs <command> [options]
```

- `help` (or no command) — print full usage (exit 0 for `help`).

**npm:** `npm run github:admin -- <command> [options]` runs the same script. For **Codex or similar clients** where approvals are per command prefix, prefer the **`node skills/github-admin/scripts/github-admin.mjs …`** form so one narrow prefix covers all subcommands instead of repeated generic `npm` approvals.

**Approval hygiene (optional):** Approve a single reusable prefix once if your client supports it:

```text
node skills/github-admin/scripts/github-admin.mjs
```

For PR creation only:

```text
node skills/github-admin/scripts/github-admin.mjs create-pr
```

## Typical maintenance commands

```bash
node skills/github-admin/scripts/github-admin.mjs help
node skills/github-admin/scripts/github-admin.mjs sync-labels
node skills/github-admin/scripts/github-admin.mjs list-issues --limit 50
node skills/github-admin/scripts/github-admin.mjs report
```

Adjust flags and SHAs to the task at hand; full command list: [reference.md](reference.md).

## Pull requests (Daneel task workflow)

Align PR titles with the repository task workflow: **`[<Task tag>] Title`**, for example `[T2.7] Make agent recency and heartbeat state update live`.

Create a PR (default base `main`; link issue when applicable):

```bash
node skills/github-admin/scripts/github-admin.mjs create-pr --head <branch> --base main --title "[<Task tag>] Title" --issue <n> --body "<markdown>"
```

See [references/commands/create-pr.md](references/commands/create-pr.md) for all flags. After opening the PR, link the task to the merge request and set project status per your workflow doc.

## Command reference

- **Index:** [reference.md](reference.md) → per-command files under [`references/commands/`](references/commands/).
- **Script mechanics:** [references/script.md](references/script.md).
- **Environment:** [references/environment.md](references/environment.md).

**When adding commands or flags:** update `usage()` in `skills/github-admin/scripts/github-admin.mjs`, add or edit `references/commands/<command>.md`, and update the index in [reference.md](reference.md) if the command set changes.
