# Reviewer — Greptile-style review bot

This repo ships an automated **reviewer** agent that mimics Greptile: it reviews every
PR diff, returns structured findings, and a GitHub Action renders them as a single
updating PR comment (plus inline threads).

## Where things live

| path | what it is |
|------|------------|
| [`REVIEW.md`](../../REVIEW.md) | **The rules.** The repo's review rules / style checks the reviewer enforces. Edit this; the bot re-reads it each run. |
| [`SKILL.md`](SKILL.md) | **The reviewer agent.** Its identity, operating rules, tools, and the **output JSON schema**. This is the reviewer's "skills and info". |
| [`skills/`](skills/) | Optional extra reviewer lenses (one markdown skill per focus, e.g. `rust.md`, `ci.md`). The reviewer loads the ones it needs. |
| [`../../.github/scripts/review_bot.py`](../../.github/scripts/review_bot.py) | The orchestration: builds the prompt, runs the agent harness, posts the comment(s). Harness-agnostic. |
| `../../.github/workflows/review*.yaml` | The harness wiring (one per track: `opencode` / `pi`). |

## How a review run works

1. A `pull_request` event fires (`opened` / `synchronize` / `reopened`).
2. The workflow checks out the PR head with full history and calls
   `review_bot.py`.
3. `review_bot.py` reads `REVIEW.md` + `SKILL.md` (and the task note if one exists),
   builds the diff + context into a single prompt, and runs the chosen harness
   (`opencode` or `pi`) against OpenRouter.
4. The harness returns the JSON review. The bot parses it and:
   - **upserts** the single top-level review comment (create, or update the previous
     one so PRs don't drown in comments), and
   - posts **inline threads** for findings that carry a valid `file`+`line`.

## Adding a rule (the common case)

Just edit [`REVIEW.md`](../../REVIEW.md). Add a short bullet under the relevant
heading and note its severity. Nothing else to wire — the reviewer reads it every run.

## Adding a reviewer skill/lens

Add a markdown file under [`skills/`](skills/) (e.g. `skills/rust-idioms.md`),
describe the lens, and give the reviewer one instruction to load it. Keep lenses
focused; the default instructions already cover the repo's conventions.

## Configuring the model / harness

- The model is set by the workflow (`OPENROUTER_MODEL`, default
  `deepseek/deepseek-v4-flash`) and the API key by `OPENROUTER_API_KEY` (GitHub
  secret / env). Both go through **OpenRouter**.
- Each track pins its harness: the `opencode` track uses `opencode` (v2 beta), the
  `pi` track uses `pi`.

## Inline threads & resolving

- Findings with a valid `file`+`line` become **inline review threads**; the bot
  dedups them across runs by a stable FID so it updates instead of re-posting.
- If the model returns a `line` that's not actually part of the diff (it sometimes
  guesses), the GitHub API rejects it (422) and the bot keeps the finding in the
  summary comment instead — no user-visible failure.
- **Resolving threads:** the bot reconciles threads to the current findings — it
  resolves a thread whose finding is no longer flagged, and reopens one that is.
  Resolving uses GraphQL, which the default `GITHUB_TOKEN` cannot do
  ("Resource not accessible by integration"). To enable full resolve/open, configure
  an optional `REVIEW_BOT_TOKEN` secret (a PAT with `repo` scope); without it the bot
  posts/updates threads but resolves best-effort (logged, non-fatal).

## Keeping this healthy

- A review that can't be parsed (bad JSON, harness error) leaves the previous
  comment in place and surfaces the raw error in the run log — don't delete the old
  review on a transient failure.
- If the reviewer agent or its config changes, bump the `REVIEW_BOT_MARKER` version
  in the workflow so the bot posts a fresh comment instead of editing an old schema.
