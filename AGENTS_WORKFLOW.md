# Daneel Task Workflow

## Goal

This document defines the standard workflow for implementing a new task in Daneel.

Use this workflow for GitHub issues, milestone tasks, and other scoped feature work.

## Standard Flow

1. Create a feature branch named `task/<tag>`.

Examples:

- `task/T2_7`

2. Set the GitHub task status to `In Progress`.

3. Implement the task on the feature branch.

4. Debug as needed and run relevant unit tests during implementation.

5. When implementation is complete, run the heavier validation passes:

- integration tests
- manual visual verification against the running app

6. Run a refactoring pass to simplify the code.

Follow Martin Fowler style refactoring principles:

- make intent clearer
- reduce duplication
- simplify conditionals
- isolate responsibilities
- keep behavior unchanged while cleaning structure

When the task reaches this refactoring step, use the `refactoring` skill and load the specific refactoring reference files that match the code smells you are addressing.

7. Remove code that is not strictly necessary for the completed feature.

This includes:

- dead code
- temporary debug code
- redundant helpers
- unnecessary indirection introduced during implementation

8. Run the full verification pass again after refactoring and cleanup.

This includes:

- formatting
- unit tests
- integration tests
- manual visual verification

9. Push the feature branch.

10. Create a merge request or pull request with the title format:

```text
[<Task tag>] Title
```

Example:

```text
[T2.7] Make agent recency and heartbeat state update live
```

11. Link the task to the merge request.

12. Set the GitHub task status to `Ready for Merge`.

13. Add relevant implementation notes to the merge request description.

Include:

- what changed
- how it was tested
- any follow-up work or known limitations
- screenshots when the UI changed materially

14. Submit the merge request and provide the link.

## Testing Expectations

During implementation:

- run fast unit tests often
- validate behavior incrementally instead of waiting until the end

Before opening the merge request:

- run formatting
- remove warnings
- run integration tests
- perform manual visual verification using the repo verification workflow

## Branch And Review Discipline

- Keep each branch focused on one task.
- Do not mix unrelated cleanup into the same branch unless it is required for the task.
- Prefer small, reviewable commits even if the branch will later be squashed.
- Keep the task status, branch, and merge request aligned.
- During cleanup, prefer explicit Fowler refactorings over vague “code polish” changes.

## Done Criteria

A task is ready for review when:

- implementation is complete
- unnecessary code has been removed
- refactoring pass is complete
- automated tests pass
- manual visual verification is complete when UI changed
- branch is pushed
- merge request is opened and linked to the task
- GitHub task status is `Ready for Merge`
