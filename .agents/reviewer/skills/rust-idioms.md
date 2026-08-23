---
name: rust-idioms
description: Focused review lens for this repo's Rust crates. Load when the diff touches `libs/` `.rs` files.
---

# Reviewer lens: Rust idioms

This repo is a Cargo workspace of `vs-*` crates under `libs/`. Load this lens when the
diff touches `.rs` files and flag specific recoverable issues.

- Own explicit resources; prefer `RAII` and smart pointers over manual cleanup.
- Prefer `Result`/`anyhow`/`thiserror`; don't `unwrap()`/`expect()` on input that
  could legitimately be absent. On library boundaries, prefer returning an error.
- `unsafe` must carry a comment justifying the safety invariant.
- Don't `clone()` large buffers when a borrow or `Arc` is intended; but don't
  micro-optimize at the cost of clarity.
- Keep public API surface minimal; `pub(crate)` unless external consumers need it.
- Time/numeric conversions: avoid silent truncation; document rounding.
- New public API should be reflected in the crate README / doc comments.

Return only concrete `bug`/`style`/`suggestion` findings; if the Rust code is clean,
say nothing rather than padding.
