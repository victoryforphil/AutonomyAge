#!/usr/bin/env python3
"""
Greptile-style fix prompts. Reuses the harness plumbing from review_bot.py.

Triggered by an `issue_comment` that asks the bot to fix something (the workflow
filters the comment). Builds a fix-generation prompt from REVIEW.md + the reviewer
skill + the PR diff + the requested instruction, runs the harness, and upserts a
single fix-suggestion comment on the PR.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from review_bot import (  # noqa: E402
    invoke_harness,
    gh,
    log,
    read_text,
)

FIX_MARKER = "<!-- review-bot-fix:v1 -->"


def build_fix_prompt(reviewer_md, rules_md, diff, changed_files, pr, instruction,
                     task_note=""):
    parts = []
    parts.append("You are the repo's reviewer agent, now asked to produce a FIX.")
    parts.append("")
    parts.append("===== REVIEWER INSTRUCTIONS + CONVENTIONS =====")
    parts.append(reviewer_md)
    parts.append("")
    parts.append("===== REPO REVIEW RULES (REVIEW.md) =====")
    parts.append(rules_md)
    parts.append("")
    parts.append("===== PR CONTEXT =====")
    parts.append(f"PR #{pr.get('number')}: {pr.get('title', '')}")
    parts.append("")
    parts.append(f"Changed files ({len(changed_files)}):")
    for f in changed_files:
        parts.append(f"- {f}")
    parts.append("")
    parts.append("===== DIFF =====")
    parts.append(diff or "(no textual diff)")
    parts.append("")
    if task_note:
        parts.append("===== TASK NOTE (context) =====")
        parts.append(task_note[:6000])
        parts.append("")
    parts.append("===== REQUESTED FIX =====")
    parts.append(instruction)
    parts.append("")
    parts.append("Produce a concrete, minimal fix that addresses the requested issue "
                 "in THIS PR. Return:")
    parts.append("1. A fenced ```diff block with the exact change (preferred), OR")
    parts.append("   a short list of file-by-file edits with before/after lines; and")
    parts.append("2. A 1-2 sentence explanation of the fix.")
    parts.append("If you cannot safely fix it, say so and explain what's blocking.")
    return "\n".join(parts)


def strip_fences(text):
    """Turn a ```diff/... fenced response into a readable comment body."""
    text = text.strip()
    # remove any wrapping ``` ... ``` and keep the inner content
    text = re.sub(r"^```[a-zA-Z]*\n", "", text)
    text = re.sub(r"\n```\s*$", "", text)
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    ap.add_argument("--pr", required=True, type=int)
    ap.add_argument("--base", required=True)
    ap.add_argument("--head", required=True)
    ap.add_argument("--harness", required=True, choices=["opencode", "pi"])
    ap.add_argument("--rules", default="REVIEW.md")
    ap.add_argument("--reviewer", default=".agents/reviewer/SKILL.md")
    ap.add_argument("--instruction", required=True, help="the fix request text")
    ap.add_argument("--task-note", default="")
    ap.add_argument("--marker", default=FIX_MARKER)
    ap.add_argument("--max-diff", type=int, default=100_000)
    ap.add_argument("--dry-run", action="store_true",
                    help="generate the fix but do not post to GitHub")
    args = ap.parse_args()

    repo_dir = os.getcwd()
    owner, _, repo = args.repo.partition("/")
    api_key = os.environ.get("OPENROUTER_API_KEY", "")
    model_id = os.environ.get("OPENROUTER_MODEL", "deepseek/deepseek-v4-flash")

    diff_res = subprocess.run(
        ["git", "diff", f"{args.base}...{args.head}", "--no-ext-diff"],
        capture_output=True, text=True, cwd=repo_dir)
    if diff_res.returncode != 0:
        log(f"git diff failed: {diff_res.stderr.strip()}"); sys.exit(1)
    diff = diff_res.stdout
    changed = subprocess.run(
        ["git", "diff", "--name-only", f"{args.base}...{args.head}"],
        capture_output=True, text=True, cwd=repo_dir).stdout.split()
    if len(diff) > args.max_diff:
        diff = diff[: args.max_diff] + "\n...(diff truncated)"

    pr = gh("GET", f"repos/{owner}/{repo}/pulls/{args.pr}") or {}
    prompt = build_fix_prompt(read_text(args.reviewer), read_text(args.rules),
                              diff, changed, pr, args.instruction,
                              read_text(args.task_note))

    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
        f.write(prompt)
        prompt_path = f.name
    try:
        raw = invoke_harness(args.harness, prompt_path, model_id, api_key, repo_dir)
    finally:
        try:
            os.unlink(prompt_path)
        except OSError:
            pass

    if not raw:
        log("fix harness returned no output"); sys.exit(1)

    body = (f"{args.marker}\n"
            f"## Suggested fix\n\n"
            f"**Request:** {args.instruction}\n\n"
            f"{strip_fences(raw)}\n"
            f"{args.marker}  <!-- end -->")

    if args.dry_run:
        log("DRY RUN — skipping GitHub POST. Suggested fix follows:\n")
        print(body)
        sys.exit(0)

    # upsert the single fix comment (idempotent by marker)
    comments = gh("GET", f"repos/{owner}/{repo}/issues/{args.pr}/comments")
    existing_id = None
    if isinstance(comments, list):
        for c in comments:
            if args.marker in (c.get("body", "") or ""):
                existing_id = c.get("id")
                break
    if existing_id:
        gh("PATCH", f"repos/{owner}/{repo}/issues/comments/{existing_id}",
           {"body": body})
        log(f"updated fix comment {existing_id}")
    else:
        gh("POST", f"repos/{owner}/{repo}/issues/{args.pr}/comments", {"body": body})
        log("posted fix comment")
    log("done.")


if __name__ == "__main__":
    main()
