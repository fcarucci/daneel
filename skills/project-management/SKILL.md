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
The calling workflow passes task context and an event name. This skill
resolves the work item, performs the matching project-management update,
and leaves an audit trail on GitHub.

| Checkpoint in workflow | Event |
|------------------------|-------|
| Branch created, work begins | `started` |
| Work is blocked by an external dependency | `blocked` |
| PR opened and all verification passes | `ready-for-merge` |
| PR merged or task closed | `done` |

## Operating rules

Read [`skills/github-admin/SKILL.md`](../github-admin/SKILL.md) before running
any commands.

Non-negotiable rules:

- **No ambiguous mutations.** If issue resolution is ambiguous, stop and exit
  cleanly. Do not guess.
- **Verify every mutating command.** Parse the JSON output and confirm it shows
  the intended effect before moving on.
- **Treat comments as idempotent.** Before posting an audit comment, list
  existing comments and skip posting if an identical marker comment already
  exists.
- **Project status updates are best-effort.** If the item is not on the project
  board, log that fact and continue with labels/comments.
- **Never throw on “no match found.”** Log and exit cleanly instead.

Use marker prefixes in automation comments so reruns stay quiet:

- `"[project-management:started]"`
- `"[project-management:blocked]"`
- `"[project-management:pr-title-warning]"`
- `"[project-management:ready-for-merge-summary]"`

## Step 1: Resolve task to a work item

Determine the issue number from the available context. Work through the
signals below **in order**, stopping only when one produces an unambiguous
same-repository match.

### Signal 1 — direct issue number

If the calling context includes an explicit issue number, use it immediately.
Skip all remaining signals.

### Signal 2 — PR closing reference

If a PR number is available, inspect its title and body for closing
references. Only accept **same-repo** references of the form:

- `Closes #N`
- `Fixes #N`
- `Resolves #N`

Use `get-issue` because it returns PR title and body as JSON:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  get-issue --number <PR_NUMBER>
```

Resolution rule:

- If there is exactly one closing reference, use that issue number.
- If there are zero closing references, continue to Signal 3.
- If there are multiple closing references, log the candidates and exit
  cleanly without mutating anything.

Ignore plain mentions like `#123` that are not attached to a closing keyword.
Ignore cross-repo references like `owner/repo#123`.

### Signal 3 — structured task ID

If the task context contains a structured ID such as `T2.7` or `T1.12`,
search for it by title:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<TASK_ID>" --state open
```

If zero results, retry with `--state all`.

Resolution rule:

- If exactly one result is returned, use it.
- If multiple results are returned, keep only titles that contain the exact
  task tag token.
- If exactly one exact-tag result remains, use it.
- Otherwise, log the candidates and exit cleanly without mutating anything.

### Signal 4 — branch name decoding

If a branch name is available, extract a task ID from it:

- Strip common prefixes: `task/`, `feature/`, `fix/`, `hotfix/`
- Normalize separators: replace `_` and `-` with `.` in the task segment
- Example: `task/T2_7` -> `T2.7`

Then apply Signal 3 with the extracted ID.

### Signal 5 — task name keyword search

Search by the most distinctive words from the task name or description:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  list-issues --title-contains "<DISTINCTIVE_KEYWORD>" --state open
```

Run narrow searches first; broaden only if zero results.

Resolution rule:

- If exactly one result remains, use it.
- If multiple candidates remain at any point, log them and exit cleanly.
- Do **not** auto-pick the lowest issue number as a tiebreaker.

### No match found

If all signals are exhausted without a match:

- Log: `No work item found for task '<TASK>'; skipping project management.`
- Exit cleanly. Do not error or throw.

## Step 2: Act on the event

For every mutating command below, inspect the JSON output before proceeding.

### Shared verification rules

Interpret command output like this:

- `set-issue-status`: the returned `updatedIssues` array must contain the issue
  number. If it does not, log that the issue is not on the project board and
  continue with labels/comments.
- `label-issue --action add`: verify the output contains the issue number and
  the label set you intended to add.
- `label-issue --action remove`: if the label is absent, suppress the error and
  continue.
- `update-issue`: verify the returned issue number matches the target item.
- `comment-issue`: verify the returned comment id exists.
- `link-pr-task`: verify the JSON fields `patchedBody` and `postedComment`; both
  may be `false` on reruns and that is still success.

### `started` — work begins

```bash
# 1. Set project status (best-effort)
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "In Progress"

# 2. Remove blocked label if present (safe to fail)
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true

# 3. Only post the started comment if it is not already present
node skills/github-admin/scripts/github-admin.mjs \
  issue-comment --action list --issue <ISSUE_NUMBER>

node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "[project-management:started] Work started on branch \`<BRANCH_NAME>\`."
```

Skip Step 3 when an identical marker comment already exists.

### `blocked` — work cannot proceed

```bash
# 1. Set project status (best-effort)
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Blocked"

# 2. Apply the blocked label
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action add --number <ISSUE_NUMBER> --labels blocked

# 3. Only post the blocked comment if it is not already present
node skills/github-admin/scripts/github-admin.mjs \
  issue-comment --action list --issue <ISSUE_NUMBER>

node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "[project-management:blocked] Blocked: <REASON>"
```

Skip Step 3 when an identical marker comment already exists.

### `ready-for-merge` — PR is open, work is complete

Run these steps in order.

#### A. Fetch issue metadata

```bash
node skills/github-admin/scripts/github-admin.mjs \
  get-issue --number <ISSUE_NUMBER>
```

Capture:

- `labels`
- `milestone.number`
- `assignees`

#### B. Set project board status

```bash
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Ready for Merge"
```

If `updatedIssues` does not include the issue, log the miss and continue.

#### C. Remove `blocked` label if present

```bash
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true
```

#### D. Link PR to issue

`link-pr-task` is rerun-safe. It only patches the PR body or posts the linked
PR comment when they are missing.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  link-pr-task --pr <PR_NUMBER> --issue <ISSUE_NUMBER> --close
```

#### E. Mirror issue labels onto the PR

GitHub does not automatically copy labels from an issue to its PR.
Apply all issue labels to the PR except `blocked`.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action add --number <PR_NUMBER> \
  --labels <COMMA_SEPARATED_ISSUE_LABELS_EXCEPT_BLOCKED>
```

Skip this step if the filtered label set is empty.

#### F. Mirror issue milestone onto the PR

GitHub treats PRs as issues for milestone purposes.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <PR_NUMBER> --milestone <ISSUE_MILESTONE_NUMBER>
```

Skip this step if the issue has no milestone.

#### G. Mirror issue assignees onto the PR

```bash
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <PR_NUMBER> --assignees <COMMA_SEPARATED_ISSUE_ASSIGNEES>
```

Skip this step if the issue has no assignees.

#### H. Verify PR title references the task

Read the PR title from `get-issue --number <PR_NUMBER>`. If the title does not
reference either the task tag or `#<ISSUE_NUMBER>`, warn once.

```bash
node skills/github-admin/scripts/github-admin.mjs \
  issue-comment --action list --issue <PR_NUMBER>

node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <PR_NUMBER> \
  --body "[project-management:pr-title-warning] PR title does not reference the task or issue. Consider updating it to make the connection explicit."
```

Only post the warning if:

- the title genuinely lacks the reference
- no identical marker warning comment already exists

#### I. Post implementation summary on the issue

List existing comments first, then post only if the exact marker comment does
not already exist:

```bash
node skills/github-admin/scripts/github-admin.mjs \
  issue-comment --action list --issue <ISSUE_NUMBER>

node skills/github-admin/scripts/github-admin.mjs \
  comment-issue --number <ISSUE_NUMBER> \
  --body "[project-management:ready-for-merge-summary] <SUMMARY>"
```

The summary should cover:

- what changed
- how it was tested
- any known limitations or follow-up items

### `done` — PR merged or task closed manually

```bash
# 1. Set project status (best-effort)
node skills/github-admin/scripts/github-admin.mjs \
  set-issue-status --issues <ISSUE_NUMBER> --status "Done"

# 2. Remove blocked label if present (safe to fail)
node skills/github-admin/scripts/github-admin.mjs \
  label-issue --action remove --number <ISSUE_NUMBER> --label blocked \
  2>/dev/null || true

# 3. Close the issue if not already closed by the merge
node skills/github-admin/scripts/github-admin.mjs \
  update-issue --number <ISSUE_NUMBER> --state closed
```

## Required subagent output

At the end of the skill run, report back:

- resolved issue number, or `none`
- which resolution signal matched
- every command executed
- whether project status updated, or was skipped because the item was off-board
- whether each comment was posted or skipped as a rerun duplicate
- any ambiguity or roadblock encountered

## Error handling

- **No work item found** — exit gracefully with a log message; never error.
- **Ambiguous work item resolution** — log candidates and exit gracefully with
  no mutations.
- **Item not in project board** — skip board status updates; still perform
  labels/comments.
- **Status already correct** — `set-issue-status` is effectively idempotent.
- **Label not present on remove** — suppress the error (`2>/dev/null || true`).
- **Duplicate marker comment** — skip posting; do not create comment spam.
