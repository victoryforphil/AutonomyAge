---
title: Victory Dir — Updated Plan (research complete)
type: plan
status: todo
tags:
  - plan
  - dir
  - research
---

# Victory Dir — Updated Plan

> **Status: research complete, decisions resolved.** This advances
> `docs/plan/victory-dir.plan.md` (PR #37). It resolves the four open questions
> (env-override semantics, repo-root markers, no-repo fallback, `dirs` vs
> hand-rolled), fixes the crate-name mapping, and produces a **concrete
> `libs/vs-dir` API** with a usage example.

## 1. Purpose (unchanged)

A reusable directory-locator utility for the AutonomyAge `vs-*` workspace.
Resolves where an app keeps its dot-directory (logging, configs, artifacts),
preferring an env override, then a user-home or repo-root base, creating the
directory on demand.

Scope: **path resolution only.** It returns paths; it does not own logging
setup, config parsing, or artifact lifecycle (those belong to the consumer, or
`vs-logging`).

## 2. Resolved decisions

### 2.1 Crate name → `vs-dir`

- Workspace members are `vs-*` (`libs/vs-data-store`, `libs/vs-wtf`,
  `libs/vs-broker`; `Cargo.toml` `members`). A `victory-dir` crate would break
  the convention next to the existing `vs-*` crates.
- **Decision:** crate `vs-dir` at `libs/vs-dir`, added to workspace `members`.
- `victory-dir` remains the *idea / PR / plan* name. Record the mapping in
  `docs/designs/victory-dir.md` (mirroring the `vs-logging` decision) so future
  `victory-*`→`vs-*` renames follow the same rule.

### 2.2 `{APP}_DIR` env override semantics → **final dot-dir (waldo), not a parent base**

- **waldo** uses `FIREFLY_DIR` *directly* as the final dot-dir (`PathBuf::from(dir)`),
  then create-if-missing.
- **lil** treats `LIL_DIR` as a *parent base* and appends `.lil/logs`.
- The two conflict. **Decision: waldo / final-dot-dir semantics** — the env value
  is the authoritative complete path for the app's dot-dir root; the dot name is
  **not** appended to it.

**Precise behavior:**

- Env key for app `X` = `X` uppercased with `-`→`_`, plus `_DIR`
  (e.g. `lil` → `LIL_DIR`, `my-tool` → `MY_TOOL_DIR`). The key is derived from
  **`app`**, not from `dot`.
- If that env var is set **and non-empty** → the dot-dir root is `PathBuf::from(value)`
  verbatim (absolutized against cwd if relative). This **bypasses** both home-base
  and repo-root discovery. It is **not** joined with the dot name.
- If unset/empty → compute the base (home or repo root), then append the dot name
  (`home/.{dot}` or `repo/{dot}`).
- `logs_dir`/`data_dir` always append to the resolved root
  (`{dot_dir}/logs`, `{dot_dir}/data`), whether or not the root came from env.
  So `LIL_DIR=/data/lil` → `/data/lil`, `/data/lil/logs`, `/data/lil/data`.
- An empty value (`LIL_DIR=""`) is treated as unset.
- **Why final-dot-dir:** it is the intuition of `{APP}_DIR` (compare `CARGO_HOME`,
  `RUSTUP_HOME`, `CARGO_TARGET_DIR` — each names the *final* home/root, not a
  parent). It also keeps `logs_dir`/`data_dir` independent of which source won.
  Documented as the authoritative-escape-hatch: overriding the base without
  appending a dot never surprises the user.

### 2.3 Repo-root markers → default `{ .git, AGENTS.md, [workspace] }`, configurable

- **agentbox** `project_context.resolve_workspace_root()` walks up until `.git`
  exists **or** `AGENTS.md` is a file (falls back to cwd).
- **mad-rs** walks up for a dir named `mad-rs`, **or** a `Cargo.toml` containing
  `[workspace]` and `tools/geo_simplify`.
- **Decision:** default marker set = **any-of**, applied per ancestor from the
  nearest upward; the **nearest ancestor that matches any marker** is the root.

Default markers (in priority by signal strength, but selection is "first ancestor
that matches any" from `start` upward):

1. `.git` — exists as a **dir** (normal repo) **or a file** (worktree / submodule).
2. `AGENTS.md` — is a **regular file**.
3. `[workspace]` manifest — `Cargo.toml` that **contains** `[workspace]`.

- **Configurable:** `repo_root_from(start)` uses the default set;
  `repo_root_from_with(start, markers: &[RepoMarker])` overlays a caller-selected
  set. `RepoMarker` is a small enum (`GitDir`, `AgentsMd`, `WorkspaceManifest`) so
  callers toggle signals without a closure and without a `dirs`-style build.
- Note: `.git` is the only marker guaranteed in AutonomyAge (`.git` is a dir; no
  `AGENTS.md`). `AGENTS.md`/`[workspace]` generalize to other repos
  (agentbox, mad-rs) and to vendored crates. Ordering (nearest-ancestor-wins)
  already handles nested repos/submodules correctly.
- Edge: if `start` is a **file** (e.g. a manifest path), start the walk at its
  parent.

### 2.4 No-repo fallback → **`~/.{dot}` (home) by default**; Cwd and Error exposed

- Options: fall back to `~/.{dot}`, use cwd, or error.
- **Decision: default = `NoRoot::Home`** (i.e. `~/.{dot}`), matching the
  tremor-nodekit `~/.tremor` model. Rationale: a dot-dir locator that *errors*
  just because you ran outside a repo is hostile (CLIs run anywhere), and the
  cwd fallback (waldo/mad-rs) pollutes whatever directory the user happens to be
  in with hidden dirs — a known footgun. `~/.{dot}` is stable, predictable, and
  matches the "user-local app data" model.
- **Decision:** expose the other policies explicitly via a `NoRoot` enum so
  callers can opt into strictness or the legacy cwd behavior:
  `NoRoot::{Home, Cwd, Error}` (`Default = Home`). `repo_dot_dir(...)` uses the
  default; `repo_dot_dir_with(..., on_no_root)` takes the policy.
- Note: the env override has **already been checked** before the no-repo branch,
  so if `{APP}_DIR` is set the fallback is never reached.

### 2.5 `dirs` crate vs hand-rolled → **hand-rolled, zero extra deps**

- **tremor-nodekit** hand-rolls `dirs_or_home()` (`HOME` on unix, `USERPROFILE`
  on non-unix). No repo in scope uses `directories::ProjectDirs`; one transiently
  used `dirs::home_dir()` (chell). The org's established pattern is hand-rolled.
- **Decision:** `vs-dir` depends on **`anyhow` only**. `home_dir()` reads
  `HOME` (unix) then `USERPROFILE` (windows) and errors if neither is set.
  Rationale: it is dependency-light, matches the org convention, covers the dev
  (unix) and fallback (windows) platforms we care about, and avoids pulling
  `dirs`' gated platform deps for marginal benefit.
- **Escape hatch:** the public `home_dir()` is the only place that touches home
  resolution. If cross-platform robustness (macOS edge cases, `SNAP`, etc.) is
  ever needed, swap its body to `dirs::home_dir()`/`.ok_or_else(anyhow...)`
  **without changing the API**. Document this so a future change is localized.

### 2.6 Error type → `anyhow::Result<PathBuf>` (no custom enum)

- The workspace crates use `anyhow::Result` broadly (`vs-broker`, `vs-data-store`),
  and `anyhow` is already in `Cargo.lock`. **Decision:** every path-returning fn
  is `anyhow::Result<PathBuf>`, so `?` works in `main`. **No custom error enum** —
  it would add surface without a consumer. `repo_root_from*` returns `Option`
  (best-effort, no I/O failure in normal flow; callers choose the fallback).

## 3. Source implementations (confirmed by re-reading raw sources)

| Repo | Path | Key content (confirmed) |
|------|------|------------------------|
| project-firefly | `src_core/waldo/src/dir_utils.rs` | `get_firefly_dir()` = `FIREFLY_DIR` env **directly** (final path, no append) else `CWD/.firefly`, create-if-missing, `anyhow::Result`. **Source of the final-dot-dir decision.** |
| agentbox | `crates/tremor-nodekit/src/lib.rs` | **Base skeleton.** `tremor_home()`=`~/.tremor`, `logs_dir()`, `data_dir()`, `ensure_repo_symlink()`; hand-rolled `dirs_or_home()` (`HOME`/`USERPROFILE`); all create-on-demand, `anyhow::Result<PathBuf>`. |
| mad-rs | `cursed/mad_common/src/mad_dir_utils.rs` | `madrs_dir()`/`ensure_madrs_dir()`, `find_dev_repo_root()` walks ancestors for dir named `mad-rs` or a `[workspace]` + `tools/geo_simplify` manifest. **Source of ancestor repo-root walk + unit tests.** |
| agentbox | `nodes/tremor-executor/src/project_context.rs` | `resolve_workspace_root()` walks up until `.git` or `AGENTS.md` file, falls back to cwd. **Source of `.git`/`AGENTS.md` markers.** |
| lil-hopps | `lil-rerun/src/lib.rs` | `.lil/logs`; env `LIL_DIR` else `CARGO_MANIFEST_DIR`, then `create_dir_all(...).unwrap()`. **Runs the parent-base alternative we rejected; also a panic-on-error counterexample.** |

## 4. Final `vs-dir` API (concrete)

### 4.1 `libs/vs-dir/Cargo.toml`

```toml
[package]
name = "vs-dir"
version = "0.0.1"
edition = "2021"

[dependencies]
anyhow = "1.0.104"
```

No other deps. (Note: `log`, `tracing`, `thiserror` etc. intentionally omitted.)

### 4.2 Public functions & doc comments

```rust
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

/// Env override key for an app's dot-dir: app uppercased, `-`→`_`, + `_DIR`.
/// `app_dir("my-tool")` → `Ok("MY_TOOL_DIR")`.
///
/// The key is derived from `app`, **not** from `dot` (a dot-name may have a
/// different stem than the env convention, though in practice they match).
pub fn env_name(app: &str) -> String;

/// Dot-dir name for an app: `"." + app`, tolerating a leading dot already present.
/// `dot_name("lil")` → `".lil"`; `dot_name(".lil")` → `".lil"`.
pub fn dot_name(app: &str) -> String;

/// What to do when `repo_dot_dir*` cannot find a repo root and no env override
/// applies. Default is `Home`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoRoot {
    /// Use `~/.{dot}` (default; matches tremor's `~/.tremor` model).
    Home,
    /// Use `{cwd}/{dot}` (legacy waldo/mad-rs behavior).
    Cwd,
    /// Return an error instead of picking a fallback.
    Error,
}

impl Default for NoRoot {
    fn default() -> Self { Self::Home }
}

/// Repo-root marker signals tested per ancestor. `repo_root_from*` picks the
/// nearest ancestor that matches **any** configured marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepoMarker {
    /// `.git` exists (as a dir, or a file for worktrees/submodules).
    GitDir,
    /// `AGENTS.md` is a regular file.
    AgentsMd,
    /// `Cargo.toml` whose text contains `[workspace]`.
    WorkspaceManifest,
}

/// Default marker set: `.git`, `AGENTS.md`, `[workspace]` manifest.
pub const DEFAULT_REPO_MARKERS: &[RepoMarker] = &[
    RepoMarker::GitDir,
    RepoMarker::AgentsMd,
    RepoMarker::WorkspaceManifest,
];

/// Resolve the user home dir. `$HOME` (unix), else `%USERPROFILE%` (windows),
/// else error. **Hand-rolled, no `dirs` dep.** Swap this body to
/// `dirs::home_dir()` if cross-platform edge cases ever matter.
pub fn home_dir() -> Result<PathBuf>;

/// Resolve an app's **home-based** dot-dir (`~/{dot}`). If `{APP}_DIR` is set
/// and non-empty, use that **verbatim** as the final dot-dir (final-dot-dir
/// semantics — the dot name is NOT appended). Creates the dir if missing.
///
/// `dot` is the literal dot-dir name including its leading dot (e.g. `".lil"`);
/// use [`dot_name`] to derive it from `app`.
pub fn app_dir(app: &str, dot: &str) -> Result<PathBuf>;

/// Resolve an app's **repo-scoped** dot-dir under the nearest repo root
/// (`{repo_root}/{dot}`). Precedence: env `{APP}_DIR` (verbatim, final) → repo
/// root → `NoRoot` fallback (default `Home` = `~/{dot}`). Creates the dir.
///
/// `start` is the directory (or file; its parent is used) from which to walk up
/// for a repo root. See [`repo_root_from`].
pub fn repo_dot_dir(app: &str, dot: &str, start: &Path) -> Result<PathBuf>;

/// Same as [`repo_dot_dir`] but with an explicit no-repo fallback policy.
pub fn repo_dot_dir_with(app: &str, dot: &str, start: &Path, on_no_root: NoRoot) -> Result<PathBuf>;

/// [`repo_dot_dir`] starting from `std::env::current_dir()`.
pub fn repo_dot_dir_from_cwd(app: &str, dot: &str) -> Result<PathBuf>;

/// `{dot-dir of app}/logs`, creating it. The dot-dir root is resolved by
/// [`app_dir`] (home-based), so `{APP}_DIR` relocates the whole tree.
pub fn logs_dir(app: &str, dot: &str) -> Result<PathBuf>;

/// `{dot-dir of app}/data`, creating it. See [`logs_dir`].
pub fn data_dir(app: &str, dot: &str) -> Result<PathBuf>;

/// Best-effort walk up from `start` (or its parent, if `start` is a file) to the
/// nearest repo root matching any marker in `DEFAULT_REPO_MARKERS`. Returns
/// `None` if not found. Does **not** create directories.
pub fn repo_root_from(start: &Path) -> Option<PathBuf>;

/// [`repo_root_from`] with a caller-selected marker set.
pub fn repo_root_from_with(start: &Path, markers: &[RepoMarker]) -> Option<PathBuf>;
```

### 4.3 Behavior / design notes

- `dot` = literal dot-dir name **including the leading dot** (`".lil"`,
  `".tremor"`, `".vs"`). It is the same string in home mode (`~/{dot}`) and repo
  mode (`{repo_root}/{dot}`); use [`dot_name`] to avoid repeating `"…"`+app.
- `repo_root_from*` are the **only** functions without create-dir; they return
  `Option` (best-effort) and never fail on I/O.
- Env override (final-dot-dir) is applied **first** in every `*_dir` path, so it
  bypasses both home and repo discovery. It is never joined with `dot`.
- Empty env value (`FOO_DIR=""`) is treated as unset.
- Env-relative paths are absolutized against `current_dir()` before `create_dir_all`.
- Creation failures surface through `anyhow::Result` (no `unwrap`/panic), fixing
  lil's `create_dir_all(...).unwrap()`.

## 5. Usage example

```rust
use std::path::Path;
use anyhow::Result;
use vs_dir::{app_dir, data_dir, logs_dir, repo_dot_dir, repo_dot_dir_with, NoRoot};
use vs_dir::dot_name;

fn main() -> Result<()> {
    // Home-based dot-dir: $HOME/.lil. Created on demand.
    // $LIL_DIR=/data/lil would instead give /data/lil (verbatim, final-dot-dir).
    let home = app_dir("lil", &dot_name("lil"))?;   // "$HOME/.lil"
    let logs = logs_dir("lil", &dot_name("lil"))?;  // "$HOME/.lil/logs"
    let data = data_dir("lil", &dot_name("lil"))?;  // "$HOME/.lil/data"

    // Repo-scoped dot-dir: nearest repo root + "/.lil", else falls back to $HOME/.lil.
    // $LIL_DIR always wins over both.
    let repo = repo_dot_dir(
        "lil",
        &dot_name("lil"),
        Path::new("/work/lil/tools/geo_simplify"),
    )?;

    // Opt into strict behavior: error if no repo root is found (no home fallback).
    let strict = repo_dot_dir_with("lil", &dot_name("lil"), std::path::Path::new("."), NoRoot::Error)?;

    // Best-effort repo discovery (no create, returns Option).
    if let Some(root) = vs_dir::repo_root_from(std::env::current_dir()?) {
        println!("repo root: {}", root.display());
    }

    println!("home={} logs={} data={}", home.display(), logs.display(), data.display());
    Ok(())
}
```

## 6. Coordination with `vs-logging` (directory boundary)

- `vs-logging` currently computes `~/.vs/logs` in a `default_logs_dir()` and adds a
  `VS_LOG` **file-path** override on top (`logs_dir()` returns `VS_LOG.parent()`).
  Its own override variable is `VS_LOG`, not `VS_DIR`.
- When `vs-dir` lands first, `vs-logging` should **swap the body of
  `default_logs_dir()`** to `vs_dir::logs_dir("vs", &vs_dir::dot_name("vs"))`
  (which resolves `~/.vs/logs`) while **keeping** its own `VS_LOG`-override layer
  above it. Do **not** have `vs-logging` call `vs_dir::app_dir` unconditionally,
  or it would inherit a `VS_DIR` env override it does not own.
- Concretely: `vs_dir::logs_dir("vs", ".vs")` == `vs-logging`'s `default_logs_dir()`
  (`$HOME/.vs/logs`). This gives a single source of truth for the home base while
  keeping the logging-specific file override local to `vs-logging`.
- This is the boundary to record in `docs/designs/victory-dir.md`.

## 7. Integration

- Add `"libs/vs-dir"` to the workspace `Cargo.toml` `members`.
- Create `libs/vs-dir/Cargo.toml` (§4.1), `src/lib.rs`, and a `README.md`
  mirroring the other `libs/vs-*` crates.
- Version `0.0.1`, edition `2021`.
- Add `libs/vs-dir` to `docs/agents/repo-index.md`'s org-facts note once the crate
  is created (record it as the first `vs-dir`).

## 8. Tests to add

Port the mad-rs `find_dev_repo_root` tests plus new cases:

- `repo_root_from` finds nearest `.git` (dir) and `.git` (file, worktree/submodule).
- `repo_root_from` finds a `[workspace]` manifest; does **not** match a lone
  package `Cargo.toml` (no `[workspace]`).
- `repo_root_from` with a file path starts at its parent.
- `repo_root_from_with` toggling markers off (e.g. `&[RepoMarker::GitDir]`).
- `env_name`/`dot_name` mapping incl. `-`→`_` and leading-dot tolerance.
- `app_dir` uses `{APP}_DIR` verbatim (final dot-dir, no dot appended), and an
  empty env value is treated as unset.
- `app_dir`/`repo_dot_dir` create ancestors (`create_dir_all`) and error cleanly
  on an unwritable path (no panic).
- `repo_dot_dir` Home/Cwd/Error policies when no repo root exists.
- `logs_dir`/`data_dir` append `logs`/`data` under an env-relocated root.

## 9. Risks / remaining blockers

- **API surface breadth** — `NoRoot` + `RepoMarker` + `_with` variants add a few
  knobs. Kept minimal and each has a clear default; the common path is
  `app_dir` / `repo_dot_dir` / `logs_dir` / `data_dir`.
- **`dirs` swap seam** — public `home_dir()` is the single swap point; keep the
  hand-rolled body unless cross-platform edge cases demand `dirs`.
- **`VS_DIR` naming collision** — if a consumer sets `VS_DIR` it would relocate
  the `vs` dot-dir; `vs-logging` deliberately uses `VS_LOG` above that seam.
- **`[workspace]` substring heuristic** — `Cargo.toml` "contains `[workspace]`"
  is a cheap text check (mad-rs style), not a `toml` parse; acceptable and avoids
  a new dep. Only used as a weaker fallback marker (below `.git`).
- **Relative env value** — absolutized against cwd; document that a relative
  `{APP}_DIR` is resolved relative to launch cwd.

## 10. Next steps

1. Branch/worktree `vfp/agent/plan/victory-dir`; add `libs/vs-dir` to `members`.
2. Implement `vs-dir` (§4) against the tremor-nodekit skeleton + waldo env
   semantics + mad-rs repo walk.
3. Port the repo-root tests and add the §8 cases.
4. Add a doc example resolving `.lil` with and without an env override (§5).
5. Optionally re-point `vs-logging::default_logs_dir()` at
   `vs_dir::logs_dir("vs", ".vs")` (§6).
6. Write `docs/designs/victory-dir.md` recording the name mapping +
   env/final-dot-dir + marker + fallback decisions.

## 11. Links

- Idea: `docs/ideas/victory-dir.idea.md`
- Prior plan: `docs/plan/victory-dir.plan.md`
- Related: `docs/plan/victory-logging.plan.md` (directory boundary, §6)
- Sources: `project-firefly/src_core/waldo/src/dir_utils.rs`;
  `agentbox/crates/tremor-nodekit/src/lib.rs`;
  `mad-rs/cursed/mad_common/src/mad_dir_utils.rs`;
  `agentbox/nodes/tremor-executor/src/project_context.rs`;
  `lil-hopps/lil-rerun/src/lib.rs`.
- Repo index: `docs/agents/repo-index.md`.
