# Change Reference to Value

Official source: https://refactoring.com/catalog/changeReferenceToValue.html

## Description

Replace shared identity semantics with explicit value semantics.

## Scope

APIs and identity/value boundaries that need to evolve safely while preserving behavior.

## When To Apply

Apply when you need to evolve a contract or identity model without breaking users all at once.

## Steps

1. Introduce the new contract or identity model in a way that can coexist briefly with the old one.
2. Migrate call sites or object construction incrementally.
3. Remove compatibility code only after all callers are on the new form.
4. Run tests after each migration step and again after cleanup.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
