# get-issue

Fetches a single issue (or pull request) by number and prints its title, state, labels, milestone, and assignees as JSON. Works on both issue and PR numbers since GitHub's REST API exposes PRs through the issues endpoint.

```bash
node skills/github-admin/scripts/github-admin.mjs get-issue --number <n>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--number` | Yes | Issue or pull request number |

**Output fields:** `number`, `title`, `state`, `labels[]`, `milestone` (`number` + `title`), `assignees[]`, `url`, `isPullRequest`.
