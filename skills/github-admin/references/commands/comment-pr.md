# comment-pr

**Alias of** [comment-issue.md](comment-issue.md): same handler. Adds a comment on the given number (issue or pull request).

```bash
node skills/github-admin/scripts/github-admin.mjs comment-pr --number <n> --body <text>
```

| Flag | Required | Notes |
|------|----------|-------|
| `--number` | Yes | Issue or PR number |
| `--body` | Yes | Comment body (markdown) |
