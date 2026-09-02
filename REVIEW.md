# REVIEW.md — Repo Review Rules

This is the **single source of truth** for what the `reviewer` agent checks on every
pull request. It is plain markdown on purpose: edit it, the bot re-reads it each run.

The reviewer agent (`REVIEWING AGENT` in `.agents/reviewer/SKILL.md`) applies these
rules to the PR diff and returns structured findings. See
[`.agents/reviewer/README.md`](.agents/reviewer/README.md) for how this is wired.

## Severity scale

Findings are tagged with one severity, in decreasing priority:

| severity    | meaning                                                        |
|-------------|----------------------------------------------------------------|
| `bug`       | Will cause incorrect behavior, a crash, or a data-loss bug.    |
| `style`     | Violates an idiomatic/readability convention; not behavior.    |
| `suggestion`| Optional improvement, nice-to-have, or a question.            |

## What to check

Review **the diff**, plus the surrounding code needed to judge it. Be concrete: cite
`file:line` and quote the offending code. Do **not** flag pre-existing code that the
PR didn't touch, unless the PR interacts with it.

### Correctness
- Behavior matches the commit/PR intent; no silent fall-through or ignored errors.
- No off-by-one, wrong comparator, or inverted boolean.
- Changes to shared/multi-threaded state are safe (locking, atomicity, ordering).
- No assumption that a value is non-null/non-empty without a guard (Rust: `unwrap` /
  `expect` on externally-controlled input is a `bug`).
- Numeric/time conversions use checked or documented rounding; no silent truncation.

### Rust idiomatic & ownership
- Prefer types over raw buffers; prefer newtype wrappers for distinct units.
- Use `Result`/`anyhow`/`thiserror` consistently; don't swallow errors.
- No `unsafe` without a documented safety invariant in a comment.
- Avoid `clone()` on large data when a borrow or `Arc<Rc>` is clearly intended.
- Keep public surface minimal; mark things `pub(crate)` unless needed externally.

### Error handling
- Errors are propagated, not logged-and-continued where the caller needs the result.
- Fatal conditions return an error rather than panicking (unless truly invariant).
- Read/txn/IO resources are closed; no leaked handles.

### Tests
- New behavior has a focused test (unit, and integration/nextest where relevant).
- A bug fix adds a regression test that would fail on the old code.
- Tests assert the outcome, not just "doesn't crash".

### Docs & scope
- Public API changes update the README / doc comments.
- No scope creep: unrelated changes, renames, or formatting churn are `style`.
- Changelog-worthy behavior adjustments are noted (see `OPEN_RISKS.md` for dep bumps).

### Security & secrets
- No credentials/tokens committed. Config/secrets via env vars or GitHub secrets.
- No unsafe deserialization of untrusted input without validation.
- Shell/script steps don't interpolate untrusted input directly (injection risk).

### CI / repo hygiene (for PRs that touch CI)
- Workflows pin actions (`@v5` etc.), not mutable branches, unless intentional.
- Secrets referenced via `secrets.*` / `env`, not hardcoded.
- `concurrency` groups used to cancel stale runs where useful.
- New actions get a permission block (`permissions:`) that's as narrow as needed.

## Output contract

The reviewer returns **one JSON object** (see schema in
`.agents/reviewer/SKILL.md`). If nothing is wrong, return an empty `findings` list and
a short `summary`. The bot renders it into the PR review comment and inline threads.

`verdict` maps to: `approve` | `changes_requested` | `info`. `risk_level` is one of
`low` | `medium` | `high`.
