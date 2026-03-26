---
name: refactoring
description: Use when a user asks to refactor, simplify, clean up, or restructure code while preserving behavior; also use when project or team workflow requires a dedicated refactoring pass at the end of each implementation slice (features and bug fixes). Applies Martin Fowler style refactoring discipline and points to one reference file per refactoring from the official catalog.
---

# Refactoring Fowler

Use this skill when the goal is to improve code structure without changing observable behavior.

When your **repository agent guide or team workflow** mandates it, run this skill as a **required** pass after each implementation slice finishes green—feature or bug fix—before calling that slice complete or moving to the next review gate. In-loop tidy-ups while coding are not a substitute.

The official source is Martin Fowler's *Refactoring* and the online catalog:

- https://martinfowler.com/books/refactoring.html
- https://refactoring.com/catalog/

Read [references/fowler-refactoring.md](references/fowler-refactoring.md) for the high-level principles.
Read exactly one per-refactoring reference file from `references/refactorings/` when you decide which transformation to apply.

## Core Rules

- Refactor in small, behavior-preserving steps.
- Keep the code working at each meaningful checkpoint.
- Run fast tests often while refactoring.
- Prefer many safe changes over one large rewrite.
- Rename for clarity early when names are misleading.
- Remove duplication only when it is real duplication, not coincidence.
- Inline indirection that no longer earns its keep.
- Delete dead code, debug scaffolding, and speculative abstractions.

## Standard Workflow

1. Identify the behavior that must not change.
2. Find or add the smallest useful tests around that behavior.
3. Choose the lightest refactoring that addresses the current smell.
4. Make one small structural change.
5. Run the fastest relevant tests.
6. Repeat until the design is materially simpler.
7. Run the broader validation pass.

## Refactoring Index

Use the linked file for the exact mechanics you want. Do not load all references by default.
- [Change Function Declaration](references/refactorings/change-function-declaration.md): Safely evolve a function's name, parameters, or calling contract.
- [Change Reference to Value](references/refactorings/change-reference-to-value.md): Replace shared identity semantics with explicit value semantics.
- [Change Value to Reference](references/refactorings/change-value-to-reference.md): Replace duplicate values with shared canonical references.
- [Collapse Hierarchy](references/refactorings/collapse-hierarchy.md): Remove an unnecessary inheritance layer by folding behavior together.
- [Combine Functions into Class](references/refactorings/combine-functions-into-class.md): Group related functions and shared data into one cohesive class.
- [Combine Functions into Transform](references/refactorings/combine-functions-into-transform.md): Unify related derivations into one explicit transformation step.
- [Consolidate Conditional Expression](references/refactorings/consolidate-conditional-expression.md): Merge several conditions that lead to the same outcome.
- [Decompose Conditional](references/refactorings/decompose-conditional.md): Split a dense conditional into named condition and branch helpers.
- [Encapsulate Collection](references/refactorings/encapsulate-collection.md): Hide direct collection mutation behind intention-revealing operations.
- [Encapsulate Record](references/refactorings/encapsulate-record.md): Wrap open record access in a clearer data abstraction.
- [Encapsulate Variable](references/refactorings/encapsulate-variable.md): Control access to mutable state through a defined boundary.
- [Extract Class](references/refactorings/extract-class.md): Split one overloaded class into smaller cohesive responsibilities.
- [Extract Function](references/refactorings/extract-function.md): Pull a coherent code fragment into a named function.
- [Extract Superclass](references/refactorings/extract-superclass.md): Move shared behavior and state into a common parent.
- [Extract Variable](references/refactorings/extract-variable.md): Give a complex expression a name that explains its purpose.
- [Hide Delegate](references/refactorings/hide-delegate.md): Stop callers from reaching through an object to its internals.
- [Inline Class](references/refactorings/inline-class.md): Remove a class that no longer justifies its existence.
- [Inline Function](references/refactorings/inline-function.md): Remove a function that adds only indirection.
- [Inline Variable](references/refactorings/inline-variable.md): Remove a temporary that does not improve clarity.
- [Introduce Assertion](references/refactorings/introduce-assertion.md): Make an important assumption explicit in the code.
- [Introduce Parameter Object](references/refactorings/introduce-parameter-object.md): Group related parameters into one meaningful object.
- [Introduce Special Case](references/refactorings/introduce-special-case.md): Represent an edge case with a dedicated object or code path.
- [Move Field](references/refactorings/move-field.md): Relocate state to the type that really owns it.
- [Move Function](references/refactorings/move-function.md): Relocate behavior to the module or type it belongs with.
- [Move Statements into Function](references/refactorings/move-statements-into-function.md): Pull repeated setup or policy statements into the callee.
- [Move Statements to Callers](references/refactorings/move-statements-to-callers.md): Push statements outward when only some callers need them.
- [Parameterize Function](references/refactorings/parameterize-function.md): Replace several near-identical functions with one parameterized function.
- [Preserve Whole Object](references/refactorings/preserve-whole-object.md): Pass the source object instead of multiple values extracted from it.
- [Pull Up Constructor Body](references/refactorings/pull-up-constructor-body.md): Move shared constructor steps into a common parent path.
- [Pull Up Field](references/refactorings/pull-up-field.md): Move duplicate subclass state into the superclass.
- [Pull Up Method](references/refactorings/pull-up-method.md): Move duplicate subclass behavior into the superclass.
- [Push Down Field](references/refactorings/push-down-field.md): Move superclass state down to only the subclasses that use it.
- [Push Down Method](references/refactorings/push-down-method.md): Move superclass behavior down to the subclasses that need it.
- [Remove Dead Code](references/refactorings/remove-dead-code.md): Delete code that is no longer reachable or needed.
- [Remove Flag Argument](references/refactorings/remove-flag-argument.md): Replace boolean mode switches with explicit entry points.
- [Remove Middle Man](references/refactorings/remove-middle-man.md): Let callers talk directly to a collaborator when delegation adds no value.
- [Remove Setting Method](references/refactorings/remove-setting-method.md): Prevent uncontrolled mutation by eliminating the setter.
- [Remove Subclass](references/refactorings/remove-subclass.md): Delete a trivial subclass and fold its behavior into a simpler model.
- [Rename Field](references/refactorings/rename-field.md): Rename a field so its purpose is clear.
- [Rename Variable](references/refactorings/rename-variable.md): Rename a variable so its role is obvious.
- [Replace Command with Function](references/refactorings/replace-command-with-function.md): Turn a lightweight command object back into a plain function.
- [Replace Conditional with Polymorphism](references/refactorings/replace-conditional-with-polymorphism.md): Move variant-specific branches into dedicated implementations.
- [Replace Constructor with Factory Function](references/refactorings/replace-constructor-with-factory-function.md): Use a named factory instead of a direct constructor call.
- [Replace Control Flag with Break](references/refactorings/replace-control-flag-with-break.md): Use structured exits instead of sentinel control flags.
- [Replace Derived Variable with Query](references/refactorings/replace-derived-variable-with-query.md): Compute a derived value on demand instead of storing it.
- [Replace Error Code with Exception](references/refactorings/replace-error-code-with-exception.md): Use exceptions for exceptional flow instead of return codes.
- [Replace Exception with Precheck](references/refactorings/replace-exception-with-precheck.md): Use an explicit guard when failure is expected and testable.
- [Replace Function with Command](references/refactorings/replace-function-with-command.md): Turn a complex function into an object with an explicit lifecycle.
- [Replace Inline Code with Function Call](references/refactorings/replace-inline-code-with-function-call.md): Replace repeated inline logic with one named call.
- [Replace Loop with Pipeline](references/refactorings/replace-loop-with-pipeline.md): Rewrite collection-oriented loops as clearer pipeline operations.
- [Replace Magic Literal](references/refactorings/replace-magic-literal.md): Replace an unexplained literal with a named symbol or object.
- [Replace Nested Conditional with Guard Clauses](references/refactorings/replace-nested-conditional-with-guard-clauses.md): Flatten nested branching by handling special cases first.
- [Replace Parameter with Query](references/refactorings/replace-parameter-with-query.md): Let the callee derive data instead of receiving it as a parameter.
- [Replace Primitive with Object](references/refactorings/replace-primitive-with-object.md): Wrap a primitive in a richer domain abstraction.
- [Replace Query with Parameter](references/refactorings/replace-query-with-parameter.md): Pass in a value when deriving it internally is the wrong dependency.
- [Replace Subclass with Delegate](references/refactorings/replace-subclass-with-delegate.md): Use delegation instead of subclassing for specialization.
- [Replace Superclass with Delegate](references/refactorings/replace-superclass-with-delegate.md): Replace inheritance with delegation when reuse is the only reason for the relationship.
- [Replace Temp with Query](references/refactorings/replace-temp-with-query.md): Replace a mutable temp with a side-effect-free query.
- [Replace Type Code with Subclasses](references/refactorings/replace-type-code-with-subclasses.md): Replace explicit type codes with dedicated subtype behavior.
- [Return Modified Value](references/refactorings/return-modified-value.md): Return the updated value explicitly instead of hiding mutation.
- [Separate Query from Modifier](references/refactorings/separate-query-from-modifier.md): Split a method that both returns data and changes state.
- [Slide Statements](references/refactorings/slide-statements.md): Move related statements together to clarify and prepare the code.
- [Split Loop](references/refactorings/split-loop.md): Break one loop with several jobs into focused loops.
- [Split Phase](references/refactorings/split-phase.md): Separate work into distinct sequential stages with clear outputs.
- [Split Variable](references/refactorings/split-variable.md): Stop reusing one variable for multiple meanings.
- [Substitute Algorithm](references/refactorings/substitute-algorithm.md): Swap in a simpler or clearer algorithm while preserving behavior.

## Validation Discipline

- Run fast unit tests often during the refactor.
- Run integration tests before declaring the work done.
- If UI changed, do a manual visual verification pass after automated tests.
- If behavior is uncertain, stop and add characterization coverage before proceeding.
