---
name: project-management
description: >
  Synchronises GitHub Project status and issue comments whenever a task or issue
  transitions lifecycle state (started, blocked, ready-for-merge, done).
  Use when the workflow reaches a status-transition checkpoint: starting a branch,
  opening a PR, merging, or blocking work.  Depends on the github-admin skill.
---

# Project Management

## When to invoke

Use this skill **as a subagent** at each lifecycle checkpoint listed below.
The calling workflow provides the issue number and the event name.

| Checkpoint in workflow | Event |
|------------------------|-------|
| Branch created, work begins | `started` |
| Work is blocked by an external dependency | `blocked` |
| PR opened and all verification passes | `ready-for-merge` |
| PR merged or issue closed | `done` |

## Prerequisites

Read [`skills/github-admin/SKILL.md`](../github-admin/SKILL.md) (or its
`.cursor` symlink) before running any commands.  Auth, repo defaults, and
approval-prefix guidance live there.

## Step-by-step for each event

### `started` — work begins

```bash
# 1. Set project board status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "In Progress"

# 2. Comment on the issue with the branch being worked on
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "Work started on branch \`<BRANCH_NAME>\`."
```

### `blocked` — work cannot proceed

```bash
# 1. Set project board status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Blocked"

# 2. Comment with the blocking reason
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "Blocked: <REASON>"
```

### `ready-for-merge` — PR is open, work is complete

```bash
# 1. Set project board status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Ready for Merge"

# 2. Link PR to issue (adds closing keyword + issue comment)
node skills/github-admin/scripts/github-admin.mjs \
  link-pr-task --pr <PR_NUMBER> --issue <ISSUE_NUMBER> --close
```

### `done` — PR merged or issue closed manually

```bash
# 1. Set project board status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Done"

# 2. Close the issue if not already closed by the merge
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <ISSUE_NUMBER> --state closed
```

## Lookup: find the issue number

If the issue number is not already known, use:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<task-id-or-keyword>" --state open
```

## Error handling

- If `set-issue-status` throws "item not found in project", the issue has not
  been added to the configured Project Board.  Add it via the GitHub UI or skip
  the status step and only post the comment.
- If the issue is already in the target status, the command is idempotent and
  exits cleanly — no action needed.
- Never skip the comment step; it provides a human-readable audit trail even
  when the board status is already correct.
