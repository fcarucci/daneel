# Extract Function

Official source: https://refactoring.com/catalog/extractFunction.html

## Description

Pull a coherent code fragment into a named function.

## Scope

Expressions, code fragments, classes, and shared behavior that need a clearer boundary or name.

## When To Apply

Apply when a fragment has a coherent purpose, deserves a name, or needs to be reused or isolated for testing.

## Steps

1. Choose a fragment that does one thing and whose boundaries are understandable.
2. Identify inputs and outputs, then create a function with a strong explanatory name.
3. Replace the original fragment with the new call.
4. Run tests and keep extracting until the surrounding function reads cleanly.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
