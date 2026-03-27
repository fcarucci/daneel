# comment-pr-verification

```bash
node skills/github-admin/scripts/github-admin.mjs comment-pr-verification --number <n> --artifact-url <url> [--route <route>] [--latest-session-count <n>] [--connected-ribbon <true|false>] [--screenshot <path>] [--dom <path>] [--video <path>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--number` | Yes | Pull request number |
| `--artifact-url` | Yes | URL to verification artifact; alias `artifactUrl` in code |
| `--route` | No | Default `/agents` |
| `--latest-session-count` | No | Alias `latestSessionCount` in code |
| `--connected-ribbon` | No | `true` or `false`; alias `connectedRibbon` in code |
| `--screenshot` | No | Path or URL note for screenshot |
| `--dom` | No | Path or URL note for DOM snapshot |
| `--video` | No | Local path to recorded video (included in comment text) |
