---
title: valley — validation framework port
type: task
key: valley
branch: vfp/agent/plan/valley
pr: https://github.com/victoryforphil/AutonomyAge/pull/34
desc: Port project-firefly's validation framework into a reusable vs-* crate (plan).
status: active
update: SkyPose blocker resolved — port to vs-wtf::transforms; exact vs-* API mapping + std Mutex decided
last_updated: 2026-08-23
---

## Context

project-firefly's `valley` is the only validation framework in the org. Model:
`Validator` trait + `ValidatorResult` (`Passing`/`ExitSuccess`/`ExitFailure`/`Failing`) +
`ValidationResultInfo` (reason/data/failed_at/label) + `SILValidator` runner + 5 built-in
validators. Port to a reusable `libs/vs-valley` crate de-coupled from `wingman-*`, targeting
`vs-data-store`, `vs-broker`, `vs-wtf`. **The code is the spec** (README's `ShowValidator`
API is aspirational).

## Todos

- [x] Survey org for validation frameworks (only project-firefly)
- [x] Draft `docs/plan/valley.plan.md`
- [x] Resolve the `SkyPose` dependency — port to `vs-wtf::transforms` (see Risks/State)
- [ ] Add `transforms` module to `vs-wtf` (SkyPose + SkyVec3/SkyVec2/SkyVec4 + Quaternion + SkyFrame)
- [ ] Create `libs/vs-valley` + register in workspace
- [ ] Port `result.rs` / `validator/mod.rs` / five validators / `SILValidator` with exact API mapping
- [ ] Wire valley into the `forest` runner's validation middleware

## State

- Plan drafted; PR #34 open. Full updated plan at `docs/plan/valley.plan.md` (branch) and `/tmp/opencode/vfp-research/cont/valley.plan.updated.md`.
- **SkyPose decision made:** port `SkyPose` (+ `SkyVec3`, `SkyVec2`, `SkyVec4`, `Quaternion`, `SkyFrame`, `CoordinateSystem`) into a new `vs-wtf::transforms` module. Keeps `pose_comparsion.rs` line-for-line and is reusable by the other project-firefly ports (forest samples, whisp, showkit, commander, whisper). Leaner fallback if vs-wtf must stay minimal: `SkyPose { position: SkyVec3 }` only (un-declared fields ignored by deserializer). **Not** a local vs-valley pose type.
- **Mutex decision made:** `std::sync::Mutex`; `tick` becomes a **sync** fn; drop `tokio`.
- **std::time::Instant** replaces `tokio::time::Instant` in `validate_frequency`.
- **vs-wtf `Timecode` has no `Display`** — `sil_validator` must render `failed_at` via `t.secs()`, not `t.to_string()`.

## Risks

- **Time serde / output drift (open):** wingman `Timecode{seconds,microseconds}` + custom string serde vs `vs_wtf::Timecode{secs,nanos}` derived serde. Config YAML + `.csv/.html/.log` output change; constructors renamed (`new_secs_f64` → `new_secs`).
- `TopicKey::from_str` is infallible (no `unwrap` / `?`); `TopicPath` was fallible — minor churn in `pose_comparsion` / `topic_comparsion`.
- Latent index-align bug in `SILValidator::get_results`/`print_results` (pairs `validators[i]` with `current_results[i]`); preserved, safe only because one result per validator per tick.
- `SkyFrame`/`CoordinateSystem` datastore serde untested (not exercised by the pose validator).

## Human help

- Decide whether `vs-wtf` should carry the full `SkyPose { position, rotation, frame }` (recommended, fidelity + reuse) or the leaner `SkyPose { position }`. Flag if a future port needs full pose serde through the datastore.

## Followups

- Implement `libs/vs-valley`; wire into the `forest` runner's validation middleware (sync `tick`).
- Add `"libs/vs-valley"` to root `[workspace].members`.

## Links

- Plan: `docs/plan/valley.plan.md` (+ `/tmp/opencode/vfp-research/cont/valley.plan.updated.md`)
- Idea: `docs/ideas/valley.idea.md`
- Source repo: `project-firefly/src_core/valley/`
- Target APIs: `libs/vs-data-store/src/database/view.rs`, `libs/vs-broker/src/broker/time.rs`, `libs/vs-wtf/src/*`

## Open questions

- Should the pose validator also assert orientation (quaternion) or frame, or stay position-only as in source? (Source is position-only; recommend keeping position-only to match behavior.)
- Should vs-valley expose reusable `SkyPose` builders for config YAML (`new_from_config`), or rely on serde directly?

## Advice / lessons

- Trust the code over the README (`ShowValidator` API is aspirational).
- Port the type names and .position.x/y/z accessor path exactly so `pose_comparsion` is a near-mechanical change.
- Keep the validation framework sync (std Mutex + std Instant) — it only ever touched async for the old `tokio::sync::Mutex` handle.
