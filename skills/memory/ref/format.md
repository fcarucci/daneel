# Memory Format Reference

## Memory file structure

Memories live in two files with identical structure:

- **User**: `~/.agents/memory/MEMORY.md` (created on first use via `init-user`)
- **Project**: `<repo>/MEMORY.md` (committed to version control)

Required structure (same for both files):

```markdown
# Agent Memory

## Experiences

<!-- Newest first. Format: - **YYYY-MM-DD** [context] {entities: e1, e2} Narrative memory text. -->

## World Knowledge

<!-- Verified, objective facts about the project and environment. Format:
- {entities: e1} Fact text. (confidence: 0.XX, sources: N) -->

## Beliefs

<!-- Agent's subjective judgments that evolve over time. Format:
- {entities: e1} Belief text. (confidence: 0.XX, formed: YYYY-MM-DD, updated: YYYY-MM-DD) -->

## Reflections

<!-- Higher-level patterns synthesized from multiple experiences and beliefs. Format:
- **YYYY-MM-DD** {entities: e1, e2} Reflection text. -->

## Entity Summaries

<!-- Synthesized profiles of key entities, regenerated when underlying memories change. Format:
### entity-name
Summary paragraph. -->
```

## Experience format

Each experience is one bullet with a narrative, self-contained description:

```text
- **2026-03-26** [debug] {entities: integration-tests, port-5432} The integration test suite hung indefinitely because another process was already bound to port 5432. Killing the stale database process resolved the issue.
```

Rules:

- Use today's date in `YYYY-MM-DD` format.
- Context tag is optional but strongly encouraged.
- **Entity tags are required.** Use `{entities: name1, name2}` inline.
- Text must be **narrative and self-contained**: a reader with no context
  should understand what happened, why it mattered, and what was learned.
  Avoid fragments like "port 5432 conflict" — instead write the full story.
- Keep entries newest-first.
- No duplicates or near-duplicates.

## Causal links

Experiences and reflections can optionally annotate cause-effect
relationships using inline causal tags. These are lightweight directed
edges that help the reflect operation trace reasoning chains.

### Format

Place causal tags after entity tags, before the narrative text:

```text
- **2026-03-26** [debug] {entities: dev-server, port-5432} {caused-by: build-watcher} The dev server crashed because the build watcher left a child process bound to port 5432.
- **2026-03-26** [debug] {entities: build-watcher} {causes: port-5432} The build watcher does not clean up child processes on exit, leaving ports bound.
```

### Supported causal tags

| Tag | Meaning | Example |
|-----|---------|---------|
| `{causes: entity}` | This event caused a problem in `entity` | Stale process → port conflict |
| `{caused-by: entity}` | This event was caused by `entity` | Port conflict ← stale process |
| `{enables: entity}` | This event makes `entity` possible | Config change → feature works |
| `{prevents: entity}` | This event blocks `entity` | Missing dep → build fails |

### Rules

- Causal tags are **optional**. Most experiences won't have them.
- Only add causal tags when the cause-effect relationship is clear
  from the experience, not speculative.
- The entity in the causal tag should also appear somewhere in the
  memory bank (as an entity tag on another entry, or as a known entity).
- Multiple causal tags are allowed on one entry.
- Causal tags are metadata for the reflect operation — they help the
  subagent trace impact chains during counterfactual analysis.

### Querying causal chains

Use keyword search to find causal relationships:

```bash
python3 skills/memory/scripts/memory-recall.py --keyword "caused-by: dev-server" --json
python3 skills/memory/scripts/memory-recall.py --keyword "causes:" --json
```

> **Design note:** In the Hindsight paper, causal links are edges in a
> graph database traversed with spreading activation. Our flat-file
> equivalent uses inline annotations that the subagent follows manually
> during reflect. This trades automatic traversal for simplicity and
> zero infrastructure.

## Narrative quality standard

Memories must be narrative, not fragmentary. Each entry should read as a
self-contained story that captures the full context: what happened, why,
and what was learned.

Bad (fragmented):
```text
- **2026-03-26** [debug] Port 5432 conflict.
```

Good (narrative):
```text
- **2026-03-26** [debug] {entities: integration-tests, port-5432} The integration test suite hung indefinitely because another process was already bound to port 5432. Killing the stale database process resolved the hang and allowed the test suite to complete normally.
```

## World knowledge format

Each world fact is one bullet with confidence and source count:

```text
- {entities: postgresql} PostgreSQL 16 requires explicit listen_addresses configuration for remote connections. (confidence: 0.95, sources: 3)
```

Rules:

- Entity tags required.
- Confidence is a float in `[0.0, 1.0]`.
- Sources count is the number of independent experiences supporting this.
- Facts must be objective, verifiable, and project-relevant.
- No duplicates.

> **Design note:** In the Hindsight paper, only opinions carry confidence
> scores — world facts are treated as objectively true. This implementation
> adds confidence to world knowledge as a practical extension: in real
> projects, "verified facts" can have varying certainty, and the sources
> count maps to the paper's convergence concept. This is a deliberate
> departure, not an oversight.

## Belief format

Each belief is one bullet with confidence and temporal metadata:

```text
- {entities: dev-server, build-watcher} Running the combined dev command is more reliable than starting the server alone for day-to-day development. (confidence: 0.70, formed: 2026-03-15, updated: 2026-03-20)
```

Rules:

- Entity tags required.
- Confidence in `[0.0, 1.0]` — represents strength of conviction.
- `formed` date is when the belief was first created.
- `updated` date changes whenever confidence is adjusted.
- Beliefs are subjective — they represent the agent's judgments, not
  verified truths.

## Reflection format

Reflections are higher-level patterns synthesized from multiple
experiences and beliefs during a reflect pass. They capture cross-entity
insights that no single experience or belief contains.

```text
- **2026-03-26** {entities: integration-tests, dev-server, port-5432} Three separate debugging sessions all involved port conflicts from stale processes. The underlying pattern is that the dev environment does not clean up child processes on exit, causing cascading issues across unrelated tools.
```

Rules:

- Use the date of the reflect pass, not the dates of source memories.
- Entity tags required — include all entities the pattern spans.
- Text must be a synthesis, not a copy of any single experience.
- Keep entries newest-first.
- Reflections are created during reflect passes, not during retain.
- A reflection should connect 2+ experiences or beliefs into a pattern
  that is more useful than any of them individually.

## Entity summary format

Each entity gets a `###` heading and a summary paragraph:

```text
### postgresql
PostgreSQL 16.x is the primary database. Config requires explicit listen_addresses for remote access. Connection pooling is handled by the application layer, not PgBouncer. Migrations run via the ORM's built-in migration tool.
```

Rules:

- One summary per entity.
- Preference-neutral: no opinions, just synthesized facts.
- Regenerated when underlying experiences or world knowledge change.
- Only created for entities with 3+ mentions across memories.

## Entity tag guidelines

Entity tags connect memories across sections. Use consistent, lowercase,
hyphenated names:

- `dev-server`, `postgresql`, `build-watcher`, `integration-tests`
- `api-gateway`, `redis`, `docker`, `ci-pipeline`
- `port-5432`, `dashboard`, `auth-service`

When in doubt about what to tag, run the entity extraction script:

```bash
python3 skills/memory/scripts/memory-manage.py extract-entities --text "your memory text"
```

## Canonical context tags

Normalize free-form context into this vocabulary:

`debug`, `testing`, `tooling`, `workflow`, `decision`, `preference`,
`infra`, `docs`, `ui`, `backend`, `security`

Omit the tag rather than inventing a noisy synonym.
