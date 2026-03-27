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
2. Read the behavioral profile (see `ref/profile.md`):
   ```bash
   cat ~/.agents/memory/profile.json 2>/dev/null || echo '{"skepticism":3,"literalism":3,"empathy":3,"bias_strength":0.2}'
   ```
   Apply the disposition parameters throughout the steps below.
3. For each belief, assess whether recent experiences reinforce or
   contradict it (applying skepticism and literalism from the profile):
   ```bash
   python3 skills/memory/scripts/memory-recall.py --entity "belief-entity" --section experiences --json
   ```
4. Compute base confidence deltas using the evolution rules below,
   modulated by the behavioral profile (see `ref/profile.md`).
5. **Apply reflect techniques** (read `ref/reflect-techniques.md`):
   - Self-verification probes — generate and check probe questions
   - Confidence calibration — weight deltas by evidence quality and β
   - Counterfactual analysis — assess dependency impact for beliefs ≥ 0.6
6. Update confidence scores with the calibrated deltas:
   ```bash
   python3 skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index N --delta <calibrated_delta>
   ```
7. **Detect belief conflicts** (technique 4, profile-aware resolution):
   ```bash
   python3 skills/memory/scripts/memory-manage.py check-conflicts
   ```
   Resolve flagged conflicts using the profile to break ties
   (see `ref/profile.md`).
8. Prune beliefs below the threshold:
   ```bash
   python3 skills/memory/scripts/memory-manage.py prune-beliefs --threshold 0.2
   ```
9. Check for entity summary opportunities:
   ```bash
   python3 skills/memory/scripts/memory-manage.py suggest-summaries
   ```
10. **Synthesize reflections** (technique 5): look for cross-cutting
    patterns across experiences and beliefs that warrant a new entry
    in `## Reflections`.
11. If you manually update summaries, beliefs, or reflections, preserve
    structure and use the guarded-write discipline from `ref/retain.md`.
12. Write the updated file.

### Confidence evolution rules

| Evidence | Delta | Example |
|----------|-------|---------|
| Reinforcing experience | +0.1 | Another session confirms the pattern |
| Mildly contradicting | -0.1 | An exception was found but doesn't invalidate |
| Strongly contradicting | -0.2 | The belief was wrong in a significant case |
| Promotion-worthy (→ World Knowledge) | n/a | Move from Beliefs to World Knowledge at 0.85+ with 3+ sources |
| Decay (no recent evidence) | -0.01 | Belief is old with no new supporting evidence |

Beliefs below `0.2` confidence are pruning candidates. If you remove one,
first make sure the information is either obsolete or preserved elsewhere.

Beliefs above `0.85` with 3+ supporting experiences can be considered for
promotion to World Knowledge, but only after repo-state verification and a
separate explicit promotion decision.

### Required output (reflect)

- Beliefs reviewed and confidence changes applied.
- For each belief reviewed: probe result, calibration multiplier used,
  and counterfactual classification (if applicable).
- Load-bearing beliefs flagged, with their dependents listed.
- Isolated low-confidence beliefs flagged for pruning.
- Belief conflicts detected and resolutions applied.
- Beliefs pruned (below threshold).
- Reflections synthesized (if any), with source memories cited.
- Entity summaries regenerated or suggested.
- Final counts per section.

## Maintain (`action: maintain`)

Run the supported maintenance cycle without adding a new memory:

1. Validate structure.
2. Review low-confidence beliefs with `prune-beliefs`.
3. Update belief confidences based on recent evidence.
4. Check entity summary opportunities with `suggest-summaries`.
5. Remove duplicates only when you can prove they are duplicates.
6. Write the updated file.

### Required output (maintain)

- Beliefs updated or pruned.
- Duplicates removed, if any.
- Entity summaries refreshed or suggested.
- Final counts per section.
