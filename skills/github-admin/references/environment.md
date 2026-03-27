# Github Admin — environment variables

Used by `skills/github-admin/scripts/github-admin.mjs`. **Keep aligned with `usage()`** when defaults change.

| Variable | Required | Default / notes |
|----------|----------|------------------|
| `GITHUB_TOKEN` or `GITHUB_PERSONAL_ACCESS_TOKEN` | Yes (or token in `~/.env`) | `Authorization: token …` |
| `GITHUB_REPOSITORY` | No | `fcarucci/daneel` |
| `GITHUB_PROJECT_NUMBER` | No | `1` |

Token fallback: script reads `GITHUB_PERSONAL_ACCESS_TOKEN` from `~/.env` when exported in shell form (`export GITHUB_PERSONAL_ACCESS_TOKEN=…`).
