---
title: quadlink
type: plan
status: todo
tags:
  - plan
  - mavlink
  - vs-mavlink
---

# quadlink (Plan) — UPDATED with concrete API + decisions

A reusable MAVLink communication & command framework crate for the AutonomyAge `vs-*` Rust workspace. It owns the whole MAVLink side of talking to a vehicle — open a connection (serial/UDP/TCP), run the transport threads, issue commands via builders, decode inbound messages into `vs-data-store` topics, bridge command requests from `vs-broker`, and stamp everything with `vs-wtf` time. This is the shared MAVLink layer so drones, sims, and ground tooling speak the same protocol.

## 1. DECISIONS RESOLVED (was "open questions")

These supersede the open questions in the prior plan draft.

### 1.1 Crate name — `vs-mavlink` (crate `vs_mavlink`)
- Path `libs/vs-mavlink/`, package `vs-mavlink`. **Recommended** over `quadlink`.
- Reason: the repo's naming convention is a `vs-*` prefix (`vs-broker`, `vs-data-store`, `vs-wtf`). `quadlink` is the *project/feature* name; `vs-mavlink` is the *crate* name that slots into the workspace. Allows `use vs_mavlink::...` alongside `vs_broker`.
- The type names **stay** `Quad*` (`QuadLinkCore`, `QuadLinkError`, `QuadlinkSystem`, `QuadPoseNED`, …) so the "quadlink" branding lives in the API, not the crate name.

### 1.2 `mavlink` crate version — `0.13.1` (**recommended**)
- Every ported source pins `0.13.1` (lil-link, whisper, devore). Only `cursed-mav` pins `0.14.1` and it is not a port source. The `vs-*` workspace has **no** existing `mavlink` dependency.
- Decision: pin `mavlink = "0.13.1"` for the first port so behavior matches the proven source exactly and the `ardupilotmega` dialect types (`COMMAND_LONG_DATA`, `SET_POSITION_TARGET_LOCAL_NED_DATA`, `SET_GPS_GLOBAL_ORIGIN_DATA`, `PositionTargetTypemask`, `MavCmd`, `MavModeFlag`, `EkfStatusFlags`, `MavSysStatusSensor`) are identical. **Defer** the 0.13 → 0.14 migration (and the `MavMessage`/dialect naming drift) to a follow-up.

### 1.3 Where the heartbeat lives — inside `QuadLinkCore::start_thread` (**gated**)
- Keep the heartbeat as a transport-maintenance thread spawned inside `start_thread` (as devore `connection.rs` and whisper `core.rs` both do), **not** in `QuadlinkSystem`/`BrokerTask`.
- Reason: heartbeat must keep flowing on a fixed cadence regardless of commander/broker ticks; it is maintenance, not command. The original concern was an **ungated** `loop` in lil-link; that is fixed by gating each send on `should_stop` (devore pattern). `BrokerTask::init` still calls `start_thread`, so startup is identical from the task's perspective.
- Optional `heartbeat_rate: Duration` (default 1s) so callers can slow it for sims.

### 1.4 Handshake placement — at connect time in `start_thread` (**gated**)
- Keep `request_parameters()` + `request_stream()` as the connect-time handshake inside `start_thread` (matches all three sources). Not a broker publish. The broker layer only handles *commands*.

### 1.5 Ack semantics — retain builders returning `None`, defer timeout/retry
- Builders keep the lil-link contract: return `Some(msg)` when not yet acked, `None` when `req.ack` is already true. `QuadlinkSystem` acks by re-publishing the request struct with `ack = true`.
- A missing-`COMMAND_ACK` timeout/retry is **out of scope** for v1 (recorded as a risk). `proc_command_ack` only logs + publishes a human-readable result string.

### 1.6 Topic namespace — fully namespaced under `quadlink/`
- All `quadlink`-owned topics are prefixed `quadlink/`, including command-request topics. No existing consumer owns these keys (new crate), so full namespacing is safe and collision-free for multi-vehicle/sim stores.
- Request/parameter `get_topic_key()` returns `quadlink/cmd/*` / `quadlink/params/*` accordingly.

## 2. Source implementations found

| Repo | Path | mavlink | Date | Notes |
|------|------|---------|------|-------|
| lil-hopps/lil-link | `lil-hopps/lil-link/src/mavlink/` | 0.13.1 | Nov 2024 | **Best base.** Public `QuadLinkCore`, `QuadLinkError`, `QuadlinkSystem`; `victory_*` = `vs_*`. Pure-serde `Quad*` types. Weakness: no graceful shutdown (un-gated heartbeat loop, disconnected `thread::spawn`, no joins). |
| project-firefly (whisper) | `project-firefly/src_flight/whisper/src/mavlink/` | 0.13.1 | Feb 2025 | Extra builders/processors to graft; but coupled to prost `whisper::common` + `wingman-*`. **`cmd_velocity.rs` is a stub (`// ... existing code ...`) — no real velocity builder exists**; must be authored. |
| project-devore | `project-devore/quad/src/ardulink/connection.rs` | 0.13.1 | Nov 2025 | **Best transport lifecycle**: `should_stop: Arc<AtomicBool>`, `thread_handles: Vec<JoinHandle<()>>`, `stop_thread()` (set flag, sleep 100 ms, drain+join), gated `send`/`recv`. Send loop `recv_timeout(100ms)`; recv loop keeps `WouldBlock` sleep. |
| cursed-mav | `cursed-mav` | 0.14.1 | ref | Only pinned-0.14 source; reference for upstream API drift only. |

Rejected (non-MAVLink/empty): `underscore_quad` (Tello binary), `victory-ground-station`, `tiny-gcs`, `delores`/`project-delores` (empty), `mavsight`/`project-mariposa` (no Rust MAVLink), `loki` (pymavlink SITL).

## 3. Confirmed vs-* API (read from real source)

- `vs_wtf::Timepoint` (`.zero()`, `.now()`, `.clone()`), `Timespan` (`.zero()`, `.ms()`, `.secs()`).
- `vs_data_store::{database::view::DataView, topics::TopicKey}`.
  - `DataView::new()`, `DataView::new_timed(time: Timepoint)`.
  - `add_latest<T: TopicKeyProvider, S: Serialize>(&mut self, topic: &T, value: S) -> Result<(), DatastoreError>`.
  - `get_latest<T: TopicKeyProvider, S: DeserializeOwned>(&self, topic: &T) -> Result<S, DatastoreError>`.
  - `TopicKey::from_str(&str) -> TopicKey`; `TopicKey` implements `TopicKeyProvider` (so pass `&TopicKey` to `get_latest`/`add_latest`/subscriptions).
- `vs_broker::{
    task::{BrokerTask, config::BrokerTaskConfig, subscription::{BrokerTaskSubscription, SubscriptionMode}, trigger::BrokerTaskTrigger},
    broker::time::BrokerTime }`.
  - `trait BrokerTask: Send { fn init(&mut self) -> Result<(), anyhow::Error> { Ok(()) } fn get_config(&self) -> BrokerTaskConfig; fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView, anyhow::Error>; }`.
  - `BrokerTaskConfig::new(name: &str)` (random id) `.with_trigger(BrokerTaskTrigger::Always)` (also `Rate(Timespan)`) `.with_subscription(BrokerTaskSubscription)`.
  - `BrokerTaskSubscription::new_latest(&dyn TopicKeyProvider)` / `new_updates_only(&dyn TopicKeyProvider)` (built from a `&TopicKey`).
  - `BrokerTime { pub time_monotonic: Timepoint, pub time_delta: Timespan, pub time_last_monotonic: Option<Timepoint> }`.
- Crate lib names: `vs-broker` → `vs_broker`, `vs-data-store` → `vs_data_store`, `vs-wtf` → `vs_wtf` (no `[lib]` override; the `broker_tcp_*` are `[[bin]]`).
- Workspace `Cargo.toml` `[workspace].members = ["libs/vs-data-store", "libs/vs-wtf", "libs/vs-broker"]` → add `"libs/vs-mavlink"`.

## 4. CONCRETE API design (port surface)

### 4.1 `lib.rs`
```rust
pub mod common;
pub mod mavlink;
```

### 4.2 `common/identifiers.rs` (renamed to `quadlink/` prefix)
```rust
pub static IDENT_BASE_STATUS: &str = "quadlink/status";
pub static IDENT_BASE_LOG:    &str = "quadlink/log";
pub static IDENT_BASE_PARAMS: &str = "quadlink/params";
pub static IDENT_BASE_POSE:   &str = "quadlink/pose";
pub static IDENT_BASE_GPS:    &str = "quadlink/gps";   // NEW (grafted)
pub static IDENT_BASE_HOME:   &str = "quadlink/home";  // NEW (grafted)

pub static IDENT_STATUS_SYSTEM: &str = "system";
pub static IDENT_STATUS_MODE:   &str = "mode";
pub static IDENT_STATUS_TEXT:   &str = "text";
pub static IDENT_STATUS_SENSORS:&str = "sensors";
pub static IDENT_STATUS_EKF:    &str = "ekf";
pub static IDENT_STATUS_HEALTH: &str = "health";
pub static IDENT_STATUS_BATTERY:&str = "battery";
pub static IDENT_STATUS_DROP_RATE_COMM: &str = "drop_rate_comm";
pub static IDENT_STATUS_VOLTAGE:&str = "voltage";
pub static IDENT_STATUS_CURRENT:&str = "current";
pub static IDENT_STATUS_COMM_ERRORS: &str = "comm_errors";
pub static IDENT_STATUS_ERRORS_COUNT: &str = "errors_count";
pub static IDENT_COMMAND_ACK:   &str = "command_ack";
pub static IDENT_POSE_NED:      &str = "ned";
pub static IDENT_ATTITUDE:      &str = "attitude";
```

Topic strings produced (`format!("{}/{}", ...)`):
- status: `quadlink/status/system`, `quadlink/status/mode`, `quadlink/status/text`, `quadlink/status/sensors`, `quadlink/status/ekf`, `quadlink/status/health`, `quadlink/status/battery`, `quadlink/status/drop_rate_comm`, `quadlink/status/voltage`, `quadlink/status/current`, `quadlink/status/comm_errors`, `quadlink/status/errors_count`.
- pose: `quadlink/pose/ned`, `quadlink/pose/attitude`.
- log: `quadlink/log/text`, `quadlink/log/command_ack/<:?>` (lowercased).
- params: `quadlink/params/<param>`.
- gps: `quadlink/gps` (publishes `Gps`), home: `quadlink/home` (publishes `QuadHome`).
- command request topics (consumed + acked): `quadlink/cmd/arm`, `quadlink/cmd/mode`, `quadlink/cmd/takeoff`, `quadlink/cmd/land`, `quadlink/cmd/waypoint`, `quadlink/cmd/set_home`, `quadlink/cmd/set_origin`, `quadlink/cmd/velocity`.

### 4.3 `common/types/*` (pure-serde, no protobuf)
- `vector3.rs`: `pub struct Vector3 { pub x: f64, pub y: f64, pub z: f64 }` + `new`, `new_f32`, `zero`, `to_array`, `distance`, `from_array`, arithmetic ops, `Default`.
- `attitude.rs`: `pub struct QuadAttitude { pub rpy_radians: Vector3 }` + `zero`, `new_xyz`, `new_f32`, `new_rpy_radians`.
- `pose_ned.rs`: `pub struct QuadPoseNED { pub position: Vector3, pub velocity: Vector3 }` + `zero`, `new_xyz`, `new_position_and_velocity`, `distance`.
- `mode.rs`: `pub enum QuadMode { Stabilize, Acro, AltHold, Auto, Guided, Loiter, Return, Land, PosHold, Brake, Follow }` + `Display` + `FromStr`.
- `autopilot_status.rs`: `pub struct QuadAutopilotStatus { custom_mode_enabled, test_enabled, auto_enabled, guided_enabled, stabilize_enabled, hil_enabled, manual_input_enabled, safety_armed: bool }`.
- `sensor_status.rs`: `pub struct QuadSensorStatus { gyro, accel, mag, abs_pressure, diff_pressure, gps, optical_flow, vision_position, laser_position, external_ground_truth, rate_control, attitude_stabilization, yaw_position, altitude_control, xy_position_control, motor_control, rc_receiver, gyro2, accel2, mag2, geofence, ahrs, terrain, reverse_motor, logging, battery, proximity, satcom, prearm_check, obstacle_avoidance, propulsion, extension: bool }` (fields match `decode_sensor_health`).
- `ekf_status.rs`: `pub struct QuadEkfStatus { attitude, vel_horiz, vel_vert, pos_horiz_rel, pos_horiz_abs, pos_vert_abs, pos_vert_agl, const_pos_mode, pred_pos_horiz_rel, pred_pos_horiz_abs, uninitialized: bool }`.
- `health_status.rs`: `pub struct QuadHealthStatus` (retained; health/ekf aggregation).
- `parameter.rs`: `pub struct QuadParameter { pub param: String, pub value: f64, pub ack: bool }` + `new`, `ack`, `byte_id() -> [u8; 16]`, `get_topic_key() -> TopicKey` (`quadlink/params/<param>`), `PartialEq`.
- Request structs (each has `new`, `get_topic_key() -> TopicKey`, `ack(&mut self)`):
  - `request_mode_set.rs`? (name is inverted in lil-link: `request_mode_set.rs` holds `QuadArmRequest`; `request_arm.rs` holds `QuadSetModeRequest`). Normalize names:
    - `request_arm.rs`: `pub struct QuadArmRequest { pub arm: bool, pub ack: bool }`, topic `quadlink/cmd/arm`.
    - `request_mode_set.rs`: `pub struct QuadSetModeRequest { pub mode: QuadMode, pub ack: bool }`, topic `quadlink/cmd/mode`.
    - `request_takeoff.rs`: `pub struct QuadTakeoffRequest { pub height: f32, pub ack: bool }`, topic `quadlink/cmd/takeoff`.
    - `request_land.rs`: `pub struct QuadLandRequest { pub ack: bool }`, topic `quadlink/cmd/land`.
- Grafted (new, pure-serde):
  - `gps.rs`: `pub struct Gps { pub latitude: f64, pub longitude: f64, pub altitude: f64 }` (whisper used f32; **use f64** to hold lat/lon precision; f32 is fine for alt). Add `new`.
  - `home.rs`: `pub struct QuadHome { pub position: Vector3 }` (or reuse `QuadPoseNED`); processor emits `quadlink/home`.

### 4.4 `mavlink/core.rs`
```rust
pub type MavlinkMessageType = mavlink::ardupilotmega::MavMessage;

#[derive(thiserror::Error, Debug)]
pub enum QuadLinkError {
    #[error("Mavlink error: {0}")] MavlinkError(mavlink::error::MessageReadError),
    #[error("Channel recv error: {0}")] ChannelRecvError(crossbeam_channel::RecvError),
    #[error("Channel send error: {0}")] ChannelSendError(crossbeam_channel::SendError<MavlinkMessageType>),
    #[error("Connection error: {0}")] ConnectionError(String),
    #[error("Generic error: {0}")] GenericError(String),
    #[error("No Pending Data")] NoData,
}

pub struct QuadLinkCore {
    recv_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
    transmit_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
    connection_string: String,
    should_stop: Arc<AtomicBool>,
    thread_handles: Vec<thread::JoinHandle<()>>,
    heartbeat_rate: Duration,   // default 1s; used ONLY inside start_thread
}

pub type QuadlinkCoreHandle = Arc<Mutex<QuadLinkCore>>;

impl QuadLinkCore {
    pub fn new(connection_string: &str) -> Result<Self, anyhow::Error>;          // bounded(500) channels
    pub fn start_thread(&mut self) -> Result<(), QuadLinkError>;                 // spawn ONE supervisor thread; returns immediately
    pub fn stop_thread(&mut self) -> Result<(), QuadLinkError>;                  // set should_stop, sleep 100ms, take+join handles
    pub fn send(&self, msg: &MavlinkMessageType) -> Result<(), QuadLinkError>;   // gated on should_stop
    pub fn recv(&self) -> Result<Vec<MavlinkMessageType>, QuadLinkError>;        // gated on should_stop
}
```
`start_thread_inner` (runs on the supervisor thread):
1. `mavlink::connect::<MavlinkMessageType>(&con_string)` → `Box<dyn MavConnection + Send + Sync>`.
2. `set_protocol_version(V2)`; send `request_parameters()` + `request_stream()`.
3. Wrap in `Arc`. Spawn **gated** heartbeat, send, recv sub-threads (all capture `should_stop`):
   - heartbeat: `while !should_stop { send_default(heartbeat_message()); sleep(heartbeat_rate) }`.
   - send: `while !should_stop { rx.recv_timeout(100ms); if should_stop break; vehicle.send(...) }`.
   - recv: `while !should_stop { vehicle.recv(); on WouldBlock sleep(10ms); on Io(other) break }`.
4. `join()` the three sub-threads inside the supervisor; the supervisor `JoinHandle` is pushed to `thread_handles`.

### 4.5 `mavlink/helpers.rs`
```rust
pub struct MavLinkHelper;
impl MavLinkHelper {
    pub fn heartbeat_message() -> mavlink::ardupilotmega::MavMessage;
    pub fn request_parameters() -> mavlink::ardupilotmega::MavMessage; // PARAM_REQUEST_LIST
    pub fn request_stream() -> mavlink::ardupilotmega::MavMessage;     // REQUEST_DATA_STREAM
    pub fn decode_mode_flag(MavModeFlag) -> QuadAutopilotStatus;
    pub fn decode_sensor_health(MavSysStatusSensor) -> QuadSensorStatus;
    pub fn decode_ekf_status(EkfStatusFlags) -> QuadEkfStatus;
    pub fn quad_mode_to_mav_mode(&QuadMode) -> ArduMode;
}
```

### 4.6 `mavlink/ardu_mode.rs` (`ArduMode` enum)
`Stabilize=0 … Turtle=28` (ordering per `ardu_modes.rs`) with `from_u32(u32) -> Option<ArduMode>`, `to_u32() -> u32`, `to_string() -> String`.

### 4.7 `mavlink/builders/*` (each returns `Option<MavlinkMessageType>`)
| module | fn | msg type |
|--------|----|----------|
| `cmd_arm.rs` | `mavlink_build_arm_message(QuadArmRequest) -> Option<MavMessage>` | `COMMAND_LONG(MAV_CMD_COMPONENT_ARM_DISARM)` |
| `cmd_mode.rs` | `mavlink_build_mode_message(QuadSetModeRequest) -> Option<MavMessage>` | `COMMAND_LONG(MAV_CMD_DO_SET_MODE)` |
| `cmd_takeoff.rs` | `mavlink_build_cmd_takeoff_message(QuadTakeoffRequest) -> Option<MavMessage>` | `COMMAND_LONG(MAV_CMD_NAV_TAKEOFF)` |
| `cmd_land.rs` | `mavlink_build_cmd_land_message(QuadLandRequest) -> Option<MavMessage>` | `COMMAND_LONG(MAV_CMD_NAV_LAND)` |
| `cmd_waypoint.rs` | `mavlink_build_cmd_waypoint_message(QuadPoseNED) -> Option<MavMessage>` | `SET_POSITION_TARGET_LOCAL_NED(MAV_FRAME_LOCAL_NED, type_mask=0b110111111000)` |
| `param_set.rs` | `mavlink_build_param_set_message(QuadParameter) -> Option<MavMessage>` | `PARAM_SET` |
| `cmd_set_home.rs` (graft) | `build_cmd_set_home(Gps) -> Option<MavMessage>` | `COMMAND_LONG(MAV_CMD_DO_SET_HOME)` (param5=lat, param6=lon, param7=alt) |
| `cmd_set_origin.rs` (graft) | `build_cmd_set_origin(Gps) -> Option<MavMessage>` | `SET_GPS_GLOBAL_ORIGIN` (lat/lon/alt scaled) |
| `cmd_velocity.rs` (NEW) | `build_cmd_velocity(QuadVelocity) -> Option<MavMessage>` | `SET_POSITION_TARGET_LOCAL_NED` with velocity `type_mask` (ignore position, enable vx/vy/vz) |

> **Graft caveat:** `cmd_velocity.rs` in whisper is only `// ... existing code ...`. There is **no catalogued velocity builder** to copy. Author a new `SET_POSITION_TARGET_LOCAL_NED` builder using a velocity-only `PositionTargetTypemask` (`0b000111000111` style: ignore x/y/z, enable vx/vy/vz (+ yaw optional)). Add `common/types/velocity.rs: QuadVelocity { pub vx,vy,vz: f64 }` (or reuse `Vector3`).

### 4.8 `mavlink/processors/*`
```rust
pub trait MavlinkMessageProcessor {
    fn on_mavlink_message(msg: MavlinkMessageType, data_view: &mut DataView) -> Result<(), anyhow::Error>;
}
```
(No `timing` param — the `output` `DataView` is already `new_timed(timing.time_monotonic.clone())`, and `add_latest` stamps the DataView's internal time. Keeps whisper's extra `timing` arg out of the signature.)

`MavlinkGenericProcessor` dispatch (add 2 grafted arms):
| msg variant | processor | publishes |
|---|---|---|
| `HEARTBEAT` | `HeartbeatProcessor` | `quadlink/status/mode` (`QuadAutopilotStatus`), `quadlink/status/system` (`String`) |
| `PARAM_VALUE` | `ParamValueProcessor` | `quadlink/params/<name>` (`f64`) |
| `STATUSTEXT` | `StatusTextProcessor` | `quadlink/log/text` (`String`) |
| `SYS_STATUS` | `SysStatusProcessor` | `quadlink/status/sensors`, `battery`, `drop_rate_comm`, `voltage`, `current`, `comm_errors`, `errors_count` |
| `COMMAND_ACK` | `CommandAckProcessor` | `quadlink/log/command_ack/<cmd>` (`String`; lowercased) |
| `ATTITUDE` | `AttitudeProcessor` | `quadlink/pose/attitude` (`QuadAttitude`) |
| `LOCAL_POSITION_NED` | `LocalPositionProcessor` | `quadlink/pose/ned` (`QuadPoseNED`) |
| `EKF_STATUS_REPORT` | `EkfHealthProcessor` | `quadlink/status/ekf` (`QuadEkfStatus`) |
| `GLOBAL_POSITION_INT` (graft) | `GpsProcessor` | `quadlink/gps` (`Gps`: lat/1e7, lon/1e7, alt/1000) |
| `HOME_POSITION` (graft) | `HomePositionProcessor` | `quadlink/home` (`QuadHome`) |

### 4.9 `mavlink/system.rs` (`QuadlinkSystem`)
```rust
pub struct QuadlinkSystem {
    mavlink: QuadlinkCoreHandle,
    last_requested_waypoint: Option<QuadPoseNED>,
}

impl QuadlinkSystem {
    pub fn new(mavlink: QuadlinkCoreHandle) -> Self;
    pub fn new_from_connection_string(connection_string: &str) -> Result<Self, anyhow::Error>;
}

impl vs_broker::task::BrokerTask for QuadlinkSystem {
    fn init(&mut self) -> Result<(), anyhow::Error>;  // self.mavlink.lock().unwrap().start_thread()
    fn get_config(&self) -> BrokerTaskConfig;         // name "quadlink-mavlink"
    fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView, anyhow::Error>;
}
```
- `get_config`: `BrokerTaskConfig::new("quadlink-mavlink").with_trigger(BrokerTaskTrigger::Always)`
  - `.with_subscription(BrokerTaskSubscription::new_updates_only(&TopicKey::from_str("quadlink/cmd")))` (child-scoped pull)
  - `.with_subscription(BrokerTaskSubscription::new_latest(&TopicKey::from_str("quadlink/cmd/waypoint")))`
  - `.with_subscription(...new_latest("quadlink/cmd/set_home"))`, `...set_origin`, `...velocity`, `...arm`, `...mode`, `...takeoff`, `...land`.
- `on_execute`:
  1. `let mut output = DataView::new_timed(timing.time_monotonic.clone());`
  2. Drain `self.mavlink.lock().unwrap().recv()?` → for each, `MavlinkGenericProcessor::on_mavlink_message(msg, &mut output)?`.
  3. For each command topic, `inputs.get_latest::<_, ReqType>(&topic)`: if `!req.ack`, build msg, `mavlink.send(...)`, then `req.ack()` and `output.add_latest(&topic, req)`.
  4. Waypoint: send only if it moved > 0.1 from `last_requested_waypoint` (or first time); update cached value.
  5. Return `Ok(output)`.

### 4.10 Module tree
```
libs/vs-mavlink/
  Cargo.toml                      # package vs-mavlink, lib vs_mavlink
  src/
    lib.rs                        # pub mod common; pub mod mavlink;
    common/
      mod.rs
      identifiers.rs              # quadlink/ prefixed
      types/
        mod.rs
        vector3.rs                # Vector3
        attitude.rs               # QuadAttitude
        pose_ned.rs               # QuadPoseNED
        mode.rs                   # QuadMode
        autopilot_status.rs       # QuadAutopilotStatus
        sensor_status.rs          # QuadSensorStatus
        ekf_status.rs             # QuadEkfStatus
        health_status.rs          # QuadHealthStatus
        parameter.rs              # QuadParameter
        request_arm.rs            # QuadArmRequest
        request_mode_set.rs       # QuadSetModeRequest
        request_takeoff.rs        # QuadTakeoffRequest
        request_land.rs           # QuadLandRequest
        gps.rs                    # Gps            (graft)
        home.rs                   # QuadHome       (graft)
        velocity.rs               # QuadVelocity   (graft)
    mavlink/
      mod.rs
      core.rs                     # MavlinkMessageType, QuadLinkError, QuadLinkCore, QuadlinkCoreHandle
      helpers.rs                  # MavLinkHelper
      ardu_mode.rs                # ArduMode enum
      system.rs                   # QuadlinkSystem (BrokerTask)
      builders/
        mod.rs
        cmd_arm.rs
        cmd_mode.rs
        cmd_takeoff.rs
        cmd_land.rs
        cmd_waypoint.rs
        param_set.rs
        cmd_set_home.rs           # graft
        cmd_set_origin.rs         # graft
        cmd_velocity.rs           # NEW (author)
      processors/
        mod.rs                    # MavlinkMessageProcessor trait + MavlinkGenericProcessor
        proc_heartbeat.rs
        proc_param_value.rs
        proc_status_text.rs
        proc_sys_status.rs
        proc_command_ack.rs
        proc_attitude.rs
        proc_local_position.rs
        proc_ekf_health.rs
        proc_gps.rs               # graft
        proc_home.rs              # graft
```

### 4.11 `Cargo.toml`
```toml
[package]
name = "vs-mavlink"
version = "0.1.0"
edition = "2021"

[dependencies]
mavlink = "0.13.1"
crossbeam-channel = "0.5"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"                     # workspace-consistent (vs-broker uses 2)
log = "0.4"
tracing = "0.1"
vs-broker = { path = "../vs-broker" }
vs-data-store = { path = "../vs-data-store" }
vs-wtf = { path = "../vs-wtf" }

[dev-dependencies]
test-log = "0.2"
env_logger = "0.11"
```
Add `"libs/vs-mavlink"` to `[workspace].members`.

## 5. Integration notes
- Use `DataView::new_timed(timing.time_monotonic.clone())` so every published datapoint carries broker time (not `Timepoint::zero()`).
- Thread model stays std threads + `crossbeam-channel`; keep `std::sync::Mutex` (NOT tokio). `BrokerTask: Send` is satisfied by `Arc<Mutex<QuadLinkCore>>`.
- `mavlink::{connect, MavConnection, MavHeader, MavlinkVersion}`; `MavConnection::send(&MavHeader::default(), &msg)`, `send_default(&msg)`, `recv() -> Result<(MavHeader, MavMessage), MessageReadError>`; `MessageReadError::Io(kind)` with `WouldBlock`.

## 6. Risks / remaining blockers
- **`cmd_velocity` must be authored** (whisper is a stub). Define `QuadVelocity` + a velocity-mask `SET_POSITION_TARGET_LOCAL_NED` builder; verify mask bits against ArduPilot.
- **mavlink 0.14 drift**: punt to follow-up; 0.13.1 types/proto are stable and match all port sources.
- **Ack timeout/retry**: out of scope for v1; missing `COMMAND_ACK` is only logged. Add per-command ack timeout + retry later.
- **`Gps` lat/lon numeric type**: whisper used f32; use f64 to avoid precision loss for deg×1e7 lat/lon. Ensure `cmd_set_origin` scaling (`*1e7`, `*1000`) is integer-safe.
- **`request_arm.rs` vs `request_mode_set.rs` naming inversion** in lil-link: normalize module→struct mapping (a `QuadArmRequest` in `request_arm.rs`, `QuadSetModeRequest` in `request_mode_set.rs`) — do not copy lil-link's swapped file contents.
- **No existing consumer** of `quadlink/*` topics; confirm the store is shared and nothing else expects bare `cmd/arm`, `status/mode`, etc.

## 7. Next steps
1. Add `libs/vs-mavlink` to `[workspace].members`.
2. Copy `common/types/*`, port to `vs_*` + `quadlink/` topic prefix.
3. Copy `mavlink/core.rs`, graft devore `should_stop`/`stop_thread`/join lifecycle; move request_parameters/stream handshake into `start_thread`; add gated heartbeat thread.
4. Copy `mavlink/helpers.rs`, `ardu_mode.rs`, all `builders/*`, all `processors/*`.
5. Graft `build_cmd_set_home`, `build_cmd_set_origin`, `proc_gps`, `proc_home` (add `Gps`, `QuadHome`); **author** `build_cmd_velocity` + `QuadVelocity`.
6. Port `system.rs` → `QuadlinkSystem` implementing `vs_broker::task::BrokerTask`; subscribe to `quadlink/cmd/*`.
7. Wire `QuadlinkSystem` as a broker task + add a smoke test against SITL or a mock `mavlink` connection (connect, receive heartbeat, publish `quadlink/status/mode`, drive `quadlink/cmd/arm` → `COMMAND_ACK`).
8. Write `docs/designs/quadlink.md`.

## 8. Links
- Idea: `docs/ideas/quadlink.idea.md`
- Plan (prior): `docs/plan/quadlink.plan.md`
- Task: `docs/tasks/quadlink.task.md`
- Sources: `lil-hopps/lil-link/src/mavlink/`, `project-devore/quad/src/ardulink/connection.rs`, `project-firefly/src_flight/whisper/src/mavlink/`
