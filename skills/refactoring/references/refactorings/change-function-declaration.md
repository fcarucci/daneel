# Change Function Declaration

Official source: https://refactoring.com/catalog/changeFunctionDeclaration.html

## Description

Safely evolve a function's name, parameters, or calling contract.

## Scope

APIs and identity/value boundaries that need to evolve safely while preserving behavior.

## When To Apply

Apply when you need to evolve a contract or identity model without breaking users all at once.

## Steps

1. Add the new function signature, wrapper, or overload without breaking current callers.
2. Migrate callers gradually to the new name or parameter list.
3. Remove the deprecated signature once all call sites have moved.
4. Run tests after each migration slice.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
