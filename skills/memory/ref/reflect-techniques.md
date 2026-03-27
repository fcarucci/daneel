# Reflect Techniques

> **When to load:** Read this file during any full reflect pass (explicit
> or auto-triggered). Not needed for the mini-reflect step inside retain
> (step 10 in `ref/retain.md`) — that only does basic confidence deltas.

These three techniques run **in order** during a full reflect, after
the basic confidence updates in `ref/reflect.md` step 3 and before
pruning in step 4.

---

## Technique 1: Self-verification probes

Before finalizing any confidence change, generate a probe question that
the belief implies should be answerable from the memory bank, then
check whether the evidence actually supports it.

### Procedure

For each belief being reviewed:

1. **Generate a probe.** Ask: "If this belief is true, what concrete
   evidence should exist in my experiences or world knowledge?" Frame
   it as a specific, answerable question.

   Example belief: "Running the combined dev command is more reliable
   than starting the server alone."

   Probe: "Are there experiences where the combined dev command
   succeeded and the standalone server failed, or vice versa?"

2. **Search for evidence.**
   ```bash
   python3 skills/memory/scripts/memory-recall.py --entity "<belief-entity>" --section experiences --json
   python3 skills/memory/scripts/memory-recall.py --keyword "<key term from probe>" --json
   ```

3. **Score the probe result.**

   | Probe outcome | Effect on confidence delta |
   |--------------|--------------------------|
   | Strong supporting evidence found | Apply the planned delta as-is |
   | Weak or indirect evidence | Halve the planned delta |
   | No evidence found | Skip the update; flag for review |
   | Contradicting evidence found | Reverse the delta sign |

4. **Report.** In the output, include for each probed belief:
   - The probe question generated
   - What evidence was found (or not)
   - How the probe result modified the confidence delta

### When to skip

- If the belief has fewer than 2 related experiences, probing adds
  little value — the evidence base is too thin to probe meaningfully.
- During auto-reflect (retain step 14), skip probes to keep latency
  low. Probes run only during explicit or full reflect passes.

---

## Technique 2: Confidence calibration

Replace fixed confidence deltas with evidence-quality-weighted deltas.
This makes strong, recent, direct evidence count more than weak, old,
indirect evidence.

### Evidence quality multiplier

| Quality signal | Multiplier | How to assess |
|---------------|------------|---------------|
| Direct + recent (≤ 7 days) | 1.5× | Experience shares entities with the belief AND is within 7 days |
| Direct + older (8–30 days) | 1.0× | Shares entities but older than a week |
| Indirect + recent (≤ 7 days) | 0.75× | Keyword match but no shared entities, within 7 days |
| Direct + stale (30+ days) | 0.5× | Shares entities but over a month old |
| Indirect + stale (30+ days) | 0.25× | Keyword match only, over a month old |

### How to apply

1. Determine the base delta from the confidence evolution rules in
   `ref/reflect.md` (+0.1, -0.1, -0.2, or -0.05).
2. Assess the strongest piece of evidence driving the update.
3. Multiply: `effective_delta = base_delta × multiplier`.
4. Round to 2 decimal places.
5. Pass the effective delta to:
   ```bash
   python3 skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index N --delta <effective_delta>
   ```

### Examples

- Base delta: +0.1 (reinforcing). Evidence is direct and recent.
  Effective: `+0.1 × 1.5 = +0.15`
- Base delta: -0.2 (strong contradiction). Evidence is indirect and stale.
  Effective: `-0.2 × 0.25 = -0.05`
- Base delta: -0.05 (decay). No multiplier applies to decay — it is
  time-based, not evidence-based. Use -0.05 as-is.

---

## Technique 3: Counterfactual analysis

For high-confidence beliefs (≥ 0.6), assess what would break if the
belief were wrong. This surfaces hidden dependencies and prevents
over-confident beliefs from going unexamined.

### Procedure

For each belief with confidence ≥ 0.6:

1. **Ask the counterfactual.** "If this belief were false, which other
   beliefs, entity summaries, or world knowledge entries would need
   revision?"

2. **Search for dependents.**
   ```bash
   python3 skills/memory/scripts/memory-recall.py --entity "<belief-entity>" --cross-section --json
   ```
   Review the results for entries that assume or build on this belief.

3. **Classify the belief's dependency impact.**

   | Dependents found | Classification | Action |
   |-----------------|---------------|--------|
   | 3+ entries depend on it | **Load-bearing** | Require stronger evidence before any downward confidence change; probe more carefully |
   | 1–2 entries depend on it | **Connected** | Standard confidence rules apply |
   | No dependents | **Isolated** | Consider whether it's worth keeping; may be a candidate for pruning even at moderate confidence |

4. **Adjust behavior based on classification.**
   - **Load-bearing beliefs:** If a contradiction is found, don't just
     lower this belief's confidence — also flag the dependent entries
     for review in the reflect output. Don't auto-modify dependents,
     just report them.
   - **Isolated beliefs** below 0.5 confidence: flag as pruning
     candidates even though they're above the 0.2 threshold, since
     they don't connect to anything.

5. **Report.** In the output, include for each analyzed belief:
   - The counterfactual question
   - Number and list of dependent entries found
   - Classification (load-bearing / connected / isolated)
   - Whether any dependents were flagged for review

### When to skip

- Beliefs below 0.6 confidence: they're already tentative, so
  dependency analysis adds little value.
- During auto-reflect: skip counterfactual analysis to keep latency
  low. Run it only during explicit or full reflect passes.

---

## Technique 4: Belief conflict detection

Detect pairs of beliefs that share entities but express opposing
judgments. Contradictory beliefs undermine epistemic clarity.

### Procedure

1. **Run the conflict detector.**
   ```bash
   python3 skills/memory/scripts/memory-manage.py check-conflicts
   ```
   The script compares all belief pairs and flags those that share
   entities but have opposing sentiment signals.

2. **Review each conflict.** For each flagged pair, the output includes:
   - Both belief texts with their indices and confidence scores
   - Shared entities
   - A recommendation (keep the higher-confidence one, or merge)

3. **Resolve conflicts.** For each pair, choose one of:

   | Resolution | When to use |
   |-----------|-------------|
   | Keep higher-confidence belief, prune the other | Clear winner with stronger evidence |
   | Merge into a nuanced belief | Both capture valid partial truths |
   | Lower confidence on both | Insufficient evidence to resolve |
   | Keep both | They aren't actually contradictory after review |

4. **Report.** In the output, list each conflict found, the resolution
   chosen, and any confidence changes or pruning applied.

### When to skip

- If there are fewer than 2 beliefs, no conflicts are possible.
- During auto-reflect: run conflict detection (it's script-driven and
  fast) but defer resolution to the next explicit reflect if conflicts
  are ambiguous.

---

## Technique 5: Hierarchical reflections

Synthesize cross-cutting patterns from multiple experiences and beliefs
into higher-level reflection entries. These go in the `## Reflections`
section (see `ref/format.md` for format).

### When to generate reflections

Generate a new reflection when you notice any of:

- **Recurring pattern**: 3+ experiences share a common root cause or
  theme that isn't captured by any single entry.
- **Cross-entity connection**: events affecting different entities turn
  out to be related by a shared mechanism.
- **Belief cluster**: multiple beliefs point to the same underlying
  principle that could be stated more clearly as one reflection.

### Procedure

1. **Identify candidates.** After reviewing experiences and beliefs,
   look for clusters that share entities or themes:
   ```bash
   python3 skills/memory/scripts/memory-recall.py --entity "<shared-entity>" --cross-section --json
   ```

2. **Draft the reflection.** Write a synthesis that:
   - Connects 2+ source memories into a pattern
   - States the insight at a higher level than any individual memory
   - Uses entity tags spanning all relevant entities
   - Is useful for future decision-making, not just a summary

3. **Check for duplicate reflections.** Before adding:
   ```bash
   python3 skills/memory/scripts/memory-manage.py check-duplicate --section reflections --candidate "the reflection text"
   ```

4. **Write the reflection** to the `## Reflections` section using the
   standard guarded-write path.

### Quality bar

A reflection should pass this test: "Would reading this single entry
save the agent from having to re-derive the pattern from scattered
experiences?" If yes, it's worth creating. If it's just a restatement
of one experience, it doesn't qualify.

### Examples

Bad (just restating one experience):
```text
- **2026-03-26** {entities: port-5432} Port 5432 was already bound.
```

Good (synthesizing a cross-cutting pattern):
```text
- **2026-03-26** {entities: integration-tests, dev-server, port-5432} Three separate debugging sessions involved port conflicts from stale processes. The dev environment does not clean up child processes on exit, which causes cascading issues across unrelated test suites and the dev server.
```

### When to skip

- During auto-reflect: skip hierarchical reflection synthesis. It
  requires careful cross-memory analysis that is better done during
  explicit reflect passes.
- If there are fewer than 5 total experiences, the evidence base is
  too thin for meaningful pattern extraction.
