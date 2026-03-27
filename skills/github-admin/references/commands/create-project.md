# create-project

Creates a GitHub **ProjectV2** owned by the **owner of `GITHUB_REPOSITORY`** (user or organization). The GitHub `createProjectV2` mutation does not set visibility, so this command immediately follows creation with `updateProjectV2` **`public: true`** unless you pass **`--private`**.

```bash
node skills/github-admin/scripts/github-admin.mjs create-project --title <title> [--private] [--dry-run]
```

| Flag | Description |
|------|-------------|
| `--title` | Project title (required). |
| `--private` | Skip the public step; leave the project at GitHub’s default visibility for new projects. |
| `--dry-run` | Print intended owner kind and whether the project would be made public; no mutation. |

Requires token scopes that can create projects for that owner and update project settings.
