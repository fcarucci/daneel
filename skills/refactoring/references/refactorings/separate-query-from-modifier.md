# Separate Query from Modifier

Official source: https://refactoring.com/catalog/separateQueryFromModifier.html

## Description

Split a method that both returns data and changes state.

## Scope

Branching logic that is too dense, repetitive, or hard to reason about.

## When To Apply

Apply when branching hides the main path, duplicates outcomes, or mixes decision logic with work.

## Steps

1. Separate decision logic from the work performed in each branch.
2. Simplify or extract the conditions and branch bodies into named units.
3. Flatten or merge branches where outcomes are equivalent.
4. Run tests after each branch-level simplification.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
