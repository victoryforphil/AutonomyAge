---
title: quadlink — MAVLink framework plan
type: task
key: quadlink
branch: vfp/agent/plan/quadlink
pr: https://github.com/victoryforphil/AutonomyAge/pull/35
status: active
update: Research deepened — concrete API + decisions resolved, plan/task rewritten
last_updated: 2026-08-23
---

## Context

Best base is `lil-hopps/lil-link` (literally "quadlink", pure-serde `Quad*` types,
already wired to `victory-*`/`vs-*`). Graft graceful-shutdown lifecycle from
`project-devore` and extra builders/processors from `project-firefly/whisper`.
Read direct from source: `libs/vs-*` APIs confirmed; whisper `cmd_velocity.rs` is a
stub (no real velocity builder — must be authored).

## Todos

- [x] Compare lil-link / whisper / project-devore sources
- [x] Draft `docs/plan/quadlink.plan.md`
- [x] Resolve open questions (crate name, mavlink version, heartbeat home, topic ns)
- [x] Confirm vs-* APIs (`BrokerTask`, `DataView`, `BrokerTaskConfig/Subscription/Trigger`)
- [x] Write updated concrete plan + task (research continuation)
- [ ] Port lil-link + graft devore lifecycle + whisper builders/processors
- [ ] Author `build_cmd_velocity` + `QuadVelocity` (whisper is a stub)
- [ ] Wire `QuadlinkSystem` as a `vs_broker` task + SITL/mock smoke test
- [ ] Write `docs/designs/quadlink.md`

## State

- Plan + task rewritten with concrete API + decisions in `/tmp/opencode/vfp-research/cont/`.
- PR #35 open; implementation not started in-repo.

## Risks

- `mavlink` 0.13.1 baseline matches all port sources; 0.14.1 drift deferred.
- `cmd_velocity.rs` is `// ... existing code ...` — author a velocity-only
  `SET_POSITION_TARGET_LOCAL_NED` builder (verify `PositionTargetTypemask` bits).
- lil-link lacks graceful shutdown — graft devore `should_stop`/`stop_thread`/joins.
- `Gps` lat/lon should be f64 (lat/lon deg×1e7 precision); whisper used f32.
- lil-link swaps module names: `request_arm.rs` holds `QuadSetModeRequest`,
  `request_mode_set.rs` holds `QuadArmRequest` — normalize on port.

## Human help

- Confirm crate name `vs-mavlink` (crate `vs_mavlink`) vs `quadlink`.
- Confirm `quadlink/` full topic namespace (incl. command topics) is acceptable.
- Confirm heartbeat stays in `start_thread` (gated) rather than BrokerTask.

## Followups

- Implement `libs/vs-mavlink`; integrate with `vs-broker`/`vs-data-store`/`vs-wtf`.
- Add per-command `COMMAND_ACK` timeout/retry (deferred to v2).

## Links

- Plan (updated): `/tmp/opencode/vfp-research/cont/quadlink.plan.updated.md`
- Task (updated): `/tmp/opencode/vfp-research/cont/quadlink.task.updated.md`
- Plan (prior): `docs/plan/quadlink.plan.md`
- Idea: `docs/ideas/quadlink.idea.md`
- Sources: `lil-hopps/lil-link/src/mavlink/`, `project-devore/quad/src/ardulink/connection.rs`,
  `project-firefly/src_flight/whisper/src/mavlink/`

## Open questions

- Velocity `type_mask` bits to enable vx/vy/vz and ignore x/y/z (not yet pinned).
- Ack timeout/retry for a missing `COMMAND_ACK` (not yet in scope).

## Advice / lessons

- Thread model stays std threads + `crossbeam-channel`; keep `std::sync::Mutex` (not tokio).
- Keep whisper's `timing: &BrokerTime` out of `MavlinkMessageProcessor::on_mavlink_message`;
  the `output` `DataView` is already `new_timed(timing.time_monotonic.clone())`.
- This crate's lib names: `vs-broker`→`vs_broker`, `vs-data-store`→`vs_data_store`,
  `vs-wtf`→`vs_wtf` (no `[lib]` override in the workspace).
