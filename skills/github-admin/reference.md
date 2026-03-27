# Github Admin CLI — reference index

Authoritative behavior: `skills/github-admin/scripts/github-admin.mjs`. **When changing commands or flags:** update `usage()` in that file, the matching file under [`references/commands/`](references/commands/), and this index if command names are added or removed.

## Shared docs (small context)

- [Script: parser, flags, help](references/script.md)
- [Environment variables](references/environment.md)

## Grouped commands

Commands with `--action` dispatching:

| Command | Doc |
|---------|-----|
| `issue-comment` | [issue-comment.md](references/commands/issue-comment.md) |
| `project` | [project.md](references/commands/project.md) |
| `pr-review` | [pr-review.md](references/commands/pr-review.md) |
| `release-asset` | [release-asset.md](references/commands/release-asset.md) |

## Commands

| Command | Doc |
|---------|-----|
| `comment-issue` | [comment-issue.md](references/commands/comment-issue.md) |
| `comment-pr` | [comment-pr.md](references/commands/comment-pr.md) |
| `comment-pr-verification` | [comment-pr-verification.md](references/commands/comment-pr-verification.md) |
| `create-issue` | [create-issue.md](references/commands/create-issue.md) |
| `create-project` | [create-project.md](references/commands/create-project.md) |
| `create-pr` | [create-pr.md](references/commands/create-pr.md) |
| `get-issue` | [get-issue.md](references/commands/get-issue.md) |
| `delete-issue` | [delete-issue.md](references/commands/delete-issue.md) |
| `ensure-release` | [ensure-release.md](references/commands/ensure-release.md) |
| `label-issue` | [label-issue.md](references/commands/label-issue.md) |
| `link-pr-task` | [link-pr-task.md](references/commands/link-pr-task.md) |
| `list-issues` | [list-issues.md](references/commands/list-issues.md) |
| `list-prs` | [list-prs.md](references/commands/list-prs.md) |
| `list-tasks` | [list-tasks.md](references/commands/list-tasks.md) |
| `merge-pr` | [merge-pr.md](references/commands/merge-pr.md) |
| `report` | [report.md](references/commands/report.md) |
| `set-issue-status` | [set-issue-status.md](references/commands/set-issue-status.md) |
| `set-project-visibility` | [set-project-visibility.md](references/commands/set-project-visibility.md) |
| `sync-labels` | [sync-labels.md](references/commands/sync-labels.md) |
| `update-issue` | [update-issue.md](references/commands/update-issue.md) |
| `update-pr` | [update-pr.md](references/commands/update-pr.md) |
| `upload-release-asset` | [upload-release-asset.md](references/commands/upload-release-asset.md) |

## Completing or closing work (generic pattern)

There are no repo-specific “complete issue” commands. Use:

1. [comment-issue.md](references/commands/comment-issue.md) (or `comment-pr`) to post an implementation note with commit link.
2. [update-issue.md](references/commands/update-issue.md) `--state closed` when the issue should close.
3. [set-issue-status.md](references/commands/set-issue-status.md) if the issue is on the configured Project and you need a Status column update (e.g. `Done`).

To find issues by title, use [list-issues.md](references/commands/list-issues.md) with `--title-prefix` or `--title-contains`.
