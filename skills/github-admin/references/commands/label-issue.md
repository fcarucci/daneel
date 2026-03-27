# label-issue

Adds or removes labels on an issue **without replacing the full label set**. Use this instead of `update-issue --labels` when you want to touch only specific labels and preserve the rest.

```bash
node skills/github-admin/scripts/github-admin.mjs label-issue --action add --number <n> --labels <a,b,...>
node skills/github-admin/scripts/github-admin.mjs label-issue --action remove --number <n> --label <name>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--action` | Yes | `add` or `remove` |
| `--number` | Yes | Issue number |
| `--labels` | For `add` | Comma-separated label names to add |
| `--label` | For `remove` | Single label name to remove |
