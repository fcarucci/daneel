# Github Admin — script behavior

Implementation: `skills/github-admin/scripts/github-admin.mjs`.

**Maintenance:** When adding or changing commands or flags, update `usage()` in that file, the matching file under [`commands/`](commands/), and the command list in [`../reference.md`](../reference.md).

## Invocation

```text
node skills/github-admin/scripts/github-admin.mjs <command> [options]
```

```bash
node skills/github-admin/scripts/github-admin.mjs help
```

## Argument parser

- Flags are `--name` with an optional value: if the next argv token does not start with `--`, it becomes the value; otherwise the flag is boolean `true`.
- Some hyphenated flags also accept camelCase keys in code (e.g. `--comment-id` / `commentId`). See each command file under [`commands/`](commands/).

## Boolean vs value flags

**Boolean (presence = true):** `--dry-run`, `--draft`, `--prerelease`, `--close` (where that command documents `--close`), `--public`, `--private` (for `set-project-visibility` or `create-project` to keep a new project non-public).

**Value flags:** `--action`, `--assignee`, `--limit`, `--issue`, `--comment-id`, `--number`, `--title`, `--body`, `--body-file`, `--state`, `--labels`, `--milestone`, `--assignees`, `--tag`, `--name`, `--file`, `--label`, `--artifact-url`, `--route`, `--latest-session-count`, `--connected-ribbon`, `--screenshot`, `--dom`, `--video`, `--title-prefix`, `--title-contains`, `--asset-id`, `--thread-id`, `--method`, `--message`, `--head`, `--base`, `--issues`, `--status`, `--commit`, `--note`, `--pr`.
