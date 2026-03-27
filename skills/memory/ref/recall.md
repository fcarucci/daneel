# Recall and Show Operations

## Show (`action: show`)

Produces a compact, context-ready digest of memories. No subagent
is needed — the calling agent runs the command directly.

```bash
python3 scripts/memory-recall.py --show                # both scopes (default)
python3 scripts/memory-recall.py --show --scope user   # user only
python3 scripts/memory-recall.py --show --scope project # project only
python3 scripts/memory-recall.py --show --last 10      # last 10 experiences
python3 scripts/memory-recall.py --show --days 7       # last 7 days
python3 scripts/memory-recall.py --show --last 999     # all experiences
```

The digest contains, for each scope with content:

1. **World Knowledge** — all verified facts with confidence scores
2. **Beliefs** — all beliefs with confidence scores
3. **Entity Summaries** — all synthesized entity profiles
4. **Recent Experiences** — bounded by `--last N` (default: 5) or `--days N`

Output is plain text, not JSON. Display directly to the user.

## Recall (`action: recall`)

Recall searches **both** user and project memory by default, tagging
results with `[user]` or `[project]` so the caller knows the source.

```bash
python3 scripts/memory-recall.py --keyword "database" --json
python3 scripts/memory-recall.py --keyword "database" --scope user --json
python3 scripts/memory-recall.py --keyword "database" --scope project --json
python3 scripts/memory-recall.py --entity "api-gateway" --cross-section --json
python3 scripts/memory-recall.py --since 2026-03-01 --until 2026-03-31 --json
python3 scripts/memory-recall.py --section beliefs --keyword "reliable" --json
python3 scripts/memory-recall.py --stats
python3 scripts/memory-recall.py --stats --scope user
```

When to use recall vs. full read:

- < 20 total memories: reading the full file is fine
- 20–50 memories: recall for targeted queries, full read for maintenance
- 50+ memories: always use recall for queries

## Required output (recall subagent)

- Query parameters used.
- Number of results per section.
- The matched memories (raw text).
