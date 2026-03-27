# Reflect and Maintain Operations

## Reflect (`action: reflect`)

The reflect operation reviews the belief network and entity summaries
without adding new memories.

Reflect runs in two modes:
- **Explicit**: the caller requests `action: reflect` directly.
- **Automatic**: triggered by the retain workflow after a successful
  write when trigger conditions are met (see `ref/retain.md`,
  "Auto-reflect" section). In this mode, reflect runs inside the
  same subagent invocation as the retain — no separate spawn needed.

### Workflow

1. Read and parse `MEMORY.md`.
2. For each belief, assess whether recent experiences reinforce or
   contradict it:
   ```bash
   python3 scripts/memory-recall.py --entity "belief-entity" --section experiences --json
   ```
3. Update confidence scores using the management script.
4. Prune beliefs below the threshold:
   ```bash
   python3 scripts/memory-manage.py prune-beliefs --threshold 0.2
   ```
5. Check for entity summary opportunities:
   ```bash
   python3 scripts/memory-manage.py suggest-summaries
   ```
6. For entities with stale summaries (new experiences since last summary
   was written), regenerate the summary paragraph.
7. Write the updated file.

### Confidence evolution rules

| Evidence | Delta | Example |
|----------|-------|---------|
| Reinforcing experience | +0.1 | Another session confirms the pattern |
| Mildly contradicting | -0.1 | An exception was found but doesn't invalidate |
| Strongly contradicting | -0.2 | The belief was wrong in a significant case |
| Promotion-worthy (→ World Knowledge) | n/a | Move from Beliefs to World Knowledge at 0.85+ with 3+ sources |
| Decay (no recent evidence) | -0.05 | Belief is old with no new supporting evidence |

Beliefs below `0.2` confidence should be removed. Their text is lost
unless it is still useful as an experience.

Beliefs above `0.85` with 3+ supporting experiences can be promoted to
World Knowledge with the belief removed.

### Required output (reflect)

- Beliefs reviewed and confidence changes applied.
- Beliefs pruned (below threshold).
- Entity summaries regenerated or suggested.
- Final counts per section.

## Maintain (`action: maintain`)

Run the full maintenance cycle without adding a new memory:

1. Validate structure.
2. Prune stale experiences (> 90 days with no ongoing relevance).
3. Update belief confidences based on recent evidence.
4. Prune low-confidence beliefs.
5. Check entity summary freshness.
6. Remove duplicates and near-duplicates.
7. Write the updated file.

### Required output (maintain)

- Experiences pruned (with reasons).
- Beliefs updated or pruned.
- Duplicates removed.
- Entity summaries refreshed.
- Final counts per section.
