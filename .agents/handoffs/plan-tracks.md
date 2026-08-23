---
title: Plan Tracks Board
type: design
status: active
tags:
  - handoff
  - swarm
---

# Plan Tracks Board

Shared swarm state for cross-repo research → `docs/plan/*.plan.md` PRs in
`AutonomyAge`. One track per idea.

Conventions:
- Branch: `vfp/agent/plan/{idea}` (worktree in `.worktrees/plan-{idea}`).
- Commit/PR title: `{Idea} // Plan // <desc> (plan)`.
- Base = fresh `origin/main`.
- Survey index: `docs/agents/repo-index.md`.

## Tracks

| Idea | Best source | Status | PR |
|------|-------------|--------|----|
| repo-index | n/a | done | #32 |
| forest | project-firefly `src_tools/forest` | PR open | #33 |
| valley | project-firefly `src_core/valley` | PR open | #34 |
| quadlink | lil-hopps `lil-link` + whisper + project-devore | PR open | #35 |
| victory-logging | tremor-nodekit `init_logger` + dark-factory/tinyverse | PR open | #36 |
| victory-dir | waldo `dir_utils.rs` + tremor-nodekit model | PR open | #37 |
| vs-viz | lil-rerun design + SkyCanvas/loki helpers | PR open | #38 |
| vs-renderkit | three-d design + EmberSim wgpu + zmesh; faultline spec | PR open | #39 |
| vs-appkit | tremor-ui + mad_common + underscore_quad | PR open | #40 |

## Swarm discipline

- One subagent per track returned the final plan doc markdown; the lead wrote the
  doc into the track worktree, then committed/pushed/PR'd.
- Thin leads are noted in the plan docs (e.g. vs_renderkit's "three-d unused");
  none blocked.

## Next round ideas (if resumed)

- Migrate the plan docs into implementation: `libs/vs-{dir,logging,valley,forest,mavlink,viz,appkit,renderkit_*}`.
- Resolve cross-cutting decisions: `victory-*` vs `vs-*` crate naming; egui pin (0.34)
  vs 0.33; rerun pin (0.28.x); the missing `SkyPose`/transforms type for valley.
