# Collapse Hierarchy

Official source: https://refactoring.com/catalog/collapseHierarchy.html

## Description

Remove an unnecessary inheritance layer by folding behavior together.

## Scope

Inheritance structures that carry accidental complexity rather than useful variation.

## When To Apply

Apply when the inheritance tree adds indirection without paying for itself in shared behavior.

## Steps

1. Decide which type structure best represents the real variation.
2. Move behavior and state toward the simplified hierarchy shape.
3. Update callers and constructors to the new structure.
4. Delete the no-longer-useful hierarchy layer and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
