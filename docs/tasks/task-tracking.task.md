---
title: task-tracking — task note skill
type: task
key: task-tracking
branch: vfp/agent/docs/task-tracking
pr: https://github.com/victoryforphil/AutonomyAge/pull/41
desc: Add the task-tracking skill (docs/tasks/{key}.task.md) and link it from the docs skill.
status: PR open — skill + self task note added
last_updated: 2026-08-22
---

## Context

The docs/tasks convention + skill so a task (usually one per branch/PR) has a single
live note for checking in on how it's going, readable by humans and agents.

## Todos

- [x] Create `.agents/skills/task-tracking/SKILL.md`
- [x] Add the `docs/tasks/` note + task-skill link to the docs skill
- [x] Add a self task note (`docs/tasks/task-tracking.task.md`)
- [ ] Each other task's `.task.md` lives in that task's own PR (not here)

## State

- PR #41 open (stacked on #32). This PR carries only the skill + docs link + its own
  task note.

## Risks

- Task notes go stale if not updated; the skill says to update `status`/`last_updated` often.

## Human help

- Confirm the frontmatter fields (key/branch/pr/desc/status/last_updated) are what you
  want to check in on via PRs.

## Followups

- New tasks should add their own `docs/tasks/{key}.task.md` in their own PR.

## Links

- Skill: `.agents/skills/task-tracking/SKILL.md`
- Docs skill: `.agents/skills/docs/SKILL.md`
- Stacked on: `vfp/agent/docs/repo-index` (PR #32)

## Open questions

- Standardize on `key` = idea slug (e.g. `forest`) vs the full branch name.

## Advice / lessons

- Keep bullets to one line; link to plan docs for detail.
