---
name: memory
description: >
  Persistent memory system for the agent. ALWAYS invoked as a subagent.
  Maintains episodic memories and curated facts in MEMORY.md at the repo
  root. Every run uses a deterministic read-plan-verify-write workflow:
  sanitize candidate input, prune stale memories, promote only strongly
  evidenced facts, curate the facts list, detect conflicts, and perform a
  single final write to MEMORY.md.
---

# Memory System

## When to invoke

Use this skill **exclusively as a subagent**. Never run its steps inline.

Invoke when **any** of these conditions is true:

- The model encounters information that will be useful in future sessions
  such as a lesson learned, workaround, decision, surprise, or durable
  preference.
- The user explicitly says "remember this", "note that", "keep in mind",
  or equivalent phrasing.
- A workflow produces a useful postmortem outcome such as a root cause, a
  verified fix, or a stable design decision.

The calling agent passes:

| Field | Required | Description |
|------|----------|-------------|
| `action` | yes | `remember` or `maintain` |
| `content` | if remember | Concise, self-contained memory candidate |
| `context` | no | Optional canonical tag such as `debug`, `testing`, `tooling`, `workflow`, `decision`, or `preference` |

- `remember`: stage a candidate episodic memory, then run the full
  maintenance pass.
- `maintain`: do not add a new candidate; run maintenance only.

## Scope and safety

This skill manages **only** `MEMORY.md`.

Non-negotiable:

1. **Read before write.** Always read `MEMORY.md` in full before planning
   any changes.
2. **Single final write.** Do not write partial state. Compute the final
   target content first, then write once.
3. **Re-read before write.** Immediately before writing, read
   `MEMORY.md` again. If it changed since the first read, merge against the
   latest state and recompute the final output instead of overwriting.
4. **No side effects.** Do not edit `AGENTS.md`, code, config, or any
   other file. If repo documentation appears out of sync, report it in the
   subagent output; do not auto-fix it here.
5. **Preserve structure.** Never remove or rename the section headers or
   HTML comments in `MEMORY.md`.
6. **Idempotent reruns.** Running the skill twice with the same input must
   produce the same final file.
7. **Fail closed.** If you cannot safely determine the correct final state,
   stop and report a blocked result rather than guessing or deleting data.
8. **Facts are verified only.** The Facts section never contains guesses,
   placeholders, or `[unverified]` entries.

## Sensitive data policy

Memory quality is less important than safety. Never persist raw sensitive
data.

Never store:

- Access tokens, API keys, passwords, secrets, cookies, SSH material, or
  credential-like strings.
- Full private URLs that embed secrets or internal-only parameters.
- Raw logs, stack traces, or command output if they may contain secrets.
- Personal data about the user or third parties unless the user explicitly
  asked for it to be remembered **and** it materially improves future help.

When useful knowledge depends on sensitive input:

- Store the sanitized lesson, not the raw value.
- Prefer redaction such as ``<redacted token>``.
- Prefer abstraction such as "the gateway token came from the OpenClaw
  config" over the token itself.

If a candidate memory contains sensitive data, redact it before any
duplicate check or write.

## Memory file

All memories live in **`MEMORY.md`** at the repository root.

Required structure:

```markdown
# Daneel Agent Memory

## Episodic Memories

<!-- Newest first. Format: - **YYYY-MM-DD** [context] Memory text -->

## Facts

<!-- Curated, verified facts. One fact per bullet. -->
```

### Episodic memory format

Each episodic memory is one bullet:

```text
- **2026-03-26** [debug] The mock-gateway test hangs if port 18789 is already bound.
```

Rules:

- Use today's date in `YYYY-MM-DD` format.
- The context tag is optional but strongly encouraged.
- The text must be complete and self-contained.
- Keep entries newest-first.
- Do not store duplicates or near-duplicates.

### Canonical context tags

Normalize free-form inputs into this vocabulary where possible:

- `debug`
- `testing`
- `tooling`
- `workflow`
- `decision`
- `preference`
- `infra`
- `docs`
- `ui`
- `backend`
- `security`

If no tag fits, omit the tag instead of inventing a noisy synonym.

### Fact format

Each fact is one bullet:

```text
- Tailwind 4 uses `@tailwindcss/cli` instead of the legacy `tailwindcss` CLI package.
```

Rules:

- One fact per bullet. No date tags.
- Facts must be general, durable, project-relevant, and self-contained.
- Facts must be verified against the current repo state before promotion or
  retention.
- No duplicates or near-duplicates.
- The facts list must stay intentionally small.

## Deterministic normalization

Before comparing, pruning, promoting, or writing memories, normalize
candidate text and existing entries using the same rules:

1. Trim surrounding whitespace and collapse internal repeated spaces.
2. Redact any sensitive values first.
3. Normalize context tags to the canonical vocabulary above.
4. Ignore date, tag, and obvious one-off framing such as "today",
   "in this session", or branch names when comparing semantic duplicates.
5. Normalize wording for tense and trivial synonyms where the meaning is
   unchanged, for example "hangs" vs "will hang" or "requires" vs "needs".

Use conservative duplicate detection:

- If two items clearly mean the same thing after normalization, keep the
  clearer one and treat the other as a duplicate.
- If equivalence is uncertain, keep both and report the ambiguity instead
  of deleting one.

## Operating workflow

Every invocation follows this exact order:

1. Read and parse `MEMORY.md`.
2. Validate structure and stop if the file is unsafe to rewrite.
3. Stage a sanitized candidate episodic memory, if `action=remember`.
4. Prune episodic memories.
5. Evaluate fact candidates from episodic patterns.
6. Curate the Facts section.
7. Render the final `MEMORY.md` content in memory.
8. Re-read `MEMORY.md` immediately before writing and detect concurrent
   changes.
9. If unchanged, write the final content once.
10. Re-read after write and verify the final file matches the planned
    content.

## Step 1: Parse and validate

Read `MEMORY.md` in full and confirm all of the following:

- The `# Daneel Agent Memory` heading exists.
- The `## Episodic Memories` and `## Facts` headings exist.
- Both HTML comment lines exist.
- Episodic entries, if present, match the bullet format.
- Fact entries, if present, are plain bullets.

If `MEMORY.md` does not exist, create the standard empty template in memory
and continue.

If the file is malformed but recoverable, repair it only if you can do so
without discarding user-visible content.

If the file is malformed and **not** safely recoverable, do not overwrite
it. Report a blocked result and require human review.

## Step 2: Stage the candidate episodic memory

*Skip this step if `action` is `maintain`.*

1. Sanitize the incoming `content` using the sensitive-data policy.
2. Normalize the `context` tag to the canonical vocabulary, or omit it.
3. Check for semantic duplicates against existing episodic memories.
4. If a clear duplicate exists, do not stage a new entry.
5. Otherwise, stage a new entry at the top of the Episodic Memories
   section in memory only. Do not write yet.

## Step 3: Prune episodic memories

After every invocation, review each episodic memory:

1. **Age.** Older than about 30 days is a pruning candidate. Older than
   about 90 days is a strong candidate.
2. **Ongoing relevance.** Is it still useful, still true, and still likely
   to matter in future sessions?
3. **Redundancy.** Has the knowledge been captured better elsewhere, such
   as a verified fact?
4. **Sensitivity.** Does the entry contain information that should now be
   redacted or removed under the sensitive-data policy?

Decision guide:

| Condition | Action |
|----------|--------|
| Recent, relevant, not redundant | keep |
| Recent, relevant, absorbed into a fact | usually keep for short-term context |
| Stale, redundant, or no longer useful | remove |
| Contains disallowed sensitive data | redact if possible, otherwise remove |

When pruning, prefer deletion over explanatory clutter. Do not leave tomb
stones in the file.

## Step 4: Extract fact candidates

**Fact promotion has a deliberately high bar.** Most episodic memories
should remain episodic.

### Promotion gates

A candidate fact must pass **all** gates below:

1. **Convergence.** At least **three independent episodic memories** point
   to the same underlying truth. Different phrasings are fine; the
   underlying lesson must be the same.
2. **Consistency.** The supporting memories do not materially contradict
   one another.
3. **Verification.** The candidate is confirmed against the current repo
   state. Verification is mandatory, not implied.
4. **Relevance.** The fact is non-obvious, actionable, and likely durable.
5. **Generality.** The wording removes one-off context while keeping
   accurate technical specifics.

### Verification rubric

For every promoted, retained, rewritten, removed, or demoted fact, record
the evidence used in your subagent output:

- **Claim:** the fact text being evaluated.
- **Evidence type:** one or more of `code`, `config`, `docs`, `tests`, or
  `user-stated preference`.
- **Evidence checked:** the exact file, symbol, config key, or command
  inspected.
- **Outcome:** verified, contradicted, redundant, or unverifiable.

If a claim cannot be verified, it is **not** a fact. Keep it as episodic
memory only if it is still useful and safe.

### What does not qualify as a fact

- A single observation, even if important.
- A preference stated once.
- A decision that has not held up across later work.
- Information already obvious from reading `AGENTS.md`,
  `DANEEL_WORKFLOW.md`, or nearby code.
- Any speculative, forward-looking, or unverifiable claim.

## Step 5: Curate the Facts section

Review every existing fact with the same rigor used for promotion.

1. **Accuracy.** Remove facts that are no longer true.
2. **Verification.** If a fact cannot be verified now, remove it from
   Facts. If the underlying lesson is still useful, keep or create an
   episodic memory instead.
3. **Contradiction.** Resolve conflicts by checking the source of truth.
4. **Redundancy.** Merge or remove near-duplicates.
5. **Quality.** Rewrite vague or overly specific facts only if they can be
   improved without weakening accuracy.
6. **Evidence debt.** If you cannot identify three independent episodic
   memories that support an existing fact, demote it out of Facts.

### Facts size budget

The Facts section should stay small and high-signal:

- Target: 5 to 15 facts.
- Soft cap: 20 facts.
- If adding a fact would exceed 20, prune or merge lower-value facts first.

## Step 6: Documentation drift check

This skill does **not** edit repo documentation.

If you notice that `AGENTS.md` or other docs no longer mention the memory
system accurately, report the drift in the subagent output with a concise
recommended fix. Do not edit those files from this skill.

## Step 7: Render and write once

After staging, pruning, extraction, and curation:

1. Render the full final `MEMORY.md` content in memory.
2. Validate that the rendered file preserves the required headings and
   comments.
3. Re-read the current on-disk `MEMORY.md`.
4. If the on-disk content changed since the initial read, merge against the
   newer content and recompute the final file before writing.
5. Write the final content once.
6. Re-read and confirm the file now matches the rendered target exactly.

## Required subagent output

At the end of the run, report:

- Whether a new episodic memory was staged and written, or skipped as a
  duplicate.
- Any sanitization applied, especially secret or personal-data redaction.
- Number of episodic memories pruned, redacted, or retained with one-line
  reasons.
- Fact extraction summary:
  - candidates considered
  - which promotion gates each candidate passed or failed
  - any fact promoted, with the fact text and the three+ supporting
    episodic memories
- Fact curation summary:
  - facts retained, rewritten, removed, or demoted
  - evidence used for each decision, following the verification rubric
- Whether concurrent-write detection triggered and how it was resolved.
- Any documentation drift noticed, with a suggested fix.
- Final counts: total episodic memories and total facts.
- If blocked, exactly why the skill refused to write.

## Invocation examples

### Remembering something

The calling agent spawns a subagent with this context:

```text
Read and follow skills/memory/SKILL.md.

action: remember
content: The e2e_mock_gateway test requires port 18789 to be free; it hangs if another process is already bound to that port.
context: testing
```

### Maintenance-only pass

```text
Read and follow skills/memory/SKILL.md.

action: maintain
```

## Error handling

- **`MEMORY.md` does not exist**: create the standard template in memory
  and continue.
- **Recoverable format issue**: repair only if you can preserve all
  existing user-visible content.
- **Unrecoverable format issue**: stop and report a blocked result. Do not
  overwrite the file.
- **Sensitive data discovered in existing memory**: redact if the lesson can
  be preserved safely; otherwise remove the entry and report the action.
- **Ambiguous duplicate detection**: keep both entries and report the
  ambiguity rather than deleting data.
- **Ambiguous fact accuracy**: do not keep uncertain entries in Facts.
  Remove or demote them to episodic memory if still useful.
