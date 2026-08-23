---
title: Repo Index
type: design
status: active
tags:
  - index
  - agents
---

# Repo Index

Cross-repo survey that backs the `docs/plan/*.plan.md` files. This indexes what
each `victoryforphil` / `AndreasLabs` repo contains that is relevant to porting
reusable `vs-*` crates into this workspace, and records which implementation is
the "best version" to port (weighted by recency + coupling).

The plans in `docs/plan/` link back here. Add new repos/notes here when an agent
surveys more of the org.

## Method

- Cloned candidate repos to `~/repos/vfp/{repo}` and `~/repos/AndreasLabs/{repo}`
  (private repos included; git over SSH via `gh`).
- Used `gh search code` (reaches private repos) to find implementations by symbol
  across the org, plus recursive tree/contents inspection of the clones.
- Weighted implementations by: completeness of abstraction, recency (git log
  dates), and coupling to the legacy `wingman-*` / `victory-*` (now `vs-*`)
  crates.

## Key org facts

- `victoryforphil/project-firefly` (private, Feb–Apr 2025) is **the source repo**
  for most of these ideas: `forest` (scenario runner), `valley` (validation),
  `waldo` (dir locator), `whisper` (MAVLink), `lit-rerun` (Rerun), `showkit-rs`,
  plus the `wingman-*` core/data/task crates.
- The `AutonomyAge` workspace (`vs-*`) is a **rename** of the Oct-2024
  `victory-*` repos (`victory-broker`→`vs-broker`, `victory-data-store`→
  `vs-data-store`, `victory-time`→`vs-wtf`). The Feb-2025 `project-firefly` code
  compiles against `wingman-*`, which is **API-divergent** from `vs-*` (see
  `OPEN_RISKS.md`). Porting firefly crates means a real migration, not a rename.
- No `forest` / `valley` / `victory-dir` / `vs-dir` crate exists anywhere yet.
  `three-d` is essentially unused by owned code (only a stale fork exists).

## Repos and what they have

### Project-firefly family (the primary sources)

- **project-firefly** (Rust workspace, private, ~Apr 2025)
  - `src_tools/forest/` — **forest** scenario runner. File-driven `config.yaml` +
    pluggable `ForestMiddleware` + `Commander` commands (trigger→action) +
    `ShowRunner` (`show.yaml`). Heavily coupled to `wingman-*`.
  - `src_core/valley/` — **valley** validation. `Validator` trait,
    `ValidationResult` (Passing/ExitSuccess/ExitFailure/Failing), `SILValidator`
    runner, 5 validators (pose/topic/frequency/num_datapoints/timeout). Coupled to
    `wingman-*`; needs a pose type (no `SkyPose` in `vs-wtf`).
  - `src_core/waldo/src/dir_utils.rs` — **victory-dir** source. `get_firefly_dir()`
    = `FIREFLY_DIR` env or `CWD/.firefly`, create-if-missing.
  - `src_flight/whisper/src/mavlink/` — **quadlink** candidate. Full framework
    (`core.rs` transport+threads, `builders/*`, `processors/*`, `helpers.rs`,
    `identifiers.rs`, `ardu_mode.rs`). Newer but coupled to protobuf
    `whisper::common` types + `wingman-*`.
  - `src_tools/lit-rerun/` — **vs-viz** candidate (rerun 0.21). `LitRerun`
    session/mode skeleton; no `vs-*` integration.
  - `src_ground/designer/showkit-rs/` — `rerun_preview.rs` + OBJ→points gen.
  - `src_ground/ground_control/` — conductor/gcs-bridge/gcs-cli/gcs-tauri.

- **lil-hopps** (Rust, ~Nov 2024) — the most `vs-*`-aligned family.
  - `lil-link/src/mavlink/` — **best quadlink base**. Public types named
    `QuadLinkCore`/`QuadLinkError`/`QuadlinkSystem`; `builders/*`,
    `processors/*`, `helpers.rs`, `system.rs` (+`common/types/*` pure-serde
    `Quad*` structs). Already implements `BrokerTask` against `victory-*`
    (=`vs-*`). Weakness: no graceful shutdown.
  - `lil-rerun/` — **best vs-viz design** (rerun 0.19). `RerunMode`/run-id,
    `DataView` topic iteration, `Primitives`→Rerun mapping, `Timepoint`→timeline.
    Written against now-drifted `vs-*` API.
  - `lil-quad`, `lil-gcs`, `lil-launcher` — consumers of `lil-link`.

- **basher** (Rust, ~Oct 2024) — `basher_rerun/` (rerun 0.18) + `system/viz/`.
  Best `RerunQuadPose::log_scalar/log_vector/log_pose` helper. Time-only `vs-*`
  integration.

- **victory-broker / victory-data-store / victory-time** — the Oct-2024 originals
  that `vs-*` are renamed from. Useful only to understand the target API.

### App / UI / CLI

- **agentbox** (Rust, ~Jun 2026) — strongest modern helpers.
  - `crates/tremor-nodekit/` — **best victory-logging base** (`init_logger(filter,
    dir, file)` with RUST_LOG precedence + guarded `non_blocking` file writer +
    `OnceLock<WorkerGuard>`) and **best victory-dir model** (`tremor_home()`,
    `logs_dir()`, `data_dir()`, `ensure_repo_symlink()`, HOME/USERPROFILE aware).
  - `crates/tremor-ui/` — **best vs_appkit base** (plain `egui` 0.34,
    no `eframe`): `UiOptions`/`UiTheme`, `Toolbar`, `fonts`, `icons`.
  - `nodes/tremor-visor/`, `nodes/tremor-chat/` — eframe panel + `egui_dock`
    shells. `nodes/tremor-executor/src/project_context.rs` — marker-based repo-root
    discovery.
  - `labs/hello_rig`, `labs/hello_zenoh` — duplicate `init_logger`.

- **mad-rs** (Rust + Bevy, ~Mar 2026) — rich egui/UI kit (`cursed/mad_common/`).
  - `ui/theme/{palette,style,frames,fonts}.rs` — high-quality theming.
  - `ui/widgets/` — `modal_scaffold`, `debug_pane_shell`, `debug_dock_state`,
    `sparkline`, `stats_grid`, `trace_overlay`, `performance_overlay`.
    Most are `bevy_egui::egui` (portable by swapping import); a few
    (`performance_overlay_*_plugin.rs`, `draw_performance_window.rs`,
    `derive(Resource)`) are Bevy-bound.
  - `camera/mad_pan_camera.rs` — pure pan/zoom math.
  - `logging.rs` — another `init_logger` (env_filter + compact stdout + rolling
    file). `mad_dir_utils.rs` — `.madrs` repo-relative dir resolver + `find_dev_repo_root`.
  - `cursed/mad_config` — layered CLI config (defaults<file<env<cli).

- **underscore_quad** (Rust, ~Mar 2026) — clean eframe app scaffold.
  `src/ui/{mod,app,theme}.rs`, `top_bar.rs`, `status_bar.rs`, `video_view.rs`.
  Best `apply_theme` + panel-layout scaffolding + state-struct→`draw(ui,&state)`.

- **tui_kit** (Rust, standalone) — `vfp_tuikit` crate: cleanest reusable CLI/TUI
  primitives (`cli` renderers + `tui` picker + palette/theme/metrics). Supersedes
  `chell`'s `ui.rs`.

- **chell** (Rust, jun 2026) — monolithic clap CLI. Reusable: TOML default
  read/write, `~/.chell` config dir, `--format auto|pretty|table`.
  Terminal UI superseded by `tui_kit`.

- **cursed-kit** — egui/eframe 0.28 template (low quality, demo only).

- **rerun-interview** — eframe 0.31 + wgpu viewer, but a **custom telemetry SDK**
  (`rust_sdk`), **not** a Rerun wrapper. `viewer/`, `python_sdk/`.

### Rerun / viz

- **AndreasLabs/loki** (Rust, ~Jun 2026) — `tools/firmware_buddy/src/rerun_bridge.rs`
  (rerun 0.23). Best multi-sink `RerunOptions{spawn,save,app_id}` bridge; has
  `docs/designs/rerun-serial-logging.md`.
- **AndreasLabs/SkyCanvas** (Rust gen2 + Python, ~Jan 2026) — UAV automation
  (ArduPilot longs-exposure). `gen2/quad_app/src/common/log_rerun.rs` is the
  **newest, cleanest small Rerun wrapper** (rerun 0.28.2): `log_status_text`,
  `log_lla` (`GeoPoints`), `log_ned` (`Points3D`). Also `link/mav_io`,
  `mav_tasks/*`.
- **victoryforphil/firewatch** — oldest Rerun usage (0.17, gRPC image→Rerun).

### MAVLink others

- **victoryforphil/project-devore** (~Nov 2025) — `quad/src/ardulink/` has the
  **best transport lifecycle** (Vec<JoinHandle>, `start_thread`/`stop_thread`
  that joins, `should_stop` gate). But its pubsub/exec/auto architecture has no
  `vs-*` integration.
- **cursed-mav** — CLI scaffold (TODO); the only repo pinning `mavlink 0.14.1`.
- **victory-ground-station**, **tiny-gcs**, **delores**, **project-delores** — empty
  / scaffold (no source).
- **underscore_quad** — DJI Tello uses UDP binary, **not** MAVLink.
- **AndreasLabs/mavsight**, **project-mariposa** — no Rust MAVLink link layer.

### Rendering / mesh

- **AndreasLabs/EmberSim** (Rust, ~May 2026) — `engine/ember_app/` is the **best
  wgpu 29 + winit 0.30 scaffold** (device/surface/queue/config + resize/render
  loop). `ember_dir` is a stub.
- **AndreasLabs/zmesh** (Rust) — `zmesh-lib`: CPU OBJ load/write, `TriMesh`,
  decimation (subdivision is a stub). Good mesh-asset companion.
- **victoryforphil/three-d** — stale fork snapshot (v0.18, glow/GL). Feature-complete
  *design* (scene/object/mesh/material/light/camera/control/effect/headless/egui-gui)
  but GL, not wgpu. **Best architectural blueprint for vs_renderkit.**
- **victoryforphil/rendering-learning** — `learn-wgpu-rust` (wgpu 29 scaffold),
  `game-of-questioning-life` (C/Vulkan Bazel), OpenGL book clone, `raytracer_weekend_cpp`.
- **victoryforphil/faultline_games** (C++/bgfx/SDL3, ~Apr 2026) — monorepo engine.
  `engine/faultline_core` RenderKit (frame+pass buckets, `IRenderer::submit`),
  SceneKit (node graph, MeshNode/LightNode, debug-draw sidecar), MathKit (unit-aware
  math), AppKit/GameKit/ToolKit; `faultline_uikit` (ImGui). **Best renderer
  *architecture* reference** (not portable to Rust/wgpu as-is).
- **glow_but_higher** — vendored `glow` 0.13.2 bindings (dependency under three-d).
- **vulkan-fun**, **ray-ray**, **AndreasLabs/tectonic** — empty/learning,
  nothing reusable.

## Leads vs. stuck points

- **Strong leads:** `victory-dir` (waldo + tremor-nodekit model), `victory-logging`
  (tremor-nodekit + dark-factory/tinyverse), `quadlink` (lil-link + whisper +
  project-devore), `vs-viz` (lil-rerun design + SkyCanvas/loki helpers),
  `vs_appkit` (tremor-ui + mad_common + underscore_quad), `forest`/`valley`
  (project-firefly, the only implementations).
- **Thin / stuck:** `vs_renderkit` — no owned `three-d` usage; fork is stale GL.
  Plan around a wgpu port using the three-d design + EmberSim scaffold + zmesh.
  `forest`/`valley` porting effort is high (full `wingman-*`→`vs-*` migration +
  missing `SkyPose` type).
- **Unavailable by code-search:** several relevant private repos (`dark-factory`,
  `tinyverse`, `cursed-tanks`) were not cloneable here; their logging helpers were
  read via GitHub raw and are marked as such.
