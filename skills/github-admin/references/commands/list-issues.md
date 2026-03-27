# list-issues

Lists repository issues (not pull requests), with optional filters. Paginates up to 50 GitHub API pages (100 issues per page).

```bash
node skills/github-admin/scripts/github-admin.mjs list-issues [--state <open|closed|all>] [--title-prefix <text>] [--title-contains <text>] [--limit <n>]
```

| Flag | Required | Default | Notes |
|------|----------|---------|-------|
| `--state` | No | `all` | `open`, `closed`, or `all` |
| `--title-prefix` | No | | Issue title must start with this string |
| `--title-contains` | No | | Substring match on title |
| `--limit` | No | `500` | Max issues after filtering (cap 2000) |

**Example (task lookup by title prefix):**

```bash
node skills/github-admin/scripts/github-admin.mjs list-issues --title-prefix "[T2.7]"
```

**Example (list all open issues):**

```bash
node skills/github-admin/scripts/github-admin.mjs list-issues --state open --limit 200
```

Compare output to your local task doc manually, or script outside this CLI.
