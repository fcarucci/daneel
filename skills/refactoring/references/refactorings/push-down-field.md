# Push Down Field

Official source: https://refactoring.com/catalog/pushDownField.html

## Description

Move superclass state down to only the subclasses that use it.

## Scope

Fields, functions, methods, and constructor logic whose current owner is not the best home.

## When To Apply

Apply when behavior or data depends more on another module or type than on its current owner.

## Steps

1. Choose the destination that owns the data or responsibility more naturally.
2. Move the field or behavior while keeping a temporary forwarding path if callers need a staged migration.
3. Update callers incrementally to the new location.
4. Remove the forwarding layer or duplicate member and rerun tests.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
- Avoid leaving permanent forwarding layers behind unless they improve the design.
