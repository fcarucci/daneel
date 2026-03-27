---
name: executing-plans
description: Use when you have a written implementation plan to execute in a separate session with review checkpoints
---

# Executing Plans

## Overview

Load plan, review critically, execute all tasks, report when complete.

**Announce at start:** "I'm using the executing-plans skill to implement this plan."

**Note:** Tell your human partner that this workflow works much better with access to subagents. If subagents are available, use `subagent-driven-development` instead of this skill.

## The Process

### Step 1: Load and Review Plan
1. Read plan file
2. Review critically - identify any questions or concerns about the plan
3. If concerns: Raise them with your human partner before starting
4. If no concerns: Create a task tracker and proceed

### Step 2: Signal work start (project management)

Before touching any code, check whether `skills/project-management/SKILL.md` exists.
If it does, gather any available context: issue number, branch name, PR number.

Spawn a subagent with the available context and instruct it to read and follow `skills/project-management/SKILL.md` with event **`started`**:

```
Event:  started
Issue:  <ISSUE_NUMBER_if_known>
Branch: <BRANCH_NAME_if_known>
```

The subagent sets the project status to `In Progress` and posts a branch comment.
Skip any field that is not yet known — the skill handles missing context gracefully.
If the skill file is absent, skip this step entirely.

### Step 3: Execute Tasks

For each task:
1. Mark as in_progress
2. Follow each step exactly (plan has bite-sized steps)
3. Run verifications as specified
4. **End-of-implementation refactoring (mandatory):** run the `refactoring` skill on all files you changed for this task (features **and** bug fixes), then rerun the plan's fast checks. Do this **proactively**—do not wait for the user to say "refactor." Ad-hoc tidying while coding does not count.
5. Mark as completed

### Step 4: Complete Development

After all tasks complete and verified:
- Announce: "I'm using the finishing-a-development-branch skill to complete this work."
- **REQUIRED SUB-SKILL:** Use `finishing-a-development-branch`
- Follow that skill to verify tests, present options, execute choice

### Step 5: Signal work complete (project management)

After the PR is opened, check whether `skills/project-management/SKILL.md` exists.
If it does, gather any available context: issue number, PR number, branch name.

Compose a brief implementation summary covering what changed, how it was tested, and any known limitations or follow-up items.

Spawn a subagent with the available context and instruct it to read and follow `skills/project-management/SKILL.md` with event **`ready-for-merge`**:

```
Event:   ready-for-merge
Issue:   <ISSUE_NUMBER_if_known>
PR:      <PR_NUMBER_if_known>
Summary: <IMPLEMENTATION_SUMMARY>
```

The subagent sets the project status to `Ready for Merge`, links the PR to the issue, and posts the summary as a comment.
Skip any field that is not yet known — the skill handles missing context gracefully.
If the skill file is absent, skip this step entirely.

## When to Stop and Ask for Help

**STOP executing immediately when:**
- Hit a blocker (missing dependency, test fails, instruction unclear)
- Plan has critical gaps preventing starting
- You don't understand an instruction
- Verification fails repeatedly

**Ask for clarification rather than guessing.**

## When to Revisit Earlier Steps

**Return to Review (Step 1) when:**
- Partner updates the plan based on your feedback
- Fundamental approach needs rethinking

**Don't force through blockers** - stop and ask.

## Remember
- Review plan critically first
- Follow plan steps exactly
- Don't skip verifications
- Reference skills when plan says to
- Stop when blocked, don't guess
- Never start implementation on main/master branch without explicit user consent

## Integration

**Required workflow skills and practices:**
- **`using-git-worktrees`** - REQUIRED: Set up an isolated workspace before starting
- **`writing-plans`** - Creates the plan this skill executes
- **`finishing-a-development-branch`** - Complete development after all tasks
- **`project-management`** - Optional: invoked at start (Step 2) and completion (Step 5) when an issue number is known
