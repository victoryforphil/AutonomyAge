---
title: victory-dir — directory locator plan
type: task
key: victory-dir
branch: vfp/agent/plan/victory-dir
pr: https://github.com/victoryforphil/AutonomyAge/pull/37
desc: Port a generic directory locator utility into a vs-* crate (plan).
status: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

Functional source is `project-firefly/src_core/waldo/dir_utils.rs` (env `{APP}_DIR` or
CWD `.firefly`, create-if-missing). Best model to copy is `agentbox/crates/tremor-nodekit`
(`tremor_home`/`logs_dir`/`data_dir`, HOME/USERPROFILE aware) + mad-rs repo discovery +
waldo env override. Plan makes it generic with the dot-dir name as a parameter.

## Todos

- [x] Survey directory-locator patterns
- [x] Draft `docs/plan/victory-dir.plan.md`
- [ ] Implement `libs/vs-dir`

## State

- Plan drafted; PR #37 open.

## Risks

- `{APP}_DIR` env override base-vs-final semantics unresolved.
- Repo-root marker ambiguity (`.git` vs `AGENTS.md` vs `[workspace]` manifest).

## Human help

- Decide the `victory-dir` vs `vs-dir` crate name (workspace uses `vs-*`).
- Choose the "no repo root found" fallback (home / error / cwd).

## Followups

- Implement `libs/vs-dir`; optionally re-point `vs-logging` at `vs_dir::logs_dir()`.

## Links

- Plan: `docs/plan/victory-dir.plan.md`
- Idea: `docs/ideas/victory-dir.idea.md`
- Sources: `project-firefly/src_core/waldo/dir_utils.rs`, `agentbox/crates/tremor-nodekit`

## Open questions

- Use `dirs` crate vs hand-rolled `HOME`/`USERPROFILE` read (keep it dependency-light).

## Advice / lessons

- No repo uses `directories::ProjectDirs`; `dirs::home_dir()` is the established pattern.
