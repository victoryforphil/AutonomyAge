---
title: valley
type: plan
status: todo
tags:
  - plan
  - validation
  - valley
  - research
---

# valley — Plan (updated)

Port project-firefly's `valley` validation framework into a reusable `vs-*` crate.
Supersedes `docs/plan/valley.plan.md` on branch `vfp/agent/plan/valley`. This revision
resolves the `SkyPose` blocker, pins the exact `vs-*` API mapping, and fixes the
std-vs-tokio and time-serde questions. **The code is the spec** — the README's
`ShowValidator`/`ValidationConfig` API is aspirational and does not exist.

## Source implementations found

| Repo | Path | Date | Notes |
|------|------|------|-------|
| project-firefly | `src_core/valley/` | Feb 2025 | **Only** implementation in the org (confirmed via `gh search code "trait Validator"` / `"ValidatorResult"`). |

Files to port: `src/result.rs`, `src/sil_validator.rs`,
`src/validator/{mod,pose_comparsion,topic_comparsion,validate_frequency,validate_num_datapoints,validate_timeout}.rs`.

## The model to port (unchanged shape)

- `result.rs` — `enum ValidatorResult { Passing, ExitSuccess, ExitFailure, Failing }`
  (each wraps `ValidationResultInfo`); `type ValidationResult<E> = Result<ValidatorResult,E>`;
  `struct ValidationResultInfo { reason: Option<String>, data: HashMap<String,String>,
  failed_at: Option<Timecode>, label: Option<String> }` + builders `failed`, `passed`,
  `failed_at`, `with_field`, `with_label`; aggregators `print_batch_results`,
  `get_passed_batch_results`, `get_failed_batch_results`.
- `validator/mod.rs` — `struct ValidatorInfo { name: String }`; 
  `pub trait Validator { fn validate(&mut self, data: &DataView, time: BrokerTime) -> ValidationResult<anyhow::Error>; fn get_info(&self) -> &ValidatorInfo; }` (sync, `&mut self`).
- Built-ins with serde `Config` + `new_from_config`: `ValidatePose` (`pose_comparsion`),
  `TopicComparisonValidator` (`topic_comparsion`), `ValidateFrequency`, `ValidateNumDatapoints`, `ValidateTimeout`.
- `sil_validator.rs` — `SILValidator { validators: Vec<Arc<Mutex<dyn Validator+Send+Sync>>>, current_results }`;
  `tick(data,time)`; `get_failed_results`/`get_passing_results`/`get_exit_results`/`should_exit`/
  `get_results(ResultType::{Csv,Pretty,Html})`/`print_results`.

## SKY-POSE BLOCKER — RESOLVED

**Decision: port `SkyPose` (+ its support types) into a new `vs-wtf::transforms` module.**
Do **not** define a local pose type in `vs-valley`.

Rationale:
- `pose_comparsion.rs` calls `data.get_struct::<SkyPose>(&topic_path)` and reads
  `pose.position.x/y/z`. Keeping the exact `SkyPose` name + `.position: SkyVec3` lets
  that validator port **line-for-line**.
- `vs-wtf` is already the shared base dep of `vs-data-store` and `vs-broker`, and the
  plan already routes wingman time types → `vs_wtf::*`. A `transforms` module is the
  consistent extension and is reusable by the other project-firefly ports that use
  `SkyPose` (forest samples, whisp, showkit, commander waypoints, whisper).
- A local `vs-valley` pose type would diverge from source semantics and block reuse.

Types to add to `vs-wtf::transforms` (fields exactly as wingman):

```rust
// vector.rs
pub struct SkyVec2 { pub x: f64, pub y: f64 }
pub struct SkyVec3 { pub x: f64, pub y: f64, pub z: f64 }
pub struct SkyVec4 { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }

// quaternion.rs (keep identity/normalize/normalized/conjugate/inverse/soft_equals;
// drop from_rpy/to_rpy so vs-wtf stays serde-only — no quaternion-core dep)
pub struct Quaternion { pub w: f64, pub x: f64, pub y: f64, pub z: f64 }

// frame.rs
pub enum CoordinateSystem { NED, NEU, ENU, Custom(String) }
pub struct SkyFrame { pub name: String, pub parent_frame: Option<String>, pub offset: SkyVec3, pub axis: CoordinateSystem }

// pose.rs
pub struct SkyPose { pub position: SkyVec3, pub rotation: Quaternion, pub frame: SkyFrame }
// + SkyPose::new(position, rotation, frame), ::identity(frame), ::soft_equals(other, eps)
```

The validator only uses `SkyPose.position.x/y/z`; `rotation`/`frame` are retained for
deserialization fidelity (`get_latest::<_, SkyPose>` reads the stored child topics) and
for reuse. **Leaner fallback** (only if vs-wtf must stay minimal): port `SkyPose {
position: SkyVec3 }` only — `PrimitiveDeserializer` ignores un-declared child fields, so
it still decodes full stored poses. Recommended option is the full port.

Note: `SkyFrame`/`CoordinateSystem` round-trip through the `PrimitiveDeserializer` is
**not exercised** by the pose validator (it only reads `position`), so treat any serde
gap there as out of scope unless a future port stores full poses.

## Std vs tokio Mutex — RESOLVED

**Recommend `std::sync::Mutex`.** `Validator::validate` is sync (`&mut self`); the lock
is taken then released fully synchronously inside `tick` (no `.await` while held), so a
std Mutex is safe even when called from an async `forest` loop. This makes `SILValidator::tick`
a **sync** fn `fn tick(&mut self, data: &DataView, time: BrokerTime) -> Result<(), anyhow::Error>`,
drops the `tokio` dependency from `vs-valley`, and matches the std ecosystem of the other
`vs-*` crates. (Source used `tokio::sync::Mutex` only as a convenience handle.)

## Concrete API mapping (wingman → vs-*) — exact names

Imports:

| wingman (firefly) | target (vs-*) | exact path |
|---|---|---|
| `wingman_data_rs::datastore::dataview::Dataview` | `vs_data_store::database::view::DataView` | `use vs_data_store::database::view::DataView;` |
| `wingman_task_rs::broker::time::BrokerTime` | `vs_broker::broker::time::BrokerTime` | `use vs_broker::broker::time::BrokerTime;` |
| `wingman_core_rs::time::Timecode` | `vs_wtf::Timecode` | `use vs_wtf::Timecode;` |
| `wingman_core_rs::time::Timepoint` | `vs_wtf::Timepoint` | `use vs_wtf::Timepoint;` |
| `wingman_core_rs::time::Timespan` | `vs_wtf::Timespan` | `use vs_wtf::Timespan;` |
| `wingman_core_rs::transforms::SkyPose` | `vs_wtf::transforms::SkyPose` (NEW) | `use vs_wtf::transforms::SkyPose;` |
| `wingman_core_rs::transforms::{SkyVec3,Quaternion,SkyFrame}` | `vs_wtf::transforms::*` (NEW) | `use vs_wtf::transforms::*;` |
| `wingman_data_rs::topic::TopicPath` | `vs_data_store::topics::TopicKey` | `use vs_data_store::topics::TopicKey;` |
| `wingman_data_rs::primitives::Primitives` | `vs_data_store::primitives::Primitives` | `use vs_data_store::primitives::Primitives;` |
| `wingman_data_rs::datapoints::Datapoint` | `vs_data_store::datapoints::Datapoint` | `use vs_data_store::datapoints::Datapoint;` |

`BrokerTime` shape (vs-broker): `struct BrokerTime { pub time_monotonic: Timepoint, pub time_delta: Timespan, pub time_last_monotonic: Option<Timepoint> }`,
`Default`, `update(&mut self, delta_time: Timespan)`, `Clone`. `time.time_monotonic.time`
yields a `Timecode` (Copy); `time.time_monotonic > self.timeout` uses `Timepoint`'s `Ord`.

Method calls inside the validators (wingman body → vs-* body):

| wingman call | target call | returns |
|---|---|---|
| `data.get_datapoint(&topic_path)` | `data.get_datapoint(&topic_key)` | `Option<&Datapoint>` (`.value: Primitives`) |
| `data.get_struct::<SkyPose>(&topic_path)` | `data.get_latest::<_, SkyPose>(&topic_key)` | `Result<SkyPose, DatastoreError>` |
| `data.datapoints.len()` | `data.get_all_datapoints().len()` | `Vec<Datapoint>` (owned) |
| `TopicPath::from_str(&self.topic)` | `TopicKey::from_str(&self.topic)` | `TopicKey` (**infallible**, no unwrap / no `?`) |
| `time.time_monotonic.time` | `time.time_monotonic.time` | `Timecode` (Copy) — unchanged |
| `time.time_monotonic > self.timeout` | `time.time_monotonic > self.timeout` | bool (Timepoint Ord) — unchanged |
| (optional) `get_latest_map` | `data.get_latest_map(&topic_key)` | `Result<HashMap<TopicKey,Datapoint>, DatastoreError>` |
| (optional) `get_struct_after` | `data.get_struct_after(&topic_key, &timepoint)` | `Result<Option<S>, DatastoreError>` |
| (optional) `get_datapoints_after` | `data.get_datapoints_after(&topic_key, &timepoint)` | `Vec<&Datapoint>` |

`Primitives` variants (vs-data-store) that `topic_comparsion` matches:
`Integer(i64)`, `Float(f64)`, `Text(String)`, `Boolean(bool)` + more (`Unset, Instant,
Duration, Blob, List, Reference, StructType`). All `_ =>` catch arms still compile.

## Concrete per-file changes during the port

1. `result.rs` — change `use wingman_core_rs::time::timecode::Timecode;` → `use vs_wtf::Timecode;`. Logic unchanged.
2. `validator/mod.rs` — swap imports to `DataView` + `BrokerTime`. `ValidatorInfo` + trait unchanged.
3. `pose_comparsion.rs` — `SkyPose` from `vs_wtf::transforms`; `let topic_key = TopicKey::from_str(&self.topic);`
   (drop `.unwrap()`); `data.get_struct::<SkyPose>(&topic_path)` → `data.get_latest::<_, SkyPose>(&topic_key)`.
4. `topic_comparsion.rs` — `check(&self, data: &DataView)`; `TopicKey::from_str(&self.topic)` (drop `?`);
   `data.get_datapoint(&topic_key)`; `Primitives` arms unchanged.
5. `validate_frequency.rs` — `tokio::time::Instant` → `std::time::Instant`. `data` param `&DataView` (unused).
6. `validate_num_datapoints.rs` — `data.datapoints.len()` → `data.get_all_datapoints().len()`.
7. `validate_timeout.rs` — `Timepoint` from `vs_wtf`. `data` param `&DataView` (unused).
8. `sil_validator.rs` — `tokio::sync::Mutex` → `std::sync::Mutex`; `tick` becomes **sync**;
   `validator.lock().await` → `validator.lock().expect("validator lock poisoned")`; 
   `info.failed_at.map_or(.., |t| t.to_string())` → `|t| t.secs().to_string()` (**vs-wtf `Timecode` has no `Display`**).

## Proposed crate layout

`libs/vs-valley` (new; add `"libs/vs-valley"` to root `[workspace].members`):

```
libs/vs-valley/
  Cargo.toml
  src/
    lib.rs              # pub mod result; pub mod sil_validator; pub mod validator;
    result.rs
    sil_validator.rs
    validator/
      mod.rs            # ValidatorInfo, Validator trait, re-exports
      pose_comparsion.rs
      topic_comparsion.rs
      validate_frequency.rs
      validate_num_datapoints.rs
      validate_timeout.rs
```

`libs/vs-wtf` additions:

```
libs/vs-wtf/src/
  lib.rs                # add `pub mod transforms;` + `pub use transforms::*;`
  transforms/
    mod.rs              # pub use pose::SkyPose; pub use vector::{SkyVec2,SkyVec3,SkyVec4};
                        # pub use quaternion::Quaternion; pub use frame::{SkyFrame,CoordinateSystem};
    vector.rs
    quaternion.rs
    frame.rs
    pose.rs
```

`libs/vs-valley/Cargo.toml` deps:
`vs-data-store = { path = "../vs-data-store" }`,
`vs-broker = { path = "../vs-broker" }`, `vs-wtf = { path = "../vs-wtf" }`,
`anyhow`, `serde = { version="1", features=["derive"] }`, `log`,
`prettytable-rs = "0.10"`, `serde_yaml = "0.9"`. **Drop** `tokio`, `clap`, `tracing`,
`wingman-*`, `whisper`, `thiserror` (keep anyhow error type). `vs-wtf` stays serde-only.

## Validator trait + result signatures (against real vs-* APIs)

```rust
// validator/mod.rs
use vs_data_store::database::view::DataView;
use vs_broker::broker::time::BrokerTime;
use crate::result::ValidationResult;

#[derive(Debug, Clone)]
pub struct ValidatorInfo { pub name: String }

pub trait Validator {
    fn validate(&mut self, data: &DataView, time: BrokerTime) -> ValidationResult<anyhow::Error>;
    fn get_info(&self) -> &ValidatorInfo;
}

// sil_validator.rs (std Mutex, sync tick)
use std::sync::{Arc, Mutex};
pub struct SILValidator {
    pub validators: Vec<Arc<Mutex<dyn Validator + Send + Sync>>>,
    pub current_results: Vec<ValidatorResult>,
}
impl SILValidator {
    pub fn tick(&mut self, data: &DataView, time: BrokerTime) -> Result<(), anyhow::Error> { /* sync, lock().expect() */ }
    pub fn get_failed_results(&self) -> Vec<ValidatorResult>;
    pub fn get_passing_results(&self) -> Vec<ValidatorResult>;
    pub fn get_exit_results(&self) -> Vec<ValidatorResult>;
    pub fn should_exit(&self) -> bool;
    pub fn get_results(&self, result_type: ResultType) -> String;
    pub fn print_results(&self);
}
```

`result.rs` is unchanged from source except the `Timecode` import path; `ValidationResultInfo`
still holds `failed_at: Option<vs_wtf::Timecode>`.

## Risks

- **Time serde / output drift (still real):** wingman `Timecode{seconds,microseconds}` w/
  custom string serde (`"5s"`,`"500ms"`,`"500us"`) **vs** `vs_wtf::Timecode{secs,nanos}`
  w/ derived serde (`{secs,nanos}`). Config YAML + `.csv/.html/.log` output change;
  constructors differ (`new_secs_f64` → `new_secs`).
- **`Timecode` has no `Display`** in vs-wtf — `sil_validator` must render `failed_at` via
  `t.secs()`/`t.ms()`, not `t.to_string()`.
- **Latent index-align bug** in source `get_results`/`print_results`: it pairs
  `self.validators[i]` with `self.current_results[i]` by position. Safe only because each
  validator yields exactly one result per `tick`. Preserve behavior; consider a comment.
- **`topic_comparsion` semi-correct port:** `TopicKey` has no `TopicPath::is_child_of`
  semantics for the validator's flat topic; uses exact `get_datapoint` key lookup — matches
  source behavior.
- **`SkyFrame`/`CoordinateSystem` datastore serde** untested (not exercised by the validator).
- **`add_struct`-stored poses** need `/topic/_type` markers; `get_latest` requires the pose
  be stored as child topics of the queried topic (matches source `get_struct`).

## Next steps

1. Add `transforms` module to `vs-wtf` (SkyPose + SkyVec3/SkyVec2/SkyVec4 + Quaternion + SkyFrame).
2. Create `libs/vs-valley` and register it in the workspace.
3. Port `result.rs` / `validator/mod.rs` / the five validators / `SILValidator` with the
   exact mapping above.
4. Wire `valley` into the `forest` runner's validation middleware (sync `tick`).

## Links

- Idea: `docs/ideas/valley.idea.md`
- Sources: `docs/agents/repo-index.md`
- Source repo: `project-firefly/src_core/valley/`
- Related: `forest.plan.md`
- Task note: `/tmp/opencode/vfp-research/cont/valley.task.updated.md`
