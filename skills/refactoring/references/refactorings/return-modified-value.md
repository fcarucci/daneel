# Return Modified Value

Official source: https://refactoring.com/catalog/returnModifiedValue.html

## Description

Return the updated value explicitly instead of hiding mutation.

## Scope

Mutation-heavy flows where making outputs explicit improves comprehension and testing.

## When To Apply

Apply when mutation is implicit and making data flow explicit would simplify callers and tests.

## Steps

1. Find where mutation is implicit and hard to follow.
2. Introduce an explicit returned value or split operation while preserving current behavior.
3. Update callers to use the explicit result.
4. Remove the old implicit path and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
