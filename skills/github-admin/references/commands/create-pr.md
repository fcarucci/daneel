# create-pr

```bash
node skills/github-admin/scripts/github-admin.mjs create-pr --head <branch> --title <title> [--base <branch>] [--body <text>] [--issue <n>] [--draft]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--head` | Yes | Source branch |
| `--title` | Yes | PR title |
| `--base` | No | Default `main` |
| `--body` | No | Description |
| `--issue` | No | Related issue number |
| `--draft` | No | Boolean: open as draft |
