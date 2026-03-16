# Replace Temp with Query

Official source: https://refactoring.com/catalog/replaceTempWithQuery.html

## Description

Replace a mutable temp with a side-effect-free query.

## Scope

Legacy mechanisms, conditionals, primitives, loops, temporaries, and object structures that need a better design shape.

## When To Apply

Apply when an older mechanism works but makes extension, testing, or reasoning harder than it should be.

## Steps

1. Characterize the current behavior so the redesign has a safety net.
2. Introduce the new mechanism alongside the old one with the smallest bridging code possible.
3. Migrate one caller, branch, or usage path at a time.
4. Remove the old mechanism and rerun the full relevant test set.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
- Do not delete the old path until the new one is covered and callers have migrated.
