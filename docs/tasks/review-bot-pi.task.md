---
title: review-bot-pi — Greptile-style review bot (pi track)
type: task
key: review-bot-pi
branch: vfp/agent/feature/review-bot-pi
pr: https://github.com/victoryforphil/AutonomyAge/pull/45
desc: Build a Greptile-like PR review GitHub Action powered by pi + OpenRouter + DeepSeek V4 Flash.
status: active
update: Scaffold done — reviewer config, REVIEW.md rules, orchestrator, pi workflow; ready to iterate on CI.
last_updated: 2026-08-22
---

## Context
Mimic Greptile's PR review behaviour in this repo using the `pi` coding agent as the
harness. Parallel track: `review-bot-opencode` (opencode v2 beta). Shared review config
lives in `.agents/reviewer/` + root `REVIEW.md`; only the harness wiring differs.

## Todos
- [x] Add `.agents/reviewer/SKILL.md` (reviewer agent: identity, rules, output schema)
- [x] Add root `REVIEW.md` (repo review rules / style checks)
- [x] Add `.github/scripts/review_bot.py` (harness-agnostic orchestrator)
- [x] Add `.github/workflows/review-pi.yaml` (pi wiring)
- [ ] Push branch + open PR; iterate on the review comment/CI until green
- [ ] Match more Greptile features: inline comments, threads, resolve/unresolve, fix prompts, diagrams, PR-desc suggestions

## State
- Scaffold committed; not yet pushed/iterated. Validated pi runs headless against
  OpenRouter (`pi -p --mode json --thinking off --no-session`).
- pi emits JSONL events; the orchestrator parses the last `agent_end` message.

## Risks
- pi `--mode json` can interleave ANSI to stderr (not stdout); captured separately.
- Agent output is JSON; a non-JSON reply fails the run (falls back to leaving the old comment).

## Human help
- Confirm the default model (`deepseek/deepseek-v4-flash`) and OpenRouter key are right for pi.

## Followups
- Compare against `review-bot-opencode`; reconcile shared `.agents/reviewer/` + `REVIEW.md`.
- Move any repeated findings into `REVIEW.md` so rules accumulate.

## Links
- Reviewer config: `.agents/reviewer/README.md`
- Rules: `REVIEW.md`
- Parallel track: `docs/tasks/review-bot-opencode.task.md`

## Open questions
- Should the pi track load the reviewer via `--skill` in addition to the injected prompt?

## Advice / lessons
- Use `--thinking off` on pi; DeepSeek V4 Flash emits large thinking blocks that bloat output/cost.
- Run pi with `@<prompt-file>` so the prompt is included without argv limits.
