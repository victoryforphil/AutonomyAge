---
title: vs-viz
type: plan
status: todo
tags:
  - plan
  - viz
  - rerun
---

# vs-viz (Plan)

A reusable visualization utilities/helpers crate for the AutonomyAge `vs-*` Rust workspace. It starts with **Rerun** as the first backend but is designed to be extensible to more sinks (custom viewers, plain logs). It provides thin, ergonomic wrappers over Rerun's API — log helpers, timestamps, scene building, standard telemetry payloads — behind a backend-agnostic core so other backends can be dropped in later. It integrates with `vs-wtf` time primitives and `vs-data-store` topics so data can be logged as it flows through the broker.

## 1. Purpose

- Give every `vs-*` app/drone/sim one shared visualization layer instead of each re-implementing Rerun plumbing.
- Provide a small, backend-agnostic core (`VizSession` + `VizSink`) free of `rerun` imports, so more sinks can be added without touching call sites.
- Ship a Rerun backend that wraps `RecordingStreamBuilder` modes (Save / Live / Spawn), sets `app_id`/`recording_id`, and writes to a repo-relative `.logs/{group}` dir.
- Standardize common telemetry concepts: scalar, text/status, pose, point clouds, geo/LLA, images.
- Integrate with `vs-wtf` time (`Timepoint` → Rerun timeline) and `vs-data-store` topics (`TopicKey` → Rerun entity path) so a `BrokerTask` can stream a `DataView` into a sink.
- Consolidate the Rerun usage scattered across VictoryForPhil/AndreasLabs into one crate.

## 2. Source implementations found

| Repo | Path | rerun version | Date | Notes |
|------|------|---------------|------|-------|
| lil-hopps | `lil-rerun/` | 0.19.0 | Nov 2024 | **Best design base.** `LilRerun { name, group, run_id, rerun: Option<RecordingStream>, rerun_override }`, `RerunMode { Save, Live, Spawn }`, `create_rerun()` (`.save/.spawn/.connect`, `.recording_id(run_id)`, `app_id = "{group}/{name}"`), `get_rerun_mode()`/`get_rerun_env()` (env `RERUN_MODE`, default `Save`), save path `.{root}/logs/{group}`. `RerunSystem` implements `victory_broker::task::BrokerTask`, maps a `DataView` (match `Primitives` → `Scalar`/`TextDocument`) and `Timepoint`→`set_time_seconds`. Scene building (`log_static world/floor/asset`, pose marker via `Boxes3D + quaternions`). **Warning**: written against old `victory-*` crate names — must rename to `vs-*` and re-verify APIs. |
| project-firefly | `src_tools/lit-rerun/` | 0.21.0 | Feb 2025 | Same `LitRerun`/mode skeleton; no `vs-*` integration; README is stubs. Confirmatory only. |
| basher | `src/basher_rerun/` | 0.18.2 | ref | Same `RerunMode` skeleton. `RerunQuadPose::log_scalar/log_vector/log_pose` — pose decomposition (position/velocity/angular/acceleration, path-prefixed) + `Points3D`. **Best pose helper.** |
| AndreasLabs/loki | `tools/firmware_buddy/src/rerun_bridge.rs` | 0.23.0 | Jun 2026 | **Best multi-sink pattern.** `RerunOptions { spawn, save: Option<PathBuf>, app_id }` and a `Vec<RecordingStream>` (spawn + save coexist). |
| AndreasLabs/SkyCanvas | `gen2/quad_app/src/common/log_rerun.rs` | 0.28.2 | Jan 2026 | **Newest & cleanest small wrapper.** `LogRerun { name, rec }`; helpers `log_status_text` (`TextLog`), `log_lla` (`GeoPoints`), `log_ned` (`Points3D`). |

Rejected / non-Rerun: `rerun-interview` (custom telemetry SDK), `rerun-vl53` (empty), `underscore_quad` (no rerun), `firewatch` (old 0.17, no `vs-*`).

**Rerun version drift is the #1 risk**: 0.17 / 0.18.2 / 0.19 / 0.21 / 0.23 / 0.28.2 across repos. Pin one (target ~0.28.x) and gate the rerun layer behind an optional cargo feature.

## 3. Best version to port

Port the **lil-rerun design as the shape** (backend-agnostic session/mode/skeleton), then modernize with three grafts:

1. **Multi-sink streams from loki `rerun_bridge.rs`**: hold a `Vec<RecordingStream>` (Save + Live/Spawn coexist) behind a `RerunOptions { spawn, save: Option<PathBuf>, app_id }` config.
2. **SkyCanvas `log_rerun.rs` helpers as the `VizSink` surface**: lift `log_status_text` (`TextLog`), `log_lla` (`GeoPoints`), `log_ned` (`Points3D`).
3. **basher `RerunQuadPose` decomposition for `log_pose`**, plus an orientation marker (`Boxes3D + quaternions + centers`) from lil-rerun.

Then re-map `victory_*`→`vs_*` and confirm every `vs-data-store`/`vs-broker`/`vs-wtf` API call. Do **not** port `rerun::Logger` (removed in newer Rerun).

### Crate shape

New member under `libs/`: `libs/vs-viz` (package `vs-viz`, crate `vs_viz`).

```
libs/vs-viz/src/
  lib.rs        # re-exports VizSession, VizMode, VizSink, VizPath, VizPose, VizGeoPoint, VizImage, RerunSink, RerunOptions, scene helpers
  session.rs    # VizSession + VizMode (backend-agnostic; NO rerun imports)
  sink.rs       # trait VizSink + shared payload types (backend-agnostic)
  backend.rs    # Backend trait + registry + NullSink
  path.rs       # VizPath (topic -> rerun path)
  telemetry.rs  # standard payload builders
  rerun/        # RerunBackend, RerunSink, scene, pose, topic (behind `rerun` feature)
  system.rs     # VizSystem: impl vs_broker::task::BrokerTask that streams a DataView
```

Dependencies (align to workspace; `rerun` optional):

```toml
[features]
default = ["rerun"]
rerun   = ["dep:rerun"]

[dependencies]
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"
log = "0.4"
tracing = "0.1"
vs-wtf = { path = "../vs-wtf" }
vs-data-store = { path = "../vs-data-store" }
vs-broker = { path = "../vs-broker" }
rerun = { version = "0.28", optional = true }
nalgebra = "0.33"
```

## 4. Core API surface

Everything below is backend-agnostic (no `rerun` imports except inside `rerun/`).

**Session / mode (`session.rs`)**:
- `enum VizMode { Save, Live, Spawn }`.
- `struct VizSession { name, group, run_id, mode_override }` — `new(name, group)` (`run_id = Timepoint::now().ms()`), `app_id() = "{group}/{name}"`, `get_mode()`, `env_mode()` (env `RERUN_MODE`), `set_save/set_live/set_spawn`.

**Backend dispatch (`backend.rs`)**:
- `trait Backend { fn open(&self, session: &VizSession) -> anyhow::Result<Box<dyn VizSink>>; fn name(&self) -> &'static str; }`
- `struct BackendRegistry` + `NullSink` (no-op default) so `VizSink` is usable headless/in tests. The Rerun backend registers itself under the `rerun` feature.

**Sink trait (`sink.rs`)** — free of `rerun`; paths are `impl Into<VizPath>`; time is `vs_wtf::Timepoint`:
```rust
pub trait VizSink: Send {
    fn set_time(&mut self, t: &Timepoint) -> anyhow::Result<()>;
    fn set_duration(&mut self, t: &Timepoint) -> anyhow::Result<()>;
    fn log_scalar(&mut self, path: impl Into<VizPath>, value: f64) -> anyhow::Result<()>;
    fn log_text(&mut self, path: impl Into<VizPath>, text: &str) -> anyhow::Result<()>;
    fn log_points(&mut self, path: impl Into<VizPath>, pts: &[[f64; 3]]) -> anyhow::Result<()>;
    fn log_geo(&mut self, path: impl Into<VizPath>, lla: &[(f64, f64)]) -> anyhow::Result<()>;
    fn log_pose(&mut self, path: impl Into<VizPath>, pose: &VizPose) -> anyhow::Result<()>;
    fn log_image(&mut self, path: impl Into<VizPath>, img: &VizImage) -> anyhow::Result<()>;
    fn flush(&mut self) -> anyhow::Result<()>;
}
```
Shared payload types: `VizPose { position, velocity, angular_velocity, acceleration, angular_acceleration, attitude: [f64;4] }`, `VizGeoPoint { lat, lon }`, `VizImage { bytes, width, height, format }`, `Vec3([f64;3])` (keeps `nalgebra` out of the stable trait).

**Rerun layer (`rerun/`)**:
- `struct RerunOptions { spawn, save: Option<PathBuf>, connect: Option<String>, app_id }` + `RerunBackend` (builds `app_id`/`recording_id`, `Save`→`.save(...)`, `Spawn`→`.spawn()`, `Live`→`.connect()`; multiple streams in a `Vec<RecordingStream>`).
- `struct RerunSink` implements `VizSink`: `log_scalar`→`Scalar`, `log_text`→`TextDocument`/`TextLog`, `log_points`→`Points3D`, `log_geo`→`GeoPoints`, `log_pose`→decomposed scalars + `Points3D` (+ `Boxes3D` orientation marker), `log_image`→`Image`.
- Save dir: `{VS_VIZ_DIR | CARGO_MANIFEST_DIR}/.logs/{group}`, created with `create_dir_all`.

**Scene building (`rerun/scene.rs`)**: `log_world(sink, ..)` (`ViewCoordinates::RIGHT_HAND_Z_UP`), `log_floor(sink, size)` (`Boxes3D`), `log_asset(sink, path)` (`Asset3D`), `log_pose_marker`, `log_waypoints`, `log_home`.

**Topic mapping (`rerun/topic.rs`)**: `topic_path(topic)` via `topic.key().display_name()`; `log_dataview(sink, view)` drains `view.get_latest_map(&TopicKey::empty())` and dispatches each `Datapoint.value` (`Primitives`): `Float`/`Integer`→`Scalar`, `Text`→`TextDocument`, `Boolean`→`TextDocument`+`Scalar`, others logged as text or `warn`.

## 5. Integration notes

- **vs-wtf → `vs_wtf`**: `Timepoint`/`Timespan`/`Timecode`; `Timepoint::now().ms()` for run id; `secs()`/`ms()`/`us()`/`ns()` feed `set_time_seconds`/`set_duration_secs`. Prefer `timing.time_monotonic` over `Timepoint::zero()`.
- **vs-data-store → `vs_data_store`**: `DataView`, `TopicKey`, `TopicKeyProvider`, `Primitives`, `Datapoint`. **Gotcha**: `TopicKey::to_string()` returns the numeric id — always use `topic.key().display_name()` (or `format!("{}", key)`) for the Rerun entity path. `add_latest` takes a `Serialize` struct and flattens it; read `Datapoint.value` (a `Primitives`) directly.
- **vs-broker → `vs_broker`**: `task::BrokerTask`, `task::config::BrokerTaskConfig::new(name)`, `subscription::BrokerTaskSubscription` (built from a `TopicKey`), `trigger::BrokerTaskTrigger::Always / Rate(Timespan)`, `broker::time::BrokerTime`.
- **`rerun::Logger` is gone** — do not port it; wire `log`/`tracing` to a `TextLog` sink yourself if needed.
- **Feature-gating**: `rerun` behind an optional cargo feature so the core compiles without it.
- **Rename mapping**: `victory_wtf`→`vs_wtf`, `victory_data_store`→`vs_data_store`, `victory_broker`→`vs_broker`.

## 6. Risks / open questions

- **Rerun pin**: 0.17→0.28.2 drift. Recommend pinning `rerun = "0.28"` and gating the layer behind a feature. Confine churn to `rerun/rerun_sink.rs`.
- **Host: broker task vs simple loop**: ship `VizSystem` as a `BrokerTask` (recommended for app integration) and keep `VizSession`/`RerunSink` usable directly in a plain loop (standalone/tooling).
- **`create_session` scoping**: expose `RerunOptions { spawn, save, connect }` so Save + Live can coexist; keep `VizMode` as a simple convenience.
- **Topic namespace**: consider a per-app prefix so multiple vehicles don't collide; keep the unchanged topic path as the entity path.
- **Scene/asset convenience**: generalize via `log_asset(sink, path)`; do not hardcode a demo asset.
- **`Vec3`/nalgebra in the trait**: define a tiny `Vec3` payload to keep `VizSink` free of external geom crates; expose nalgebra behind a feature if needed.
- **Thread-safety**: `RecordingStream` is cheaply cloneable; decide the sharing model (wrap in `Arc<Mutex<...>>` if multiple tasks log concurrently).

## 7. Next steps

1. Add `libs/vs-viz` to the workspace `[members]` with the deps/features above.
2. Implement the backend-agnostic core first (`VizMode`, `VizSession`, `Backend`/`BackendRegistry` + `NullSink`, `VizSink`, payload types). Verify it compiles with **no** `rerun` feature.
3. Port the Rerun backend (`RerunOptions`, `RerunBackend`, `RerunSink`) with the SkyCanvas helper bodies.
4. Port the basher `RerunQuadPose` decomposition into `log_pose`, plus scene helpers (`log_world`/`log_floor`/`log_asset`/`log_waypoints`/`log_home`).
5. Write `rerun/topic.rs` (`topic_path`, `log_dataview`) and `system.rs` (`VizSystem: BrokerTask`).
6. Add a smoke test: build a `VizSession`, open the Rerun sink (or `NullSink` for headless), log scalar/pose/geo; add a `NullSink` unit test exercising the whole trait without Rerun.
7. Write a short design note in `docs/designs/` covering the backend registry, topic→path mapping, time mapping, and the broker-vs-loop decision.

## 8. Links

- Idea: [`docs/ideas/vs-viz.idea.md`](../ideas/vs-viz.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
