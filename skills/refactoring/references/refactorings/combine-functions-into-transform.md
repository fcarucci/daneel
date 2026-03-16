# Combine Functions into Transform

Official source: https://refactoring.com/catalog/combineFunctionsIntoTransform.html

## Description

Unify related derivations into one explicit transformation step.

## Scope

Related functions or derivations that should be grouped around one concept or transform.

## When To Apply

Apply when several functions clearly revolve around one concept and callers must stitch them together manually.

## Steps

1. Identify the functions that clearly belong to one concept or derived result.
2. Introduce the new class or transform boundary.
3. Move the related logic into that boundary and reduce duplication among callers.
4. Run tests and then simplify call sites to take advantage of the new grouping.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
