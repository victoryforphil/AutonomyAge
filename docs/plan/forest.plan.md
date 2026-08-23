---
title: forest
type: plan
status: todo
tags:
  - plan
  - scenario-runner
---

# forest — Plan

Port project-firefly's file-driven scenario runner into a reusable crate.

## Purpose

A scenario runner framework: scenarios defined as files (markup/data) describing a
sequence of steps, so content and runner logic stay separate. Steps are pluggable:
register step handlers and compose scenarios from reusable building blocks. Composes
with `vs-broker` / `vs-data-store` to drive task steps and with `valley` for
validating outcomes. Useful for sims, hardware-in-the-loop, and replay.

## Source implementations found

| Repo | Path | Date | Notes |
|------|------|------|-------|
| project-firefly | `src_tools/forest/` | Feb 2025 | The **only** implementation in the org (confirmed via `gh search code "ScenarioRunner"` / `sil_runner`). |

No newer or alternative implementation exists anywhere in `victoryforphil` /
`AndreasLabs`. This is not a "pick the best of N" idea — it is a "migrate the one
real implementation (it is the newest) and de-couple it" idea.

## How scenarios are defined (as files/data)

The app is file-driven from `config.yaml`:

- `ForestAppConfig { name, num_whisps, docker, whisp }` (`config/mod.rs`,
  `load_from_yaml`). `config_app.rs` / `config_whisp.rs` are empty stubs — all real
  config lives in `config/mod.rs`.
- `WhispTemplateConfig` expands into per-instance `ForestWhipConfig` via
  `generate_whisp_config(index, log_dir)` (port offsets, per-index log dirs).
- "Steps" are:
  1. A `Commander` task's `commands` — a `Vec<Command { name, trigger, action }>`.
     `trigger` is a serde-tagged enum (`Instant | Time | RelativeTime |
     TopicStringEquals | TopicComparison`); `action` is `SetArm | SetMode | Takeoff
     | Land | Waypoint | Idle | SetHome | EnterShow`. Executed by
     `CommanderCommand::execute(name, task, out: &mut Dataview, timing)`.
  2. A `ShowRunner` task loads `show.yaml` → a timed show scenario.
- Pluggability is `ForestMiddleware` (`middleware/mod.rs`): trait
  `on_init/on_start/on_tick/on_stop/on_postprocess`, with
  `ForestMiddlewareResult { Continue, Stop, ResultSuccess(bool), ResultFailure(bool) }`
  and `ForestMiddlewareContext { broker, node, node_info, last_tick, delta,
  validations, log_directory }`. Concrete: `save_datastore`, `save_validation`,
  `print_validation`.
- Runner loop: `SILRunner` (`runners/sil_runner.rs`) + `ForestApplication`
  (`application.rs`): `setup → start → loop{tick} → stop → post_process`.

## Best version to port

Port `project-firefly/src_tools/forest` **as-is conceptually**, but migrate off the
`wingman-*` APIs onto `vs-*`, and genericize the task/step registry so it isn't
hard-wired to flight-specific tasks.

## What to de-couple for a reusable crate

| firefly (wingman) | target (vs-*) |
|-------------------|---------------|
| `wingman-task-rs` `BrokerServer`/`BrokerNode`/`BrokerTask`/`BrokerNodeInfo`, TCP adapters | `vs-broker` equivalents |
| `wingman-data-rs` `Dataview`/`DatastoreFilter`/`TopicPath` | `vs-data-store` `DataView`/`TopicKey` |
| `wingman-core-rs` `Timespan`/`Timepoint` | `vs-wtf` |
| `ForestTaskType`/`ForestValidationType` concrete enums | user-supplied factory traits (generic step/validator registry) |
| `waldo::DirUtils::get_firefly_dir()` | new `vs-dir` (see `victory-dir.plan.md`) |
| `valley::SILValidator` | new `vs-valley` (see `valley.plan.md`) |

Also drop the hard `whisper`/`whisp`/`commander`/`showkit-rs`/`lit-rerun` coupling.

## Note on migration

The `vs-*` crates are a **rename of the Oct-2024 `victory-*` repos**, not the
Feb-2025 `wingman-*` crates forest currently compiles against. So this is an API
migration, not a find-and-replace. Known breaks: `vs-broker` uses `Broker` +
`broker.tick(delta)` (sync) vs `BrokerServer`/`broker.run(delta).await`;
`vs-data-store` `DataView` has no `with_query`/`DatastoreFilter`; `vs-wtf`
`Timecode{secs,nanos}` vs wingman `{seconds,microseconds}` and different
constructor names (`new_secs(f64)` vs `new_secs_f64(f64)`).

## Risks / open questions

- **High coupling** — porting effort is the largest of the `vs-*` ideas. Doing a
  generic (non-flight) registry is the main new design work.
- **Async vs sync** — forest is tokio-heavy (full tokio, `async_trait` middleware,
  `tokio::spawn`); `vs-broker`'s `tick` looks sync. Reconcile the runner loop.
- **README drift** — `forest/README.md` documents an `AppConfig`/`AppRunner` API
  that doesn't exist. The code is the spec.
- `ForestGeneratedPort` / `DockerComposeConfig` are domain-adjacent; decide whether
  to keep or make optional.

## Next steps

1. Add `libs/vs-forest` to the workspace, depending on `vs-broker`/`vs-data-store`/`vs-wtf`.
2. Extract the generic `Step`/`Scenario`/`Middleware` traits; move concrete
   flight tasks into an `examples/` or a `flight` feature.
3. Port `waldo`→`vs-dir` and `valley`→`vs-valley`, then wire forest to them.
4. Add a file-based scenario example + a `valley`-driven validation example.

## Links

- Idea: [`docs/ideas/forest.idea.md`](../ideas/forest.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
- Related: `valley.plan.md`
