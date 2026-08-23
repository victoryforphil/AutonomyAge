---
title: vs-viz
type: plan
status: todo
tags:
  - plan
  - viz
  - rerun
  - vs-wtf
  - vs-data-store
  - vs-broker
---

# vs-viz (Updated Plan)

Backend-agnostic visualization utilities crate for the AutonomyAge `vs-*` workspace.
The **Rerun** backend is the first sink, but the core (`VizSession` + `VizSink`) is
free of `rerun` imports so more sinks can be added later. Integrates `vs-wtf` time
primitives and `vs-data-store` topics so a `BrokerTask` can stream a `DataView` into a
sink.

This is the **deepened research revision** of `docs/plan/vs-viz.plan.md`. Section 1
resolves the previously-open questions against the real `vs-*` source; the API in
Section 5 is concrete and verified against those sources.

---

## 0. Decisions (summary)

| # | Question | Decision |
|---|----------|----------|
| 1 | Rerun version pin | **`rerun = "0.28"`** (resolves to **0.28.2**), behind optional feature `rerun`. |
| 2 | `rerun` dep | **Optional** (`rerun = ["dep:rerun", "dep:nalgebra"]`); core compiles with `--no-default-features`. |
| 3 | Host model | BOTH. `VizSystem` implements `vs_broker::task::BrokerTask` **and** `VizSession`+`RerunSink`/`NullSink` are usable directly in a plain loop. |
| 4 | Topic/namespace prefix | Per-`RerunOptions.namespace: Option<String>` (default `None` = keep verbatim topic path). Recommend apps set `namespace = Some(app_id)` when multiple apps share a viewer. |
| 5 | Payload `Vec3` | Own tiny `Vec3([f64;3])` in the core (keeps `nalgebra` out of the stable trait). |
| 6 | `log_scalar` archetype | Use **`rerun::Scalars::single(f64)`** (not legacy `Scalar`). |
| 7 | `TopicKey`→path | Use `key.display_name()` (or `format!("{}", key)`), **never** `.to_string()` (returns numeric ids). |
| 8 | Object safety | `VizSink` is `Box<dyn VizSink>` → trait path params are `&VizPath`, not `impl Into<VizPath>`. |

---

## 1. Confirmed `vs-*` APIs (read from source)

All names verified in `libs/` (branch `vfp/agent/plan/vs-viz`). The original plan's
`get_latest_primitive` claim for `DataView` was **wrong** — it lives on `Datastore`.

### `vs_wtf`
- `Timepoint::{now, zero, new, new_secs, new_ms, new_us, new_ns}`, `.secs()/.ms()/.us()/.ns()`, `.time: Timecode`.
  `Timepoint::now().ms()` → `f64` (epoch millis) used for `run_id`.
- `Timespan::{new_secs, new_ms, zero}`, `.secs()/.ms()/.us()/.ns()`.
- `Timecode` (in `vs_wtf::timecode`).

### `vs_data_store`
- `data_store::database::view::DataView`
  - `DataView::new() -> DataView`
  - `DataView::new_timed(time: Timepoint) -> DataView`
  - `get_latest_map<T: TopicKeyProvider>(&self, topic: &T) -> Result<HashMap<TopicKey, Datapoint>, DatastoreError>` (parent-matching via `is_child_of`)
  - `get_latest<T: TopicKeyProvider, S: DeserializeOwned>(&self, topic: &T) -> Result<S, DatastoreError>`
  - `get_datapoint<T>(&self, topic: &T) -> Option<&Datapoint>`
  - `get_value<T>(&self, topic: &T) -> Option<&Primitives>`
  - `get_all_datapoints(&self) -> Vec<Datapoint>`
- `vs_data_store::database::Datastore`:
  - `get_latest_primitive<T: TopicKeyProvider>(&self, topic: &T) -> Option<Primitives>`
  - `get_latest_primitives<T>(&self, topics: HashSet<T>) -> HashMap<TopicKey, Primitives>`
- `vs_data_store::topics::TopicKey` / `TopicKeyHandle` / `TopicKeyProvider`
  - `TopicKey::{empty, from_str, display_name, to_string, is_child_of, add_prefix, remove_prefix}`
  - **GOTCHA**: `TopicKey::to_string()` (inherent) returns numeric ids joined by `/`.
    `Display` (used by `format!("{}", key)`) and `display_name()` return the human path.
    Always use `display_name()` (or `format!("{}", key)`) for a Rerun entity path.
- `vs_data_store::datapoints::Datapoint { topic: TopicKeyHandle, time: Timepoint, value: Primitives }`
- `vs_data_store::primitives::Primitives` enum:
  `Unset, Instant(Timepoint), Duration(Timespan), Integer(i64), Float(f64), Text(String), Blob(VicBlob), Boolean(bool), List(Vec<Primitives>), Reference(TopicIDType), StructType(String)`

### `vs_broker`
- `vs_broker::task::BrokerTask` trait (`Send`):
  - `fn init(&mut self) -> Result<(), anyhow::Error> { Ok(()) }`
  - `fn get_config(&self) -> BrokerTaskConfig`
  - `fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView, anyhow::Error>`
- `vs_broker::task::config::BrokerTaskConfig::new(name)`, `.with_trigger(...)`, `.with_subscription(...)`, `.with_flag(...)`; `BrokerCommanderFlags::NonBlocking`.
- `vs_broker::task::subscription::BrokerTaskSubscription::{new_latest(&dyn TopicKeyProvider), new_updates_only(...)}`; `SubscriptionMode::{Latest, NewValues}`.
- `vs_broker::task::trigger::BrokerTaskTrigger::{Always, Rate(Timespan)}`.
- `vs_broker::broker::time::BrokerTime { time_monotonic: Timepoint, time_delta: Timespan, time_last_monotonic: Option<Timepoint> }`.

---

## 2. Rerun version pin (resolved)

**Pin `rerun = "0.28"` (→ 0.28.2), gated behind the `rerun` feature.**

Rationale:
- **Only version whose exact API surface is confirmed by a working in-org wrapper.** SkyCanvas
  `gen2/quad_app/src/common/log_rerun.rs` pins `rerun = "0.28.2"` (Cargo.toml line 13) and
  uses the exact calls we want: `RecordingStreamBuilder::new(name).spawn()`,
  `rec.log(path, &rerun::TextLog::new(txt).with_level(rerun::TextLogLevel::INFO))`,
  `GeoPoints::from_lat_lon(...)`, `Points3D::new(...)`, `Radius::new_ui_points`, `Color::from_rgb`.
- **Cross-version drift note.** Latest on crates.io is **0.36.2** (0.36.1/0.36.2 released
  2026-08). 0.28.2 (2026-01-08) is ~8 minors behind. The core archetypes we use
  (`Scalars`, `Points3D`, `GeoPoints`, `Boxes3D`, `TextLog`, `TextDocument`, `Image`,
  `ViewCoordinates`, `RecordingStreamBuilder`) are stable across 0.28→0.36; a later task can
  bump the pin with only the build-verified calls inside `rerun/`.
- **MSRV / build.** Workspace has no `rust-toolchain`; installed toolchain is `rustc 1.95.0`
  (2026-04-14), well above rerun 0.28's MSRV (~1.85+). `rerun` is a heavy dependency
  (arrow ^56.1, tokio ^1.47.1, `re_*` crates, cmake build/nasm for video) → **must stay an
  optional feature** so `--no-default-features` keeps the core and workspace builds fast.

> Note: `rerun::Logger` was removed in newer Rerun — do **not** port it. Wire `log`/`tracing`
> to a `TextLog` sink yourself if needed.

---

## 3. Host model (resolved: BOTH entry points)

Ship **two** entry points, sharing the same `RerunSink`/`NullSink`.

1. **`VizSystem: BrokerTask`** (`system.rs`) — for app integration. Opens a sink in `init()`,
   streams the recieved `DataView` in `on_execute()`, flushes after each tick.
   `get_config()` subscribes `new_latest(&TopicKey::empty())`, triggers
   `BrokerTaskTrigger::Always`, and flags `NonBlocking`.
2. **`VizSession` + `RerunSink`/`NullSink`** — for a plain loop / standalone tooling:
   ```rust
   let session = VizSession::new("demo", "group");
   let options = RerunOptions::from_session(&session);
   let mut sink = RerunSink::open(&options, &session)?;
   sink.set_time(&Timepoint::now())?;
   sink.log_scalar(&VizPath::from("demo/temperature"), 21.5)?;
   sink.flush()?;
   ```

The `rerun` layer lives in `rerun/` behind the feature. `VizSystem` is generic over
`Box<dyn Backend>` so it works headless (`NullBackend`) or with Rerun.

---

## 4. Topic/namespace mapping (resolved)

- A `TopicKey` maps to a Rerun entity path via `key.display_name()` (e.g. `pose/ned/x`,
  no leading slash — always a valid Rerun path because `from_str` filters empty sections).
- **Per-app prefix** is optional and explicit: `RerunOptions::namespace: Option<String>`.
  - `None` (default) → use the topic path verbatim (topics already carry their own section
    hierarchy).
  - `Some("viz")` / `Some(app_id)` → prepend, e.g. `viz/pose/ned/x`.
- `topic_path(topic, namespace)` in `rerun/topic.rs` builds the `VizPath`. `log_dataview`
  drains `view.get_latest_map(&TopicKey::empty())` and dispatches each `Datapoint.value`.
- Recommendation for multi-vehicle / multi-app viewers: set `namespace = Some(app_id)` so
  entity spaces don't collide; leave `None` for a flat single-instance session.

---

## 5. Concrete `libs/vs-viz` API

New workspace member `libs/vs-viz` (package `vs-viz`, crate `vs_viz`).

### Feature / dependency layout

```toml
[package]
name = "vs-viz"
version = "0.1.0"
edition = "2021"

[features]
default = ["rerun"]            # convenient; disable with --no-default-features for fast/headless builds
rerun   = ["dep:rerun", "dep:nalgebra"]

[dependencies]
serde = { version = "1.0.229", features = ["derive"] }
anyhow = "1.0.104"
thiserror = "2.0.20"
log = "0.4.34"
tracing = "0.1.44"
vs-wtf = { path = "../vs-wtf" }
vs-data-store = { path = "../vs-data-store" }
vs-broker = { path = "../vs-broker" }
# Optional, feature-gated:
rerun    = { version = "0.28", optional = true }
nalgebra = { version = "0.33", optional = true }

[dev-dependencies]
env_logger = "0.11.11"
```

> `nalgebra` is only used by the Rerun pose-marker math; it is pulled in by the `rerun`
> feature. The core `VizSink` trait and payloads use the own `Vec3`/`[f64;4]` so no geom
> crate leaks into the stable trait.

### Module layout

```
libs/vs-viz/src/
  lib.rs        # re-exports
  session.rs    # VizSession + VizMode (backend-agnostic, NO rerun imports)
  sink.rs       # trait VizSink + payload types (backend-agnostic)
  path.rs       # VizPath (topic -> entity path)
  backend.rs    # Backend trait + BackendRegistry + NullSink/NullBackend
  system.rs     # VizSystem: impl vs_broker::task::BrokerTask
  rerun/        # (feature `rerun`)
    mod.rs
    backend.rs  # RerunOptions + RerunBackend
    sink.rs     # RerunSink: impl VizSink
    pose.rs     # VizPose -> scalars + Points3D + orientation Boxes3D marker
    scene.rs    # log_world / log_floor / log_asset / log_waypoints / log_home
    topic.rs    # topic_path + log_dataview (DataView -> Primitives -> sink)
```

### `session.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizMode { Save, Live, Spawn }

pub struct VizSession {
    pub name: String,
    pub group: String,
    pub run_id: String,
    mode_override: Option<VizMode>,
}

impl VizSession {
    pub fn new(name: impl Into<String>, group: impl Into<String>) -> Self;   // run_id = Timepoint::now().ms().to_string()
    pub fn with_run_id(name: impl Into<String>, group: impl Into<String>, run_id: impl Into<String>) -> Self;
    pub fn app_id(&self) -> String;              // "{group}/{name}"
    pub fn get_mode(&self) -> VizMode;           // mode_override.unwrap_or(Self::env_mode())
    pub fn env_mode() -> VizMode;                // reads RERUN_MODE (SAVE/LIVE/SPAWN, default SAVE)
    pub fn set_save(&mut self) / set_live(&mut self) / set_spawn(&mut self);
    pub fn save_dir(&self, root: impl AsRef<std::path::Path>) -> std::path::PathBuf; // {root}/.logs/{group}
}
```

### `path.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizPath(String);

impl VizPath {
    pub fn new(path: impl Into<String>) -> Self;
    pub fn from_topic(topic: &vs_data_store::topics::TopicKey, namespace: Option<&str>) -> Self;
    pub fn join(&self, child: impl AsRef<str>) -> Self;
    pub fn as_str(&self) -> &str;
}
impl From<&str> for VizPath;
impl From<String> for VizPath;
impl From<&vs_data_store::topics::TopicKey> for VizPath;   // via display_name()
impl std::fmt::Display for VizPath;                         // forwards as_str()
```

### `sink.rs` (backend-agnostic payloads + trait)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3(pub [f64; 3]);
impl Vec3 { pub fn new(x: f64, y: f64, z: f64) -> Self; pub fn x(&self) -> f64; pub fn y(&self) -> f64; pub fn z(&self) -> f64; }

#[derive(Debug, Clone, PartialEq)]
pub struct VizPose {
    pub position: Vec3,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub acceleration: Vec3,
    pub angular_acceleration: Vec3,
    pub attitude: [f64; 4],   // quaternion [x, y, z, w] (matches nalgebra UnitQuaternion.coords and Rerun::Quaternion)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VizGeoPoint { pub lat: f64, pub lon: f64 }

#[derive(Debug, Clone, PartialEq)]
pub enum VizImageFormat { Rgb8, Rgba8 }

#[derive(Debug, Clone, PartialEq)]
pub struct VizImage { pub bytes: Vec<u8>, pub width: u32, pub height: u32, pub format: VizImageFormat }

/// Backend-agnostic sink. Object-safe (used as `Box<dyn VizSink>`); path params are `&VizPath`.
pub trait VizSink: Send {
    fn name(&self) -> &'static str { "sink" }
    fn set_time(&mut self, t: &vs_wtf::Timepoint) -> anyhow::Result<()>;
    fn set_duration(&mut self, span: &vs_wtf::Timespan) -> anyhow::Result<()>;
    fn log_scalar(&mut self, path: &VizPath, value: f64) -> anyhow::Result<()>;
    fn log_text(&mut self, path: &VizPath, text: &str) -> anyhow::Result<()>;
    fn log_points(&mut self, path: &VizPath, pts: &[Vec3]) -> anyhow::Result<()>;
    fn log_geo(&mut self, path: &VizPath, lla: &[VizGeoPoint]) -> anyhow::Result<()>;
    fn log_pose(&mut self, path: &VizPath, pose: &VizPose) -> anyhow::Result<()>;
    fn log_image(&mut self, path: &VizPath, img: &VizImage) -> anyhow::Result<()>;
    fn flush(&mut self) -> anyhow::Result<()>;
}
```

### `backend.rs`

```rust
pub trait Backend: Send {
    fn name(&self) -> &'static str;
    fn open(&self, session: &VizSession) -> anyhow::Result<Box<dyn VizSink>>;
}

pub struct BackendRegistry { backends: Vec<Box<dyn Backend>> }
impl BackendRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, backend: Box<dyn Backend>);
    pub fn get(&self, name: &str) -> Option<&dyn Backend>;
    pub fn open(&self, session: &VizSession) -> anyhow::Result<Box<dyn VizSink>>; // first registered else NullSink
}

pub struct NullSink;                                   // no-op, all methods Ok(())
pub fn null_sink() -> Box<dyn VizSink>;
pub struct NullBackend;                                // `open` returns NullSink
impl Backend for NullBackend { /* name() = "null" */ }
```

### `rerun/backend.rs` (feature `rerun`)

```rust
#[derive(Clone, Debug)]
pub struct RerunOptions {
    pub app_id: String,
    pub run_id: Option<String>,
    pub spawn: bool,
    pub save: Option<std::path::PathBuf>,   // .rrd path; None = don't save
    pub save_name: Option<String>,          // default "{name}.rrd"
    pub connect: Option<String>,            // gRPC addr for Live; None = don't connect
    pub namespace: Option<String>,          // per-app entity prefix (default None)
    pub create_dir: bool,                   // create save parent dir (default true)
}
impl RerunOptions {
    pub fn from_session(session: &VizSession) -> Self;   // app_id = session.app_id(); run_id = session.run_id
    pub fn with_spawn(mut self, v: bool) -> Self;
    pub fn with_save(mut self, path: std::path::PathBuf) -> Self;
    pub fn with_connect(mut self, addr: impl Into<String>) -> Self;
    pub fn with_namespace(mut self, ns: impl Into<String>) -> Self;
    pub fn enabled(&self) -> bool;                         // spawn || save.is_some() || connect.is_some()
    pub fn save_dir(&self, session: &VizSession) -> std::path::PathBuf; // {VS_VIZ_DIR|CARGO_MANIFEST_DIR}/.logs/{group}
}

pub struct RerunBackend { pub options: RerunOptions }
impl Backend for RerunBackend {
    fn name(&self) -> &'static str { "rerun" }
    fn open(&self, session: &VizSession) -> anyhow::Result<Box<dyn VizSink>> {
        Ok(Box::new(RerunSink::open(&self.options, session)?))
    }
}
```

`RerunSink::open` builds a `Vec<RecordingStream>` (Save + Spawn + Connect can coexist, per
loki `rerun_bridge.rs`), sets `app_id`/`recording_id`, and `create_dir_all`s the save dir.

### `rerun/sink.rs` (feature `rerun`)

```rust
pub struct RerunSink {
    streams: Vec<rerun::RecordingStream>,
    namespace: Option<String>,
}
impl RerunSink {
    pub fn open(options: &RerunOptions, session: &VizSession) -> anyhow::Result<Self>;
    pub fn streams(&self) -> &[rerun::RecordingStream];
}

impl VizSink for RerunSink {
    fn set_time(&mut self, t: &Timepoint)      { for s in &self.streams { s.set_time_seconds("viz-time", t.secs()); } }
    fn set_duration(&mut self, span: &Timespan){ for s in &self.streams { s.set_duration_secs("viz-time", span.secs()); } }
    fn log_scalar(..., value: f64)             { for s in &self.streams { s.log(a_path, &rerun::Scalars::single(value))?; } }
    fn log_text(..., text: &str)               { for s in &self.streams { s.log(a_path, &rerun::TextLog::new(text).with_level(rerun::TextLogLevel::INFO))?; } }
    fn log_points(..., pts: &[Vec3])           { let ps: Vec<rerun::Vec3D> = pts.iter().map(to_vec3d).collect();
                                                 for s in &self.streams { s.log(a_path, &rerun::Points3D::new(ps).with_radii(...).with_colors(...))?; } }
    fn log_geo(..., lla: &[VizGeoPoint])       { let ll: Vec<(f64,f64)> = lla.iter().map(|p|(p.lat,p.lon)).collect();
                                                 for s in &self.streams { s.log(a_path, &rerun::GeoPoints::from_lat_lon(&ll).with_radii(...).with_colors(...))?; } }
    fn log_pose(..., pose: &VizPose)           { self.log_pose_impl(path, pose)?; }   // -> pose.rs
    fn log_image(..., img: &VizImage)          { for s in &self.streams { s.log(a_path, &rerun::Image::from_rgb8((img.width, img.height), &img.bytes))?; } }
    fn flush(&mut self)                        { for s in &self.streams { s.flush_blocking()?; } }
}
```

`a_path` is `path.as_str()` prefixed by `namespace` when set.

### `rerun/pose.rs` (feature `rerun`)

Decomposition ported from basher `RerunQuadPose`:
- `position`, `velocity`, `angular_velocity`, `acceleration`, `angular_acceleration` each logged
  (under `{path}/<field>`) as a `Points3D` single point + three `Scalars::single` (`x`,`y`,`z`).
- Orientation marker from lil-rerun: a `Boxes3D::from_sizes(vec![Vec3D::new(0.5,0.5,0.1)])`
  `.with_quaternions(vec![[x,y,z,w]])` `.with_centers(vec![Vec3D::new(px,py,pz)])`.
  `[x,y,z,w]` from `nalgebra::UnitQuaternion::from_euler_angles(r,p,y).coords`.
- NED → ENU convention: `pos.z` → `-pos.z` (matches lil-rerun/basher).

### `rerun/scene.rs` (feature `rerun`) — Rerun-only (static archetypes, not on the trait)

- `log_world(sink, path)` → `ViewCoordinates::RIGHT_HAND_Z_UP` via `log_static`.
- `log_floor(sink, path, size)` → `Boxes3D::from_sizes(vec![Vec3D::new(size, size, 0.01)])`.
- `log_asset(sink, path, file)` → `Asset3D::from_file(path)` via `log_static`.
- `log_waypoints(sink, path, pts)` → `Points3D`.
- `log_home(sink, path)` → single `Points3D` at origin.

### `rerun/topic.rs` (feature `rerun`)

```rust
pub fn topic_path(topic: &TopicKey, namespace: Option<&str>) -> VizPath;   // namespace {viz/} + topic.display_name()
pub fn log_dataview(sink: &mut dyn VizSink, view: &DataView, namespace: Option<&str>) -> anyhow::Result<()>;
```

`log_dataview` iterates `view.get_latest_map(&TopicKey::empty())?`, and for each
`(key, datapoint)`:
- `sink.set_time(&datapoint.time)` (per-datapoint `data-time`).
- dispatch on `&datapoint.value` (`Primitives`):
  - `Float(v)` → `log_scalar(path, *v)`
  - `Integer(v)` → `log_scalar(path, *v as f64)`
  - `Boolean(v)` → `log_scalar(path, *v as i64 as f64)` + `log_text(path, "<bool>")`
  - `Text(v)` → `log_text(path, v)`
  - `Duration(d)` → `log_scalar(path, d.secs())`
  - `Instant(t)` → text (or set a separate timeline)
  - `Blob(_)` / `List(_)` / `Reference(_)` / `StructType(_)` → `log_text(path, debug)` + `warn!`
  - `Unset` → skip

### `system.rs`

```rust
pub struct VizSystem {
    session: VizSession,
    backend: Box<dyn Backend>,
    sink: Option<Box<dyn VizSink>>,
    namespace: Option<String>,
}
impl VizSystem {
    pub fn new(name: impl Into<String>, group: impl Into<String>) -> Self;      // NullBackend default
    pub fn with_backend(backend: Box<dyn Backend>) -> Self;
    #[cfg(feature = "rerun")]
    pub fn rerun(options: RerunOptions) -> Self;                                 // RerunBackend
    pub fn open(&mut self) -> anyhow::Result<()>;
    pub fn sink_mut(&mut self) -> Option<&mut dyn VizSink>;
    pub fn set_namespace(&mut self, ns: Option<String>);
}
impl vs_broker::task::BrokerTask for VizSystem {
    fn init(&mut self) -> Result<(), anyhow::Error> { self.open() }
    fn get_config(&self) -> BrokerTaskConfig {
        BrokerTaskConfig::new("viz-system")
            .with_trigger(BrokerTaskTrigger::Always)
            .with_subscription(BrokerTaskSubscription::new_latest(&TopicKey::empty()))
            .with_flag(BrokerCommanderFlags::NonBlocking)
    }
    fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView, anyhow::Error> {
        if let Some(sink) = self.sink_mut() {
            log_dataview(sink, inputs, self.namespace.as_deref())?;
            sink.flush()?;
        }
        Ok(DataView::new())
    }
}
```

`VizSystem` is `Send` (`dyn Backend`/`dyn VizSink` are `Send`; `RecordingStream` is `Send`).

### `lib.rs` re-exports

Backend-agnostic (always): `VizSession`, `VizMode`, `VizSink`, `VizPath`, `Vec3`,
`VizPose`, `VizGeoPoint`, `VizImage`, `VizImageFormat`, `Backend`, `BackendRegistry`,
`NullBackend`, `NullSink`, `null_sink`, and (under `vs-broker`) `VizSystem`, `log_dataview`.
Under `feature = "rerun"`: `RerunOptions`, `RerunBackend`, `RerunSink`, `topic_path`,
`log_world`/`log_floor`/`log_asset`/`log_waypoints`/`log_home`.

---

## 6. Update plan for the implementation next steps

1. Add `libs/vs-viz` to `[workspace].members`.
2. Implement core first (no `rerun` feature): `VizSession`/`VizMode`, `VizPath`,
   `Vec3`/`VizPose`/`VizGeoPoint`/`VizImage`, `VizSink`, `Backend`/`BackendRegistry`,
   `NullSink`/`NullBackend`. Verify `cargo check -p vs-viz --no-default-features`.
3. Implement `rerun/` (feature `rerun`): `RerunOptions`/`RerunBackend`, `RerunSink`
   (Scalars/TextLog/Points3D/GeoPoints/Image), `pose.rs`, `scene.rs`, `topic.rs`.
4. Implement `system.rs` (`VizSystem: BrokerTask`).
5. Smoke tests: open `NullSink`/`RerunSink`, log scalar/pose/geo/image; a
   `log_dataview` test built from a `DataView::new_timed(...)` with `add_latest`.
6. Design note in `docs/designs/` (backend registry, topic→path, time mapping, broker-vs-loop).
7. Build-verify the `rerun` 0.28.2 archetype calls (`Boxes3D::with_quaternions`,
   `Asset3D::from_file`, `flush_blocking`) — these come from 0.18/0.19 sources and need a
   compile check against 0.28.2.

---

## 7. Risks / open questions

- **Rerun build weight**: `rerun` pulls arrow/tokio/`re_*`/cmake/nasm. Keep it optional and
  default the feature ON only for convenience; CI/headless builds use `--no-default-features`.
- **0.28 vs 0.36 drift**: we pin 0.28.2 to match the confirmed SkyCanvas wrapper. Bumping to
  a newer 0.3x is a separate task; the archetypes used are stable across the range.
- **Build-only API verification**: `Boxes3D::with_quaternions/from_sizes`, `Asset3D::from_file`,
  `flush_blocking`, `ViewsCoordinates::RIGHT_HAND_Z_UP` come from 0.18/0.19 and need a
  compile check against 0.28.2. The `Scalars`/`TextLog`/`Points3D`/`GeoPoints`/`Image` calls
  are confirmed by SkyCanvas/loki 0.23/0.28 sources.
- **Time axis**: per-datapoint `data-time` vs broker tick `broker-time` — documented; the broker
  path sets `viz-time` from `BrokerTime.time_monotonic`, the loop path sets it from `Timepoint`.
- **Concurrency**: `RecordingStream` is `Send`+clone; if multiple tasks log concurrently, wrap
  the sink in `Arc<Mutex<dyn VizSink>>`. Default is single-threaded per `VizSystem`.

## 8. Links

- Idea: `docs/ideas/vs-viz.idea.md`
- Prior plan: `docs/plan/vs-viz.plan.md`
- Sources: `lil-hopps/lil-rerun/`, `basher/src/basher_rerun/`,
  `AndreasLabs/SkyCanvas/gen2/quad_app/src/common/log_rerun.rs`,
  `AndreasLabs/loki/tools/firmware_buddy/src/rerun_bridge.rs`,
  `project-firefly/src_tools/lit-rerun/`.
