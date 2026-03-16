# Extract Superclass

Official source: https://refactoring.com/catalog/extractSuperclass.html

## Description

Move shared behavior and state into a common parent.

## Scope

Expressions, code fragments, classes, and shared behavior that need a clearer boundary or name.

## When To Apply

Apply when a fragment has a coherent purpose, deserves a name, or needs to be reused or isolated for testing.

## Steps

1. Isolate a coherent fragment and identify its inputs, outputs, and side effects.
2. Create the new function, class, or superclass with the smallest correct interface.
3. Replace the original fragment with the new abstraction and keep names intention-revealing.
4. Run fast tests, then simplify parameters, locals, and callers if the new boundary exposed more cleanup.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
