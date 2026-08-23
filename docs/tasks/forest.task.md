---
title: forest — scenario runner plan
type: task
key: forest
branch: vfp/agent/plan/forest
pr: https://github.com/victoryforphil/AutonomyAge/pull/33
desc: Port project-firefly's file-driven scenario runner into a reusable vs-* crate (plan).
status: active
update: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

project-firefly's scenario runner is the only implementation in the org. Scenarios
are file-driven (`config.yaml` commander commands trigger→action, `show.yaml` timed
show) with pluggable `ForestMiddleware` handlers.

## Todos

- [x] Survey org for scenario runners (only project-firefly)
- [x] Draft `docs/plan/forest.plan.md`
- [ ] Migrate off `wingman-*` onto `vs-broker`/`vs-data-store`/`vs-wtf`

## State

- Plan drafted; PR #33 open.

## Risks

- Highest coupling of the vs-* ideas; API drift, not a rename. `vs-broker` `tick` is
  sync vs wingman `broker.run().await`. Heavy tokio usage.

## Human help

- Decide whether keep flight-specific tasks in a feature or `examples/`.
- Confirm the generic step/validator registry design is desired.

## Followups

- Implement `libs/vs-forest` after plan review; depends on `vs-dir` and `vs-valley`.

## Links

- Plan: `docs/plan/forest.plan.md`
- Idea: `docs/ideas/forest.idea.md`
- Source repo: `project-firefly/src_tools/forest/`

## Open questions

- Keep `ForestGeneratedPort` / `DockerComposeConfig` or make optional?

## Advice / lessons

- forest/valley READMEs document an API that doesn't exist; trust the code.
