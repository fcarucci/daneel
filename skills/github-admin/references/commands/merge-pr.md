# merge-pr

```bash
node skills/github-admin/scripts/github-admin.mjs merge-pr --number <n> [--method <merge|squash|rebase>] [--title <title>] [--message <text>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--number` | Yes | Pull request number |
| `--method` | No | Default `merge` |
| `--title` | No | Merge commit title |
| `--message` | No | Merge commit message |
