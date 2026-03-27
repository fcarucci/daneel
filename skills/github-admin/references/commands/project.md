# project

Project automation: link PRs to their tracked issues, or close the configured GitHub Project. Supports `[--dry-run]` on both actions.

```bash
node skills/github-admin/scripts/github-admin.mjs project --action link-prs [--title-prefix <text>] [--dry-run]
node skills/github-admin/scripts/github-admin.mjs project --action close-project [--dry-run]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--action` | Yes | `link-prs` or `close-project` |
| `--title-prefix` | No | For `link-prs`: restrict to issues whose title starts with this string |
| `--dry-run` | No | Report what would happen without making changes |

| `--action` | Description |
|------------|-------------|
| `link-prs` | Matches issues to PRs (by task id or closing keywords) and ensures each issue has a `Closes #n` in the PR body and a linked-PR comment on the issue |
| `close-project` | Closes the configured GitHub Project (`GITHUB_PROJECT_NUMBER`) |
