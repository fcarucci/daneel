---
name: memory
description: >
  Persistent memory system for the agent. ALWAYS invoked as a subagent
  for writes. Two-tier storage: user (~/.agents/memory/MEMORY.md) and
  project (<repo>/MEMORY.md). Four memory networks: experiences, world
  knowledge, beliefs, entity summaries. Read-only inspection (show) can
  be run directly without a subagent.
---

# Memory System

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

### Four memory networks

| Network | Epistemic role | Has confidence? |
|---------|---------------|----------------|
| **Experiences** | What the agent observed or did | No |
| **World Knowledge** | Verified objective facts | Yes |
| **Beliefs** | Subjective judgments that evolve | Yes |
| **Entity Summaries** | Synthesized entity profiles | No |

### Action dispatch

Each action needs only specific reference files. Read SKILL.md (this
file) for routing, then read the listed refs for the action you need.

| Action | Subagent? | Read these refs | Description |
|--------|-----------|----------------|-------------|
| `show` | No | `ref/recall.md` | Display memory digest |
| `remember` | Yes | `ref/format.md` + `ref/retain.md` | Store a new memory (includes auto-reflect) |
| `recall` | Yes | `ref/recall.md` | Query by keyword/entity/date |
| `reflect` | Yes | `ref/reflect.md` | Review and update beliefs |
| `maintain` | Yes | `ref/format.md` + `ref/reflect.md` | Full maintenance cycle |
| `promote` | Yes | `ref/promote.md` | Copy user → project |

**Auto-reflect:** The `remember` action automatically triggers a
reflect pass when beliefs are stale, low-confidence, or unsupported
by recent experiences. No explicit scheduling is needed — reflection
is a built-in part of the retain workflow. See `ref/retain.md` for
trigger conditions.

Full script reference: `ref/scripts.md`

## Memory inspection (no subagent — run directly)

When the user asks to **see** memories, run `scripts/memory-recall.py`
directly and display the output.

| User says | Command |
|-----------|---------|
| "What do you remember?" | `python3 scripts/memory-recall.py --show` |
| "What are your last memories?" | `python3 scripts/memory-recall.py --show --last 10` |
| "Show me your user memories" | `python3 scripts/memory-recall.py --show --scope user` |
| "Show me the project memories" | `python3 scripts/memory-recall.py --show --scope project` |
| "What happened last week?" | `python3 scripts/memory-recall.py --show --days 7` |
| "What do you know about \<topic\>?" | `python3 scripts/memory-recall.py --entity "<topic>" --cross-section` |
| "Any memories about \<keyword\>?" | `python3 scripts/memory-recall.py --keyword "<keyword>"` |

## Memory writes (subagent required)

Spawn a subagent for any operation that modifies memory. The subagent
reads this file for routing, then reads the ref files listed in the
dispatch table above.

Invoke when:

- The model encounters information useful in future sessions.
- The user explicitly says "remember this" or equivalent.
- A workflow produces a postmortem outcome.
- A maintenance pass is needed.

### Subagent parameters

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

### Show (no subagent)

```bash
python3 scripts/memory-recall.py --show
python3 scripts/memory-recall.py --show --scope user
python3 scripts/memory-recall.py --show --scope project
python3 scripts/memory-recall.py --show --last 20
python3 scripts/memory-recall.py --show --days 7 --scope user
```

### Remember (subagent)

```text
Read skills/memory/SKILL.md, then ref/format.md and ref/retain.md.

action: remember
content: The integration test suite requires port 5432 to be free; it hangs if another process is already bound to that port. Killing the stale process fixes the issue.
context: testing
```

### Remember to project scope

```text
Read skills/memory/SKILL.md, then ref/format.md and ref/retain.md.

action: remember
content: PostgreSQL 16 requires explicit listen_addresses for remote connections.
context: infra
scope: project
```

### Recall

```text
Read skills/memory/SKILL.md, then ref/recall.md.

action: recall
query: entity=api-gateway, cross-section=true
```

### Promote

```text
Read skills/memory/SKILL.md, then ref/promote.md.

action: promote
section: experiences
index: 0
```

### Reflect

```text
Read skills/memory/SKILL.md, then ref/reflect.md.

action: reflect
```

### Maintain

```text
Read skills/memory/SKILL.md, then ref/format.md and ref/reflect.md.

action: maintain
```
