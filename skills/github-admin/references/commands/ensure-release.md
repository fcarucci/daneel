# ensure-release

```bash
node skills/github-admin/scripts/github-admin.mjs ensure-release --tag <tag> [--name <title>] [--body <text>] [--draft] [--prerelease]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--tag` | Yes | Git tag |
| `--name` | No | Release title |
| `--body` | No | Release notes |
| `--draft` | No | Boolean: presence enables draft |
| `--prerelease` | No | Boolean: presence enables prerelease |
