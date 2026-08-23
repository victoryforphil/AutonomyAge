---
title: vs-renderkit — wgpu 3D rendering kit plan
type: task
key: vs-renderkit
branch: vfp/agent/plan/vs-renderkit
pr: https://github.com/victoryforphil/AutonomyAge/pull/39
desc: Port a composable wgpu 3D rendering kit into a vs-* crate (plan).
status: active
update: PR open — plan drafted (thin lead)
last_updated: 2026-08-22
---

## Context

Honest lead: `three-d` is essentially unused by owned code (stale v0.18 glow/GL fork),
so vs_renderkit is a **wgpu rebuild**, not a port. Build from three-d architecture
(design) + EmberSim wgpu 29 scaffold (substrate) + zmesh (CPU mesh: OBJ load/write,
decimation) + faultline_core render boundary (spec).

## Todos

- [x] Survey rendering/mesh candidates
- [x] Draft `docs/plan/vs_renderkit.plan.md`
- [ ] Implement `vs_renderkit_{mesh,core,egui}`

## State

- Plan drafted; PR #39 open.

## Risks

- wgpu/winit/edition drift (EmberSim wgpu 29 + winit 0.30 + ed2024 vs egui 0.31–0.34).
- No owned WGSL shaders — must be authored fresh. zmesh is f64/non-indexed.

## Human help

- Confirm the wgpu/egui version matrix (the gating decision) before coding.

## Followups

- Implement the 3-crate kit; position as in-process/embeddable path (Rerun stays the viewer).

## Links

- Plan: `docs/plan/vs_renderkit.plan.md`
- Idea: `docs/ideas/vs_renderkit.idea.md`
- Sources: `three-d`, `AndreasLabs/EmberSim/engine/ember_app/`, `AndreasLabs/zmesh`, `faultline_games`

## Open questions

- Keep the mesh crate CPU-only by default (wgpu conversion behind a feature)?
- Scope creep risk into a full viewer.

## Advice / lessons

- Don't vendor the stale three-d fork; it drags in the wrong (GL) backend.
