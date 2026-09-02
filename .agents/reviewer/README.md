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
| [`../../.github/scripts/review_bot.py`](../../.github/scripts/review_bot.py) | The trusted orchestration: builds the prompt, runs pi, validates the response, and posts comments. |
| `../../.github/workflows/review-pi.yaml` | The trusted-base GitHub Actions wiring. |

## How a review run works

1. A `pull_request_target` event fires (`opened` / `synchronize` / `reopened`).
2. The workflow checks out the trusted PR base revision, fetches the PR head only for
   `git diff`, then calls `review_bot.py`. It never executes PR-controlled code.
3. `review_bot.py` reads trusted `REVIEW.md` + `SKILL.md` (and the task note), builds
   the diff + context into a single prompt, and runs pi against OpenRouter with only
   read-only filesystem tools.
4. Pi returns the JSON review. The bot validates the response schema and:
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

## Configuring the model

- The model is set by the workflow (`OPENROUTER_MODEL`, default
  `deepseek/deepseek-v4-flash`) and the API key by `OPENROUTER_API_KEY` (GitHub
  secret / env). Both go through **OpenRouter**.
- The workflow pins pi. The reviewer has only `read`, `grep`, `find`, and `ls` tools;
  prompt instructions are not the security boundary.

## Inline threads & resolving

- Findings with a valid `file`+`line` become **inline review threads**; the bot
  dedups them across runs by a stable FID so it updates instead of re-posting.
- If the model returns a `line` that's not actually part of the diff (it sometimes
  guesses), the GitHub API rejects it (422) and the bot keeps the finding in the
  summary comment instead — no user-visible failure.
- The bot does not resolve or reopen threads. GitHub Actions' default token cannot
  perform those GraphQL mutations reliably.

## Keeping this healthy

- A review that cannot be parsed or fails schema validation leaves the previous
  comment in place and logs a bounded excerpt of the invalid response.
- If the reviewer agent or its config changes, bump the `REVIEW_BOT_MARKER` version
  in the workflow so the bot posts a fresh comment instead of editing an old schema.
