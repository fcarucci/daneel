# Split Phase

Official source: https://refactoring.com/catalog/splitPhase.html

## Description

Separate work into distinct sequential stages with clear outputs.

## Scope

Variables, loops, statements, and processing stages that currently mix multiple responsibilities.

## When To Apply

Apply when one construct is carrying several meanings or stages and that coupling makes change risky.

## Steps

1. Identify the current phases that are coupled together, such as parsing and rendering.
2. Define an explicit output structure for the first phase.
3. Refactor the second phase to consume only that intermediate result.
4. Run tests, then simplify each phase independently.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
