# list-tasks

Lists pending issues from the configured GitHub Project board, sorted by priority then issue number. Excludes items in `In Progress`, `Done`, or `Ready for Merge` status — shows only `Ready` and `Backlog` items.

```bash
node skills/github-admin/scripts/github-admin.mjs list-tasks [--limit <n>]
```

| Flag | Required | Notes |
|------|----------|-------|
| `--limit` | No | Max items returned; default `10` |
