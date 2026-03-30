# ensure-release

```bash
node skills/github-admin/scripts/github-admin.mjs ensure-release --tag <tag> [--name <title>] [--body <text>] [--draft] [--prerelease]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--tag` | Yes | Git tag |
| `--name` | No | Release title |
| `--body` | No | Release notes |
| `--draft` | No | Boolean: presence marks the release as a draft (default: published) |
| `--prerelease` | No | Boolean: presence marks the release as prerelease (default: stable) |
