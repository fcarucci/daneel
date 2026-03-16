# Hide Delegate

Official source: https://refactoring.com/catalog/hideDelegate.html

## Description

Stop callers from reaching through an object to its internals.

## Scope

Object relationships where callers know too much about internal structure.

## When To Apply

Apply when callers have to reach through one object to another and this leaks internal structure.

## Steps

1. Locate the object boundary that is leaking too much structure.
2. Introduce the simpler delegation or direct access path.
3. Update callers to the new interaction style.
4. Remove the old navigation path and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
