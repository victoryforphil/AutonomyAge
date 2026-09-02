---
title: review-bot-pi — Greptile-style review bot (pi track)
type: task
key: review-bot-pi
branch: vfp/agent/feature/review-bot-pi
pr: https://github.com/victoryforphil/AutonomyAge/pull/45
desc: Build a Greptile-like PR review GitHub Action powered by pi + OpenRouter + DeepSeek V4 Flash.
status: active
update: Pi hardening complete — required PR checks pass; ready to merge.
last_updated: 2026-09-02
---

## Context
Mimic Greptile's PR review behaviour in this repo using pi as the sole agent harness.
Reviewer configuration lives in `.agents/reviewer/` and rules in `REVIEW.md`.

## Todos
- [x] Add `.agents/reviewer/SKILL.md` (reviewer agent: identity, rules, output schema)
- [x] Add root `REVIEW.md` (repo review rules / style checks)
- [x] Add `.github/scripts/review_bot.py` (pi orchestration)
- [x] Add `.github/workflows/review-pi.yaml` (trusted-base Pi wiring)
- [x] Harden trusted execution, tool access, and review schema validation
- [x] Remove unsupported inline-thread resolution mutations
- [x] Required PR checks pass; ready to merge

## State
- Pi runs headlessly with `--mode json`, `--thinking off`, `--no-session`, and an
  enforced read-only tool allowlist.
- The review and fix workflows run trusted base-revision scripts/configuration and
  fetch the PR head only to generate a diff.
- Parser tests cover valid JSON plus required-schema and malformed-finding rejection.
- Required GitHub checks pass: crates, coverage, and all three build-test jobs.

## Risks
- Review quality remains model-dependent; tune rules from observed findings after merge.

## Human help
- No human input required.

## Followups
- Monitor review usefulness after merge; tune `REVIEW.md` from observed findings.
- Add thread resolution only with a tested GitHub credential model.

## Links
- Reviewer config: `.agents/reviewer/README.md`
- Rules: `REVIEW.md`
- PR: https://github.com/victoryforphil/AutonomyAge/pull/45

## Open questions
- None.

## Advice / lessons
- Do not execute PR-controlled code in a workflow that has secrets or write tokens.
- Schema validation must reject incomplete JSON rather than render it as an approval.
- Pass prompts by file and enforce Pi's tool allowlist; prompts are not a security boundary.
