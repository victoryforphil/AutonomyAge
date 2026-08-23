---
title: vs_renderkit
type: plan
status: todo
tags:
  - plan
  - rendering
  - wgpu
---

# vs_renderkit (Plan)

A composable 3D rendering kit for the AutonomyAge `vs-*` Rust workspace. It provides **scene, shader, mesh, and graphics helpers** built on **`wgpu`** (not `three-d`), and is reusable/embeddable in three contexts — inside an `egui` app, as a standalone window, or headless (offscreen, for screenshots/CI). Like `vs_appkit` but for 3D: small opt-in components rather than one monolith.

## 1. Purpose

- Give every `vs-*` tool a clean way to build and render 3D scenes without owning all the `wgpu` boilerplate (instance → surface → adapter → device → queue → config → render pass).
- Be **composable**: `vs_renderkit_mesh`, `vs_renderkit_core`, `vs_renderkit_egui` are separate opt-in crates, mirroring the `vs_appkit` split between an egui-only core and an optional eframe shell.
- Support three embed modes with one renderer: **windowed** (winit/wgpu app shell), **embedded** (render to offscreen texture displayed via `egui::TextureHandle`), **headless** (offscreen, for screenshots/CI).
- Port the strongest available **design** (the `three-d` scene/object/material/light/camera/control architecture) onto a **`wgpu`** substrate, reusing the best scaffold (EmberSim) and mesh code (zmesh).
- Keep domain types out of the renderer: it consumes meshes/materials/cameras and small data structs, not `vs-wtf`/`vs-data-store`/`vs-broker` types.

## 2. Source implementations found

> Honest note up front: **`three-d` is essentially UNUSED by owned code.** A stale fork exists at `/home/vfp/repos/vfp/three-d` (v0.18.0, `glow`/GL); the workspace `Cargo.lock` does not reference it. Its value is purely as a design blueprint, not code to import.

| Repo | Path | Backend | Date | Notes |
|------|------|---------|------|-------|
| **three-d** | `victoryforphil/three-d/src/` | `glow` 0.13 (GL), features `window`/`headless`/`egui-gui` | v0.18.0 | **Best design blueprint.** Feature-complete layers: `core` (buffer/program/render_states/render_target/texture/uniform), `renderer` (camera/control/effect/geometry/light/material/object), `window`, `gui`. GL, not wgpu — port the architecture, not the code. |
| **EmberSim** | `AndreasLabs/EmberSim/engine/ember_app/` (`app.rs`, `app_state.rs`, `lib.rs`) | wgpu 29.0.3 + winit 0.30, edn 2024 | ~May 2026 | **Best wgpu scaffold.** Clean `winit::ApplicationHandler` + `State` owning real `wgpu::Instance`/`Surface`/`Adapter`/`Device`/`Queue`/`SurfaceConfiguration`, sRGB surface format, `resize()`, `render()` clear pass, `queue.submit`. Missing pipeline/shader/mesh layer. |
| **zmesh** | `AndreasLabs/zmesh/zmesh-lib/src/` | CPU-only, `wavefront_obj` 11 | current | **Best mesh asset companion.** `TriMesh { faces: Vec<[[f64;3];3]> }`, OBJ load/write, decimation (842-line module), vertex dedup. Subdivision is a stub. Drop `f64`→`f32` for vertex buffers. |
| **faultline_core** | `victoryforphil/faultline_games/engine/faultline_core/` | C++/bgfx/SDL3 | ~Apr 2026 | **Architecture spec, not portable.** RenderKit (`RenderFrame`/`RenderPass`/`PassItems`/`IRenderer::submit`), SceneKit (retained `Scene`/`Node`/`MeshNode`/`LightNode` + debug-draw sidecar), MathKit, AppKit/GameKit/ToolKit. Copy the design. |
| rerun-interview | `victoryforphil/rerun-interview/viewer/src/viewer.rs` | eframe 0.31 + `Renderer::Wgpu` | ref | egui-embed reference. Has a TODO for `egui_wgpu::Callback`. |
| underscore_quad | `victoryforphil/underscore_quad/src/ui/video_view.rs` | egui | ref | `egui::TextureHandle` image-widget pattern. |

**Not reusable**: `vulkan-fun` (empty), `AndreasLabs/tectonic` (scaffold only), `ray-ray` (CPU raytracer learning), `glow_but_higher` (vendored glow bindings), `rendering-learning` (blank wgpu scaffold + raw C/Vulkan).

## 3. Best version to build from

There is no single repo to port wholesale. Build a **wgpu renderer from four pieces**:

1. **Architecture = `three-d` design.** Use its layering as the target shape: a `context`/core that owns the GPU device/queue/rendertarget, and a `Scene` (object/mesh/material/light) that extracts to a frame. Re-implement on wgpu rather than depending on the stale fork.
2. **Substrate = EmberSim wgpu scaffold.** Start from `engine/ember_app/`'s `State` + `ApplicationHandler` (instance/surface/adapter/device/queue/config, sRGB, resize, clear render). Extend with a simple mesh pipeline.
3. **Mesh = zmesh.** Import the CPU mesh layer as a standalone crate (OBJ load/write, `TriMesh`, decimation, vertex dedup) + wgpu-facing conversion helpers.
4. **Spec = faultline_core.** Use the RenderKit/SceneKit boundary as the contract: a retained scene that extracts into a per-frame render frame, opaque handles, and a debug-draw sidecar.

**The one decision that shapes everything**: this must be a **wgpu port**. `three-d` is a GL codebase that is unused by owned code; building on it would couple us to a deprecated backend and a stale fork.

## 4. Realistic first scope

A `vs_renderkit` workspace with **three crates** under `libs/` (added to the root `workspace.members`):

```
libs/
├── vs_renderkit_mesh/   # crate: vs_renderkit_mesh
├── vs_renderkit_core/   # crate: vs_renderkit_core
└── vs_renderkit_egui/   # crate: vs_renderkit_egui
```

- **`vs_renderkit_mesh`** (from zmesh, no GPU deps): `TriMesh { faces: Vec<[[f64;3];3]> }`, OBJ load/write (`wavefront_obj` 11), decimation, vertex dedup; a `MeshData` conversion layer (de-duplicated `Vec<[f32;3]>` positions + `Vec<u32>` indices + optional normals) with vertex/index buffer helpers behind a `wgpu` feature. Skip subdivision (stub).
- **`vs_renderkit_core`** (from EmberSim, the wgpu engine): `Instance` (`windowed()`/`headless()`), `Renderer` (owns `Adapter`/`Device`/`Queue`, optional `Surface`/`SurfaceConfiguration`), `resize()`, `render()` (clear pass + a simple mesh pipeline), `Shader`/`Pipeline` helpers, `Camera` + a lightweight `RenderFrame`/`RenderPass` frame/pass boundary (faultline concept). Milestone: windowed app that clears to a color and draws one indexed mesh.
- **`vs_renderkit_egui`** (optional, feature-gated): eframe + `Renderer::Wgpu` shell; render the offscreen scene to a `wgpu::Texture`, display via `egui::TextureHandle`; later `egui_wgpu::Callback`.
- **Headless path**: `Renderer::headless()` + `render_to_texture` → encode to PNG, run in CI.

## 5. Notes / risks

- **`three-d` is stale GL, and unused** — don't vendor it. It drags in the wrong backend and a deprecated winit.
- **wgpu / winit / edition drift** is the biggest practical risk: EmberSim uses wgpu 29.0.3 + winit 0.30 + edition 2024; the egui/eframe ecosystem is 0.31–0.34 and pulls a different wgpu minor. Pin `vs_renderkit_core` to the same wgpu major as the chosen `eframe`, or bridge through `egui_wgpu`.
- **zmesh gaps**: `TriMesh` is `f64`, non-indexed, O(n²) dedup. Plan for `f64`→`f32`, index generation, faster dedup.
- **No owned shaders**: the first pipeline needs WGSL authored from scratch (minimal textured/unlit vertex+fragment); three-d's GLSL is not directly portable.
- **Position vs Rerun**: Rerun is the de-facto 3D viewer today (`vs-viz`). Position `vs_renderkit` as the **in-process/embeddable** path (an interactive 3D view inside a `vs_appkit` tool), not a Rerun replacement.
- **Don't leak GPU resources**: use opaque handles and a frame/pass boundary; keep domain types out of the renderer.
- **Keep the mesh crate CPU-only by default**; put GPU conversion behind a feature.

## 6. Next steps

1. **Lock the wgpu/egui version matrix** — the gating decision.
2. **Add the workspace members**: `libs/vs_renderkit_{mesh,core,egui}`.
3. **Port `vs_renderkit_mesh`** from zmesh (standalone, testable, no GPU): `TriMesh`, OBJ load/write, decimation, dedup, `MeshData` + vertex/index helpers behind a feature.
4. **Port `vs_renderkit_core`** from EmberSim: `Instance` (windowed/headless), `Renderer` (device/queue, surface config, resize), `render()` clear pass → add a minimal WGSL shader + mesh pipeline.
5. **Add the frame/pass boundary** (lightweight `RenderFrame`/`RenderPass` extraction) porting the faultline boundary rather than a full SceneKit.
6. **Build `vs_renderkit_egui`** (feature-gated): eframe + `Renderer::Wgpu`, render to a texture, display via `egui::TextureHandle`; fill the `egui_wgpu::Callback` TODO if time permits.
7. **Add a headless screenshot path** (offscreen render → PNG) as a CI smoke test.
8. **Write a small demo** loading a zmesh OBJ, drawn windowed and embedded.
9. **Unit tests** for the pure math (mesh conversion/dedup, camera) + a headless render smoke test.

## 7. Links

- Idea: [`docs/ideas/vs_renderkit.idea.md`](../ideas/vs_renderkit.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
