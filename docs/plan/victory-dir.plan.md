---
title: Victory Dir
type: plan
status: todo
tags:
  - plan
  - dir
---

# Victory Dir — Plan

A reusable directory-locator utility for the AutonomyAge `vs-*` Rust workspace.
Resolves where an app keeps its dot-directory (logging, configs, artifacts),
preferring an env override, then a user-home or repo-root base, creating the
directory on demand.

## Purpose

Today every tool hand-rolls its own "where do I store `.firefly` / `.madrs` /
`.tremor` / `.lil` / `.chell`?" logic, with subtly different answers each time
(env override vs home vs repo-root vs CWD, and whether the dir is auto-created).
This crate collapses that into one generic locator so a new tool gets a correct,
create-on-demand dot-dir in a single call, and the org stops copy-pasting
slightly-divergent implementations.

Scope: **path resolution only.** It returns paths; it does not own logging setup,
config parsing, or artifact lifecycle (those belong to the consumer, or `vs-logging`).

## Source implementations found

| Repo | Path | Date | Notes |
|------|------|------|-------|
| project-firefly | `src_core/waldo/src/dir_utils.rs` | Feb 2025 | `DirUtils::get_firefly_dir()` — env `FIREFLY_DIR`, else `CWD/.firefly`, create-if-missing, `anyhow::Result`. **Single-purpose**, firefly-named. `waldo` deps: `anyhow`/`log`/`thiserror`. |
| agentbox | `crates/tremor-nodekit/src/lib.rs` | Jun 2026 | **Newest, best model.** `tremor_home()` (`~/.tremor`, `HOME` + `USERPROFILE` fallback), `logs_dir()`, `data_dir()`, `ensure_repo_symlink(repo_root)`, all create-on-demand, `anyhow::Result<PathBuf>`. |
| mad-rs | `cursed/mad_common/src/mad_dir_utils.rs` | recent | `madrs_dir()`/`ensure_madrs_dir()`/`find_dev_repo_root()` — walks ancestors for a dir named `mad-rs` or a workspace `Cargo.toml` containing `tools/geo_simplify`. **Has unit tests**. |
| agentbox | `nodes/tremor-executor/src/project_context.rs` | (agentbox) | `resolve_workspace_root()` — walks up until `.git` exists or `AGENTS.md` is a file, falling back to cwd. |
| chell | `src/main.rs` | (chell) | `dirs::home_dir()?.join(".chell")` — simplest `dirs::home_dir()` pattern, no env, no create. |
| lil-hopps | `lil-rerun/src/lib.rs` | (lil-hopps) | `.lil/logs`; env `LIL_DIR` else `CARGO_MANIFEST_DIR`, then `create_dir_all(...).unwrap()` (panics on error). |
| project-firefly | `src_ground/ground_control/conductor/src/main.rs` | Feb 2025 | Manifest-relative `.firefly` via `CARGO_MANIFEST_DIR.parent().parent()` — fragile, build-machine specific, no env. |

**Notable absence**: no repo uses `directories::ProjectDirs`; all use `dirs::home_dir()`
or a hand-rolled `HOME`/`USERPROFILE` read. The established pattern is **env override →
home/repo base → create-if-missing**, not platform-config-dir discovery.

## Best version to port

Port **agentbox `tremor-nodekit`** as the skeleton (newest, closest to a reusable
locator), fold in three missing pieces, and make the app name a parameter:

- **Home layout** (tremor-nodekit): `home_dir()` with `HOME`→`USERPROFILE` fallback + `logs_dir()`/`data_dir()` subpaths.
- **Repo-root discovery** (mad-rs + project_context): walk `ancestors()` for a stop marker (`.git` dir, `AGENTS.md` file, or a `Cargo.toml` containing `[workspace]`).
- **Env override first** (waldo + lil): for app `X`, prefer `${X}_DIR` (uppercased, `-`→`_`) before the home/repo base.
- **Create-if-missing** (all): every returned dot-dir is `create_dir_all`'d; errors surface as `anyhow::Result`.
- **Make it generic**: the dot-dir name is a **parameter**, not hardcoded — that generalizes `.tremor`/`.madrs`/`.firefly`/`.lil`/`.chell` into one function.

## Proposed API surface

```rust
/// Resolve the user home dir. `HOME`, else `USERPROFILE`, else `dirs::home_dir()`.
pub fn home_dir() -> anyhow::Result<PathBuf>;

/// Resolve an app's dot-dir under the repo root. Env `${APP}_DIR` wins if set.
/// Creates the dir if missing. Falls back to home if no repo root is found.
pub fn repo_dot_dir(app: &str, dot: &str, start: &Path) -> anyhow::Result<PathBuf>;

/// Convenience: repo-dot-dir starting from `std::env::current_dir()`.
pub fn repo_dot_dir_from_cwd(app: &str, dot: &str) -> anyhow::Result<PathBuf>;

/// Resolve an app's home-based dot-dir (`~/.{dot}`), env `{APP}_DIR` override first.
pub fn app_dir(app: &str, dot: &str) -> anyhow::Result<PathBuf>;

/// `{app dot-dir}/logs` (creates it).
pub fn logs_dir(app: &str, dot: &str) -> anyhow::Result<PathBuf>;

/// `{app dot-dir}/data` (creates it).
pub fn data_dir(app: &str, dot: &str) -> anyhow::Result<PathBuf>;

/// Best-effort walk up from `start` to the nearest repo root (markers: `.git`,
/// `AGENTS.md`, or a `[workspace]` manifest). Returns `None` if not found.
pub fn repo_root_from(start: &Path) -> Option<PathBuf>;
```

Design notes:

- `repo_root_from` returns `Option` (best-effort; callers choose the fallback), and is the only function without `create_dir_all`.
- Env override is applied **first** in every `*_dir` path.
- All path-returning fn are `anyhow::Result<PathBuf>` so `?` works in `main`.
- The dot name and env name are separate args for generality; a single-arg convenience collapses them when they match.

## Integration / conventions

- **Crate name: `vs-dir`** under `libs/vs-dir/`, matching `libs/vs-broker`, `vs-data-store`, `vs-wtf`. Add it to the workspace `members`.
- **Naming question (open)**: the idea/plan say `victory-dir`; the workspace renamed `victory-*`→`vs-*`, so `vs-dir` is consistent. The doc title may stay "Victory Dir" as a human label. Decide and record the mapping in the idea file so future `vs-*` crates follow the same rule.
- **Dependencies**: `anyhow` only (already in `Cargo.lock`); optionally `dirs` for a platform-aware `home_dir()`. Keep it dependency-light.
- **No logging/init coupling** — `init_logging`-style helpers stay in `vs-logging`.
- **Version**: `0.0.1`, `edition 2021`.

## Risks / open questions

- **`{APP}_DIR` override semantics**: is the env value the final dot-dir or a parent base (waldo uses it directly; lil appends `.lil/logs`)? Decide and document.
- **Repo-root marker ambiguity**: different tools want different markers (`.git`, `AGENTS.md`, `[workspace]`). Consider an optional marker param.
- **No-repo fallback**: when no repo root is found, fall back to `~/.{dot}`, error, or use CWD? Pick one default and expose the others.
- **Home vs repo precedence**: make home- vs repo-scoped explicit at the call site (two functions), not a hidden heuristic.
- **Error-vs-panic**: use `anyhow::Result` to avoid `.unwrap()` on unwritable locations.

## Next steps

1. Add `libs/vs-dir` to the workspace members.
2. Implement the API surface above.
3. Port the mad-rs `find_dev_repo_root` tests + add tests for env override, home fallback, create-if-missing.
4. Decide/document the env-override semantics and encode in doc comments.
5. Add a doc example resolving `.vs`/`.mytool` with and without an env override.
6. Optionally re-point `vs-logging` (when built) at `vs-dir::logs_dir`.

## Links

- Idea: [`docs/ideas/victory-dir.idea.md`](../ideas/victory-dir.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
