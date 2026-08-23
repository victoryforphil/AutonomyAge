---
title: Victory Logging — Updated Plan (research complete)
type: plan
status: todo
tags:
  - plan
  - logging
  - tracing
  - research
---

# Victory Logging — Updated Plan

> **Status: research complete, decisions resolved.** This is an advance of
> `docs/plan/victory-logging.plan.md` (PR #36). It resolves the three open
> questions (crate name, facade, guard lifetime), fixes the test-init design,
> and produces a **concrete migration list** for `vs-broker` / `vs-data-store`.

## 1. Purpose (unchanged)

A reusable logging/tracing setup helper for the `vs-*` workspace that collapses
hand-rolled init into one library-shaped helper: pretty STDOUT, env-driven log
levels, timestamped file logging, and an extension seam — with a test-safe init.

Scope: **logging/tracing setup only.** Directory-locator duties stay split out of
`vs-logging` (see `victory-dir`; seam documented in §6).

## 2. Resolved decisions

### 2.1 Crate name → `vs-logging`

- Workspace members are `vs-*` (`vs-data-store`, `vs-wtf`, `vs-broker`; see
  `Cargo.toml` `members`). Naming the crate `victory-logging` would break the
  convention and be surprising next to `VS_LOG`/`VS_LOG_LEVEL` env vars.
- **Decision:** `vs-logging`, at `libs/vs-logging`, added to workspace `members`.
- `victory-logging` remains the *idea / PR* name. Record the mapping in a design
  note (`docs/designs/victory-logging.md`) so `victory-*`/`vs-*` naming is explicit.

### 2.2 Facade → `tracing` canonical, `log` passthrough via a `log` feature

- Both target crates already **mix** `log::*` for events with `tracing::*` for
  spans/instrumentation. Standardizing on `tracing` subsumes `log`.
- Mechanism (verified against `tracing-subscriber` 0.3.x docs): the `log`
  passthrough is provided by the **`tracing-log`** feature of `tracing-subscriber`.
  `SubscriberInitExt::{try_init,init,set_default}` install a `log` compatibility
  layer automatically **when `tracing-log` is enabled**, so `log::Record`s are
  consumed as `tracing` `Event`s. No separate logger wiring is needed.
- **Decision:** `vs-logging` builds the subscriber with `.try_init()` and ships a
  `log` feature that maps to `tracing-subscriber/tracing-log` (on by default).
  `log::*` call sites from third-party crates keep working with no code change.

### 2.3 Guard lifetime → keep `OnceLock<WorkerGuard>` internal (do NOT expose)

- The `tracing_appender::non_blocking` `WorkerGuard` must live for the process;
  a `static FILE_LOG_GUARD: OnceLock<WorkerGuard>` (tremor-nodekit) achieves this.
- **Decision:** keep the guard internal. Do **not** return it. This is the
  simplest reason-preserving design; a returned guard would make every caller
  responsible for keeping it alive and would fight the "set once" contract.
- Scoped/embedded loggers are explicitly out of scope for v1 (documented as
  Future). Any future `with_scoped_logger` would return a `DefaultGuard`, not the
  file writer guard.

### 2.4 Test-safe init → regular public fn, NOT `#[cfg(test)]`

- The prior plan marked `init_logging_test` with `#[cfg(test)]`. **This is wrong
  for consumption:** a `#[cfg(test)]` item in `vs-logging` is only compiled when
  *vs-logging's own* tests run — not when `vs-data-store`/`vs-broker` test modules
  call it. Downstream test code needs a normally-visible function.
- **Decision:** `init_logging_test` is a **public, always-compiled** function
  (document as "test-safe, for `#[cfg(test)]`/bench callers"). It is idempotent
  and race-free by calling the core init and **swallowing** the already-set
  `TryInitError`. It does not rely on `tracing::dispatcher::has_been_set()`
  (which would be racy across threads); instead it treats "already initialized"
  as success.

## 3. Source implementations (confirmed by re-reading raw sources)

| Repo | Path | Date | Key content (confirmed) |
|------|------|------|------------------------|
| agentbox | `crates/tremor-nodekit/src/lib.rs` | current | **Base.** `init_logger(filter, root, file)`, `init_logging()`, `logs_dir()`, `timestamped_log_file_name()`; env precedence `EnvFilter::try_from_default_env().or_else(try_new(filter)).unwrap_or(info)`; `OnceLock<WorkerGuard>` + `non_blocking`; stdio `.compact().without_time().with_target(false)`, file `.with_ansi(false).with_target(false).without_time().compact()`. No `clap`/`time` needed for the logging cores (timestamp computed from `SystemTime`). |
| dark-factory | `frontends/dark_cli/src/logging.rs` | current | **Graft A (confirmed).** `use_ansi = stdout().is_terminal() && NO_COLOR.is_none()`; `RUST_LOG`-derived `EnvFilter`; `BoxMakeWriter` + `Arc<Mutex<File>>` `SharedFileWriter`; path override `DARK_CLI_LOG` (generalize → `VS_LOG`) with parent-dir creation + cwd/.darkfactory/logs fallback; `Registry::default().with(filter).with(stdout).with(file).try_init()`. |
| tinyverse | `tinyverse_cli/src/logging.rs` | current | **Graft B (confirmed).** `InitOptions { stdout_enabled, default_level }`; custom level env var `RUST_INFO` (generalize → `VS_LOG_LEVEL`); `NO_COLOR`; custom `FormatEvent`/`FormatFields` (`FancyFormat`, `EventMessageVisitor`) for panel/state rendering; `stdout_enabled` gating via `filter_fn`. |

**Confirmed finding — tinyverse file layer uses `with_file/with_line_number/with_thread_ids/with_thread_names`; dark-cli does not.** Recommend defaults: rich file layer (target + file + line + thread) to match tinyverse; keep `compact().without_time().with_ansi(false)` for both layers to preserve tremor neutrality.

## 4. Final `vs-logging` API (concrete)

### 4.1 Constants

```rust
pub const VS_LOG_LEVEL: &str = "VS_LOG_LEVEL";   // custom level override (defaults to default_level)
pub const VS_LOG: &str = "VS_LOG";               // per-run log file path override
pub const DEFAULT_LOG_LEVEL: &str = "info";      // fallback when nothing else set
```

### 4.2 Directory locators

```rust
/// ~/.vs/logs (unix: $HOME/.vs/logs; non-unix: %USERPROFILE%\.vs\logs). Creates it.
pub fn default_logs_dir() -> anyhow::Result<PathBuf>;

/// Resolve the logs dir: if VS_LOG is set, its parent (creating parents);
/// otherwise default_logs_dir(). Creates it.
pub fn logs_dir() -> anyhow::Result<PathBuf>;
```

`logs_dir()` semantics: if `VS_LOG` is a non-empty path, use `path.parent()`
(or `.` when it has no parent — i.e. a bare filename in cwd). This generalizes
dark-cli's `DARK_CLI_LOG` override.

### 4.3 `InitOptions`

```rust
#[derive(Clone, Debug)]
pub struct InitOptions {
    pub stdout_enabled: bool,       // if false, don't install the STDOUT layer
    pub default_level: &'static str,// `VS_LOG_LEVEL` -> `RUST_LOG` -> this -> "info"
    pub pretty_stdout: bool,        // `.pretty()` instead of `.compact()`; default false
}

impl InitOptions {
    pub fn cli_default() -> Self { Self { stdout_enabled: true, default_level: "info", pretty_stdout: false } }
    pub fn tui_mode() -> Self { Self { stdout_enabled: false, default_level: "info", pretty_stdout: true } }
}
impl Default for InitOptions { fn default() -> Self { Self::cli_default() } }
```

Field/behavior detail:
- `stdout_enabled`: when `false`, the stdout `fmt` layer is **not added** (not
  filter-gated). File logging still happens.
- `default_level`: `&'static str` (keeps `EnvFilter::try_new` simple; callers pass
  a literal). Used only when neither `RUST_LOG` nor `VS_LOG_LEVEL` is set.
- `pretty_stdout`: `true` → `.pretty()`; `false` → `.compact().without_time()`.
  **Caveat to verify at implementation:** the `pretty` formatter may render a
  timestamp regardless of `.without_time()`; keep the default `false` for the
  migration so nothing changes, and confirm pretty+without_time behavior when
  enabling it.

### 4.4 Init entry points

```rust
/// Convenience: resolves write via logs_dir(), uses VS_LOG_LEVEL/RUST_LOG/default,
/// writes an `app-{timestamp}.log` file under the resolved dir.
/// Executes the filter: RUST_LOG > VS_LOG_LEVEL > options.default_level > "info".
pub fn init_logging(options: InitOptions) -> anyhow::Result<()>;

/// Generic base (tremor-nodekit shape): caller supplies filter string, root dir,
/// and base file name. RUST_LOG still wins over log_filter.
pub fn init_logger(log_filter: &str, logs_root: &Path, log_file_name: &str) -> anyhow::Result<()>;

/// Timestamped file name `{stem}-{secs}-{millis:03}.{ext}` (SystemTime; no time crate).
pub fn timestamped_log_file_name(log_file_name: &str) -> String;

/// Test-safe, idempotent init. Public (NOT #[cfg(test)]). Only installs a
/// subscriber if none is set; swallows TryInitError/AlreadySet. Used by
/// downstream #[cfg(test)]/bench modules in place of sensible_env_logger::safe_init!().
pub fn init_logging_test(options: InitOptions);
```

Behavior shared by `init_logging`/`init_logger` (derived from tremor):
1. Filter: `EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(level)).unwrap_or_else(|_| EnvFilter::new("info"))`.
2. Create `logs_root` (parent of `VS_LOG`, or `~/.vs/logs`).
3. If `VS_LOG` set and non-empty → `rolling::never(parent, file_name_of(VS_LOG))`;
   else `rolling::never(logs_root, timestamped_log_file_name(base))`.
4. `let (writer, guard) = tracing_appender::non_blocking(appender); let _ = FILE_LOG_GUARD.set(guard);`
5. Build `registry().with(env_filter).with(stdout?).with(file?)` and `.try_init()`.
   - stdout layer (if `stdout_enabled`): `fmt::layer().with_target(false).without_time()` + (`.pretty()` or `.compact()`) + `.with_ansi(use_ansi)`; `use_ansi = stdout_enabled && stdout().is_terminal() && NO_COLOR.is_none()`.
   - file layer: `fmt::layer().with_target(true).with_file(true).with_line_number(true).with_thread_ids(true).with_thread_names(true).with_ansi(false).with_writer(writer)`.
6. `tracy` feature: optionally `.with(tracing_tracy::TracyLayer::new())`.

### 4.5 Feature flags (`libs/vs-logging/Cargo.toml`)

```toml
[features]
default = ["log"]
log   = ["tracing-subscriber/tracing-log"]   # log -> tracing bridge (SubscriberInitExt installs LogTracer)
tracy = ["dep:tracing-tracy"]                # optional profiling layer

[dependencies]
anyhow = "1.0.104"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter", "fmt", "ansi", "tracing-log", "registry", "std"] }
tracing-appender = "0.2.3"                   # required (non_blocking file writer)
```

`tracing-appender` stays a required (non-optional) dep — file logging is a v1
feature. `time`/`clap` are **not** deps (timestamped file name uses `SystemTime`).

## 5. Extension seams (kept from prior plan, now concrete)

- **Format seam (tinyverse graft):** `InitOptions { pretty_stdout }` is the
  simplest knob. Full `FormatEvent`/`FormatFields` pluggability (FancyFormat /
  EventMessageVisitor) is deferred behind a `fmt` feature using an `Arc<dyn ...>`
  for the event formatter; not required for the migration.
- **Layer composition (tracy):** because `init_logging*` calls `try_init()` on a
  registry it owns, consumers cannot add layers to the *same* subscriber after
  init. Resolved by feature-gating the one currently-needed consumer layer
  (`tracy`). General closure/`IntoIterator<Layer>` composition is Future (generics
  make it awkward in a boxed registry).
- **Dir boundary:** if `victory-dir` (`vs-dir`) lands first, `vs-logging` calls
  `vs_dir::logs_dir()` instead of computing `~/.vs/logs` itself. Seam: keep
  `default_logs_dir()` as the single place to swap.

## 6. CONCRETE MIGRATION LIST

### 6.1 `libs/vs-data-store`

Source call sites:
- `src/main.rs:1` `use log::info;` → `use tracing::info;`
- `src/main.rs:40` `env_logger::init();` → `vs_logging::init_logging(vs_logging::InitOptions::default()).expect("init vs-logging");`
- `src/buckets/mod.rs:6` `use log::{debug, info, trace, warn};` → `use tracing::{debug, info, trace, warn};` (module already uses `#[tracing::instrument]`)
- `src/database/mod.rs:11` `use log::{debug, trace, warn};` → `use tracing::{debug, trace, warn};` (module already uses `tracing::debug_span`/`info_span`/`instrument`)
- `src/database/view.rs:10` `use log::{debug, warn};` → `use tracing::{debug, warn};`
- `src/primitives/serde/deserializer.rs:1` `use log::trace;` → `use tracing::trace;` (module already uses `tracing::instrument`)
- `src/primitives/serde/mod.rs:10` (test module) `use log::trace;` → `use tracing::trace;`
- `src/primitives/serde/serialize.rs:3` already `use tracing::instrument;` (no change)

Test init sites (replace `sensible_env_logger::safe_init!()`):
- `src/primitives/serde/serialize.rs:816`
- `src/primitives/serde/mod.rs:60`
- `src/primitives/serde/deserializer.rs:650`
- `src/database/view.rs:329`
- `src/database/mod.rs:567` and `:617`
- Each → `vs_logging::init_logging_test(vs_logging::InitOptions::default());`

Cargo.toml (deps):
- Remove `env_logger = "0.11.11"` (replaced by vs-logging).
- Remove `pretty_env_logger = "0.5.0"` (declared but **unused**).
- Remove `tracing-subscriber = "0.3.23"` (declared but **unused**; moves to vs-logging).
- Remove `sensible-env-logger = "0.3.2"` from `[dev-dependencies]` (replaced by init_logging_test).
- Move `tracing-tracy` + `tracy_full` behind `vs-logging/tracy` feature (currently
  declared but **no `TracyLayer` is ever registered** — latent; keep only if tracy
  profiling is planned, else prune).
- Remove direct `log = "0.4.34"` (all `log::*` calls migrated to `tracing::*`;
  `tracing` is already a dep).
- Add `vs-logging = { path = "../vs-logging" }`.

### 6.2 `libs/vs-broker`

Binaries (init + event macros):
- `bin/broker_tcp_server.rs:6` `use log::{info, warn};` → `use tracing::{info, warn};`
- `bin/broker_tcp_server.rs:28` `pretty_env_logger::init();` → `vs_logging::init_logging(vs_logging::InitOptions::default()).expect("init vs-logging");`
- `bin/broker_tcp_client.rs:7` `use log::info;` → `use tracing::info;`
- `bin/broker_tcp_client.rs:29` `pretty_env_logger::init();` → `vs_logging::init_logging(...).expect(...)`

Library event macros (`log::*` → `tracing::*`):
- `src/broker/mod.rs:7` `use log::{debug, info, trace, warn};` → `use tracing::{debug, info, trace, warn};` (already imports `tracing::instrument`; the module also uses `tracing::span!`)
- `src/broker/time.rs:1` `use log::info;` → `use tracing::info;` (module already uses `tracing::trace`)
- `src/adapters/tcp/tcp_server.rs:1` `use log::{debug, info, warn};` → `use tracing::{debug, info, warn};`
- `src/adapters/tcp/tcp_client.rs:3` `use log::info;` → `use tracing::info;`
- `src/adapters/tcp/connection.rs:3` `use log::{info, warn};` → `use tracing::{info, warn};`
- `src/adapters/channel/mod.rs:4` `use log::warn;` → `use tracing::warn;`
- `src/commander/mock.rs:1` `use log::{debug, info};` → `use tracing::{debug, info};`
- `src/commander/linear.rs:1` `use log::info;` → `use tracing::info;`
- `src/node/mod.rs:4` `use log::{debug, info};` → `use tracing::{debug, info};`
- `src/node/info.rs:2` `use log::debug;` → `use tracing::debug;`

Test macro (`test_env_log::test`):
- `src/lib.rs:22`, `src/broker/mod.rs:429` — these are **test-only** per-test log
  capture. Keep `test-env-log` (used); it is a dev-dependency, not a redundant
  runtime logging dep. If per-test capture is replaced, call
  `vs_logging::init_logging_test(...)` in each `#[test]` instead. Do **not**
  mix `test_env_log::test` (which sets its own global subscriber) with a
  process-global `init_logging_test` in the same binary.

Cargo.toml (deps):
- Remove `pretty_env_logger = "0.5.0"` (replaced by vs-logging).
- Remove `env_logger = "0.11.11"` (declared but **unused**; only `pretty_env_logger` was used).
- Remove `test-log = "0.2.21"` (**unused**; only `test-env-log` is used).
- Remove direct `log = "0.4.34"` after `log::*` → `tracing::*` migration
  (transitive `log` still flows via `tracing-subscriber/tracing-log`).
- Keep `tracing` (already a dep). Keep `test-env-log` (used).
- Add `vs-logging = { path = "../vs-logging" }`.

### 6.3 Cross-cutting

- Workspace `Cargo.toml`: add `"libs/vs-logging"` to `members`.

## 7. Feature mapping (updated)

| Feature | Status | Implementation |
|---------|--------|----------------|
| Pretty STDOUT | Now | `compact().without_time()` default; `.pretty()` via `InitOptions::pretty_stdout`; ANSI auto-disabled when not a TTY or `NO_COLOR` set. |
| ENV log levels | Now | `RUST_LOG` > `VS_LOG_LEVEL` > `default_level` > `"info"`. |
| File logging | Now | `rolling::never` + `non_blocking` + `WorkerGuard` in `OnceLock`; timestamped filename; `VS_LOG` path override. |
| `log` passthrough | Now | `log` feature → `tracing-subscriber/tracing-log`; `try_init` installs `LogTracer`. |
| Extensions | Now (seam) | `InitOptions`; feature-gated `tracy` layer. |
| Pluggable `FormatEvent`/`FormatFields` | Future | `fmt` feature + `Arc<dyn>` formatter. |
| Rerun / state-change logging | Future | Custom `FormatEvent`/`FormatFields` + rate-limiting layer. |
| Scoped/embedded logger | Future | Return a `DefaultGuard` (not the file-writer guard). |

## 8. Risks / remaining blockers

- **`pretty` + `without_time` interaction** — verify at implementation; default off.
- **`VS_LOG` override edge cases** — bare filename (no parent) → use cwd as the
  rolling dir; empty value → ignore override.
- **Bad `RUST_LOG`/`VS_LOG_LEVEL` values** — `EnvFilter::try_new` may fail; the
  fallback chain `try_from_default_env().or_else(try_new(level)).unwrap_or(info)`
  already degrades gracefully. `VS_LOG_LEVEL` must feed into the same `try_new`.
- **`test_env_log::test` vs global `init_logging_test`** — do not mix in one
  binary (both want to own the global subscriber).
- **tracy is latent** — no `TracyLayer` is registered today; keep it behind the
  feature or prune.
- **`vs-dir` ordering** — if `vs-dir` lands first, swap `default_logs_dir()` body.

## 9. Next steps

1. Branch/worktree `vfp/agent/plan/victory-logging`; add `libs/vs-logging` to `members`.
2. Implement `vs-logging` (§4) from the tremor base + dark-cli/tinyverse grafts.
3. Add `init_logging_test` + an integration test asserting double-init does not panic.
4. Migrate `vs-data-store` (§6.1) and `vs-broker` (§6.2).
5. Prune redundant deps; `cargo build`/`cargo test`.
6. Port remaining tinyverse `FancyFormat`/`EventMessageVisitor` (if the panel
   rendering is wanted) behind the `fmt` feature.
7. Write `docs/designs/victory-logging.md` recording name mapping + facade decision.

## 10. Links

- Idea: `docs/ideas/victory-logging.idea.md`
- Prior plan: `docs/plan/victory-logging.plan.md`
- Related: `victory-dir.plan.md` (naming + directory boundary)
- Sources: `agentbox/crates/tremor-nodekit/src/lib.rs`; dark-factory `frontends/dark_cli/src/logging.rs`; tinyverse `tinyverse_cli/src/logging.rs` (GitHub raw).
