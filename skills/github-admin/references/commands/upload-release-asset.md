# upload-release-asset

```bash
node skills/github-admin/scripts/github-admin.mjs upload-release-asset --tag <tag> --file <path> [--label <text>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--tag` | Yes | Existing release tag |
| `--file` | Yes | Local file path to upload |
| `--label` | No | Display label for the asset |
