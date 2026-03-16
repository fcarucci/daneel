# Fowler Refactoring Notes

This skill is based on Martin Fowler's refactoring guidance and official supporting material.

Primary sources:

- Martin Fowler, *Refactoring: Improving the Design of Existing Code*:
  https://martinfowler.com/books/refactoring.html
- Refactoring.com definition and catalog:
  https://refactoring.com/
- Catalog of Refactorings:
  https://refactoring.com/catalog/

## Source-backed principles

- Refactoring is a controlled technique for improving design while preserving observable behavior.
- The safest process is a series of small transformations, each low risk on its own.
- Testing is a core part of refactoring, not a separate afterthought.
- Code smells are cues to investigate, not rules to obey mechanically.
- The catalog exists to help choose a fitting transformation rather than inventing one from scratch.

## Practical mapping for agent work

When the code smells like:

- duplication: prefer extract, move, consolidate, or parameterize
- unclear intent: rename, extract, inline, or split phase
- tangled conditionals: decompose, add guard clauses, consolidate fragments
- scattered state logic: move function, encapsulate data, centralize derived state
- unnecessary layers: inline and delete

## Important guardrails

- Do not call a behavior-changing rewrite a refactor.
- Do not batch many semantic changes under the label of cleanup.
- Do not introduce abstractions "for the future" without a real current pressure.
- Prefer one explicit simplification over a framework-shaped abstraction.
