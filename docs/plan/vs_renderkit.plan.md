---
title: vs_renderkit
type: plan
status: todo
tags:
  - plan
  - rendering
  - wgpu
  - vs-renderkit
  - version-matrix
  - 3d
---

# vs_renderkit (Plan — UPDATED)

> Research track UPDATED 2026-08-23. This revision **resolves the wgpu/winit/edition + eframe version matrix** (the gating decision), locks the **mesh crate default**, and produces a **concrete three-crate layout + first-milestone API** with signatures. Supersedes the original plan draft.

A composable 3D rendering kit for the AutonomyAge `vs-*` Rust workspace. This is a **wgpu rebuild** (not a `three-d` port); the value comes from porting the `three-d` *architecture*, the EmberSim wgpu-29 *scaffold*, and the zmesh *mesh asset* code onto one coherent, version-locked substrate.

## 1. Purpose

- Give every `vs-*` tool a clean way to build and render 3D scenes without owning wgpu boilerplate (instance → surface → adapter → device → queue → config → render pass).
- Be **composable** (three opt-in crates, mirroring the `vs_appkit` split): `vs_renderkit_mesh`, `vs_renderkit_core`, `vs_renderkit_egui`.
- Support three embed modes with one core renderer: **windowed** (winit/wgpu shell), **embedded** (render to offscreen `wgpu::Texture`, shown in an `egui` image), **headless** (offscreen → PNG, for screenshots/CI).
- Port the strongest design (three-d scene/object/material/light/camera/control) onto a wgpu substrate, reusing EmberSim (wgpu 29 scaffold) and zmesh (CPU mesh: OBJ load/write, decimation, dedup).
- Keep domain types out: the renderer consumes plain data structs (`MeshData`, `Camera`, handles), not `vs-wtf`/`vs-data-store`/`vs-broker` types.

## 2. Version matrix — RESOLVED ✅

Empirically read from the reference repos and `crates.io` dependency graphs on **2026-08-23**. The eframe→egui-wgpu→wgpu→winit chain is the constraint.

**Recommended coherent set (ONE):**

| Component | Version | Rationale |
|-----------|---------|-----------|
| **wgpu** | `29.0.3` | Direct read from **EmberSim** `engine/ember_app/Cargo.toml`. Stable, demonstrated scaffold. |
| **winit** | `0.30.13` | Read from EmberSim; also the exact pin in eframe 0.34/0.35/0.36. |
| **edition** | **2024** | EmberSim uses it; rustc **1.95.0** supports it. Matches the reference scaffold. |
| **eframe / egui / egui-wgpu** | **0.35.0** | Bundles **wgpu ^29** + winit ^0.30.13 → exact match with core's wgpu 29. |
| **pollster** | `0.4.0` | EmberSim; used to block on async device/queue. |
| **anyhow** | `1.0` | Existing vs-* convention; error plumbing. |
| **image** | latest | Headless PNG encode (CI smoke test). |

**Version-bundle mapping (verified against crates.io dependency metadata):**

| eframe | bundled wgpu | bundled winit | notes |
|--------|--------------|----------------|-------|
| 0.31.0 | ^24.0.0 | ^0.30.7 | too old for wgpu-29 core |
| 0.33.0 | ^27.0.1 | ^0.30.12 | wgpu 27 ≠ 29 |
| **0.34.0** | **^29.0.0** | ^0.30.13 | matches, egui 0.34 |
| **0.35.0** | **^29.0** | ^0.30.13 | **matches, egui 0.35 — chosen** |
| 0.36.0 | ^30.0 | ^0.30.13 | wgpu 30 (too new / pre-release) |

**Trade-off:** choosing **eframe 0.35** (2026-06-25) keeps wgpu on stable **29** — the exact version EmberSim already builds + runs — rather than jumping to eframe 0.36 (2026-08-07) which bumps to **wgpu 30**, a version not yet demonstrated by any reference repo and likely pre-release. The cost is being one egui minor behind the very latest. If wgpu 30 stabilizes and a reference appears, bump core + eframe together in one commit (the `egui_wgpu::Callback` bridge is the only wgpu-touching surface in the egui crate).

**Consequence for interop:** `vs_renderkit_core` pins `wgpu = "29.0.3"`; `vs_renderkit_egui` pins `eframe`/`egui`/`egui-wgpu = "0.35"`. Because both resolve to **wgpu 29**, the core renderer's `wgpu::Device`/`Queue`/`Texture` can be shared with `egui-wgpu` (currently via CPU readback → `egui::TextureHandle`; later natively via `egui_wgpu::Callback`). Do **not** let Cargo resolve two wgpu majors — that breaks texture/device sharing.

**wgpu 29 API deltas to code against (read from EmberSim `app_state.rs`):**
- `Instance::new(InstanceDescriptor { backends: Backends::PRIMARY, flags: Default::default(), memory_budget_thresholds: Default::default(), backend_options: Default::default(), display: None })`.
- `instance.create_surface(window)` returns `Surface` (with `'static` lifetime when the window is `Arc<Window>`).
- Headless: prefer `instance.request_adapter(&RequestAdapterOptions { compatible_surface: None, .. })`; windowed: `instance.enumerate_adapters(...)` + filter `adapter.is_surface_supported(&surface)`.
- `adapter.request_device(&DeviceDescriptor { label, required_features, experimental_features: ExperimentalFeatures::disabled(), required_limits: Limits::default(), memory_hints, trace: Trace::Off })`.
- `surface.get_current_texture()` returns a `Result<_, CurrentSurfaceTexture>` enum in wgpu 29 (EmberSim matches `Success`/`Suboptimal`/`Timeout`/`Occluded`/`Validation`/`Outdated`/`Lost`) — the old `SurfaceError` shape is gone.
- `RenderPassColorAttachment` has a `depth_slice` field; `RenderPassDescriptor` has `multiview_mask` and `occlusion_query_set`. Surface config uses `desired_maximum_frame_latency`.

## 3. Mesh crate default — DECIDED ✅

**`vs_renderkit_mesh` stays CPU-only by default.** It has **no wgpu dependency** unless you opt in. GPU conversion lives behind a `wgpu` cargo feature.

- **Default features:** `[]` (CPU only). Deps: `wavefront_obj = "11"`, `anyhow`, `thiserror`, optional `rayon`.
- **Optional feature `wgpu`:** pulls `wgpu = "29"` and exposes `GpuMesh` / buffer builders. `vs_renderkit_core` enables it when it depends on the mesh crate.
- **Optional feature `parallel`:** `rayon` for QEM decimation (mirrors zmesh's `parallel` flag).

**`MeshData` type (f32, indexed, GPU-ready):**

```rust
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,  // de-duplicated vertex positions
    pub indices:   Vec<u32>,       // triangle list (len = 3 * triangle_count)
    pub normals:   Vec<[f32; 3]>,  // per-vertex; empty until compute_normals()
}
```

**OBJ load/write/decimation scope to port from zmesh** (not the whole crate):

| Port | From zmesh | Scope |
|------|-----------|-------|
| `TriMesh { faces: Vec<[[f32;3];3]> }` | `mesh.rs` | **f64 → f32** for vertex-buffer compat; keep as the CPU ops intermediate. |
| `load_obj` / `load_obj_str` | `loaders/loader_obj.rs` | `wavefront_obj::parse` → convert to `MeshData` (with dedup + index generation). |
| `write_obj` / `write_obj_str` | `write/write.rs` | Serialize `MeshData`/`TriMesh` back to OBJ text. |
| `decimate(target_faces)` QEM edge-collapse | `operations/decimation.rs` (842 lines) | Port the QEM (quadric error metric) edge-collapse; keep its `parallel`/`sequential` split. |
| vertex dedup + index generation | `mesh.rs::deduplicate_vertices` | Replace the O(n²) loop with a **spatial hash** (the existing `SpatialHash` in `decimation.rs` is reusable) so it scales. |
| `compute_normals()` | new | Cross-product per face, accumulate to vertices, normalize. |
| subdivision | `operations/subdivision.rs` | **Skip** — it is a stub in zmesh. |

The mesh crate returns `MeshData` for the GPU path and keeps `TriMesh` only where a CPU pipeline needs the raw face soup. Seed data: port a small `sphere.obj`-style fixture into `tests/data/` so the round-trip + decimation tests (ported from zmesh) run under `cargo test`.

## 4. Concrete three-crate layout + first-milestone API

```
libs/
├── vs_renderkit_mesh/   # CPU-only, optional wgpu/parallel features
├── vs_renderkit_core/   # wgpu 29 engine; depends on mesh (wgpu feature)
└── vs_renderkit_egui/   # eframe 0.35 shell (feature-gated); depends on core
```

Add all three to the root `workspace.members`.

### 4.1 `vs_renderkit_mesh` — types + functions (signatures)

```rust
// lib.rs
pub mod mesh;      // MeshData, TriMesh, dedup, normals
pub mod loaders;   // obj
pub mod write;     // obj
pub mod operations;// decimate (qem)

// mesh/mesh_data.rs
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub indices:   Vec<u32>,
    pub normals:   Vec<[f32; 3]>,
}
impl MeshData {
    pub fn empty() -> Self;
    pub fn from_faces(faces: &[[[f32; 3]; 3]]) -> Self;   // dedup + index
    pub fn vertex_count(&self) -> usize;
    pub fn triangle_count(&self) -> usize;
    pub fn compute_normals(&mut self);                    // flat (per-vertex accumulate)
}

// mesh/tri_mesh.rs  (CPU ops intermediate)
pub struct TriMesh { pub faces: Vec<[[f32; 3]; 3]> }
impl TriMesh {
    pub fn new() -> Self;
    pub fn from_obj(obj: &wavefront_obj::obj::ObjSet) -> Self;
    pub fn face_count(&self) -> usize;
    pub fn to_mesh_data(&self) -> MeshData;              // dedup + index
}

// loaders/obj.rs
pub fn load_obj(path: impl AsRef<Path>) -> Result<MeshData>;
pub fn load_obj_str(text: &str) -> Result<MeshData>;

// write/obj.rs
pub fn write_obj(mesh: &MeshData) -> Result<String>;
pub fn write_obj_path(mesh: &MeshData, path: impl AsRef<Path>) -> Result<()>;

// operations/decimate.rs
pub fn decimate(mesh: &MeshData, target_faces: usize) -> Result<MeshData>;

// gpu.rs  (behind `feature = "wgpu"`)
pub use wgpu;
pub struct GpuMesh {   // owns GPU buffers
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer:  wgpu::Buffer,
    pub vertex_count:  u32,
    pub index_count:   u32,
}
pub fn upload_mesh(device: &wgpu::Device, queue: &wgpu::Queue, mesh: &MeshData) -> GpuMesh;
```

### 4.2 `vs_renderkit_core` — the wgpu engine (signatures)

```rust
// lib.rs
pub mod instance;   // Instance (windowed/headless)
pub mod renderer;   // Renderer + render_to_texture / read_*_png
pub mod shader;     // Shader + pipeline helpers
pub mod camera;     // Camera + CameraUniform
pub mod frame;      // RenderFrame / RenderPass / PassItems (faultline boundary)

// instance.rs
pub struct Instance { pub instance: wgpu::Instance }
impl Instance {
    pub fn new(window: Option<wgpu::SurfaceTarget<'_>>, backends: wgpu::Backends) -> Self;
    pub fn windowed(backends: wgpu::Backends) -> Self;
    pub fn headless(backends: wgpu::Backends) -> Self;      // no surface
}
// Async device/queue acquisition (pollster wraps these for non-web):
pub async fn init_device(instance: &Instance) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue)>;
pub async fn init_surface(instance: &Instance, window: &Arc<Window>)
    -> Result<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>;

// renderer.rs  (owns device/queue + optional surface)
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
}
impl Renderer {
    pub async fn windowed(window: Arc<Window>) -> Result<Self>;
    pub async fn headless() -> Result<Self>;
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn render(&mut self, frame: &RenderFrame) -> Result<()>;            // surface path
    pub fn render_offscreen(&mut self, frame: &RenderFrame, width: u32, height: u32)
        -> Result<wgpu::Texture>;                                          // headless + embedded
    pub fn read_texture_png(&mut self, texture: &wgpu::Texture, width: u32, height: u32)
        -> Result<Vec<u8>>;                                                // copy→map→png
    pub fn upload_mesh(&mut self, mesh: &MeshData) -> Result<GpuMesh>;     // from vs_renderkit_mesh
}

// shader.rs
pub struct Shader(pub wgpu::ShaderModule);
impl Shader {
    pub fn from_wgsl(device: &wgpu::Device, wgsl: &str) -> Result<Self>;
}
pub struct MeshPipeline {   // one vertex+fragment pipeline w/ bind group layout
    pub layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}
pub fn make_mesh_pipeline(device: &wgpu::Device, shader: &Shader, target_format: wgpu::TextureFormat, camera_bind_group_layout: &wgpu::BindGroupLayout) -> MeshPipeline;

// camera.rs  (column-major [[f32;4];4], matches wgpu mat4x4<f32>; port of faultline Camera)
pub struct Camera {
    pub eye: [f32; 3], pub target: [f32; 3], pub up: [f32; 3],
    pub fov_y_degrees: f32, pub aspect: f32, pub near: f32, pub far: f32,
}
impl Camera {
    pub fn build_view(&self) -> [[f32; 4]; 4];
    pub fn build_projection(&self) -> [[f32; 4]; 4];
    pub fn view_projection(&self) -> [[f32; 4]; 4];
}
#[repr(C)] pub struct CameraUniform { pub view_proj: [[f32; 4]; 4] }

// frame.rs  (lightweight faultline RenderKit boundary)
pub enum RenderPass { Shadow, MainOpaque, MainTransparent, Debug, UI }
pub struct MeshInstance { pub mesh: MeshHandle, pub transform: [[f32; 4]; 4] }
#[derive(Clone)] pub struct MeshHandle(pub std::sync::Arc<GpuMesh>);
pub struct PassItems {
    meshes: Vec<MeshInstance>,
}
impl PassItems {
    pub fn mesh(&mut self, mesh: MeshHandle, transform: [[f32; 4]; 4]) -> &mut Self;
    pub fn clear(&mut self);
    pub fn meshes(&self) -> &[MeshInstance];
}
pub struct RenderFrame {
    pub camera: Camera,
    passes_: Vec<(RenderPass, PassItems)>,
}
impl RenderFrame {
    pub fn pass(&mut self, pass: RenderPass) -> &mut PassItems;  // lazy-reuse bucket
    pub fn passes(&self) -> &[(RenderPass, PassItems)];
    pub fn clear(&mut self);
}
```

**First milestone (windowed):** `Instance::windowed` → `init_surface`/`init_device` → `Renderer`; a `Camera` on a `RenderFrame`; one `MeshData` (generated cube/sphere or an OBJ fixture) uploaded via `Renderer::upload_mesh` → `GpuMesh`; a `Shader::from_wgsl` (unlit vertex+fragment) → `make_mesh_pipeline`; `render()` clears to a color, binds the camera uniform, and draws the indexed mesh.

### 4.3 `vs_renderkit_egui` — feature-gated shell (signatures)

Enabled behind `feature = "egui"` (default `["egui"]` for the crate, but `egui` is the only heavy dep). Uses **eframe 0.35** + `Renderer::Wgpu`.

```rust
// lib.rs
pub struct RenderView {          // owned by the egui app state
    pub renderer: vs_renderkit_core::Renderer,
    pub mesh: vs_renderkit_mesh::MeshData,
    pub texture: Option<egui::TextureHandle>,
    pub frame: RenderFrame,
}
impl RenderView {
    pub fn new() -> Result<Self>;
    pub fn draw(&mut self, ui: &mut egui::Ui);          // figure out size, render_offscreen, show image
}

pub fn run_app(title: &str, view: RenderView) -> eframe::Result<()>;
// eframe::run_native(..., NativeOptions { renderer: Renderer::Wgpu, .. }, ...)
```

**Embed mode (milestone):** in `draw`, size the offscreen target to the widget rect, call `renderer.render_offscreen(&frame, w, h)`, read back to `egui::ColorImage` (CPU readback — simple but a copy), and `ui.image((texture_id, size))`. This is the `underscore_quad` `video_view.rs`/`TextureHandle` pattern.

**Later (`egui_wgpu::Callback`):** instead of CPU readback, submit the scene's draw directly into egui's wgpu render pass via `egui_wgpu::Callback` (fills the `rerun-interview/viewer/src/viewer.rs` TODO at line 149 — "optimize point rendering by using `egui_wgpu::Callback`"). Requires sharing the `wgpu::Device`/`Queue` that eframe 0.35 owns, which is only coherent because both sides are wgpu **29**.

## 5. What to WRITE FRESH (WGSL shaders) + headless path

- **WGSL from scratch (none portable):** three-d is GLSL and cannot be reused. Author two minimal shaders, embedded via `include_str!` in `vs_renderkit_core`:
  - `mesh.vert.wgsl` — bind group 0 (camera uniform `view_proj`), per-vertex `position` + `normal` + optional `color`; output `clip_position`.
  - `mesh.frag.wgsl` — unlit (flat `color`/normal dot light) at first; add a single directional light for a basic shaded look later.
  - Optionally a separate pass for a wireframe/debug draw (matches the faultline `Debug` pass).
- **Headless screenshot path (CI smoke test):** `Instance::headless()` → `init_device` (no compatible surface) → offscreen `wgpu::Texture` sized to the render target with `TextureUsages::RENDER_ATTACHMENT | COPY_SRC` → `render_offscreen(&frame, w, h)` → `copy_texture_to_buffer` → `buffer.map_async`/`get_mapped_range` → `image` crate PNG encode → `read_texture_png(...)`. Run it as a `cargo test --test headless_smoke` that asserts a non-empty, non-uniform PNG.

## 6. Workspace / build wiring

- Root `Cargo.toml` `workspace.members`: add `"libs/vs_renderkit_mesh"`, `"libs/vs_renderkit_core"`, `"libs/vs_renderkit_egui"`.
- Set `edition = "2024"` in all three (rustc 1.95 supports it; matches EmberSim). Note existing `vs-*` libs are `2021` — edition is a per-crate style choice and does **not** affect interoperability.
- Crate deps: `core` → `vs_renderkit_mesh = { version, path, features = ["wgpu"] }`; `egui` → `vs_renderkit_core` + `eframe/egui/egui-wgpu 0.35`. `core` does **not** depend on `egui`; `egui` depends on `core`.
- The mesh crate's `default = []` (CPU only) is preserved; `wgpu` and `parallel` are opt-in.

## 7. Notes / risks

- **wgpu/winit/edition drift is now resolved:** matrix locked to wgpu 29.0.3 + winit 0.30.13 + edition 2024 + eframe/egui-wgpu 0.35. The remaining interop rule: never resolve two wgpu majors in one build.
- **three-d is stale GL + unused** — don't vendor it. It drags a deprecated backend/older winit.
- **zmesh gaps:** `TriMesh`/faces are `f64` and non-indexed; dedup is O(n²). Convert to `f32` + indexed `MeshData`, switch dedup to a spatial hash.
- **WGSL must be authored fresh**; no owned shaders exist. Minimal unlit first, light later.
- **CPU readback is a milestone shortcut** (slow). The `egui_wgpu::Callback` path is the eventual non-copy route; only runnable because both sides are wgpu 29.
- **Don't leak GPU resources:** opaque `MeshHandle(Arc<GpuMesh>)`, `RenderFrame::pass()` lazy-reuse, and a `clear()`/frame-boundary discipline (faultline contract). Domain types stay out of the renderer.
- **Position vs Rerun:** vs_renderkit is the in-process/embeddable interactive 3D view for a `vs_appkit` tool; Rerun stays the general 3D viewer. Don't drift into a full viewer (scope-creep risk).
- **Headless adapter selection:** `request_adapter` with `compatible_surface: None` may fall back to a software adapter; allow an env-var/env-var backend selection for CI determinism.

## 8. Concrete next steps (ordered)

1. **Add the three workspace members** with `edition = "2024"` and the pinned deps above.
2. **Port `vs_renderkit_mesh`**: `MeshData` (f32/indexed), `TriMesh`, `load_obj`/`load_obj_str`, `write_obj`, `decimate` (QEM), spatial-hash dedup, `compute_normals`; port zmesh tests + a `tests/data/sphere.obj` fixture. Keep CPU-only by default.
3. **Port `vs_renderkit_core`** from EmberSim: `Instance::windowed`/`headless`, `Renderer`, `resize`, `render` (clear), then add `Shader::from_wgsl` + `make_mesh_pipeline` + `upload_mesh`.
4. **Author the WGSL** (`mesh.vert.wgsl`/`mesh.frag.wgsl`) — unlit first.
5. **Add the frame/pass boundary** (`RenderFrame`/`RenderPass`/`PassItems`/`MeshInstance`/`Camera`) as the lightweight faultline port — do not build a full SceneKit.
6. **Add `Renderer::render_offscreen` + `read_texture_png`** → headless CI smoke test.
7. **Build `vs_renderkit_egui`** (eframe 0.35): offscreen → `ColorImage` → `TextureHandle` image display. Add `egui_wgpu::Callback` if time permits.
8. **Write a demo** loading the OBJ fixture, shown both windowed and embedded.
9. **Tests:** pure math (dedup/normals/decimation/camera) + headless render smoke.

## 9. Links

- Idea: `docs/ideas/vs_renderkit.idea.md`
- Sources: `docs/agents/repo-index.md`
- Reference scaffolds: `AndreasLabs/EmberSim/engine/ember_app/` (wgpu 29), `AndreasLabs/zmesh/zmesh-lib/` (mesh), `victoryforphil/faultline_games/.../RenderKit` (frame/pass spec), `victoryforphil/rerun-interview/viewer/` (eframe-embed reference, `egui_wgpu::Callback` TODO).
