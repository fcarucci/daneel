# Replace Conditional with Polymorphism

Official source: https://refactoring.com/catalog/replaceConditionalWithPolymorphism.html

## Description

Move variant-specific branches into dedicated implementations.

## Scope

Legacy mechanisms, conditionals, primitives, loops, temporaries, and object structures that need a better design shape.

## When To Apply

Apply when an older mechanism works but makes extension, testing, or reasoning harder than it should be.

## Steps

1. Identify the discriminator and the behavior that changes with it.
2. Introduce subtype or strategy implementations for each variant.
3. Move each branch into the matching implementation and route dispatch through polymorphism.
4. Delete the central conditional once all variants are migrated and tested.

## Guardrails

- Keep the change behavior-preserving; if behavior changes, treat that as feature work and separate it.
- Prefer small migrations over one large rewrite.
- Run the fastest relevant tests after each meaningful step.
- Do not delete the old path until the new one is covered and callers have migrated.
