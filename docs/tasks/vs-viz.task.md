---
title: vs-viz — visualization (Rerun) utilities plan
type: task
key: vs-viz
branch: vfp/agent/plan/vs-viz
pr: https://github.com/victoryforphil/AutonomyAge/pull/38
desc: Port a backend-agnostic visualization utilities crate (Rerun first backend) — research deepened.
status: active
update: Deepened research — resolved rerun pin, vs-* API, host model, topic prefix; concrete API drafted.
last_updated: 2026-08-23
---

## Context

Best design base is `lil-hopps/lil-rerun` (backend-agnostic `LilRerun`/`RerunMode`,
`DataView`→`Primitives` mapping, vs-* integrated), modernized onto current `vs-*` APIs.
Grafts: `RerunOptions` multi-sink (loki), newest clean `log_*` helpers (SkyCanvas 0.28.2),
pose decomposition (basher).

Research pass verified every `vs-*` call against real source and produced a concrete,
buildable `libs/vs-viz` API (see `vs-viz.plan.updated.md`).

## Todos

- [x] Compare Rerun wrappers across org (done: 0.18.2 / 0.19 / 0.21 / 0.23 / 0.28.2 + 0.36 latest)
- [x] Draft `docs/plan/vs-viz.plan.md`
- [x] Confirm `vs-*` API names against real libs
- [x] Resolve Rerun pin (0.28.2), host model (both), topic/namespace prefix
- [x] Draft concrete `libs/vs-viz` API
- [ ] Implement `libs/vs-viz` (core + rerun feature)
- [ ] Build-verify rerun 0.28.2 archetype calls (Boxes3D/Asset3D/flush_blocking) in-tree

## State

- Plan (+ updated plan) drafted; PR #38 open.
- `vs-viz.plan.updated.md` has the resolved decisions and full API.
- Open questions from the original plan are now DECIDED (see Risks).

## Decisions (new)

- **Rerun pin: `rerun = "0.28"` → 0.28.2**, optional feature `rerun = ["dep:rerun","dep:nalgebra"]`.
  Latest on crates.io is 0.36.2; 0.28.2 is pinned because it is the only version whose API is
  confirmed by an in-org wrapper (SkyCanvas `log_rerun.rs`). Archetypes used are stable 0.28→0.36.
- **`log_scalar` uses `rerun::Scalars::single(f64)`** (legacy `Scalar` removed/renamed).
- **`VizSink` is object-safe** (`Box<dyn VizSink>`): path params are `&VizPath`, not `impl Into<VizPath>`.
- **Host model = BOTH**: `VizSystem: BrokerTask` (app integration) AND `VizSession`+`RerunSink`/`NullSink`
  for a plain loop.
- **Topic prefix**: `RerunOptions::namespace: Option<String>` (default `None`). Recommend
  `namespace = Some(app_id)` for multi-app viewers.
- **Payload `Vec3`** is own `Vec3([f64;3])`; `VizPose.attitude` is `[f64;4]` quaternion `[x,y,z,w]`.

## Risks

- **Rerun build weight**: heavy deps (arrow/tokio/`re_*`/cmake/nasm). Keep optional; CI/headless
  builds use `--no-default-features`. Installed `rustc 1.95.0` is well above rerun MSRV.
- **Version drift**: pin 0.28.2; bumping to 0.36.x is a separate task.
- **Build-only API verification**: `Boxes3D::with_quaternions/from_sizes`, `Asset3D::from_file`,
  `flush_blocking`, `ViewCoordinates` come from 0.18/0.19 sources — must compile-check against 0.28.2.
- **Original plan bug fixed**: `get_latest_primitive` is on `Datastore`, not `DataView`; `log_dataview`
  uses `view.get_latest_map(&TopicKey::empty())`.
- **`TopicKey` gotcha confirmed**: `key.to_string()` returns numeric ids; use `display_name()`
  (always valid Rerun path). `format!("{}", key)` also works (Display uses `display_name`).

## Human help

- Confirm the rerun 0.28.2 pin is acceptable (vs jumping to latest 0.36.x).
- Confirm that `Scalars::single` + the scene/pose archetypes are the desired surface (build will
  lock exact signatures).
- Confirm `namespace = Some(app_id)` default is desired for the first app, or keep `None`.

## Followups

- Implement `libs/vs-viz`; wire `VizSystem: BrokerTask` into a sim.
- Build-verify rerun 0.28.2 calls; then optionally file a follow-up to bump to 0.3x.

## Links

- Plan (updated): `/tmp/opencode/vfp-research/cont/vs-viz.plan.updated.md`
- Plan (repo): `docs/plan/vs-viz.plan.md`
- Idea: `docs/ideas/vs-viz.idea.md`
- Sources: `lil-hopps/lil-rerun/`, `basher/src/basher_rerun/`,
  `AndreasLabs/SkyCanvas/.../log_rerun.rs`, `AndreasLabs/loki/.../rerun_bridge.rs`,
  `project-firefly/src_tools/lit-rerun/`

## Open questions

- Whether the `rerun` feature should be `default = ["rerun"]` (convenience) or `default = []`
  (fast headless/CI). Plan currently says default-on, flagged as a tradeoff.
- Whether per-datapoint `data-time` or single `broker-time` axis should be the default in
  `log_dataview` (both supported).

## Advice / lessons

- Keep `VizSession`/`VizSink` free of `rerun` imports; `rerun` lives only in `rerun/`.
- `VizSink` must stay object-safe. Use `&VizPath` params. Keep nalgebra out of the stable trait.
- Do not port `rerun::Logger` (removed in newer Rerun).
