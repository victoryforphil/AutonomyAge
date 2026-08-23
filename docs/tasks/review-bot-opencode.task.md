---
title: review-bot-opencode — Greptile-style review bot (opencode track)
type: task
key: review-bot-opencode
branch: vfp/agent/feature/review-bot-opencode
pr: https://github.com/victoryforphil/AutonomyAge/pull/44
desc: Build a Greptile-like PR review GitHub Action powered by opencode (v2 beta) + OpenRouter + DeepSeek V4 Flash.
status: active
update: Scaffold done — reviewer config, REVIEW.md rules, orchestrator, opencode workflow; ready to iterate on CI.
last_updated: 2026-08-22
---

## Context
Mimic Greptile's PR review behaviour in this repo using opencode as the agent harness.
Parallel track: `review-bot-pi` (pi harness). Shared review config lives in
`.agents/reviewer/` + root `REVIEW.md`; only the harness wiring differs.

## Todos
- [x] Add `.agents/reviewer/SKILL.md` (reviewer agent: identity, rules, output schema)
- [x] Add root `REVIEW.md` (repo review rules / style checks)
- [x] Add `.github/scripts/review_bot.py` (harness-agnostic orchestrator)
- [x] Add `.github/workflows/review-opencode.yaml` (opencode v2 beta wiring)
- [x] Add `.opencode/agents/reviewer.md` (opencode agent registration)
- [ ] Push branch + open PR; iterate on the review comment/CI until green
- [ ] Match more Greptile features: inline comments, threads, resolve/unresolve, fix prompts, diagrams, PR-desc suggestions

## State
- Scaffold committed; not yet pushed/iterated. Validated opencode v2 beta runs headless
  against OpenRouter (`opencode run --format json` + fresh `XDG_DATA_HOME`).
- opencode stable (1.17.x) has a local SQLite datastore bug; v2 beta is clean.

## Risks
- opencode v2 beta is beta — CLI/config may shift under us.
- Agent output is JSON; a non-JSON reply fails the run (falls back to leaving the old comment).
- Long diffs inflate token cost even at default context.

## Human help
- Confirm the default model (`deepseek/deepseek-v4-flash`) and OpenRouter key are right for opencode.

## Followups
- Compare against `review-bot-pi`; reconcile shared `.agents/reviewer/` + `REVIEW.md`.
- Move any repeated findings into `REVIEW.md` so rules accumulate.

## Links
- Reviewer config: `.agents/reviewer/README.md`
- Rules: `REVIEW.md`
- Parallel track: `docs/tasks/review-bot-pi.task.md`

## Open questions
- Should the opencode track register the reviewer as an opencode `--agent`, or keep the prompt-injected path?

## Advice / lessons
- opencode needs a clean `XDG_DATA_HOME`; a corrupted global DB causes "Unexpected server error".
- DeepSeek V4 Flash emits long thinking blocks; set `--thinking off` on the pi track to keep output parseable/cheap.
