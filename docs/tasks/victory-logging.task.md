---
title: victory-logging — logging/tracing helper plan
type: task
key: victory-logging
branch: vfp/agent/plan/victory-logging
pr: https://github.com/victoryforphil/AutonomyAge/pull/36
desc: Port a reusable logging/tracing setup helper into a vs-* crate (plan).
status: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

Best base is `agentbox/crates/tremor-nodekit` `init_logger(filter, dir, file)` (RUST_LOG
precedence, guarded non-blocking file writer, `OnceLock<WorkerGuard>`). Grafts from
dark-factory and tinyverse add ANSI/TTY detection, path override, options struct, and
the `FormatEvent`/`FormatFields` extension seam.

## Todos

- [x] Survey logging setups across org
- [x] Draft `docs/plan/victory-logging.plan.md`
- [ ] Port tremor-nodekit base + grafts

## State

- Plan drafted; PR #36 open.

## Risks

- `log` (vs-broker) vs `tracing` (vs-data-store) facade mixing; prefer `tracing`.
- Guard lifetime for the non-blocking file writer.
- Redundant `env_logger`/`pretty_env_logger`/`tracing-subscriber` deps in the workspace.

## Human help

- Decide crate name `victory-logging` vs `vs-logging` (workspace uses `vs-*`).

## Followups

- Implement `libs/vs-logging`; migrate `vs-broker`/`vs-data-store` to it; prune deps.

## Links

- Plan: `docs/plan/victory-logging.plan.md`
- Idea: `docs/ideas/victory-logging.idea.md`
- Sources: `agentbox/crates/tremor-nodekit/src/lib.rs`, dark-factory + tinyverse (raw)

## Open questions

- Env-override semantics for the log file path; expose the WorkerGuard?

## Advice / lessons

- Keep the crate library-shaped (generic signature, no hard clap/time deps).
