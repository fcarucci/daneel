---
name: memory
description: >
  Persistent memory system for the agent. Two-tier storage: user
  (~/.agents/memory/MEMORY.md) and project (<repo>/MEMORY.md).
  Five memory networks: experiences, world knowledge, beliefs,
  reflections, entity summaries.
---

# Memory System

## Step 1: Determine the action

**Read this table FIRST before doing anything else.** Match the user's
request to an action, then follow only that action's instructions.

| User says | Action | Subagent? | Read these refs |
|-----------|--------|-----------|----------------|
| "Remember this", "Don't forget", "Note that", "Keep in mind" | **remember** | **YES — spawn subagent** | `ref/format.md` + `ref/retain.md` |
| "What do you remember?", "Show me memories", "What are your last memories?" | **show** | No | `ref/recall.md` |
| "What do you know about X?", "Any memories about Y?" | **recall** | No | `ref/recall.md` |
| "Reflect on your memory", "Dream", "Time for a reflection", "Review your beliefs" | **reflect** | **YES — spawn subagent** | `ref/reflect.md` + `ref/reflect-techniques.md` + `ref/profile.md` |
| "Forget about X", "Delete that memory", "Remove the belief about Y" | **forget** | No | `ref/forget.md` |
| "Promote this to the project" | **promote** | **YES — spawn subagent** | `ref/promote.md` |

**Critical rule:** If the user asks to **store** something (remember,
don't forget, note that, keep in mind), you MUST spawn a subagent.
Do not run `--show`, do not load the digest, do not do anything else
first. Spawn the subagent immediately with the content to remember.

### How to spawn a remember subagent

```text
Read and follow skills/memory/SKILL.md.

action: remember
content: <the thing to remember, in the user's words or summarized>
context: <tag: debug|testing|tooling|workflow|decision|preference|infra|docs|ui|backend|security>
```

The subagent reads SKILL.md, follows the dispatch to `ref/format.md` +
`ref/retain.md`, and executes the full retain workflow: entity extraction,
duplicate checking, format validation, guarded write, and auto-reflect.

After the subagent completes, tell the user what was remembered.

## Architecture

Four-network memory model inspired by
[Hindsight](https://arxiv.org/abs/2512.12818), adapted for text-only
markdown storage.

### Two-tier scoping

| Scope | File | Default for |
|-------|------|-------------|
| **User** | `~/.agents/memory/MEMORY.md` | New memories (writes) |
| **Project** | `<repo>/MEMORY.md` | Promotion target only |

New memories go to **user** scope unless explicitly directed to project.
Recall searches **both** scopes by default.

### Five memory networks

| Network | Epistemic role | Has confidence? |
|---------|---------------|----------------|
| **Experiences** | What the agent observed or did | No |
| **World Knowledge** | Verified objective facts | Yes |
| **Beliefs** | Subjective judgments that evolve | Yes |
| **Reflections** | Higher-level patterns from multiple memories | No |
| **Entity Summaries** | Synthesized entity profiles | No |

**Auto-reflect:** The `remember` action automatically triggers a
reflect pass when beliefs are stale, low-confidence, or unsupported
by recent experiences. See `ref/retain.md` for trigger conditions.

Script implementation details: `ref/scripts.md`

## Automatic memory retrieval

### Session-start loading

At the start of every session, load the memory digest into context:

```bash
python3 skills/memory/scripts/memory-recall.py --show
```

This replaces reading raw `MEMORY.md`. The digest includes world
knowledge, beliefs, reflections, entity summaries, and recent
experiences from both user and project scopes.

### Pre-task recall

Before starting any task, run a targeted recall against the task topic:

```bash
python3 skills/memory/scripts/memory-recall.py --entity "<topic>" --cross-section --json
python3 skills/memory/scripts/memory-recall.py --keyword "<keyword>" --json
```

| User says | Recall command |
|-----------|---------------|
| "Fix the integration tests" | `--entity "integration-tests" --cross-section` |
| "Work on the API gateway" | `--entity "api-gateway" --cross-section` |
| "The build is failing" | `--keyword "build" --cross-section` |

If no memories match, proceed normally.

## Automatic memory capture

### Post-task sweep (after every completed task)

After completing any task — **before committing** — ask:

> "What did I learn that would be useful in a future session?"

If non-empty, **spawn a memory subagent** with `action: remember` for
each lesson, then **tell the user what was remembered**:

> **Remembered:**
> - [debug] The integration test hangs if port 5432 is already bound.
> - [workflow] Running the combined dev command avoids CSS rebuild issues.

This is **not optional** — it is part of the mandatory task completion
sequence (see `docs/agent-workflows/DANEEL_WORKFLOW.md`).

### Session-end review

When the conversation is winding down ("thanks", "goodbye", "that's
all", or task done with no follow-up):

1. Scan for uncaptured lessons, surprises, decisions, or workarounds.
2. **Spawn a memory subagent** with `action: remember` for each item.
3. **Tell the user what was remembered.**
4. If the session was substantial, also spawn `action: reflect`.

## Subagent parameters (for write operations)

| Field | Required | Description |
|------|----------|-------------|
| `action` | yes | `remember`, `recall`, `reflect`, `maintain`, or `promote` |
| `content` | if remember | Narrative memory candidate |
| `context` | no | Tag: `debug`, `testing`, `tooling`, `workflow`, `decision`, `preference`, `infra`, `docs`, `ui`, `backend`, `security` |
| `query` | if recall | Search terms, entity, or date range |
| `section` | no | Limit to one section |
| `scope` | no | `user` (default writes), `project`, or `both` (default reads) |
| `index` | if promote | Index in user memory to promote |

## Invocation examples

### Remember (spawn subagent)

```text
Read and follow skills/memory/SKILL.md.

action: remember
content: The integration test suite requires port 5432 to be free; it hangs if another process is already bound.
context: testing
```

### Remember to project scope (spawn subagent)

```text
Read and follow skills/memory/SKILL.md.

action: remember
content: PostgreSQL 16 requires explicit listen_addresses for remote connections.
context: infra
scope: project
```

### Show (run directly)

```bash
python3 skills/memory/scripts/memory-recall.py --show
python3 skills/memory/scripts/memory-recall.py --show --scope user
python3 skills/memory/scripts/memory-recall.py --show --last 20
python3 skills/memory/scripts/memory-recall.py --show --days 7
```

### Recall (run directly)

```bash
python3 skills/memory/scripts/memory-recall.py --entity "api-gateway" --cross-section --json
python3 skills/memory/scripts/memory-recall.py --keyword "database" --json
```

### Reflect (spawn subagent)

```text
Read and follow skills/memory/SKILL.md.

action: reflect
```

### Maintain (spawn subagent)

```text
Read and follow skills/memory/SKILL.md.

action: maintain
```

### Promote (spawn subagent)

```text
Read and follow skills/memory/SKILL.md.

action: promote
section: experiences
index: 0
allow_project_promotion: true
```

### Forget (run directly)

See `ref/forget.md` for the full workflow.
