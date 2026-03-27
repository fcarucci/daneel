# create-issue

```bash
node skills/github-admin/scripts/github-admin.mjs create-issue --title <title> [--body <text>] [--body-file <path>] [--labels <a,b,c>] [--milestone <n>] [--assignees <login,login>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--title` | Yes | |
| `--body` | No | Mutually exclusive with `--body-file` |
| `--body-file` | No | UTF-8 file path for description; mutually exclusive with `--body` |
| `--labels` | No | Comma-separated |
| `--milestone` | No | Milestone number |
| `--assignees` | No | Comma-separated GitHub logins |
