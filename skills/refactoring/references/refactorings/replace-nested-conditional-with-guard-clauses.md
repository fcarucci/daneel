# Replace Nested Conditional with Guard Clauses

Official source: https://refactoring.com/catalog/replaceNestedConditionalWithGuardClauses.html

## Description

Flatten nested branching by handling special cases first.

## Scope

Legacy mechanisms, conditionals, primitives, loops, temporaries, and object structures that need a better design shape.

## When To Apply

Apply when an older mechanism works but makes extension, testing, or reasoning harder than it should be.

## Steps

1. Identify exceptional or early-exit cases hidden inside nesting.
2. Turn each such case into an early return or equivalent guard.
3. Leave the main successful path as the flat fall-through case.
4. Run tests after each guard extraction so behavior stays identical.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
- Do not delete the old path until the new one is covered and callers have migrated.
