# Daneel Task Workflow

## Goal

This document defines the standard workflow for implementing a new task in Daneel.

Use this workflow for GitHub issues, milestone tasks, and other scoped feature work.

## Standard Flow

1. Create a feature branch named `task/<tag>`.

Examples:

- `task/T2_7`

2. **Spawn a subagent** to run the **[`project-management` skill](skills/project-management/SKILL.md)** with event **`started`**.

   Pass this context to the subagent:

   ```
   Event:  started
   Task:   <task-tag-or-title>
   Branch: <BRANCH_NAME>
   ```

   The subagent reads `skills/project-management/SKILL.md`, sets the GitHub Project status to `In Progress`, and posts a branch comment on the issue.

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

Open the **[`github-admin` skill](skills/github-admin/SKILL.md)** and follow **Pull requests (Daneel task workflow)** (and [references/commands/create-pr.md](skills/github-admin/references/commands/create-pr.md)) for the exact `create-pr` invocation, base branch, body, issue linking, and client approval-prefix guidance. Prefer that skill over copying ad hoc shell snippets into this document.

11. **Spawn a subagent** to run the **[`project-management` skill](skills/project-management/SKILL.md)** with event **`ready-for-merge`**.

    Pass this context to the subagent:

    ```
    Event:   ready-for-merge
    Task:    <task-tag-or-title>
    PR:      <PR_NUMBER>
    Summary: <one paragraph: what changed, how tested, known limitations>
    ```

    The subagent reads `skills/project-management/SKILL.md`, sets the GitHub Project status to `Ready for Merge`, links the PR to the issue, and posts the summary as a comment on the issue.

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
- include the resulting formatting-pass changes in the branch and pull request; do not drop `cargo fmt` edits that are part of the validated final state

## Branch And Review Discipline

- Keep each branch focused on one task.
- Do not mix unrelated cleanup into the same branch unless it is required for the task.
- Prefer small, reviewable commits even if the branch will later be squashed.
- Keep the task status, branch, and merge request aligned.
- During cleanup, prefer explicit Fowler refactorings over vague “code polish” changes.
- If formatting changes appear while validating the task, keep them in the same PR so the reviewed branch matches the tested code exactly.

## Done Criteria

A task is ready for review when:

- implementation is complete
- unnecessary code has been removed
- refactoring pass is complete
- automated tests pass
- manual visual verification is complete when UI changed
- branch is pushed
- merge request is opened and linked to the task (via `project-management` skill, `ready-for-merge` event)
- GitHub Project status is `Ready for Merge`
