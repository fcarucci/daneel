# update-issue

```bash
node skills/github-admin/scripts/github-admin.mjs update-issue --number <n> [--title <title>] [--body <text>] [--body-file <path>] [--state <open|closed>] [--labels <a,b,c>] [--assignees <login,login>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--number` | Yes | Issue number |
| `--title` | No | |
| `--body` | No | Mutually exclusive with `--body-file` |
| `--body-file` | No | UTF-8 file path for new body; mutually exclusive with `--body` |
| `--state` | No | `open` or `closed` |
| `--labels` | No | Comma-separated label names |
| `--assignees` | No | Comma-separated assignees logins |
