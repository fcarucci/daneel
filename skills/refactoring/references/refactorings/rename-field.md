# Rename Field

Official source: https://refactoring.com/catalog/renameField.html

## Description

Rename a field so its purpose is clear.

## Scope

Local names, public APIs, and data model semantics where clearer intent reduces cognitive load without changing behavior.

## When To Apply

Apply when the current name hides intent, reflects stale history, or makes callers guess the meaning.

## Steps

1. Identify the declaration and all direct and indirect usages.
2. Rename the declaration in the smallest safe scope first, using automated rename support where possible.
3. Update callers, serialization boundaries, docs, and tests that depend on the old name.
4. Run the relevant tests and remove any temporary compatibility aliases.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
