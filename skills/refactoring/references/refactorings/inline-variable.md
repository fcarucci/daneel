# Inline Variable

Official source: https://refactoring.com/catalog/inlineVariable.html

## Description

Remove a temporary that does not improve clarity.

## Scope

Helpers, temps, or classes that now add indirection without improving comprehension or changeability.

## When To Apply

Apply when the abstraction is trivial, one-off, or obscures the real flow instead of clarifying it.

## Steps

1. Confirm the abstraction adds little explanatory or reuse value.
2. Replace the call site or wrapper with the underlying code or state access.
3. Simplify now-redundant parameters, locals, or forwarding methods.
4. Delete the old abstraction and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
