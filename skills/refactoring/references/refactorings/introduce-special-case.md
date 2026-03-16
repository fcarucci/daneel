# Introduce Special Case

Official source: https://refactoring.com/catalog/introduceSpecialCase.html

## Description

Represent an edge case with a dedicated object or code path.

## Scope

Mutable state, records, collections, parameters, and edge cases that need a safer or clearer boundary.

## When To Apply

Apply when outside code can violate invariants, mutate state directly, or repeat edge-case handling.

## Steps

1. Identify the data or special case that should stop leaking across the codebase.
2. Introduce a narrow interface that exposes intention-revealing reads and writes.
3. Migrate callers away from direct mutation or repeated branching.
4. Enforce invariants at the new boundary and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
