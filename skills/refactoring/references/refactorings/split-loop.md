# Split Loop

Official source: https://refactoring.com/catalog/splitLoop.html

## Description

Break one loop with several jobs into focused loops.

## Scope

Variables, loops, statements, and processing stages that currently mix multiple responsibilities.

## When To Apply

Apply when one construct is carrying several meanings or stages and that coupling makes change risky.

## Steps

1. Identify the mixed responsibilities or multiple meanings in the current construct.
2. Create distinct variables, loops, phases, or statement groups for each responsibility.
3. Move logic into the new structure one responsibility at a time.
4. Delete the mixed form and rerun tests after each meaningful step.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
