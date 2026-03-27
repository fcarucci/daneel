# release-asset

```bash
node skills/github-admin/scripts/github-admin.mjs release-asset --action list --tag <tag>
node skills/github-admin/scripts/github-admin.mjs release-asset --action delete --asset-id <n>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--action` | Yes | `list` or `delete` |
| `--tag` | For `list` | Release tag |
| `--asset-id` | For `delete` | Asset id; alias `assetId` in implementation |
