---
title: Victory Dir — Plan
type: plan
status: active
tags:
  - plan
  - dir
---

# Victory Dir — Plan

A small reusable directory locator for the AutonomyAge `vs-*` workspace.
It follows the existing source implementations instead of introducing a
config/resource framework:

```text
configured environment base/.dot
→ current working directory/.dot
→ Git repository root/.dot
→ Cargo crate root/.dot
→ user home/.dot
```

The first existing candidate wins. The selected directory is created on demand.
`logs` and `data` are children of that same selected directory, so they follow
the exact same resolution order.

Scope: path resolution and directory creation only. Configuration parsing,
prompt loading, logging setup, and artifact lifecycle remain consumer concerns.

## 1. Decisions

### 1.1 Crate name

Create `libs/vs-dir` and add it to the workspace. The workspace uses `vs-*`
names; `victory-dir` remains the plan and PR label.

### 1.2 Keep the source-shaped API

Use one configuration struct and one locator object. Avoid a collection of
free functions, marker enums, fallback-policy enums, or separate global and
project directory APIs.

```rust
let locator = DirLocator::new(
    DirConfig::new(".lil").with_env("LIL_DIR"),
    Path::new("."),
)?;

let logs = locator.logs_dir()?;
```

`DirLocator` owns the configured dot name and launch cwd. Its member methods are
`app_dir`, `logs_dir`, `data_dir`, and `ensure_dir`. Root detection is an
internal detail of `app_dir`; callers do not select a root through a second API.

### 1.3 Environment override

`DirConfig::with_env` accepts one explicit environment variable name. There is
no automatic app-name-to-environment-name transformation and no scan of an
`ENV_*` namespace.

The configured value is a **base directory** and the locator always appends the
dot component. For example:

- `LIL_DIR=/var/lib/lil` → `/var/lib/lil/.lil`;
- empty or unset `LIL_DIR` → continue to cwd, Git root, Cargo crate root, home;
- relative values are resolved against the launch cwd.

This keeps one meaning for the variable and preserves the requested project-dot
behavior. It is deliberately simpler than supporting both final-path and
parent-base override modes.

### 1.4 Resolution order

`app_dir()` checks these candidates in order:

1. configured environment base + dot, if configured and non-empty;
2. `cwd/.dot`;
3. nearest Git root + dot;
4. nearest Cargo crate root + dot, identified by a `[package]` manifest;
5. `home_dir()/.dot`.

Candidates two through four are used only when the dot directory already exists.
The home candidate is created when no project candidate exists. This avoids
creating hidden directories in every ancestor while retaining the direct waldo,
mad-rs, and tremor-nodekit behavior.

A Cargo workspace root is not a separate default scope. The nearest package
crate root is the useful Rust boundary here; workspace-specific behavior can be
added only when a consumer needs it.

### 1.5 Home and errors

`DirLocator::home_dir()` reads `HOME`, then `USERPROFILE`. The crate has no
`dirs` dependency. Filesystem errors use `anyhow::Result` with context. Runtime
paths never use `unwrap()` or panic-on-create behavior.

## 2. Source direction

| Source | Pattern retained |
|---|---|
| project-firefly waldo | direct app-specific env override and cwd dot directory |
| agentbox tremor-nodekit | hand-rolled home resolution plus logs/data children |
| mad-rs | ancestor Git/repository discovery |
| Pi resource loader | global scope plus cwd-oriented project context |
| OpenCode config paths | project paths derive from the active cwd |

The Pi study records global plus cwd/ancestor context loading in
`/Users/alex/repos/vfp/agentbox/docs/research/pi-coding-agent.md:142-158`.
The OpenCode study records upward project discovery in
`/Users/alex/repos/vfp/agentbox/docs/research/opencode.md:75-86`.

## 3. Module layout

Keep the crate organized by responsibility:

```text
libs/vs-dir/src/
├── lib.rs                 # module declarations and public re-exports
├── config.rs              # DirConfig
└── path/
    ├── mod.rs             # path-domain module and DirLocator export
    ├── locator.rs         # resolution and member methods
    ├── utils.rs           # shared lexical path helpers
    └── tests.rs           # high-level scenario tests
```

`lib.rs` stays thin. Do not reintroduce top-level `paths.rs`, `roots.rs`, or a
free-function API when the path domain grows.

## 4. Proposed API

### 4.1 `libs/vs-dir/Cargo.toml`

```toml
[package]
name = "vs-dir"
version = "0.0.1"
edition = "2021"

[dependencies]
anyhow = "1.0.104"
```

No other dependencies.

### 4.2 Configuration and locator

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Clone, Copy, Debug)]
pub struct DirConfig<'a> {
    pub dot: &'a str,
    pub env_var: Option<&'a str>,
}

impl<'a> DirConfig<'a> {
    pub fn new(dot: &'a str) -> Self;
    pub fn with_env(self, variable: &'a str) -> Self;
}

#[derive(Clone, Debug)]
pub struct DirLocator<'a> {
    config: DirConfig<'a>,
    cwd: PathBuf,
}

impl<'a> DirLocator<'a> {
    pub fn new(config: DirConfig<'a>, cwd: &Path) -> Result<Self>;
    pub fn home_dir() -> Result<PathBuf>;
    pub fn app_dir(&self) -> Result<PathBuf>;
    pub fn logs_dir(&self) -> Result<PathBuf>;
    pub fn data_dir(&self) -> Result<PathBuf>;
    pub fn ensure_dir(&self, path: &Path) -> Result<PathBuf>;
}
```

All root walking, marker checks, and candidate selection remain private
implementation details of `DirLocator`.

## 5. Usage example

```rust
use std::path::Path;
use anyhow::Result;
use vs_dir::{DirConfig, DirLocator};

fn main() -> Result<()> {
    let config = DirConfig::new(".lil").with_env("LIL_DIR");
    let locator = DirLocator::new(config, Path::new("."))?;

    let app = locator.app_dir()?;
    let logs = locator.logs_dir()?;
    let data = locator.data_dir()?;

    println!(
        "app={} logs={} data={}",
        app.display(),
        logs.display(),
        data.display()
    );
    Ok(())
}
```

## 6. Tests

Keep tests in `src/path/tests.rs` as a small set of high-level scenarios.
Extend scenarios as behavior grows instead of adding one test per edge case:

- resolution-order scenario: cwd, Git root, Cargo crate root, and home fallback;
- environment scenario: explicit base, dot append, empty value fallback;
- subdirectory scenario: logs/data under the selected app directory;
- ensure-directory scenario: nested creation and error propagation.

Tests should use temporary directories and environment isolation. They should
assert observable selected paths and creation behavior, not private helper
structure.

## 7. Integration and risks

- Add `libs/vs-dir` to the workspace `Cargo.toml` members.
- Keep the runnable example in `libs/vs-dir/examples/locator.rs` and document
  the same API in `libs/vs-dir/README.md`.
- Record the stable boundary in `docs/designs/victory-dir.design.md`.
- Keep `vs-logging`'s `VS_LOG` file-path override separate; its directory
  default can later call `DirLocator` without inheriting unrelated settings.
- Do not add a separate Cargo workspace-root mode until a consumer needs it.
- Do not create project dot directories merely while checking candidates.
- Keep the environment variable explicit and base-directory-only.

## Links

- Idea: `docs/ideas/victory-dir.idea.md`
- Task: `docs/tasks/victory-dir.task.md`
- Pi study: `/Users/alex/repos/vfp/agentbox/docs/research/pi-coding-agent.md`
- OpenCode study: `/Users/alex/repos/vfp/agentbox/docs/research/opencode.md`
