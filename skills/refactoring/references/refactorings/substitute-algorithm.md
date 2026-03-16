# Substitute Algorithm

Official source: https://refactoring.com/catalog/substituteAlgorithm.html

## Description

Swap in a simpler or clearer algorithm while preserving behavior.

## Scope

Implementations that preserve behavior but can be simplified or clarified wholesale.

## When To Apply

Apply when the current algorithm is correct but difficult to understand, maintain, or evolve.

## Steps

1. Write or strengthen tests for the externally visible behavior first.
2. Implement the new algorithm behind the same interface.
3. Compare edge cases and performance-sensitive assumptions if they matter.
4. Delete the old algorithm once the new one is proven.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
- Do not delete the old path until the new one is covered and callers have migrated.
