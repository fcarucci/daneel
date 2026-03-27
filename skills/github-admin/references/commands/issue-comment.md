# issue-comment

```bash
node skills/github-admin/scripts/github-admin.mjs issue-comment --action list --issue <n>
node skills/github-admin/scripts/github-admin.mjs issue-comment --action delete --comment-id <n>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--action` | Yes | `list` or `delete` |
| `--issue` | For `list` | Issue number |
| `--comment-id` | For `delete` | Comment id; alias `commentId` in implementation |
