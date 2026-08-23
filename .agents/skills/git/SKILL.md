---
name: git
description: Conventions for using git / github
---

# Branches 
- Format:
  - `vfp/{_ (blank if human)/agent/{feature/branch name}/{sub-feature}}
- Examples:
    - `vfp/agent/feature/logging`
    - `vfp/agent/feature/`
    - `vfp/agent/feature/`
    - `vfp/agent/feature/`
    - `vfp/human-guided-feature`

# Commit
- Format:
  - `[{optional issue}] {Module / Category} // {Optional sub-category} // {Description} ({tags..})`
- Examples:
    - `[#123] HotFix // vs-wtf // Fixed timezone issue`
    - `Valley // Added new example validator (example, un-tested)`

# PR Title
- Format:
  - `[{optional issue}] {Module / Category} // {Optional sub-category} // {Description} ({tags..})`
- Examples:
    - `[#123] HotFix // vs-wtf // Fixed timezone issue`
    - `Valley // Added new example validator (example, un-tested)`

# Worktrees
- Use worktrees to work a branch without touching the primary checkout (leave it as-is).
- Store worktrees under `{project_dir}/.worktrees/` inside the repo.
- Setup:
  - `git worktree add -b <branch> .worktrees/<name> <base>` (e.g. `.<base> = origin/main`)
- Useful commands:
  - `git worktree list` — show all worktrees
  - `git worktree remove .worktrees/<name>` — delete a worktree
  - `git worktree prune` — clean stale worktree metadata

# Stacked PRs (`gh stack`)
- Use `gh stack` to break a large change into a chain of PRs that build on each other (each PR targets the previous branch, not `main`).
- One-time install of the official extension: `gh extension install github/gh-stack`.
- Build a stack bottom-to-top: `gh stack init <branch1> <branch2> ...` — the first branch bases on the default trunk (`main`); each later branch bases on the prior one.
- Push and create/link all PRs: `gh stack submit [--auto --open]` (`--auto` skips the editor; `--open` makes PRs ready for review so CI runs).
- Other commands: `gh stack view`, `gh stack switch|checkout`, `gh stack sync`, `gh stack rebase`, `gh stack merge`.
- Stacking on an existing PR branch (e.g. onto a hotfix PR): include that branch as `branch1`; it keeps its own PR and your branch's PR gets `baseRef = branch1`. GitHub actions still run for the stacked PR.
