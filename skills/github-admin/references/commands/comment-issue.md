# comment-issue

Adds a comment on an issue or pull request (GitHub uses the same issues API for PR comments).

```bash
node skills/github-admin/scripts/github-admin.mjs comment-issue --number <n> --body <text>
```

| Flag | Required |
|------|----------|
| `--number` | Yes |
| `--body` | Yes |

**Alias:** `comment-pr` — same implementation and flags.
