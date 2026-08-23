#!/usr/bin/env python3
"""
Greptile-style review bot.

Builds a review prompt from REVIEW.md + the reviewer SKILL.md + the PR diff/context,
runs one agent harness (opencode or pi) against OpenRouter, parses the JSON review,
then:

  1. upserts the single top-level PR review comment (create, or update the previous
     one containing the review-bot marker), and
  2. posts inline review threads for findings that carry a valid file+line.

Harness-agnostic: the only harness-specific part is how we invoke the CLI and parse
its output. See .agents/reviewer/README.md.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# --------------------------------------------------------------------------- #
# helpers
# --------------------------------------------------------------------------- #

def log(msg):
    print(f"[review-bot] {msg}", file=sys.stderr, flush=True)


def run(cmd, env=None, cwd=None, timeout=900):
    log("running: " + " ".join(_redact(cmd)))
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        env=env,
        cwd=cwd,
        timeout=timeout,
    )


_SECRET_FLAGS = {"--api-key", "--token", "--password"}
_SECRET_ENV = {"OPENROUTER_API_KEY", "GITHUB_TOKEN", "GH_TOKEN", "OPENAI_API_KEY"}


def _redact(argv):
    """Return a copy of argv with secret flag values masked, for safe logging."""
    out = []
    i = 0
    while i < len(argv):
        tok = argv[i]
        if tok in _SECRET_FLAGS and i + 1 < len(argv):
            out.append(tok)
            out.append("<redacted>")
            i += 2
            continue
        out.append(tok)
        i += 1
    return out


def gh(method, path, payload=None):
    """Call the GitHub REST API via `gh`. payload is a dict -> JSON body."""
    cmd = ["gh", "api", "-X", method, path]
    if payload is not None:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(payload, f)
            tmp = f.name
        cmd += ["--input", tmp]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    finally:
        if payload is not None:
            try:
                os.unlink(tmp)
            except OSError:
                pass
    if res.returncode != 0:
        log(f"gh api {method} {path} failed: {res.stderr.strip()}")
        return None
    if not res.stdout.strip():
        return None
    try:
        return json.loads(res.stdout)
    except json.JSONDecodeError:
        return res.stdout


# --------------------------------------------------------------------------- #
# harness invocation
# --------------------------------------------------------------------------- #

def invoke_harness(harness, prompt_path, model_id, api_key, repo_dir):
    """Run the chosen harness on prompt_path, return the raw assistant text."""
    with open(prompt_path, "r", encoding="utf-8") as fh:
        prompt = fh.read()
    # Keep the message comfortably under Linux's ~128KB per-arg limit.
    prompt = prompt[:110_000]

    if harness == "opencode":
        # opencode needs a clean datastore; a corrupt/locked global DB breaks it.
        # Pass the prompt as the message (not --file into /tmp): the agent otherwise
        # tries to `read` the /tmp file and its external-directory permission auto-
        # rejects, stalling the run.
        tmpdata = tempfile.mkdtemp(prefix="ocdata-")
        env = dict(os.environ)
        env["OPENROUTER_API_KEY"] = api_key
        env["XDG_DATA_HOME"] = tmpdata
        env["XDG_CACHE_HOME"] = tmpdata
        try:
            cmd = [
                "opencode",
                "run",
                prompt,
                "--format", "json",
                "-m", f"openrouter/{model_id}",
            ]
            res = run(cmd, env=env, cwd=repo_dir)
            if res.returncode != 0:
                log(f"opencode stderr: {res.stderr[-2000:]}")
            return extract_opencode_text(res.stdout)
        finally:
            shutil.rmtree(tmpdata, ignore_errors=True)

    if harness == "pi":
        env = dict(os.environ)
        env["OPENROUTER_API_KEY"] = api_key
        cmd = [
            "pi", "-p", f"@{prompt_path}",
            "--provider", "openrouter",
            "--model", f"openrouter/{model_id}",
            # key read from OPENROUTER_API_KEY env (never on argv -> never in logs)
            "--mode", "json",
            "--thinking", "off",
            "--no-session",
            "--no-approve",
        ]
        res = run(cmd, env=env, cwd=repo_dir)
        if res.returncode != 0:
            log(f"pi stderr: {res.stderr[-2000:]}")
        return extract_pi_text(res.stdout)

    raise ValueError(f"unknown harness: {harness}")


def extract_opencode_text(stdout):
    parts = []
    for line in stdout.splitlines():
        line = line.strip()
        if not line or not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") == "text":
            part = ev.get("part", {})
            if isinstance(part, dict) and part.get("type") == "text":
                parts.append(part.get("text", ""))
    return "".join(parts)


def extract_pi_text(stdout):
    # last agent_end carries the full messages array
    last = {}
    for line in stdout.splitlines():
        line = line.strip()
        if not line or not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") == "agent_end":
            last = ev
    msgs = last.get("messages", [])
    text = []
    for m in msgs:
        if m.get("role") == "assistant":
            content = m.get("content", [])
            for c in content if isinstance(content, list) else []:
                if isinstance(c, dict) and c.get("type") == "text":
                    text.append(c.get("text", ""))
    return "".join(text)


def parse_review_json(raw_text):
    """Pull a JSON object out of the model's reply. Tolerant of fences/prose."""
    if not raw_text:
        return None
    candidates = []
    # direct
    try:
        return json.loads(raw_text)
    except json.JSONDecodeError:
        pass
    # fenced block(s)
    for fence in re.finditer(r"```(?:json)?\s*([\s\S]*?)```", raw_text):
        candidates.append(fence.group(1))
    # first {...} block via brace matching (robust to trailing prose)
    candidates.append(_first_balanced_object(raw_text))
    for cand in candidates:
        if not cand:
            continue
        cand = cand.strip()
        try:
            obj = json.loads(cand)
            if isinstance(obj, dict):
                return obj
        except json.JSONDecodeError:
            continue
    return None


def _first_balanced_object(text):
    """Return the substring of the first balanced {...} object, or ''."""
    start = text.find("{")
    if start == -1:
        return ""
    depth = 0
    in_str = False
    esc = False
    for i in range(start, len(text)):
        ch = text[i]
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
            continue
        if ch == '"':
            in_str = True
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[start:i + 1]
    return ""


# --------------------------------------------------------------------------- #
# github rendering / posting
# --------------------------------------------------------------------------- #

VERDICT_EMOJI = {
    "approve": "✅",
    "changes_requested": "🔴",
    "info": "🎗️",
}


def render_review_md(review, stats, marker):
    verdict = review.get("verdict", "info")
    emoji = VERDICT_EMOJI.get(verdict, "🎗️")
    label = verdict.replace("_", " ").title()
    findings = review.get("findings", [])
    summary = review.get("summary", "").strip()

    lines = [f"{marker}", f"## Review — {emoji} {label}", ""]
    if summary:
        lines.append(summary)
        lines.append("")

    # stats card
    risk = review.get("risk_level", "medium")
    lines.append(f"- **Risk:** {risk}")
    lines.append(f"- **Files changed:** {stats['files']}")
    lines.append(f"- **Additions:** +{stats['additions']}")
    lines.append(f"- **Deletions:** -{stats['deletions']}")
    nbug = sum(1 for f in findings if f.get("severity") == "bug")
    nstyle = sum(1 for f in findings if f.get("severity") == "style")
    nsup = sum(1 for f in findings if f.get("severity") == "suggestion")
    lines.append(f"- **Findings:** {nbug} bug · {nstyle} style · {nsup} suggestion")
    lines.append("")

    # per-area checks checklist
    checks = review.get("checks", [])
    if checks:
        lines.append("### Checks")
        for c in checks:
            area = c.get("area", "?")
            status = c.get("status", "na")
            icon = {"pass": "✅", "issue": "⚠️", "na": "➖"}.get(status, "➖")
            note = c.get("note")
            line = f"- {icon} **{area}** — {status}"
            if note:
                line += f" ({note})"
            lines.append(line)
        lines.append("")

    # findings
    if not findings:
        lines.append("**No findings.** Looks good to ship.")
    else:
        lines.append("### Findings")
        for f in findings:
            sev = f.get("severity", "suggestion")
            title = f.get("title", "Finding")
            loc = f.get("file", "?")
            if f.get("line") is not None:
                loc = f"{loc}:{f['line']}"
            lines.append(f"- **[{sev}] {title}** — `{loc}`")
            desc = (f.get("description") or "").strip()
            if desc:
                lines.append(f"  {desc}")
            sug = (f.get("suggestion") or "").strip()
            if sug:
                lines.append(f"  _Suggestion:_ {sug}")
            lines.append("")

    # risk sources / tests / diagram
    risks = review.get("risk_sources", [])
    if risks:
        lines.append("### Risk sources")
        for r in risks:
            lines.append(f"- {r}")
        lines.append("")
    tests = review.get("suggested_tests", [])
    if tests:
        lines.append("### Suggested tests")
        for t in tests:
            lines.append(f"- {t}")
        lines.append("")
    pr_desc = review.get("pr_desc_suggestion")
    if pr_desc:
        lines.append("### Suggested PR description")
        lines.append(pr_desc)
        lines.append("")
    diagram = review.get("diagram")
    if diagram:
        lines.append("### Change diagram")
        lines.append("```mermaid")
        lines.append(str(diagram).strip())
        lines.append("```")
        lines.append("")

    lines.append(f"{marker}  <!-- review-bot end -->")
    return "\n".join(lines)


def upsert_review_comment(owner, repo, pr, marker, body):
    comments = gh("GET", f"repos/{owner}/{repo}/issues/{pr}/comments")
    existing_id = None
    if isinstance(comments, list):
        for c in comments:
            if marker in (c.get("body", "") or ""):
                existing_id = c.get("id")
                break
    if existing_id:
        log(f"updating existing review comment {existing_id}")
        return gh("PATCH", f"repos/{owner}/{repo}/issues/comments/{existing_id}",
                  {"body": body})
    log("creating new review comment")
    return gh("POST", f"repos/{owner}/{repo}/issues/{pr}/comments", {"body": body})


def finding_fid(f):
    """Stable, deterministic id for a finding, so inline threads can be matched
    across run to-run. Uses file + normalized title (not the model's line number,
    which wobbles each run and would otherwise spawn duplicate threads)."""
    title = re.sub(r"\s+", " ", (f.get("title") or "")).strip().lower()
    key = f"{f.get('file')}|{title}"
    return hashlib.md5(key.encode("utf-8")).hexdigest()[:16]


def inline_body(f, marker):
    fid = finding_fid(f)
    return (f"{marker}\n"
            f"**[{f.get('severity', 'suggestion')}] {f.get('title', 'Finding')}**\n\n"
            f"{f.get('description', '')}\n\n"
            f"_Suggestion:_ {f.get('suggestion', '')}\n"
            f"<!-- review-bot-fid:{fid} -->")


def gh_graphql(query, variables):
    """Run a GraphQL query/mutation via `gh api graphql`. Returns data or None."""
    cmd = ["gh", "api", "graphql", "-f", f"query={query}"]
    for k, v in variables.items():
        cmd += ["-F", f"{k}={v}"]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if res.returncode != 0:
        log(f"gh api graphql failed: {res.stderr.strip()}")
        return None
    try:
        return json.loads(res.stdout)
    except json.JSONDecodeError:
        return None


def reconcile_inline_threads(owner, repo, pr, current_fids):
    """Resolve stale review threads (finding no longer flagged) and unresolve ones
    that have reopened. Best-effort: never fails the run."""
    try:
        q = ("query($owner:String!,$repo:String!,$pr:Int!){"
             "repository(owner:$owner,name:$repo){"
             "pullRequest(number:$pr){"
             "reviewThreads(first:100){nodes{id isResolved "
             "comments(first:100){nodes{databaseId body}}}}} } }")
        data = gh_graphql(q, {"owner": owner, "repo": repo, "pr": pr})
        nodes = (((data or {}).get("data", {}) or {}).get("repository", {}) or {}) \
            .get("pullRequest", {}).get("reviewThreads", {}).get("nodes", [])
        for t in nodes:
            comments = t.get("comments", {}).get("nodes", [])
            fid = None
            for c in comments:
                m = re.search(r"review-bot-fid:([0-9a-f]{16})", c.get("body", "") or "")
                if m:
                    fid = m.group(1)
                    break
            if not fid:
                continue
            if fid not in current_fids:
                # finding no longer flagged -> resolved (fixed or dropped)
                if not t.get("isResolved"):
                    mutate("resolveReviewThread", t["id"])
            else:
                # finding still present -> keep it open
                if t.get("isResolved"):
                    mutate("unresolveReviewThread", t["id"])
    except Exception as e:  # noqa: BLE001
        log(f"reconcile_inline_threads error: {e}")


def mutate(mutation, node_id):
    q = (f"mutation {{ {mutation}(input: {{threadId: \"{node_id}\"}}) "
         f"{{ clientMutationId }} }}")
    res = gh_graphql(q, {})
    log(f"{mutation} on {node_id}: "
        + ("ok" if res else "failed"))


def current_fids(findings):
    return {finding_fid(f) for f in findings if f.get("file") and f.get("line") is not None}


def post_inline_comments(owner, repo, pr, head_sha, findings, marker):
    """Post/update review threads on the diff. Idempotent: finds prior threads for
    this PR by their review-bot marker and dedups by the finding FID; updates the
    existing thread body when a finding persists, otherwise posts a new one."""

    existing = gh("GET", f"repos/{owner}/{repo}/pulls/{pr}/comments")
    existing_by_fid = {}
    if isinstance(existing, list):
        for c in existing:
            body = c.get("body", "") or ""
            if marker not in body:
                continue
            m = re.search(r"review-bot-fid:([0-9a-f]{16})", body)
            if m:
                existing_by_fid[m.group(1)] = c.get("id")

    for f in findings:
        file = f.get("file")
        line = f.get("line")
        if not file or line is None:
            continue
        fid = finding_fid(f)
        body = inline_body(f, marker)
        payload = {
            "body": body,
            "commit_id": head_sha,
            "path": file,
            "line": line,
            "side": "RIGHT",
        }
        if fid in existing_by_fid:
            cid = existing_by_fid[fid]
            log(f"updating inline thread {cid} ({file}:{line})")
            gh("PATCH", f"repos/{owner}/{repo}/pulls/comments/{cid}", {"body": body})
        else:
            r = gh("POST", f"repos/{owner}/{repo}/pulls/{pr}/comments", payload)
            if r:
                log(f"inline comment posted: {file}:{line}")
            else:
                log(f"inline comment skipped (invalid line?): {file}:{line}")


# --------------------------------------------------------------------------- #
# prompt builder
# --------------------------------------------------------------------------- #

def read_text(path):
    if not path or not Path(path).exists():
        return ""
    return Path(path).read_text()


def build_prompt(reviewer_md, rules_md, diff, changed_files, pr):
    parts = []
    parts.append("You are the repo's reviewer agent. Apply the instructions below.")
    parts.append("")
    parts.append("===== REVIEWER INSTRUCTIONS + OUTPUT SCHEMA =====")
    parts.append(reviewer_md)
    parts.append("")
    parts.append("===== REPO REVIEW RULES (REVIEW.md) =====")
    parts.append(rules_md)
    parts.append("")
    parts.append("===== PR CONTEXT =====")
    parts.append(f"PR #{pr.get('number')}: {pr.get('title', '')}")
    if pr.get("body"):
        parts.append(f"Description:\n{pr['body'][:4000]}")
    parts.append("")
    parts.append(f"Changed files ({len(changed_files)}):")
    for f in changed_files:
        parts.append(f"- {f}")
    parts.append("")
    parts.append("===== DIFF =====")
    parts.append(diff or "(no textual diff)")
    parts.append("")
    parts.append("Return ONE JSON object exactly matching the schema. No markdown "
                 "fences, no commentary, no tools. If nothing is wrong, return an empty "
                 "findings array.")
    return "\n".join(parts)


# --------------------------------------------------------------------------- #
# main
# --------------------------------------------------------------------------- #

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True, help="OWNER/REPO")
    ap.add_argument("--pr", required=True, type=int)
    ap.add_argument("--base", required=True, help="base commit sha")
    ap.add_argument("--head", required=True, help="head commit sha")
    ap.add_argument("--harness", required=True, choices=["opencode", "pi"])
    ap.add_argument("--rules", default="REVIEW.md")
    ap.add_argument("--reviewer", default=".agents/reviewer/SKILL.md")
    ap.add_argument("--task-note", default="")
    ap.add_argument("--marker", default="<!-- review-bot:v1 -->")
    ap.add_argument("--max-diff", type=int, default=100_000)
    ap.add_argument("--dry-run", action="store_true",
                    help="run the harness and render, but do not post to GitHub")
    args = ap.parse_args()

    repo_dir = os.getcwd()
    owner, _, repo = args.repo.partition("/")
    api_key = os.environ.get("OPENROUTER_API_KEY", "")
    model_id = os.environ.get("OPENROUTER_MODEL", "deepseek/deepseek-v4-flash")

    # ---- gather diff ----
    diff_res = subprocess.run(
        ["git", "diff", f"{args.base}...{args.head}", "--no-ext-diff"],
        capture_output=True, text=True, cwd=repo_dir,
    )
    if diff_res.returncode != 0:
        log(f"git diff failed ({diff_res.returncode}): {diff_res.stderr.strip()}. "
            "Leaving existing review comment in place.")
        sys.exit(1)
    diff = diff_res.stdout
    changed = subprocess.run(
        ["git", "diff", "--name-only", f"{args.base}...{args.head}"],
        capture_output=True, text=True, cwd=repo_dir,
    ).stdout.split()
    if len(diff) > args.max_diff:
        diff = diff[: args.max_diff] + "\n...(diff truncated)"

    stat = subprocess.run(
        ["git", "diff", "--numstat", f"{args.base}...{args.head}"],
        capture_output=True, text=True, cwd=repo_dir,
    ).stdout
    additions = deletions = 0
    for ln in stat.splitlines():
        parts = ln.split("\t")
        if len(parts) >= 2:
            a, d = parts[0], parts[1]
            additions += int(a) if a.isdigit() else 0
            deletions += int(d) if d.isdigit() else 0
    stats = {"files": len(changed), "additions": additions, "deletions": deletions}

    # ---- pr metadata from gh ----
    pr = gh("GET", f"repos/{owner}/{repo}/pulls/{args.pr}") or {}

    # ---- build prompt ----
    reviewer_md = read_text(args.reviewer)
    rules_md = read_text(args.rules)
    task_note = read_text(args.task_note)
    prompt = build_prompt(reviewer_md, rules_md, diff, changed, pr)
    if task_note:
        prompt += "\n\n===== TASK NOTE (context) =====\n" + task_note[:6000]

    # ---- run harness (retry once on a non-JSON reply; then bail gracefully) ----
    review = None
    for attempt in (1, 2):
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
            f.write(prompt)
            prompt_path = f.name
        try:
            raw = invoke_harness(args.harness, prompt_path, model_id, api_key,
                                 repo_dir)
        finally:
            try:
                os.unlink(prompt_path)
            except OSError:
                pass
        review = parse_review_json(raw)
        if review:
            break
        log(f"attempt {attempt}: could not parse a JSON review from the harness "
            "output.")

    if not review:
        # Transient model/harness failure: keep the last good review comment; never
        # red the PR just because the model went off-script.
        log("leaving existing comment in place; exiting 0 (transient parse failure).")
        log("first 2000 chars of last raw reply:\n" + (raw or "")[:2000])
        sys.exit(0)

    # ---- render + upsert top-level comment ----
    body = render_review_md(review, stats, args.marker)

    if args.dry_run:
        log("DRY RUN — skipping GitHub POSTs. Rendered body follows.\n")
        print(body)
        log("inline findings: %d" % len(review.get("findings", [])))
        sys.exit(0)

    upsert_review_comment(owner, repo, args.pr, args.marker, body)

    # ---- inline threads ----
    head_sha = args.head
    findings = review.get("findings", [])
    post_inline_comments(owner, repo, args.pr, head_sha, findings, args.marker)
    # Resolve threads whose finding is no longer flagged; reopen ones that persist.
    reconcile_inline_threads(owner, repo, args.pr, current_fids(findings))

    log("done. verdict=%s findings=%d" % (review.get("verdict"),
                                          len(review.get("findings", []))))


if __name__ == "__main__":
    main()
