---
title: repo-index — cross-repo survey index
type: task
key: repo-index
branch: vfp/agent/docs/repo-index
pr: https://github.com/victoryforphil/AutonomyAge/pull/32
desc: Index victoryforphil/AndreasLabs repos and document the docs/agents/ convention.
status: PR open
last_updated: 2026-08-22
---

## Context

Backing research for the `vs-*` plan PRs. Surveys the org and records what each repo
has that's relevant to porting reusable crates, plus the survey method.

## Todos

- [x] Survey / clone repos and record per-repo entries
- [x] Add `docs/agents/repo-index.md`
- [x] Add `docs/agents/` note to the docs skill
- [x] Add the swarm plan-track handoff board (`.agents/handoffs/plan-tracks.md`)

## State

- PR #32 open. Includes the repo index + docs skill note + handoff board.

## Risks

- Some relevant private repos (dark-factory, tinyverse, cursed-tanks) not cloneable
  here; read via GitHub raw and marked.

## Human help

- None required; review the index for accuracy.

## Followups

- Add new repos to the index as the org grows.

## Links

- Index: `docs/agents/repo-index.md`
- Docs skill note: `.agents/skills/docs/SKILL.md`
- Handoff board: `.agents/handoffs/plan-tracks.md`

## Open questions

- Where to draw the "within relative reason" line for how many repos to survey.

## Advice / lessons

- `gh search code` reaches private repos; local grep + git trees are the source of truth.
