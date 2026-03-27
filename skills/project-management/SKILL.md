---
name: project-management
description: >
  Updates project tracking whenever a task transitions lifecycle state
  (started, blocked, ready-for-merge, done). Receives task context from the
  calling workflow, resolves the associated work item if one exists, then
  updates its status and posts a comment. Exits gracefully when no work item
  is found. Depends on the github-admin skill for GitHub operations.
---

# Project Management

## When to invoke

Use this skill **as a subagent** at each lifecycle checkpoint.
The calling workflow passes task context and an event name — this skill
resolves the work item and acts on it.

| Checkpoint in workflow | Event |
|------------------------|-------|
| Branch created, work begins | `started` |
| Work is blocked by an external dependency | `blocked` |
| PR opened and all verification passes | `ready-for-merge` |
| PR merged or task closed | `done` |

## Step 1: Resolve task to a work item

Before doing anything else, determine the issue number from the context provided.

**If an issue number was given directly** — use it.

**Otherwise, search by task ID or name:**

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<TASK_ID_OR_NAME>" --state open
```

Pick the issue whose title best matches the task. If no match is found,
**exit gracefully** — log "No work item found for task <TASK>; skipping
project management." and stop. Do not error.

Read [`skills/github-admin/SKILL.md`](../github-admin/SKILL.md) for auth
and configuration before running any commands.

## Step 2: Act on the event

### `started` — work begins

```bash
# 1. Set project status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "In Progress"

# 2. Remove blocked label if previously set (safe to fail)
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true

# 3. Comment with the branch being worked on
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "Work started on branch \`<BRANCH_NAME>\`."
```

### `blocked` — work cannot proceed

```bash
# 1. Set project status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Blocked"

# 2. Apply the blocked label
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action add --number <ISSUE_NUMBER> --labels blocked

# 3. Comment with the blocking reason
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "Blocked: <REASON>"
```

### `ready-for-merge` — PR is open, work is complete

```bash
# 1. Set project status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Ready for Merge"

# 2. Remove blocked label if previously set (safe to fail)
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true

# 3. Link PR to issue (adds closing keyword + comment)
node skills/github-admin/scripts/github-admin.mjs \
  link-pr-task --pr <PR_NUMBER> --issue <ISSUE_NUMBER> --close

# 4. Post implementation summary as an issue comment
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> --body "<SUMMARY>"
```

### `done` — PR merged or task closed manually

```bash
# 1. Set project status
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Done"

# 2. Close the issue if not already closed by the merge
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <ISSUE_NUMBER> --state closed
```

## Error handling

- **No work item found** — exit gracefully with a log message; never error.
- **Item not in project board** — skip `set-issue-status`; still post the comment.
- **Status already correct** — `set-issue-status` is idempotent; no action needed.
- **Label not present on remove** — suppress the error (`2>/dev/null || true`).
- **Never skip the comment step** — it provides an audit trail even when board status is unchanged.
