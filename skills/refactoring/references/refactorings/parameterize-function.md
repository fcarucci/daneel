# Parameterize Function

Official source: https://refactoring.com/catalog/parameterizeFunction.html

## Description

Replace several near-identical functions with one parameterized function.

## Scope

Call signatures and data passing patterns that are too noisy or fragmented.

## When To Apply

Apply when several arguments travel together or a call signature hides the real concept being passed.

## Steps

1. Find the call pattern that is noisy, repetitive, or hiding the real concept.
2. Introduce the cleaner function signature or object boundary.
3. Migrate callers in small steps, keeping compatibility shims only as long as needed.
4. Remove the old calling form and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
