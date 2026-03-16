# Remove Dead Code

Official source: https://refactoring.com/catalog/removeDeadCode.html

## Description

Delete code that is no longer reachable or needed.

## Scope

Dead or unnecessary code paths, APIs, indirection layers, and subclassing structures.

## When To Apply

Apply when the code no longer contributes to the behavior you care about and only increases maintenance cost.

## Steps

1. Search for references, runtime entry points, and config paths that still reach the code.
2. Add characterization coverage first if the code looks unused but behavior is uncertain.
3. Delete the code and any companion helpers or tests that only served it.
4. Run tests and verify no documented behavior disappeared.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
