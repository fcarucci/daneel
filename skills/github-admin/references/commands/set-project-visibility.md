# set-project-visibility

Sets visibility on a **user** GitHub Project (classic Projects are not supported here). Uses the repository owner from `GITHUB_REPOSITORY` as the user login and `GITHUB_PROJECT_NUMBER` as the default project number unless `--number` is set.

```bash
node skills/github-admin/scripts/github-admin.mjs set-project-visibility --public [--number <n>] [--dry-run]
node skills/github-admin/scripts/github-admin.mjs set-project-visibility --private [--number <n>] [--dry-run]
```

| Flag | Description |
|------|-------------|
| `--public` | Make the project visible to everyone (mutually exclusive with `--private`). |
| `--private` | Restrict the project (mutually exclusive with `--public`). |
| `--number` | Project number (default: `GITHUB_PROJECT_NUMBER`, usually `1`). |
| `--dry-run` | Print the current project metadata and the intended visibility change; no mutation. |

Requires a token with permission to update the project (GraphQL `updateProjectV2`).
