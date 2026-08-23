# OPEN_RISKS

Risks and notable changes from the external dependency version bump pass.

All version bumps were applied as **version-string-only changes** in the crate
`Cargo.toml`s. No functional/code changes were required to keep `cargo build` and
`cargo test` green — the workspace compiles and all tests pass after the full
bump pass.

## Bumps applied

| crate | from      | to        | notes |
|-------|-----------|-----------|-------|
| serde | 1.0.210   | 1.0.229   | minor |
| log   | 0.4.22    | 0.4.34    | minor |
| tracing | 0.1.40  | 0.1.44    | minor |
| thiserror | 1.0.63 | 2.0.20  | **major** — `#[derive(Error, Debug)]` API compatible, no code changes |
| anyhow | 1.0.86    | 1.0.104   | minor |
| env_logger | 0.11.5 | 0.11.11 | minor |
| clap  | 4.5.2x     | 4.6.6     | minor |
| tokio | 1.0 x / *  | 1.53.1    | tightened loose `"1.0"`/`"*"` specs |
| rmp-serde | 1.3.0  | 1.3.1     | minor |
| rmp   | 0.8.14     | 0.8.15    | minor |
| rand  | 0.8.5      | 0.10.2    | **major** — `rand::random()` calls still compile, no code changes |
| serde_json | 1.0.132 | 1.0.151 | minor |
| bincode | 1.3.3    | 2.0.1     | **major** — see risk below |
| arrow | 52.2.0     | 59.2.0    | **major** — see risk below |
| futures | 0.3.31    | 0.3.34    | minor |
| test-log | 0.2.16    | 0.2.21    | minor |
| memuse | 0.2.1      | 0.2.2     | minor |
| tracing-subscriber | 0.3.18 | 0.3.23 | minor |
| tracing-tracy | 0.11.3 | 0.11.4 | minor |
| tracy_full | 1.10.0  | 1.13.0   | minor |
| sensible-env-logger | 0.3 | 0.3.2 | minor (dev-dep) |
| divan | 0.1.14     | 0.1.21    | minor (dev-dep) |

No-op (already at latest, left unchanged): `pretty_env_logger` (0.5.0),
`test-env-log` (0.2.8), `lazy_static` (1.5.0).

## Risks / things to watch

- **bincode 2.0.1 (not 3.0.0).** `bincode` 3.0.0 is a broken/non-building
  placeholder (it errors with the xkcd "Dependency" comic URL). Bumped to the
  latest *working* stable, **2.0.1**. Note the `use bincode::de;` import in
  `libs/vs-broker/src/broker/mod.rs` has a comment flagging it is currently
  **unused** — if bincode isn't actually needed, that import and dependency can be
  removed later.

- **arrow 52.2.0 → 59.2.0.** `arrow` is declared in `libs/vs-data-store/Cargo.toml`
  but is **not used anywhere in the code**. It only contributes build time /
  dependency footprint. No runtime impact. (If it stays unused, consider removing
  it.)

- **rand 0.8.5 → 0.10.2.** Major-version jump. The code only uses
  `rand::random()` / `rand::random::<u32>()`, which still exist in 0.10.2, so no
  code changes were needed. Future rand API drift (e.g. if `gen_range` /
  `thread_rng` are introduced) may require updates.

- **thiserror 1.0.63 → 2.0.20.** Major-version jump. The derive API used
  (`#[derive(thiserror::Error, Debug)]` + `#[error(...)]`) is compatible with 2.x;
  no code changes required.

- **tokio spec tightened.** `libs/vs-broker/Cargo.toml` was `version = "1.0"`
  (floats to latest 1.x) and `libs/vs-data-store/Cargo.toml` was `version = "*"`.
  Both now pin `1.53.1`. This removes the wildcard flexibility intentionally —
  future tokio updates will need an explicit bump.

- **No functional/code changes.** Every bump compiled and tested without touching
  source logic; the only source edit is a comment on the unused `bincode::de`
  import.

- **CI workflow cleanup.** `.github/workflows/rust-lib.yaml` was an unresolved
  merge-conflict leftover referencing a non-existent `victory-time-rs` crate
  (would have broken CI parsing). Rewritten to a clean, valid workflow that
  builds and tests the workspace at the repo root.

- **Workspace resolver.** Cargo prints a warning that the virtual workspace
  defaults to `resolver = "1"` even though workspace members are edition 2021.
  Consider adding `resolver = "2"` (or `"3"`) to the root `Cargo.toml` where
  appropriate.

## Repo organisation notes

- The three first-party crates were moved under `libs/` and renamed from the
  `victory-` prefix to the `vs-` prefix:
  - `vs-broker` (was `victory-broker`)
  - `vs-data-store` (was `victory-data-store`)
  - `vs-wtf` (was `victory-wtf`)
