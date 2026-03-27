# pr-review

```bash
node skills/github-admin/scripts/github-admin.mjs pr-review --action list --number <n>
node skills/github-admin/scripts/github-admin.mjs pr-review --action resolve --thread-id <id>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--action` | Yes | `list` or `resolve` |
| `--number` | For `list` | Pull request number |
| `--thread-id` | For `resolve` | Review thread id; alias `threadId` in implementation |
