# Forget Operation (`action: forget`)

When the user asks to **forget** something, the agent runs a two-step
process: fuzzy-find matches, show them, ask for confirmation, then delete.

## Trigger phrases

| User says | Action |
|-----------|--------|
| "Forget about the port conflict" | `forget` |
| "Delete that memory about testing" | `forget` |
| "Remove the belief about the dev server" | `forget` |
| "I don't want you to remember that" | `forget` |

## Workflow (no subagent needed)

1. **Find matches** — fuzzy-search with the user's description:
   ```bash
   python3 skills/memory/scripts/memory-manage.py find-matches --query "port conflict"
   ```
   Use `--scope user` or `--scope project` if the user specifies.
   The default threshold (0.4) is intentionally lower than duplicate
   detection to cast a wide net.

2. **Show candidates** — display the matches to the user:
   > Found 2 matching memories:
   > 1. [experiences, index 0, similarity 0.82] 2026-03-26: The integration test hung because port 5432 was bound...
   > 2. [beliefs, index 1, similarity 0.45] The dev server port conflicts are systemic...
   >
   > Which ones should I forget? (all / numbers / none)

3. **Wait for confirmation** — never delete without the user confirming.

4. **Delete confirmed entries** — for each confirmed item:
   ```bash
   python3 skills/memory/scripts/memory-manage.py delete-entry --section experiences --index 0
   ```
   **Important:** delete in reverse index order within each section
   to avoid index shifting.

5. **Confirm deletion** — tell the user what was removed.
