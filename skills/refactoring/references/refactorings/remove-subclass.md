# Remove Subclass

Official source: https://refactoring.com/catalog/removeSubclass.html

## Description

Delete a trivial subclass and fold its behavior into a simpler model.

## Scope

Dead or unnecessary code paths, APIs, indirection layers, and subclassing structures.

## When To Apply

Apply when the code no longer contributes to the behavior you care about and only increases maintenance cost.

## Steps

1. Prove the code is unnecessary by searching references and checking behavior coverage.
2. Redirect any remaining legitimate callers to the simpler supported path.
3. Delete the dead or obsolete code, configuration, and tests that only existed for it.
4. Run tests and verify no behavior depended on the removed path.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
