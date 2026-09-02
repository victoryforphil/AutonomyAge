---
title: vs-renderkit — wgpu 3D rendering kit plan
type: task
key: vs-renderkit
branch: vfp/agent/plan/vs-renderkit
pr: https://github.com/victoryforphil/AutonomyAge/pull/39
desc: Port a composable wgpu 3D rendering kit into a vs-* crate (plan).
status: active
update: version matrix resolved (wgpu 29.0.3 + winit 0.30.13 + ed2024 + eframe/egui-wgpu 0.35); mesh stays CPU-only default; concrete 3-crate layout + API drafted
last_updated: 2026-08-23
---

## Context

Lead confirmed: `three-d` is essentially unused by owned code (stale v0.18 glow/GL fork),
so vs_renderkit is a **wgpu rebuild**, not a port. Build from three-d architecture
(design) + EmberSim wgpu-29 scaffold (substrate) + zmesh (CPU mesh: OBJ load/write,
decimation) + faultline_core RenderKit boundary (frame/pass spec).

**Version matrix now RESOLVED** (read off crates.io dep graphs 2026-08-23):
wgpu 29.0.3 + winit 0.30.13 + edition 2024, with **eframe/egui/egui-wgpu = 0.35**
(bundles wgpu ^29 + winit ^0.30.13). Trade-off: eframe 0.35 (2026-06-25) over 0.36
(2026-08-07, which bumps to wgpu 30) to stay on the proven wgpu 29. Single wgpu major
in the build is the hard interop rule (core + egui texture/device sharing).

**Mesh default DECIDED:** `vs_renderkit_mesh` is CPU-only by default (no wgpu dep);
GPU conversion behind `feature = "wgpu"`, decimation parallel behind `feature = "parallel"`.
`MeshData { positions: Vec<[f32;3]>, indices: Vec<u32>, normals: Vec<[f32;3]> }`.

## Todos

- [x] Survey rendering/mesh candidates
- [x] Draft `docs/plan/vs_renderkit.plan.md`
- [x] Resolve wgpu/winit/edition + eframe version matrix
- [x] Decide mesh crate default + `MeshData` type + OBJ/decimation port scope
- [x] Produce concrete three-crate layout + first-milestone API (signatures)
- [ ] Implement `vs_renderkit_{mesh,core,egui}`
- [ ] Author WGSL shaders (mesh.vert/frag) from scratch
- [ ] Headless screenshot path (offscreen → PNG CI smoke test)

## State

- Plan drafted and updated: `docs/plan/vs_renderkit.plan.md` + updated research note
  `/tmp/opencode/vfp-research/cont/vs-renderkit.plan.updated.md` (version matrix, 3-crate
  layout, signatures, first milestone). PR #39 open.

## Risks

- Single-wgpu-major invariant: never let Cargo resolve two wgpu majors or egui↔core
  texture/device sharing breaks. Bump core + eframe together if wgpu 30 is adopted.
- No owned WGSL shaders — must be authored fresh (three-d is GLSL, not portable).
- zmesh is f64 / non-indexed / O(n²) dedup — convert to f32 + indexed `MeshData`,
  switch dedup to spatial hash. QEM decimation is an 842-line port.
- CPU readback for the egui embed is a milestone shortcut (slow); the
  `egui_wgpu::Callback` non-copy path depends on both sides being wgpu 29.
- Headless adapter selection may fall back to software; needs backend selection for CI.

## Human help

- Confirm the locked matrix (wgpu 29.0.3 + winit 0.30.13 + ed2024 + eframe 0.35) before
  coding — it was the gating decision and is now concrete.
- Approve skipping zmesh subdivision (stub) and deferring a full SceneKit in favor of the
  lightweight `RenderFrame`/`RenderPass` boundary.

## Followups

- Implement the 3-crate kit; first milestone = windowed clear + one indexed mesh.
- Add headless PNG smoke test and embedded egui view (offscreen → ColorImage → TextureHandle).
- Position as in-process/embeddable path (Rerun stays the general viewer); avoid scope creep.

## Links

- Plan: `docs/plan/vs_renderkit.plan.md`
- Updated plan (research): `/tmp/opencode/vfp-research/cont/vs-renderkit.plan.updated.md`
- Idea: `docs/ideas/vs_renderkit.idea.md`
- Sources: `three-d`, `AndreasLabs/EmberSim/engine/ember_app/` (wgpu 29), `AndreasLabs/zmesh`,
  `victoryforphil/faultline_games/.../RenderKit` (frame/pass), `victoryforphil/rerun-interview/viewer/`

## Open questions

- Keep the mesh crate CPU-only by default (wgpu conversion behind a feature)? — **Yes, decided.**
- Scope-creep risk into a full viewer? — Mitigate: keep frame/pass boundary lightweight, no SceneKit.
- Use the `egui_wgpu::Callback` path now, or CPU readback for the first egui milestone?

## Advice / lessons

- Don't vendor the stale three-d fork; it drags in the wrong (GL) backend.
- Lock the version matrix first (now done) — it is the gating decision for everything else.
- Port the faultline RenderKit boundary (frame/pass/handle) rather than a full scene graph.
