---
title: victory-dir — directory locator plan
type: task
key: victory-dir
branch: vfp/agent/plan/victory-dir
pr: https://github.com/victoryforphil/AutonomyAge/pull/37
desc: Port a generic directory locator utility into a vs-* crate (plan).
status: active
update: implementation, example, README, and design note complete
last_updated: 2026-09-02
---

## Context

Functional source is `project-firefly/src_core/waldo/dir_utils.rs` (env override
and create-if-missing). The implementation keeps its direct path-building style
while making resolution order explicit:

```text
configured env base/.dot → cwd/.dot → Git root/.dot → Cargo crate root/.dot → home/.dot
```

`DirLocator` owns the configuration and launch cwd. Logging and data paths call
the same `app_dir()` method, so they cannot silently choose a different scope.

## Todos

- [x] Survey directory-locator patterns
- [x] Draft and finalize `docs/plan/victory-dir.plan.md`
- [x] Add repository Rust style guide and local style skill
- [x] Implement modular `libs/vs-dir`
- [x] Convert directory behavior to `DirLocator` member methods
- [x] Add high-level scenario tests

## State
- `src/lib.rs` is thin and re-exports `DirConfig` and `DirLocator`.
- `src/path/mod.rs` owns the path domain.
- `src/path/locator.rs` owns member methods and root selection.
- `src/path/utils.rs` contains only shared path mechanics.
- `src/path/tests.rs` contains four high-level scenarios that are extended as
  behavior grows.
- The API has one explicit env variable base and always appends the dot name.
- Resolution is configured env base → cwd → Git root → Cargo crate root → home.
- Home is the final fallback and is created only when no project candidate exists.
- `libs/vs-dir/examples/locator.rs`, `libs/vs-dir/README.md`, and the design
  note document the same stable API.

## Human help

- None blocking. Cargo workspace roots are intentionally not a separate mode;
  the nearest Cargo package root is the Rust boundary for this first version.

## Followups

- Keep `vs-logging`'s `VS_LOG` file-path override separate from `vs-dir`.
- Update the repo index when the crate is adopted by a consumer.

## Links

- Plan: `docs/plan/victory-dir.plan.md`
- Idea: `docs/ideas/victory-dir.idea.md`
- Sources: local Pi and OpenCode studies referenced by the plan

## Open questions

- None blocking.
- Deferred: whether a consumer needs a final-path override in addition to the
  base-directory environment model.

## Advice / lessons

- Keep the locator boring: one struct, one resolution path, and one fallback
  order shared by app, logs, and data directories.
- Preserve high-level scenario tests instead of accumulating one test per edge
  case.
