---
title: Victory Logging
type: plan
status: todo
tags:
  - plan
  - logging
---

# Victory Logging — Plan

A reusable **logging/tracing setup helper crate** for the AutonomyAge `vs-*` Rust
workspace. Every current `vs-*` crate hand-rolls its own logging init (or ships no
file logging at all); this crate collapses that into one consistent,
library-shaped helper: pretty STDOUT, env-driven log levels, timestamped file
logging, and an extension seam — with a test-safe init.

## 1. Purpose

Today each `vs-*` binary rolls its own logging setup:

- `vs-data-store/src/main.rs` → `env_logger::init()` (no file logging, no pretty output).
- `vs-broker/bin/broker_tcp_{server,client}.rs` → `pretty_env_logger::init()`.
- Tests across `vs-data-store` → `sensible_env_logger::safe_init!()`.

None write to a file, none share a log-level env contract, and dependencies are
pulled in redundantly (`vs-data-store` and `vs-broker` both depend on `env_logger`
**and** `pretty_env_logger` **and** `tracing-subscriber`). The helper should give
every `vs-*` crate one way to:

- Emit **pretty, timestamp-free compact output to STDOUT** (ANSI-aware).
- Respect **environment-variable log levels** (`RUST_LOG`, plus a workspace custom var).
- Write **timestamped/rotating logs to a file** under a canonical logs dir, with a runtime override.
- Be **extensible** (custom `FormatEvent`/`FormatFields`, future state-change/panel hooks).
- Be **test-safe** — a `#[cfg(test)]` init that never tears down the global subscriber twice.

Scope: **logging/tracing setup only.** Directory-locator duties are split out into `vs-dir`.

## 2. Source implementations found

| Repo | Path | Date | Notes |
|------|------|------|-------|
| agentbox | `crates/tremor-nodekit/src/lib.rs` | current | **Best base.** Library-shaped `init_logger(log_filter, logs_root, log_file_name) -> anyhow::Result<()>`, `init_logging()`, `timestamped_log_file_name()`, `logs_dir()`. Env priority `try_from_default_env().or_else(try_new(filter)).unwrap_or(EnvFilter::new("info"))`; guarded `tracing_appender::rolling::never` + `non_blocking` + `WorkerGuard` in `OnceLock`; stdio `.compact().without_time().with_target(false)`, file `.with_ansi(false).with_writer(...)`. |
| agentbox | `labs/hello_rig/src/logging.rs`, `labs/hello_zenoh/src/logging.rs` | current | Sibling near-duplicates — same shape, **no** `try_from_default_env`. |
| mad-rs | `cursed/mad_common/src/logging.rs` | current | env_filter + compact stdout + rolling file + `OnceLock`. Independent confirmation of the idiom. |
| dark-factory | `frontends/dark_cli/src/logging.rs` | current | **Graft A.** TTY/`NO_COLOR` ANSI auto-detect (`use_ansi = stdout().is_terminal() && NO_COLOR.is_none()`); per-run log-path ENV override (generalize `DARK_CLI_LOG`→`VS_LOG`); `BoxMakeWriter` + `Arc<Mutex<File>>` `SharedFileWriter`; `.pretty()`. |
| tinyverse | `tinyverse_cli/src/logging.rs` | current | **Graft B.** `InitOptions { stdout_enabled, default_level }`; custom level ENV var (generalize `RUST_INFO`→`VS_LOG_LEVEL`); custom `FormatEvent`/`FormatFields` for state-change/panel rendering. |

## 3. Best version to port

Port the **tremor-nodekit** base as the skeleton, with two grafts:

- **Base (tremor-nodekit)**: generic `init_logger(log_filter, logs_root, log_file_name)`; the env precedence chain; the `OnceLock<WorkerGuard>` non-blocking file appender; compact-stdout + ANSI-less-file dual-layer `registry().with(...)`.
- **Graft A (dark-cli)**: ANSI auto-detect; a `VS_LOG` runtime override of the log file path; optional `SharedFileWriter`.
- **Graft B (tinyverse)**: public `InitOptions { stdout_enabled, default_level }`; a `VS_LOG_LEVEL` env var; the `FormatEvent`/`FormatFields` extension seam.

Keep tremor's **library shape** (generic signature, `anyhow::Result`, no hard `clap`/`time` deps); downstream crates keep `registry().with(layer)` to add their own layers (e.g. `tracy`).

## 4. Proposed API surface

Crate name: **`vs-logging`** (default), at `libs/vs-logging` (naming decision in §6).

```rust
pub const VS_LOG_LEVEL: &str = "VS_LOG_LEVEL";
pub const VS_LOG: &str = "VS_LOG";

pub fn default_logs_dir() -> anyhow::Result<PathBuf>;   // ~/.vs/logs
pub fn logs_dir() -> anyhow::Result<PathBuf>;           // VS_LOG parent, else default

pub fn init_logging(options: InitOptions) -> anyhow::Result<()>;   // convenience
pub fn init_logger(log_filter: &str, logs_root: &Path, log_file_name: &str) -> anyhow::Result<()>;

#[derive(Clone, Debug)]
pub struct InitOptions { pub stdout_enabled: bool, pub default_level: &'static str, pub pretty_stdout: bool }
impl Default for InitOptions { fn default() -> Self { Self { stdout_enabled: true, default_level: "info", pretty_stdout: false } } }

#[cfg(test)]
pub fn init_logging_test(options: InitOptions);
```

**Test-safe init**: `init_logging_test` only initializes if the global subscriber is unset (via `tracing::dispatcher::has_been_set()`), so existing `sensible_env_logger::safe_init!()` sites can migrate to `vs_logging::init_logging_test(...)`.

**Facade — prefer `tracing`**: `vs-data-store` is `tracing`-instrumented; `vs-broker` calls `log::*`. Standardize on `tracing` (it subsumes `log` via the `tracing-subscriber` `log` feature). Migrate `vs-broker`'s `log::*` to `tracing::*`; keep `log` passthrough for third-party `log`-based crates.

## 5. Feature mapping

| Feature | Status | Implementation |
|---------|--------|----------------|
| Pretty STDOUT | Now | `fmt::layer().with_target(false).without_time().compact()`; `.pretty()` via `InitOptions::pretty_stdout`; ANSI auto-disabled when not a TTY or `NO_COLOR` set (dark-cli graft). |
| ENV log levels | Now | `RUST_LOG` > `VS_LOG_LEVEL` > `default_level` > `"info"`. |
| File logging | Now | `tracing_appender::rolling::never` + `non_blocking` + `WorkerGuard` in `OnceLock`; timestamped filename; `VS_LOG` path override; optional `SharedFileWriter`. |
| Extensions | Now (seam) | `InitOptions`; pluggable `FormatEvent`/`FormatFields`; `registry().with(...)` lets consumers add layers. |
| Rerun logging / plugin support | Future | Optional `tracing` layer forwarding events/spans to Rerun. |
| Rate-limited / state-change logging (ROS LOG style) | Future | Custom `FormatEvent`/`FormatFields` + a rate-limiting/state-change layer. |

## 6. Risks / open questions

- **Facade mixing (`log` vs `tracing`)**: `vs-broker` uses `log::*`, `vs-data-store` is `tracing`-instrumented. Standardizing on `tracing` needs a small migration.
- **Guard lifetime**: `WorkerGuard` must live for the whole process; `OnceLock` handles this. Open: whether to expose the guard for scoped/embedded loggers.
- **ANSI / TTY detection**: graft `stdout().is_terminal() && NO_COLOR.is_none()`; ensure piped/CI/headless environments emit no escape codes.
- **Crate naming `victory-logging` vs `vs-logging`**: workspace uses `vs-*`; recommend `vs-logging` (matches `VS_LOG`/`VS_LOG_LEVEL`). Record the mapping.
- **Consolidate redundant deps**: prune `env_logger`/`pretty_env_logger`/test loggers from `vs-data-store`/`vs-broker`; move `tracing-appender`/`tracing-subscriber` behind `vs-logging`. Confirm a `tracy` layer composes with the shared subscriber.
- **`OnceLock`/global-subscriber constraint**: `tracing`'s global subscriber can only be set once; document init as idempotent-by-contract; keep the test-safe wrapper.
- **`vs-dir` interdependency**: if `vs-dir` lands first, `vs-logging` should call `vs_dir::logs_dir()` rather than hardcode `~/.vs/logs`.

## 7. Next steps

1. Decide the crate name (`vs-logging` recommended) and add `libs/vs-logging` to the workspace `members`.
2. Port the tremor-nodekit base (`init_logger`, `init_logging`, `timestamped_log_file_name`, `logs_dir`).
3. Graft dark-cli ANSI/TTY + `NO_COLOR` + `VS_LOG` override + `SharedFileWriter`.
4. Graft tinyverse `InitOptions`, `VS_LOG_LEVEL`, and the `FormatEvent`/`FormatFields` seam.
5. Add `init_logging_test` + an integration test that init runs once (and doesn't panic on a second call).
6. Migrate `vs-broker` (`log::*`→`tracing::*`; `pretty_env_logger::init()` → `vs_logging::init_logging(...)`).
7. Migrate `vs-data-store` (`env_logger::init()` + `sensible_env_logger` test sites → `vs_logging`).
8. Prune redundant logging deps.
9. Write a design note in `docs/designs/`.

## 8. Links

- Idea: [`docs/ideas/victory-logging.idea.md`](../ideas/victory-logging.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
- Related: `victory-dir.plan.md` (naming + directory boundary)
