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
