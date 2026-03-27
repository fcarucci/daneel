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

Before doing anything else, determine the issue number from the available
context. Work through the signals below **in order**, stopping as soon as
one produces an unambiguous match.

Read [`skills/github-admin/SKILL.md`](../github-admin/SKILL.md) for auth
and configuration before running any commands.

---

### Signal 1 — direct issue number

If the calling context includes an explicit issue number, use it immediately.
Skip all remaining signals.

---

### Signal 2 — PR closing reference

If a PR number is available, inspect its body and title for closing
keywords (`Closes #N`, `Fixes #N`, `Resolves #N`). Extract N and use it.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  get-issue --number <PR_NUMBER>
# inspect the returned title and body for #N references
```

---

### Signal 3 — structured task ID

If the task context contains a structured ID (e.g. `T2.7`, `T1.12`),
search open issues for it. A structured ID is almost always unique.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<TASK_ID>" --state open
```

If exactly one result, use it. If zero results, try `--state all` (the
issue may already be closed).

---

### Signal 4 — branch name decoding

If a branch name is available, extract a task ID from it:

- Strip common prefixes: `task/`, `feature/`, `fix/`, `hotfix/`
- Normalise separators: replace `_` and `-` with `.` in the task segment
- Example: `task/T2_7` → `T2.7`

Then apply Signal 3 with the extracted ID.

---

### Signal 5 — task name keyword search

Search by the most distinctive words from the task name or description:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<DISTINCTIVE_KEYWORD>" --state open
```

Run narrower searches first; broaden only if zero results.

**Disambiguation when multiple results are returned:**

1. Prefer the issue whose title contains the greatest number of words from the task name (case-insensitive).
2. Among ties, prefer open over closed.
3. Among remaining ties, take the lowest issue number.
4. Log all candidates and the chosen one before proceeding.

---

### No match found

If all signals are exhausted without a match:

- Log: `No work item found for task '<TASK>'; skipping project management.`
- Exit cleanly. Do not error or throw.

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

Run these steps in order. Each step is idempotent — re-running is safe.

#### A. Fetch issue metadata

```bash
node skills/github-admin/scripts/github-admin.mjs \
  get-issue --number <ISSUE_NUMBER>
```

Capture the output: `labels`, `milestone.number`, `assignees`. You will
need these values in the steps below.

#### B. Set project board status

```bash
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Ready for Merge"
```

#### C. Remove `blocked` label if present (safe to fail)

```bash
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true
```

#### D. Link PR to issue

Adds `Closes #<ISSUE_NUMBER>` to the PR body and posts a linked-PR comment
on the issue so the connection is visible from both sides:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  link-pr-task --pr <PR_NUMBER> --issue <ISSUE_NUMBER> --close
```

#### E. Mirror issue labels onto the PR

GitHub does not automatically copy labels from an issue to its PR.
Apply all labels from the issue to the PR so both show the same
classification (component, priority, type). Filter out `blocked` if it was
present in Step A, since the PR should not be marked as blocked.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action add --number <PR_NUMBER> \
  --labels <COMMA_SEPARATED_ISSUE_LABELS>
```

Skip this step if the issue has no labels.

#### F. Mirror issue milestone onto the PR

GitHub treats PRs as issues for milestone purposes. Copy the milestone:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <PR_NUMBER> --milestone <ISSUE_MILESTONE_NUMBER>
```

Skip this step if the issue has no milestone.

#### G. Mirror issue assignees onto the PR

Copy the assignees from the issue to the PR to maintain ownership tracking:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <PR_NUMBER> --assignees <COMMA_SEPARATED_ISSUE_ASSIGNEES>
```

Skip this step if the issue has no assignees.

#### H. Verify PR title references the task

Read the PR title. If it does not reference the task ID or the issue
number, post a warning comment on the PR:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <PR_NUMBER> \
  --body "⚠️ PR title does not reference the task or issue. Consider updating it to make the connection explicit."
```

Only post this warning if the title is genuinely missing the reference.

#### I. Post implementation summary on the issue

```bash
node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> --body "<SUMMARY>"
```

The summary should cover: what changed, how it was tested, any known
limitations or follow-up items.

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
