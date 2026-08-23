---
title: quadlink — MAVLink framework plan
type: task
key: quadlink
branch: vfp/agent/plan/quadlink
pr: https://github.com/victoryforphil/AutonomyAge/pull/35
desc: Port a reusable MAVLink communication & command framework into a vs-* crate (plan).
status: active
update: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

Best base is `lil-hopps/lil-link` (literally "quadlink", pure-serde `Quad*` types,
already wired to `victory-*`/`vs-*`). Graft graceful-shutdown lifecycle from
`project-devore` and extra builders/processors from `project-firefly/whisper`.

## Todos

- [x] Compare lil-link / whisper / project-devore
- [x] Draft `docs/plan/quadlink.plan.md`
- [ ] Port lil-link + graft devore lifecycle + whisper builders/processors

## State

- Plan drafted; PR #35 open.

## Risks

- `mavlink` version: baseline 0.13.1; `cursed-mav` pins 0.14.1. Recommend 0.13.1 first.
- lil-link lacks graceful shutdown (no `should_stop`/`stop_thread`/joins) — graft devore.
- whisper's prost `whisper::common` coupling must be rewritten to pure-serde.

## Human help

- Decide crate name (`vs-mavlink` vs `quadlink`) and whether heartbeat lives in
  start_thread or the BrokerTask.

## Followups

- Implement `libs/vs-mavlink`; integrate with `vs-broker`/`vs-data-store`/`vs-wtf`.

## Links

- Plan: `docs/plan/quadlink.plan.md`
- Idea: `docs/ideas/quadlink.idea.md`
- Sources: `lil-hopps/lil-link/src/mavlink/`, `project-devore/quad/src/ardulink/`

## Open questions

- Ack timeout/retry for a missing `COMMAND_ACK` (not yet in scope).
- `quadlink/` topic prefix adoption vs bare keys.

## Advice / lessons

- Thread model stays std threads + crossbeam; keep `std::sync::Mutex` (not tokio).
