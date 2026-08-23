---
title: victory-dir — directory locator plan
type: task
key: victory-dir
branch: vfp/agent/plan/victory-dir
pr: https://github.com/victoryforphil/AutonomyAge/pull/37
desc: Port a generic directory locator utility into a vs-* crate (plan).
status: active
update: research complete — all 4 open questions resolved, concrete vs-dir API + example written
last_updated: 2026-08-23
---

## Context

Functional source is `project-firefly/src_core/waldo/dir_utils.rs` (env `{APP}_DIR`
or CWD `.firefly`, create-if-missing). Best model to copy is
`agentbox/crates/tremor-nodekit` (`tremor_home`/`logs_dir`/`data_dir`,
HOME/USERPROFILE aware) + mad-rs repo discovery + waldo env override. Plan makes
it generic with the dot-dir name as a parameter. All four open questions are now
resolved and a concrete `libs/vs-dir` API + example is written to the updated plan.

## Todos

- [x] Survey directory-locator patterns
- [x] Draft `docs/plan/victory-dir.plan.md`
- [x] Resolve the 4 open questions (env semantics, markers, no-repo fallback, dirs crate)
- [x] Produce concrete `libs/vs-dir` API + usage example (updated plan §4 §5)
- [ ] Implement `libs/vs-dir`

## State

- Plan drafted; PR #37 open. Research/decisions complete (see updated plan):
  - Crate name → `vs-dir`. `victory-dir` stays the idea/PR name.
  - Env `{APP}_DIR` = **final dot-dir** (waldo), not a parent base. Never appended
    with the dot name; `logs`/`data` always hang below it. Empty value = unset.
  - Repo markers default `{ .git, AGENTS.md, [workspace] }`, nearest-ancestor-wins,
    configurable via `RepoMarker`/`repo_root_from_with`. `.git` is the only one
    guaranteed in AutonomyAge.
  - No-repo fallback default = `NoRoot::Home` (`~/.{dot}`); CWD/Error exposed via
    `NoRoot`.
  - `dirs` crate **not** used; hand-rolled `HOME`/`USERPROFILE` in `home_dir()`
    (single swap point if cross-platform needs change). Deps = `anyhow` only.

## Risks

- API surface breadth (`NoRoot`/`RepoMarker` + `_with` variants) — kept minimal,
  each with a default; common path is `app_dir`/`repo_dot_dir`/`logs_dir`/`data_dir`.
- `VS_DIR` naming collision — `vs-logging` intentionally uses `VS_LOG` above the
  `vs_dir` seam; it should not inherit a `VS_DIR` override it does not own.
- `[workspace]` marker is a cheap text contains-check, not a `toml` parse (weakest,
  below `.git`).
- Relative `{APP}_DIR` absolutized against launch cwd — documented.

## Human help

- Approve crate name `vs-dir` (workspace uses `vs-*`; `victory-dir` = idea/PR label).
- Confirm the `VS_LOG`-above-`vs_dir` boundary with `vs-logging` (plan §6) so
  `default_logs_dir()` can swap to `vs_dir::logs_dir("vs", ".vs")`.

## Followups

- Implement `libs/vs-dir` (plan §4) and add to workspace `members`.
- Port mad-rs repo-root tests + add §8 cases (env final-dot-dir, no-repo policies,
  create-on-demand, marker toggling).
- Re-point `vs-logging::default_logs_dir()` at `vs_dir::logs_dir("vs", ".vs")`.
- Write `docs/designs/victory-dir.md` recording name mapping + decisions.

## Links

- Plan: `docs/plan/victory-dir.plan.md`; Updated: `/tmp/opencode/vfp-research/cont/victory-dir.plan.updated.md`
- Idea: `docs/ideas/victory-dir.idea.md`
- Sources: `project-firefly/src_core/waldo/dir_utils.rs`;
  `agentbox/crates/tremor-nodekit`; `mad-rs/cursed/mad_common/mad_dir_utils.rs`;
  `agentbox/nodes/tremor-executor/src/project_context.rs`; `lil-hopps/lil-rerun`.

## Open questions

- None blocking. (Closed: env semantics, markers, no-repo fallback, dirs crate.)
- Latent: whether `logs_dir`/`data_dir` should ever honor a per-subdir env var
  (e.g. `{APP}_LOGS_DIR`) — deferred, not needed by current consumers.

## Advice / lessons

- No repo uses `directories::ProjectDirs`; the org pattern is env override → home/
  repo base → create-if-missing. Keep `vs-dir` at `anyhow`-only.
- `{APP}_DIR` (like `CARGO_HOME`/`CARGO_TARGET_DIR`) should name the **final**
  directory, not a parent base — waldo got it right, lil's parent-base form is the
  footgun. Do not re-add a panic-on-error (`create_dir_all(...).unwrap()`).
- Keep `.git` first in the marker set (it is the strongest and the only one
  guaranteed in AutonomyAge); `AGENTS.md`/`[workspace]` generalize to other repos.
