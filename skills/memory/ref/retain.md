# Retain Operation (`action: remember`)

> **Prerequisite:** Read `ref/format.md` before this file — it defines
> the section formats you need to write entries correctly.

## Scope and safety

This skill manages **only** `MEMORY.md`.

Non-negotiable:

1. **Read before write.** Always read `MEMORY.md` in full before planning
   changes.
2. **Use the guarded writer.** Prefer `python3 skills/memory/scripts/memory-manage.py append-entry`
   over manual edits so secret screening, duplicate checks, entity
   canonicalization, and optimistic concurrency checks are enforced.
3. **Single final write.** Do not write partial state. Compute the final
   target content first, then write once.
4. **Re-read before write.** Immediately before writing, read `MEMORY.md`
   again. If it changed since the first read, merge against the latest
   state and recompute.
5. **No side effects.** Do not edit `AGENTS.md`, code, config, or any
   other file. Report drift in the subagent output instead.
6. **Preserve structure.** Never remove or rename section headers or HTML
   comments.
7. **Idempotent reruns.** Running the skill twice with the same input must
   produce the same file.
8. **Fail closed.** If you cannot safely determine the correct final state,
   stop and report a blocked result.
9. **World knowledge is verified only.** The World Knowledge section never
   contains guesses, placeholders, or `[unverified]` entries.

## Sensitive data policy

Never persist raw sensitive data. Never store:

- Tokens, API keys, passwords, secrets, cookies, SSH material
- Full private URLs with embedded secrets
- Raw logs or stack traces that may contain secrets
- Personal data unless the user explicitly asked for it

When useful knowledge depends on sensitive input, store the sanitized
lesson, not the raw value. Prefer abstraction ("the database password came
from the environment config") over the token itself.

## Scope

New memories go to **user** scope by default. Only use project scope when:

- The memory is about shared project infrastructure, not personal workflow.
- The user explicitly asks to store it in the project.
- The memory is being promoted from user scope.

To initialize user memory on first use:
```bash
python3 skills/memory/scripts/memory-manage.py init-user
```

## Workflow

1. Ensure user memory exists (`init-user` if needed).
2. Read and parse the target `MEMORY.md` (user scope by default).
3. Validate structure (run `python3 skills/memory/scripts/memory-manage.py validate --scope user`).
4. Screen the incoming `content` before any write:
   ```bash
   python3 skills/memory/scripts/memory-manage.py screen-text --text "the memory text"
   ```
   If the result is unsafe, store only a sanitized lesson or stop.
5. **Classify** the memory into the correct network:
   - Is it something that happened to/around the agent? → **Experience**
   - Is it an objective, verifiable fact about the project? → **World Knowledge**
   - Is it the agent's subjective judgment or preference? → **Belief**
6. **Extract entities** using the script:
   ```bash
   python3 skills/memory/scripts/memory-manage.py extract-entities --text "the memory text"
   ```
   Review the candidates and finalize the entity set.
7. **Check for duplicates** across both scopes:
   ```bash
   python3 skills/memory/scripts/memory-manage.py check-duplicate --section experiences --candidate "the memory text" --cross-scope
   ```
   If a clear duplicate exists in either scope, do not add a new entry.
8. Write the final entry via the guarded command:
   ```bash
   python3 skills/memory/scripts/memory-manage.py append-entry \
     --section experiences \
     --scope user \
     --date 2026-03-27 \
     --context testing \
     --entities "integration-tests,port-5432" \
     --text "the memory text"
   ```
   For world knowledge, also pass `--confidence` and `--sources`.
   For beliefs, also pass `--confidence` and optionally `--formed` / `--updated`.
9. For new beliefs, set initial confidence based on evidence strength:
   - `0.4–0.5`: tentative, based on a single observation
   - `0.6–0.7`: moderate, based on 2+ observations
   - `0.8+`: strong, based on repeated consistent evidence
10. **Reflect**: check whether the new memory reinforces or contradicts
    any existing beliefs. If so, update confidence scores:
    ```bash
    python3 skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index N --delta 0.1 --scope user
    ```
    Use `+0.1` for reinforcement, `-0.1` for weakening, `-0.2` for
    strong contradiction. Beliefs below `0.2` are pruning candidates.
11. Check if any entity now has 3+ mentions and lacks a summary. If so,
    write a new entity summary.
12. Verify the written file matches the plan.
13. **Auto-reflect check** (see below).

## Classification guide

| Signal | Network |
|--------|---------|
| "I found that…", "The test showed…", "When I tried…" | Experience |
| Tool version, config location, API behavior, file path | World Knowledge |
| "X is better than Y", "I think…", "Prefer…" | Belief |
| Summarizes an entity from 3+ memories | Entity Summary |

## Promotion from experience to world knowledge

An experience can be promoted to world knowledge when:

1. **Convergence**: 3+ independent experiences point to the same truth.
2. **Consistency**: supporting experiences do not contradict each other.
3. **Verification**: confirmed against the current repo state.
4. **Relevance**: non-obvious, actionable, and likely durable.
5. **Generality**: wording removes one-off context.

When promoting, set the initial confidence based on the strength of
evidence and record the source count.

## Deterministic normalization

Before comparing, pruning, or writing memories, normalize using:

1. Trim whitespace and collapse repeated spaces.
2. Redact sensitive values first.
3. Normalize context tags to the canonical vocabulary.
4. Ignore date, tag, and framing when comparing duplicates.
5. Normalize tense and trivial synonyms.

For deterministic duplicate detection, use the script:

```bash
python3 skills/memory/scripts/memory-manage.py check-duplicate --section experiences --candidate "text"
```

The script uses three complementary similarity metrics (sequence ratio,
Jaccard, overlap coefficient) with a threshold of 0.65.

## Auto-reflect

After every successful retain, check whether a full reflect pass is
warranted. This keeps beliefs and entity summaries from going stale
without requiring the caller to explicitly schedule reflection.

### Trigger conditions (any one is sufficient)

1. **Volume**: 5+ experiences exist that are newer than the most recent
   belief `updated` date. This means beliefs haven't been reviewed
   against recent evidence.
2. **Staleness**: any belief has an `updated` date older than 14 days.
3. **Low-confidence accumulation**: 2+ beliefs are below `0.3` confidence
   (candidates for pruning).
4. **Missing summaries**: an entity appears in 3+ memories but has no
   summary in the Entity Summaries section.

### How to check

After completing the retain write, run:

```bash
python3 skills/memory/scripts/memory-manage.py prune-beliefs --threshold 0.3
python3 skills/memory/scripts/memory-manage.py suggest-summaries
```

Inspect the belief `updated` dates from the file you just wrote.
If any trigger condition is met, run the full reflect workflow
(see `ref/reflect.md`) **in the same subagent invocation** — do not
return and ask the caller to spawn a separate reflect.

### What to report

In the subagent output, include:

- Whether auto-reflect was triggered and which condition(s) fired.
- If triggered: the reflect output (belief updates, pruning, summaries).
- If not triggered: "auto-reflect: no action needed."

## Required output

At the end of the run, report:

- Whether a new memory was added, and to which section.
- Classification reasoning (why experience vs. world knowledge vs. belief).
- Entity tags applied.
- Duplicate check result.
- Any belief confidence updates triggered.
- Any new entity summaries generated.
- Final counts per section.
- Whether concurrent-write detection triggered.
- Any documentation drift noticed.
- If blocked, exactly why.

## Error handling

- **`MEMORY.md` does not exist**: create the standard template and continue.
- **Recoverable format issue**: repair if you can preserve all existing content.
- **Unrecoverable format issue**: stop and report blocked.
- **Sensitive data in existing memory**: redact if possible, otherwise remove and report.
- **Ambiguous duplicate**: stop and report the ambiguity; do not bypass the guarded write path.
- **Script not found**: fall back to manual parsing but warn that deterministic operations are degraded.
