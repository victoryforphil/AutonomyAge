---
name: task-tracking
description: Conventions for live per-task tracking in docs/tasks/{key}.task.md — one file per task (usually one per branch/PR), with frontmatter (key, branch, pr, desc, status, update, last_updated) and sectioned context/todos/state/risks/human-help/followups/links/open-questions/advice. Load when creating or updating a task note, or to check in on how a task/branch/PR is going.
---

# Task Tracking

Per-task live notes for checking in on work. A **task** is a unit of work, usually
tied closely to a branch or PR. One file per task in `docs/tasks/{key}.task.md`.

These are read often — by people working the task and by agents (via PRs and other
interfaces) to get the current state quickly. Keep them current; a stale task note is
worse than none.

## File & naming

- Path: `docs/tasks/{key}.task.md`
- `{key}` is the same key used to name the task's branch (the branch's identifying
  slug). Keep it short and unique.
- `type: task` in frontmatter, matching the `.task.md` suffix (docs skill pattern).

## Frontmatter

Every task note starts with Obsidian-style YAML frontmatter:

```yaml
---
title: Human-readable Name
type: task
key: <branch key>          # same key used in the branch
branch: vfp/agent/...      # full branch name
pr: https://github.com/... # PR link / number
desc: one-line what this task does
status: active             # lifecycle: todo | active | done | deprecated
update: <short progress>   # e.g. "PR open — plan drafted", "blocked on X"
last_updated: YYYY-MM-DD
---
```

`key`, `branch`, `pr`, `desc`, `status`, `update`, `last_updated` are required.

- `status` uses the **same lifecycle enum** as the docs skill (`todo | active |
  done | deprecated`) so task metadata filters consistently with every other doc.
  Set `active` while in progress, `done` when complete, `deprecated` if abandoned.
- `update` is the one-line human-readable progress note (PR state, blockers).
- Keep `desc` and `update` to one line. Update `last_updated` on every meaningful change.

## Body sections

Use these headings, concise bullet-point selections. Include only what's relevant;
stay scannable.

- `## Context` — why this task exists, what it depends on, the big-picture one-liner.
- `## Todos` — `- [ ]` / `- [x]` checklist. Keep short; link to a plan doc for detail.
- `## State` — current concrete state: done / in progress / next.
- `## Risks` — open technical risks / blockers, one bullet each.
- `## Human help` — where a human decision/input is needed (e.g. a naming choice,
  a dependency version).
- `## Followups` — follow-on work this task enables or that should come after it.
- `## Links` — handoff notes, plan/idea/design docs, related PRs, source repos.
- `## Open questions` — unresolved questions with any trailing constraints.
- `## Advice / lessons` — gotchas, things learned, traps avoided, one-liners.

Rules:

- One line per bullet; prefer bullets over paragraphs.
- Update `status`, `update`, `last_updated`, `State`, `Todos` whenever something changes.
- Do not duplicate deep specs — link to the plan/design doc.
- If a task is blocked or needs a human, say so explicitly in `status`, `update`,
  `State`, and `Human help`.

## Reading / checking in

- `docs/tasks/{key}.task.md` holds the live state; the matching `docs/plan/` and
  `docs/agents/` docs hold the research/spec detail.
- When reviewing a PR, read its task note first for context, state, and risks.

## Surfacing on the PR

Make the task note easy to read from the PR itself so it's visible without opening
the file:

- **Embed it in the PR description.** Append the task note's body (or a condensed
  version) under a `## Task` heading in the PR description. If the PR already has a
  description, append the task section rather than replacing it.
- Or **post it as a PR comment**, and keep that comment updated as the task progresses.
- Either way, **keep it in sync**: whenever the task note's `status`/`update`/`State`
  changes, refresh the PR description/comment. A stale embed is worse than none.
- For a stacked `gh stack`, each PR carries its own task note; embed each PR's own task.

## Example

```markdown
---
title: forest — scenario runner plan
type: task
key: forest
branch: vfp/agent/plan/forest
pr: https://github.com/victoryforphil/AutonomyAge/pull/33
desc: Port project-firefly scenario runner into a reusable vs-* crate (plan).
status: active
update: PR open — plan drafted
last_updated: 2026-08-22
---

## Context
One-line why this exists.

## Todos
- [x] Survey org for existing scenario runners
- [ ] Draft migration off wingman-* onto vs-*

## State
- Plan drafted; PR open.

## Risks
- High coupling to wingman-*; API drift.

## Human help
- Confirm the vs-* crate name and whether to genericize the task registry.

## Followups
- Implement `libs/vs-forest` after plan review.

## Links
- Plan: `../plan/forest.plan.md`
- Source index: `../agents/repo-index.md`

## Open questions
- Keep flight-specific tasks in a feature or examples?

## Advice / lessons
- The README documents an API that doesn't exist; trust the code.
```
