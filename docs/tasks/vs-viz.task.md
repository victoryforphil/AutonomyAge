---
title: vs-viz — visualization (Rerun) utilities plan
type: task
key: vs-viz
branch: vfp/agent/plan/vs-viz
pr: https://github.com/victoryforphil/AutonomyAge/pull/38
desc: Port a backend-agnostic visualization utilities crate starting with Rerun (plan).
status: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

Best design is `lil-hopps/lil-rerun` (backend-agnostic `LilRerun`/`RerunMode`/
`DataView`→`Primitives` mapping, vs-* integrated) modernized onto current `vs-*` APIs.
Grafts: `RerunOptions` multi-sink (loki), newest clean `log_*` helpers (SkyCanvas 0.28.2),
pose decomposition (basher).

## Todos

- [x] Compare Rerun wrappers across org
- [x] Draft `docs/plan/vs-viz.plan.md`
- [ ] Implement `libs/vs-viz` (core + rerun feature)

## State

- Plan drafted; PR #38 open.

## Risks

- **Rerun version drift** (0.17→0.28.2) is the top risk; pin ~0.28 behind a feature.
- `lil-rerun` written against old `victory-*` API — must re-map to current `vs-*`.
- `rerun::Logger` removed in newer Rerun.

## Human help

- Confirm the rerun version pin and that the `vs-viz` `TopicKey`→path mapping
  (`display_name`, not `to_string`) is acceptable.

## Followups

- Implement `libs/vs-viz`; wire `VizSystem: BrokerTask` into a sim.

## Links

- Plan: `docs/plan/vs-viz.plan.md`
- Idea: `docs/ideas/vs-viz.idea.md`
- Sources: `lil-hopps/lil-rerun/`, `AndreasLabs/SkyCanvas/.../log_rerun.rs`, `loki`

## Open questions

- Keep `VizSystem` as a `BrokerTask` vs plain loop; whether Save+Live coexist by default.

## Advice / lessons

- Keep `VizSession`/`VizSink` free of `rerun` imports; the `rerun` crate stays in the backend.
