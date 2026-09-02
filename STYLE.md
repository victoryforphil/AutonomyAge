# Rust Style Guide

## Core rules

- Prefer simple, explicit code over clever abstractions.
- Keep the smallest correct diff; do not reshape unrelated code.
- Keep one obvious implementation path and remove dead alternatives.
- Use Rust 2021 and keep lines near 100 columns when practical.
- Follow existing names and error behavior when editing an established API.

## Module and type layout

- Keep `src/lib.rs` thin: module declarations and stable public re-exports.
- Organize related behavior into focused modules (`config`, `paths`, `roots`,
  `logging`, and similar domains).
- Prefer one primary struct, enum, or responsibility per file.
- Use directory modules with `mod.rs` when a domain has multiple child modules;
  keep leaf modules small and grep-friendly.
- Put shared runtime configuration in `config.rs` as an explicit config struct.
- Put behavior beside the type or domain that owns it; avoid kitchen-sink
  utility modules.
- Preserve a stable public API through `lib.rs` re-exports when internals move.

## APIs and errors

- Use `PascalCase` for types, `snake_case` for functions/modules, and
  `SCREAMING_SNAKE_CASE` for constants.
- Prefer borrowed inputs and slices where ownership is unnecessary.
- Use option/config structs when a shared API has multiple meaningful knobs.
- Use `anyhow::Result` with context for application/library glue; use
  `thiserror` only for a real domain error type.
- Propagate filesystem and I/O errors. Do not use `unwrap()` for runtime paths.
- Keep unsafe code minimal and document its safety invariant.

## Imports, docs, and tests

- Group standard-library imports before external crates and crate-local imports.
- Use `///` for public item documentation; explain constraints and why behavior
  exists rather than narrating obvious implementation details.

## Scenario tests

- Keep integration behavior covered by a small number of high-level scenario
  tests in the owning module's `tests.rs`.
- Extend an existing scenario as behavior grows; do not add a one-test-per-case
  list unless the behavior is genuinely independent.
- Keep tests near the module they exercise or in a focused `tests.rs` module.
- Test observable behavior, boundaries, error cases, and ordering—not private
  implementation shape.
- Run focused formatting, tests, and clippy checks for changed crates.
