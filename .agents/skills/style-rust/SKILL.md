---
name: style-rust
description: Rust module layout, API, error handling, and test conventions for AutonomyAge.
---

# Rust Style

Apply this skill to Rust source changes in AutonomyAge.

## Baseline

- Favor readable, explicit code over clever shortcuts.
- Keep diffs scoped and remove dead or duplicate paths.
- Use Rust 2021, `snake_case` modules/functions, `PascalCase` types, and
  `SCREAMING_SNAKE_CASE` constants.
- Prefer borrowed inputs and avoid allocations in hot paths.

## Module layout

- Keep `src/lib.rs` thin: declarations plus stable public re-exports.
- Split domains into focused modules such as `config.rs`, `paths.rs`, and
  `roots.rs`.
- Keep one primary type or responsibility per file.
- When a domain grows, use `src/domain/mod.rs` with focused child modules.
- Put shared runtime configuration in an explicit config struct in `config.rs`.
- Keep implementation beside the type or domain that owns it; avoid a generic
  kitchen-sink utility module.
- Preserve public behavior and re-exports when moving internals.

## APIs and errors

- Use option/config structs for APIs with multiple meaningful settings.
- Use `anyhow::Result` plus context for application and filesystem glue.
- Introduce `thiserror` only when callers need a real domain error enum.
- Propagate I/O errors; do not panic on runtime paths or configuration.
- Avoid unsafe code. When required, document the safety invariant at the item.

## Scenario tests

- Keep integration behavior in a small number of high-level scenario tests in
  the owning module's `tests.rs`.
- Extend an existing scenario as behavior grows; avoid one-test-per-case lists
  unless the behavior is genuinely independent.

## Tests and checks

- Keep tests close to the module or in a focused `tests.rs` module.
- Defend observable behavior, ordering, boundaries, and error handling.
- Group standard-library imports before external and crate-local imports.
- Document public constraints with `///` comments and explain why when useful.
- Run focused `cargo fmt`, `cargo test`, and `cargo clippy -- -D warnings` checks.
