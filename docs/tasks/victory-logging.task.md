---
title: victory-logging — logging/tracing helper plan
type: task
key: victory-logging
branch: vfp/agent/plan/victory-logging
pr: https://github.com/victoryforphil/AutonomyAge/pull/36
desc: Port a reusable logging/tracing setup helper into a vs-* crate (plan).
status: active
update: Research complete — decisions resolved, concrete migration list produced
last_updated: 2026-08-23
---

## Context

Best base is `agentbox/crates/tremor-nodekit` `init_logger(filter, dir, file)`
(RUST_LOG precedence, guarded non-blocking file writer, `OnceLock<WorkerGuard>`).
Grafts from dark-factory and tinyverse add ANSI/TTY detection, `VS_LOG` path
override, `InitOptions`, and the `FormatEvent`/`FormatFields` extension seam.

Research is now complete: all three open questions are resolved, the test-init
design was corrected, and a concrete migration list for `vs-broker` / `vs-data-store`
was produced.

## Todos

- [x] Survey logging setups across org
- [x] Draft `docs/plan/victory-logging.plan.md`
- [x] Read exact logging call sites + deps in `vs-broker` / `vs-data-store`
- [x] Resolve crate name, facade, guard lifetime, test-init design
- [x] Write concrete migration list + updated plan (`/tmp/opencode/vfp-research/cont/victory-logging.plan.updated.md`)
- [ ] Implement `libs/vs-logging`
- [ ] Migrate `vs-broker` / `vs-data-store` to it and prune deps

## State

- Plan drafted; PR #36 open.
- Research subagent advanced the plan; decisions locked (see updated plan).

## Risks

- `pretty` + `without_time` interaction (default `pretty_stdout=false`).
- `VS_LOG` override edge cases (bare filename → cwd; empty value ignored).
- Do **not** mix `test_env_log::test` and global `init_logging_test` in one test binary.
- `tracing-tracy` + `tracy_full` in `vs-data-store` are latent (no `TracyLayer`
  registered today); keep behind `vs-logging/tracy` feature or prune.
- `vs-dir` ordering: if it lands first, swap `default_logs_dir()` body.

## Human help

- None blocking now. (Was: decide `victory-logging` vs `vs-logging` → resolved
  as `vs-logging` to match the `vs-*` workspace; keep `victory-logging` as PR/idea name.)
- Sanity-check the `pretty` formatter behavior with `without_time` when enabling it.

## Followups

- Implement `libs/vs-logging`; migrate `vs-broker`/`vs-data-store`; prune deps;
  write `docs/designs/victory-logging.md` recording the name mapping.

## Links

- Plan: `docs/plan/victory-logging.plan.md`
- Updated plan: `/tmp/opencode/vfp-research/cont/victory-logging.plan.updated.md`
- Idea: `docs/ideas/victory-logging.idea.md`
- Sources: `agentbox/crates/tremor-nodekit/src/lib.rs`, dark-factory + tinyverse (raw)
- PR: https://github.com/victoryforphil/AutonomyAge/pull/36

## Open questions

- Whether to expose pluggable `FormatEvent`/`FormatFields` now (feature `fmt`) or
  defer; default is defer behind the `fmt` feature.
- Whether to keep the latent `tracy` deps (behind `vs-logging/tracy`) or prune them.

## Advice / lessons

- `init_logging_test` must be a regular public fn, **not** `#[cfg(test)]` —
  downstream test modules need it visible; make it idempotent by swallowing the
  already-set `TryInitError` (race-free) rather than `has_been_set()`.
- `log` passthrough is free via `tracing-subscriber/tracing-log`
  (`SubscriberInitExt::try_init` installs the `LogTracer`); keep a `log` feature
  mapping to it.
- Keep the crate library-shaped: no hard `clap`/`time` deps (timestamped file name
  uses `SystemTime`).
