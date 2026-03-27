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
python3 scripts/memory-manage.py extract-entities --text "your memory text"
```

## Canonical context tags

Normalize free-form context into this vocabulary:

`debug`, `testing`, `tooling`, `workflow`, `decision`, `preference`,
`infra`, `docs`, `ui`, `backend`, `security`

Omit the tag rather than inventing a noisy synonym.
