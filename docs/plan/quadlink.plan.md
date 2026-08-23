---
title: quadlink
type: plan
status: todo
tags:
  - plan
  - mavlink
---

# quadlink (Plan)

A reusable MAVLink communication & command framework crate for the AutonomyAge `vs-*` Rust workspace. It owns the whole MAVLink side of talking to a vehicle — open a connection (serial/UDP/TCP), run the transport threads, issue commands via builders, decode incoming messages into `vs-data-store` topics, bridge command requests from `vs-broker`, and stamp everything with `vs-wtf` time. It is the shared MAVLink layer so drones, sims, and ground tooling speak the same protocol.

## 1. Purpose

- Give every `vs-*` app/drone/sim a single MAVLink transport + command layer instead of each re-implementing it.
- Send/receive MAVLink messages over a connection string (`/dev/tty...`, `udp://...`, `tcp://...`) on background threads.
- Issue commands with opinionated builders (arm, land, mode, takeoff, waypoint, param set, set-home, set-origin) that return `Option<MavMessage>` and respect ack semantics.
- Decode inbound messages into typed `Quad*` serde structs and publish them as `vs-data-store` topics.
- Implement `vs_broker::task::BrokerTask` so command requests (`cmd/arm`, `cmd/mode`, `cmd/takeoff`, `cmd/land`, `cmd/waypoint`, `params/*`) are consumed from the data store and acked.
- Stamp data with `vs-wtf` time (`DataView::new_timed(timing.time_monotonic.clone())`), preferring the broker tick time.

## 2. Source implementations found

| Repo | Path | mavlink version | Date | Notes |
|------|------|-----------------|------|-------|
| lil-hopps/lil-link | `lil-hopps/lil-link/src/mavlink/` | 0.13.1 | Nov 2024 | **Best base — literally "quadlink".** Public `QuadLinkCore`, `QuadLinkError`, `QuadlinkSystem`; modules `core.rs` (transport + threads), `helpers.rs`, `system.rs` (`victory_broker::task::BrokerTask`), `builders/*` (arm, land, mode, takeoff, waypoint, param_set), `processors/*`, `ardu_modes.rs`; sibling `common/types/*` (pure-serde `Quad*` structs, no protobuf). Already wired to `victory-*` = `vs-*`. Weakness: no graceful shutdown, no cmd_set_home/set_origin, no proc_gps/proc_home. |
| project-firefly (whisper) | `project-firefly/src_flight/whisper/src/mavlink/` | 0.13.1 | Feb 2025 | Newer, fuller framework (`core.rs`, `builders/*`, `processors/*`, `helpers.rs`, `identifiers.rs`, `ardu_mode.rs`, `task.rs`) — includes `build_cmd_set_home`, `build_cmd_set_origin`, `proc_gps`, `proc_home`, `req_parameters.rs`, `req_stream.rs`, `heartbeat.rs`, `cmd_velocity.rs`. But coupled to prost protobuf `whisper::common` types + legacy `wingman-*` crates. Not portable without rewriting. |
| project-devore | `project-devore/quad/src/ardulink/` | 0.13.1 | Nov 2025 | **Best transport lifecycle.** `ArdulinkConnection` with `start_thread`/`stop_thread` (`Vec<JoinHandle>`, joins, `should_stop: Arc<AtomicBool>` gate on send/recv, `recv_timeout(100ms)` send loop, 10ms `WouldBlock` sleep on recv). Graft this lifecycle into the lil-link core. Its pubsub/exec/auto architecture and `ArdulinkConnectionType` config are NOT portable and have no `vs-*` integration. |
| cursed-mav | `cursed-mav` | 0.14.1 | ref | Newer `mavlink` 0.14.1. Only source pinning 0.14; not aligned with the 0.13.1 baseline. Reference for upstream API drift only. |

Rejected as non-MAVLink / empty: `underscore_quad` (Tello UDP binary protocol), `victory-ground-station`, `tiny-gcs`, `delores`, `project-delores` (empty), `mavsight`/`project-mariposa` (no Rust MAVLink), `loki` (ArduPilot SITL Docker/pymavlink only).

## 3. Best version to port

Port the **lil-link `mavlink/` module as the base crate**, then graft three things:

1. **Transport lifecycle from project-devore `ardulink/connection.rs`.** Replace lil-link's `start_thread` (spawns disconnected `thread::spawn`, never joins, un-gated heartbeat loop) with devore's `should_stop: Arc<AtomicBool>` + `thread_handles: Vec<JoinHandle<()>>` + `stop_thread()` (set flag, sleep 100ms, drain and join handles). Send loop uses `recv_timeout(100ms)` with an early `should_stop` break; recv loop keeps the 10ms `WouldBlock` sleep + a `should_stop` check each iteration. Gate `send()`/`recv()` on `should_stop`.

2. **Command builders + processors + helpers from whisper (rewritten to pure-serde).** Bring over `build_cmd_set_home`, `build_cmd_set_origin`, `cmd_velocity`, `heartbeat`, `req_parameters`, `req_stream`, and `proc_gps` / `proc_home`. Do NOT import `whisper::common` — introduce pure-serde `common/types` structs (`Gps`, `QuadHome`, etc.) in the lil-link style. Port the `decode_mode_flag` / `decode_sensor_health` / `decode_ekf_status` helpers already in lil-link's `helpers.rs`.

3. **Keep lil-link's `Quad*` serde types and `BrokerTask` design** (already `vs-*`-shaped); change `victory_*` → `vs_*` crate paths and namespace `identifiers.rs` base constants under a `quadlink/` prefix.

### Crate shape

New member under `libs/`, e.g. `libs/vs-mavlink` (crate `vs_mavlink`; naming open). Module tree mirrors lil-link + whisper grafts:

```
libs/vs-mavlink/src/
  lib.rs
  mavlink/
    mod.rs
    core.rs                 # QuadLinkCore + QuadLinkError + QuadlinkCoreHandle (devore lifecycle)
    helpers.rs              # MavLinkHelper (heartbeat, request_parameters/stream, decode_*)
    arduino_mode.rs         # ArduMode enum + from_u32/to_u32
    system.rs               # QuadlinkSystem implements vs_broker::task::BrokerTask
    builders/               # cmd_arm, cmd_land, cmd_mode, cmd_takeoff, cmd_waypoint,
                            #   cmd_set_home, cmd_set_origin, cmd_velocity, param_set
    processors/             # MavlinkMessageProcessor trait + MavlinkGenericProcessor dispatch
  common/
    identifiers.rs
    types/                  # pure-serde Quad* structs + Gps + Home
```

Dependencies (align to workspace): `mavlink = "0.13.1"`, `crossbeam-channel`, `serde` (derive), `anyhow`, `thiserror = "2"`, `log`, `tracing`, plus `vs-wtf`/`vs-data-store`/`vs-broker` paths.

## 4. Core API surface to port

- **Connection / transport (`core.rs`)**: `type MavlinkMessageType = mavlink::ardupilotmega::MavMessage;`, `enum QuadLinkError` (`MavlinkError`, `ChannelRecvError`, `ChannelSendError`, `ConnectionError`, `GenericError`, `NoData`), `struct QuadLinkCore { recv_channels, transmit_channels, connection_string, should_stop, thread_handles }`, `type QuadlinkCoreHandle = Arc<Mutex<QuadLinkCore>>`, `new(connection_string)`, `start_thread()`, `stop_thread()`, `send(&MavlinkMessageType)`, `recv() -> Vec<MavlinkMessageType>`.
- **Helpers (`helpers.rs` `MavLinkHelper`)**: `heartbeat_message()`, `request_parameters()` (`PARAM_REQUEST_LIST`), `request_stream()` (`REQUEST_DATA_STREAM`), `decode_mode_flag`, `decode_sensor_health`, `decode_ekf_status`, `quad_mode_to_mav_mode`.
- **Command builders (`builders/*`)** — return `Option<MavMessage>` (None when already acked): `mavlink_build_arm_message`, `mavlink_build_mode_message`, `mavlink_build_cmd_takeoff_message`, `mavlink_build_cmd_land_message`, `mavlink_build_cmd_waypoint_message`, `mavlink_build_param_set_message`, `build_cmd_set_home`, `build_cmd_set_origin`, `build_cmd_velocity`.
- **Message processors (`processors/*`)**: `trait MavlinkMessageProcessor { fn on_mavlink_message(msg, data_view: &mut DataView) -> Result<(), anyhow::Error>; }` + `MavlinkGenericProcessor` dispatch (HEARTBEAT, PARAM_VALUE, STATUSTEXT, SYS_STATUS, COMMAND_ACK, ATTITUDE, LOCAL_POSITION_NED, EKF_STATUS_REPORT, + grafted GLOBAL_POSITION_INT/HOME_POSITION).
- **Broker task wrapper (`system.rs` `QuadlinkSystem`)** implements `BrokerTask`: `init` starts thread; `get_config` (`BrokerTaskConfig::new("quadlink-mavlink")` + trigger `Always` + `cmd`/`cmd/waypoint` subscriptions); `on_execute` uses `DataView::new_timed(timing.time_monotonic.clone())`, drains `recv()` into the generic processor, reads `cmd/*` requests via `inputs.get_latest(...)`, dispatches builders, sends, and acks.
- **Shared types (`common/types/*`)**: pure-serde `Vector3`, `QuadAttitude`, `QuadPoseNED`, `QuadMode`, `QuadAutopilotStatus`, `QuadSensorStatus`, `QuadEkfStatus`, `QuadHealthStatus`, `QuadParameter`, request structs, plus portable `Gps`/`QuadHome`.
- **Topic convention (`common/identifiers.rs`)** — use a `quadlink/` prefix to avoid collisions in a shared store.

## 5. Integration notes

- **vs-wtf**: `BrokerTime { time_monotonic: Timepoint }` is the single timing source; use `DataView::new_timed(timing.time_monotonic.clone())` so published datapoints carry broker time (not `Timepoint::zero()`).
- **vs-data-store**: `DataView`, `TopicKey::from_str`, `DataView::new_timed(Timepoint)`, `add_latest(&TopicKey, value)`.
- **vs-broker**: `task::BrokerTask`, `task::config::BrokerTaskConfig`, `task::subscription::BrokerTaskSubscription` (built from a `TopicKey`, not a raw `&str`), `task::trigger::BrokerTaskTrigger`, `broker::time::BrokerTime`.
- **mavlink crate**: `Box<dyn MavConnection<MavMessage> + Send + Sync>`, `mavlink::connect`, `set_protocol_version(V2)`, `send(&MavHeader::default(), &msg)`, `recv()`, `MessageReadError::Io(WouldBlock)`. Keep `std::sync::Mutex`.
- **Rename mapping**: `victory_wtf`→`vs_wtf`, `victory_data_store`→`vs_data_store`, `victory_broker`→`vs_broker`; drop `wingman-*` in favor of pure-serde `common/types`.

## 6. Risks / open questions

- **mavlink version**: baseline 0.13.1 everywhere; `cursed-mav` pins 0.14.1. Recommend staying on 0.13.1 for the first port.
- **Coupling to whisper's prost `whisper::common`**: rewrite to pure-serde; do not pull in prost or `wingman-*`.
- **Graceful shutdown**: lil-link never joins and has an un-gated heartbeat loop; graft devore's `should_stop`/`stop_thread`/join. Decide where the heartbeat lives (recommend moving to `QuadlinkSystem`/BrokerTask so it's gated).
- **Handshake placement**: confirm whether the initial `request_parameters`/`request_stream` belongs in `start_thread` (connect-time) or is a gated command publish from the broker layer.
- **Ack semantics**: builders return `None` on already-acked requests; no timeout/retry for a missing `COMMAND_ACK` — consider adding a per-command ack timeout + retry later.
- **Topic namespace**: adopt a `quadlink/` prefix so multiple vehicles/sims can share one store; confirm no existing consumer expects the old bare keys.
- **Crate naming / workspace membership**: add `libs/vs-mavlink` (or keep `quadlink`); confirm vs-broker/vs-data-store are workspace members (they are).

## 7. Next steps

1. Add `libs/vs-mavlink` to the workspace `[members]`.
2. Import the lil-link `mavlink/` module, rename `victory_*`→`vs_*`.
3. Graft devore's `should_stop`/`stop_thread`/join lifecycle into `core.rs`.
4. Port whisper's `build_cmd_set_home`/`build_cmd_set_origin`/`cmd_velocity`/`proc_gps`/`proc_home` + add pure-serde `Gps`/`QuadHome`.
5. Adopt the `quadlink/` topic prefix in `identifiers.rs`.
6. Wire `QuadlinkSystem` as a `vs_broker` task.
7. Add a smoke test against SITL (or a mock `mavlink` connection): connect, receive heartbeat, publish `status/mode`, drive `cmd/arm` → `COMMAND_ACK`.
8. Write a short design note in `docs/designs/`.

## 8. Links

- Idea: [`docs/ideas/quadlink.idea.md`](../ideas/quadlink.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
