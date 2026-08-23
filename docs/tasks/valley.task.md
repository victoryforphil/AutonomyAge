---
title: valley — validation framework plan
type: task
key: valley
branch: vfp/agent/plan/valley
pr: https://github.com/victoryforphil/AutonomyAge/pull/34
desc: Port project-firefly's validation framework into a reusable vs-* crate (plan).
status: active
update: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

project-firefly's `valley` is the only validation framework in the org. Model:
`Validator` trait + `ValidatorResult` (Passing/ExitSuccess/ExitFailure/Failing) +
`ValidationResultInfo` (reason/data/failed_at/label) + `SILValidator` runner + 5
built-in validators.

## Todos

- [x] Survey org for validation frameworks (only project-firefly)
- [x] Draft `docs/plan/valley.plan.md`
- [ ] Resolve the `SkyPose` dependency (no transforms module in `vs-wtf`)

## State

- Plan drafted; PR #34 open.

## Risks

- **`SkyPose` blocker**: `pose_comparsion.rs` needs a pose type that `vs-wtf` lacks.
- Time type/serde drift: wingman `Timecode{seconds,microseconds}` + custom string serde
  vs `vs-wtf` `Timecode{secs,nanos}` derived serde.

## Human help

- Decide the `SkyPose` fix: add a transforms module to `vs-wtf`, define a local pose
  type, or drop the pose validator.

## Followups

- Implement `libs/vs-valley`; wire into the `forest` runner's validation middleware.

## Links

- Plan: `docs/plan/valley.plan.md`
- Idea: `docs/ideas/valley.idea.md`
- Source repo: `project-firefly/src_core/valley/`

## Open questions

- Use `std::sync::Mutex` (recommended) vs `tokio::sync::Mutex` for the validators vec.

## Advice / lessons

- Trust the code over the README (`ShowValidator` API is aspirational).
