---
title: forest
type: plan
status: todo
tags:
  - plan
  - scenario-runner
  - vs-broker
  - vs-data-store
  - vs-wtf
---

# forest — Plan (research continuation, advance)

Port project-firefly's file-driven scenario runner into a reusable `vs-forest`
crate. This is the **continuation** of the original `docs/plan/forest.plan.md`
(PR #33). It adds the precise wingman→vs migration map, resolves the open
questions with concrete decisions, and proposes the crate layout + core traits.

## Confirmed facts

- Source implementation: `project-firefly/src_tools/forest/` (Feb 2025). The **only**
  scenario runner in the org. Code is the spec; `forest/README.md` documents a
  non-existent `AppConfig`/`AppRunner` API and must be ignored.
- The runner is `SILRunner::run(app) → setup → start → { tick } → stop →
  post_process`, over one or more `WhispInstance`s. A whisp = a
  `BrokerServer<LinearBrokerCommander>` + a `BrokerNode` (which owns the tasks) +
  a channel adapter pair connecting them, plus a `SILValidator`.
- The hard coupling to drop: `whisper` (WhisperMavlink task/config, Gps,
  MavlinkConnetionType), `commander` (CommanderConfig/Command/triggers/actions),
  `whisp` (HealthCheck/ShowRunner/Planner/Navigation), `lit-rerun`, `valley`
  (SILValidator/Validators), `waldo` (DirUtils), `rusty_docker_compose`, and the
  `ForestTaskType`/`ForestValidationType` concrete enums.

> **Correction to the original plan:** it stated "`vs-broker`'s `tick` looks sync."
> That is **wrong**. `vs-broker` `Broker::tick(&mut self, delta: Timespan)` is
> `pub async fn ... -> Result<(), BrokerError>` (tokio). The async runner loop
> survives the port unchanged. The real gaps are different (see migration map): the
> vs-broker `Broker` fields are `pub(crate)`/private, and `timespan`/`datapoint`
> semantics changed.

## Precise migration map (wingman → vs)

### vs-broker (`libs/vs-broker`)

| wingman (forest) | vs-* equivalent | Concrete change |
|---|---|---|
| `wingman_task_rs::broker::server::BrokerServer` | `vs_broker::broker::Broker` | rename; generic over `TCommander` |
| `BrokerServer::new(commander)` | `Broker::new(commander)` | same |
| `broker.run(delta).await` | `broker.tick(delta).await` | `pub async fn tick(&mut self, delta: Timespan) -> Result<(), BrokerError>` |
| `broker.add_adapter(adapter)?` | `broker.add_adapter(adapter)` | vs returns `()` **no `?` / no Result** |
| `broker.datastore.read().await` (`Arc<RwLock<Datastore>>`) | **blocked** | `datastore` is `pub(crate)`; `DatastoreHandle = Arc<std::sync::Mutex<Datastore>>`. Needs a public `Broker::datastore(&self) -> &DatastoreHandle` accessor (or make pub). This is required for the `save_datastore` middleware. |
| `broker.timing.clone()` (`pub`) | **blocked** | `timing` is a **private** field. Needs `Broker::timing(&self) -> &BrokerTime` accessor (or make pub). Required for validation time. |
| `BrokerNode::new(info, adapter)` / `.init()` / `.add_task(h)?` / `.tick()` | identical | same signatures; `view` field becomes `DataView` instead of `Datastore` |
| `node.add_task(Arc::new(std::sync::Mutex::new(task)))?` | identical | `BrokerTaskHandle = Arc<Mutex<dyn BrokerTask>>` |
| `wingman...task::BrokerTask` trait | `vs_broker::task::BrokerTask` | `fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView>` (was `&Dataview`/`Dataview`) |
| `BrokerTaskConfig` | identical | `adapter_id: u32`→`AdapterID`, `connection_id: u32`→`ConnectionID`; `with_/set_/add_` builder methods unchanged |
| `SubscriptionMode::{Latest, UpdatesIndex, UpdatesTime}` | `SubscriptionMode::{Latest, NewValues}` | 3 modes → 2; drop `UpdatesIndex`/`UpdatesTime` |
| `BrokerTaskSubscription::new_latest(&TopicPath)` | `new_latest(&dyn TopicKeyProvider)` | pass `&TopicKey`; `new_updates_index/time` → `new_updates_only` |
| `BrokerCommander` / `LinearBrokerCommander` | identical | `add_task`/`get_next_tasks`/`remove_task` unchanged |
| `ChannelBrokerAdapter::new_pair()` | identical | returns `(Arc<Mutex<_>>, Arc<Mutex<_>>)` |
| `TcpBrokerServerConfig::new(addr, port)` / `TcpBrokerClientConfig` | `TcpBrokerServer::new(addr).await` / `TcpBrokerClient::new(addr).await` | config structs replaced by async constructors taking a `&str` address; `ForestGeneratedPort` port logic no longer threads a static config |
| `BrokerNodeInfo::new(name)` | `BrokerNodeInfo::new(name)` | same (rand id); `new_with_id` preserved |
| wingman `BrokerTime { time_delta: Timecode }` | vs `BrokerTime { time_delta: Timespan }` | delta is now a `Timespan` (duration), not `Timecode` |

### vs-data-store (`libs/vs-data-store`)

| wingman | vs | Concrete change |
|---|---|---|
| `wingman_data_rs::topic::TopicPath` | `vs_data_store::topics::TopicKey` | `TopicPath::from_str(s) -> Result` → `TopicKey::from_str(s) -> TopicKey` (infallible). `is_matching` → `matches`. `Topic` wrapper is gone; `TopicKey` is the canonical path type. |
| `wingman_data_rs::topic::TopicKey` (`u16`) | `vs_data_store::topics::TopicKey` (struct of `TopicKeySectionHandle`s) | unrelated concept; forest only used the string path type |
| `wingman_data_rs::datastore::Dataview` | `vs_data_store::database::view::DataView` | `Dataview { datapoints: Vec, topic_info, time_range }` → `DataView { maps: HashMap<TopicKey, Datapoint>, time }` |
| `Dataview::with_query(&ds, &path, DatastoreFilter::Always)` | `DataView::new().add_query(&mut ds, &topic)` | **`&mut Datastore`** required (mutates query_cache); filter is implicit "latest". `DatastoreFilter`/`Always` removed. |
| `Dataview::with_query(..., DatastoreFilter::UpdatedTime(t))` | `DataView::new().add_query_after(&mut ds, &topic, &t)` | time-based filter |
| `DatastoreFilter` enum | **removed** | replaced by choosing `add_query` vs `add_query_after(_per)` |
| `Dataview::extend` | `DataView::maps` (HashMap) merge | `add_query_from_view`/`add_query_after_from_view` |
| `Dataview::to_csv()/to_html()/to_pretty()` | **not present** | `SaveDatastoreMiddleware` must be reworked to format `DataView` directly (prettytable/`csv`), since `DataView` has no serializers |
| `Datapoint { topic: TopicPath, value, timestamp: Timepoint, idx: u64 }` | `Datapoint { topic: TopicKeyHandle, value, time: Timepoint }` | `timestamp`→`time`, `idx` dropped, `topic`→handle. `Datapoint::new(&impl TopicKeyProvider, time: Timepoint, value: Primitives)` |
| `wingman ...datastore::Datastore` | `vs_data_store::database::Datastore` | `Datastore::new()` + `handle(): DatastoreHandle`; buckets store `BTreeMap<Timepoint, Datapoint>` with `RetentionPolicy`; add via `add_datapoint(s)`, read via `get_latest_datapoints`/`get_datapoints_after`/`get_struct` |

### vs-wtf (`libs/vs-wtf`)

| wingman | vs | Concrete change |
|---|---|---|
| `Timespan { start: Timepoint, end: Timepoint }` | `Timespan { time: Timecode }` (**a single duration**) | **biggest semantic change.** `Timespan::new(a,b)` is **gone** → `Timespan::new_secs(f64)` / `Timespan::new_points(start,end)` / `new_ms/us/hz/ns` / `zero()`. `.duration()` is **gone** → use `.secs()/.ms()/.us()/.ns()`. `From<Duration>`→`.as_duration()`. |
| `Timespan::new_secs_f64(f64)` | `Timespan::new_secs(f64)` | ctor already takes `f64` |
| `Timespan::new(start, end)` | `Timespan::new_points(start, end)` | only if a range is truly needed |
| `Timepoint::new_secs_f64(f64)` | `Timepoint::new_secs(f64)` | f64 ctor |
| `Timepoint::zero()` / `Timepoint::min/max` | `zero()` / `Ord` (`a.min(b)`, `a.max(b)`) | same semantics |
| `Timecode { seconds: u64, microseconds: u32 }` | `Timecode { secs: u64, nanos: u32 }` (**private fields** via ctors) | `new_us(u64)`→`new_us(f64)`, `new_ms(u64)`→`new_ms(f64)`, `new_secs(u64)`→`new_secs(f64)`, `new_hz(u64)`→`new_hz(f64)`; access via `.secs()/.ms()/.us()/.ns()` |
| (env-relative) | `Timepoint::now()` | available; wingman had no `now()` on this type |

### Removed / deferred dependencies

| forest dep | target | status |
|---|---|---|
| `waldo::DirUtils::get_firefly_dir()` | `vs-dir` (see `victory-dir.plan.md`) | **not in repo yet**; vs-forest should take output dir as config, not a global |
| `valley::SILValidator` + validators | `vs-valley` (see `valley.plan.md`) | **not in repo yet**; vs-forest defines a generic validator registry trait |
| `whisper / commander / whisp / lit-rerun` | flight-specific | move to `examples/` / downstream `forest-flight` |
| `rusty_docker_compose` | optional integration | drop from core (see decision 3) |

## Decisions (resolved)

### Decision 1 — generic user-supplied registry, not a concrete enum behind a feature

**Recommendation: generic trait registry (primary); flight specifics stay out of the
library crate.**

`ForestTaskType` (WhisperMavlink/Commander/HealthCheck/ShowRunner/ExternalCommander/
Planner/Navigation/RerunSystem) and `ForestValidationType`
(TopicComparison/NumDatapoints/Timeout/Pose/Frequency) are all flight-specific and
depend on crates that are **not** in this workspace. Gating them behind a cargo
`feature` in `vs-forest` would force `vs-forest` to either vendor those deps (breaks
the build) or not compile. Instead:

- `vs-forest` defines a **generic, object-safe `StepFactory`/`ValidatorFactory`
  registry** (name → factory) that yields `vs-broker` task handles and validator
  handles.
- The concrete flight task/validator enums live in `examples/` (dev-dependencies of
  the flight crates) or a separate `forest-flight` crate. They register themselves
  into the registry at scenario build time.

Rationale: the whole point of the idea is decoupling content from runner and making
the runner reusable; a hard-wired enum contradicts it. A generic registry is the main
new design work and the part that makes `vs-forest` dependency-light. Downside
accepted: a `StepSpec` carries an opaque `serde_json::Value` payload, so a given
step's config is not statically typed inside `vs-forest` (typing happens in the
flight layer).

### Decision 2 — keep `ForestGeneratedPort`, drop `DockerComposeConfig` from core

- **`GeneratedPort` (rename from `ForestGeneratedPort`): KEEP, generalize** into the
  `vs-forest::config::port` module. `OffsetPort(base,offset) / FixedPort(port) /
  RandomPort(min,max)` + `calculate_port(index) -> u16` is genuinely useful for any
  multi-instance allocator and is flight-agnostic. No external deps.
- **`DockerComposeConfig`: DROP from the core type graph.** It drags in
  `rusty_docker_compose` and is environment orchestration, not scenario authoring.
  It does not belong on `ScenarioConfig`. Move it to an optional
  `vs-forest::compose` feature (or the downstream `forest-flight` crate) as a
  `middleware`/pre-step that brings up compose and waits. The runner loop must not
  own docker.

### Decision 3 — the "scenario" is a file-driven `ScenarioConfig`, the "step" is a `BrokerTask`

Keep the two-level model (config/scenario + step). `ScenarioConfig` is deserialized
from YAML; each `StepSpec` names a registered step and attaches subscriptions/trigger
+ an opaque payload. A `Scenario` wraps a config and, on `run`, assembles the
vs-broker `Broker` + `BrokerNode` + a channel adapter pair, registers each step via
the registry, and runs the tick loop.

## Proposed crate layout — `libs/vs-forest`

Add to workspace `members`: `"libs/vs-forest"`. `Cargo.toml` deps: `vs-broker`,
`vs-data-store`, `vs-wtf`, `serde`, `serde_json`, `serde_yaml`, `tokio`
(macros/sync/time/rt), `anyhow`, `tracing`, `async-trait`, `thiserror`, `prettytable`
(optional, for the save middleware).

```
libs/vs-forest/
  Cargo.toml
  src/
    lib.rs                 // re-exports; crate docs
    config/
      mod.rs               // ScenarioConfig, NodeSpec, StepSpec, ValidationSpec, load/save yaml
      port.rs              // GeneratedPort { OffsetPort/FixedPort/RandomPort } + calculate_port
    scenario/
      mod.rs               // trait Scenario; assemble_from(config, registry)
    step/
      mod.rs               // trait BrokerStep + StepFactory + StepRegistry (name -> StepFactory)
    validation/
      mod.rs               // trait Validator + ValidatorFactory + ValidatorRegistry; ValidationResult
    registry/
      mod.rs               // Shared registry: register_step/register_validator; build all
    runner/
      mod.rs               // ScenarioRunner (setup/start/tick/stop/post_process)
      context.rs           // RunnerContext { broker handle, node handle, node_info, last_delta, time }
    middleware/
      mod.rs               // trait Middleware + MiddlewareContext + MiddlewareResult
      save_datastore.rs    // writes DataView as csv/log/html
      save_validation.rs   // writes validation results
      print_validation.rs  // prints validation results
    report.rs              // ScenarioReport, stats, pass/fail aggregation
  examples/
    cmd_scenario.rs        // file-driven commander scenario (uses flight dev-deps)
    show_scenario.rs       // timed-show scenario
    flight/               // optional flight StepFactory/ValidatorFactory impls (dev-deps only)
```

### Core traits (signatures, against the real vs APIs)

```rust
// pub use in lib.rs
use std::sync::{Arc, Mutex};
use vs_broker::{
    broker::Broker,
    broker::time::BrokerTime,
    commander::linear::LinearBrokerCommander,
    node::{BrokerNode, info::BrokerNodeInfo},
    task::{BrokerTask, config::BrokerTaskConfig},
};
use vs_data_store::{
    database::{view::DataView, Datastore},
    topics::TopicKey,
};
use vs_wtf::{Timespan, Timepoint};

// ---------- step: a vs-broker task produced from a spec ----------
/// A unit of work in a scenario. Implemented by flight/custom task types;
/// maps 1:1 to `vs_broker::task::BrokerTask`.
pub trait Step: BrokerTask + Send {
    /// Identity + trigger + subscriptions used to register this step.
    fn spec(&self) -> &StepSpec;
}

/// Builds a typed step from an opaque spec. Trait object = user-supplied registry.
pub type StepHandle = Arc<Mutex<dyn BrokerTask>>;
pub trait StepFactory: Send + Sync {
    fn build(&self, spec: &StepSpec) -> anyhow::Result<StepHandle>;
}

// ---------- validation: produced from a spec; lives in the report ----------
pub trait Validator: Send {
    fn name(&self) -> &str;
    fn tick(&mut self, view: &DataView, time: &BrokerTime) -> anyhow::Result<()>;
    fn result(&self) -> &ValidationResult;
}
pub trait ValidatorFactory: Send + Sync {
    fn build(&self, spec: &ValidationSpec) -> anyhow::Result<Arc<Mutex<dyn Validator>>>;
}

// ---------- scenario: the file-driven whole ----------
pub trait Scenario: Send + Sync {
    fn config(&self) -> &ScenarioConfig;
    /// Assemble broker + node + channel adapter + steps + validators from the registry.
    fn build(&self, reg: &Registry) -> anyhow::Result<ScenarioRunner>;
}

// ---------- middleware lifecycle ----------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MiddlewareResult { Continue, Stop, Success(bool), Failure(bool) }

pub struct MiddlewareContext {
    pub broker: Arc<Mutex<Broker<LinearBrokerCommander>>>,
    pub node: Arc<Mutex<BrokerNode>>,
    pub node_info: BrokerNodeInfo,
    pub last_delta: Timespan,
    pub validations: Vec<Arc<Mutex<dyn Validator>>>,
    pub log_directory: std::path::PathBuf,
    pub datastore: Arc<Mutex<Datastore>>,   // shared, also owned by Broker
}

#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn on_init(&mut self, _c: &MiddlewareContext) -> anyhow::Result<MiddlewareResult> { Ok(MiddlewareResult::Continue) }
    async fn on_start(&mut self, _c: &MiddlewareContext) -> anyhow::Result<MiddlewareResult> { Ok(MiddlewareResult::Continue) }
    async fn on_tick(&mut self, _c: &MiddlewareContext) -> anyhow::Result<MiddlewareResult> { Ok(MiddlewareResult::Continue) }
    async fn on_stop(&mut self, _c: &MiddlewareContext) -> anyhow::Result<MiddlewareResult> { Ok(MiddlewareResult::Continue) }
    async fn on_postprocess(&mut self, _c: &MiddlewareContext) -> anyhow::Result<MiddlewareResult> { Ok(MiddlewareResult::Continue) }
}

// ---------- runner: the ported SILRunner loop ----------
pub struct ScenarioRunner {
    pub(crate) broker: Arc<Mutex<Broker<LinearBrokerCommander>>>,
    pub(crate) node: Arc<Mutex<BrokerNode>>,
    pub(crate) datastore: Arc<Mutex<Datastore>>,
    pub(crate) middleware: Vec<Arc<Mutex<dyn Middleware>>>,
    pub(crate) validators: Vec<Arc<Mutex<dyn Validator>>>,
    pub(crate) log_directory: std::path::PathBuf,
}
impl ScenarioRunner {
    pub async fn setup(&mut self) -> anyhow::Result<()>;
    pub async fn start(&self) -> anyhow::Result<()>;
    pub async fn tick(&mut self, delta: Timespan) -> anyhow::Result<bool>;  // true = exit
    pub async fn stop(&self) -> anyhow::Result<()>;
    pub async fn post_process(&self) -> anyhow::Result<ScenarioReport>;
}
```

Key loop detail (port of `WhispInstance::tick`): compute `delta = now - last`
(`Timespan::from_duration`, or `new_secs(dt)`), feed it to `broker.tick(delta).await`
(vs), then drive validators with the shared `datastore` handle + `BrokerTime` (via a
new `Broker::timing()` accessor). Middleware `on_tick` sees `last_delta`, not a
broker-owned time.

## Runner loop / async reconciliation (resolved)

- vs-broker `Broker::tick` is async (tokio `Mutex`), so the existing
  `tokio::spawn { node.tick() }` / `broker.tick(delta).await` loop port 1:1.
- The node side still runs as a separate spawned loop (`node.tick()`), same as forest.
- `async_trait` middleware stays. Middleware is invoked serially per tick, matching
  the original.
- The original used `whisp_instance.lock().await` (tokio Mutex) per whisp; vs-forest
  keeps `Arc<Mutex<ScenarioRunner>>` for shared handles but should prefer `&mut self`
  on the runner loop instead of locking the whole runner each tick.

## Prerequisites / small upstream changes required

1. **vs-broker**: expose `Broker::datastore(&self) -> &DatastoreHandle` and
   `Broker::timing(&self) -> &BrokerTime` (or make `datastore`/`timing` `pub`). Today
   the forest's `save_datastore` middleware and validation can't reach them.
2. **vs-broker**: confirm `TcpBrokerServer::new(addr).await` is the intended
   replacement for `TcpBrokerServerConfig` (no static port config). Bind the port
   allocation to `GeneratedPort`, not to a TCP config struct.
3. **vs-data-store**: `DataView` has no `to_csv`/`to_html`/`to_pretty`. The port
   either adds render helpers to `vs-data-store` or the `save_datastore` middleware
   renders in `vs-forest`.
4. **vs-valley / vs-dir** must land before wiring validation + output-dir, OR
   `vs-forest` ships the generic `Validator`/`ValidatorFactory` traits and a
   caller-supplied base dir (recommended path: traits now, integration once
   `vs-valley`/`vs-dir` land).

## Risks / open questions (narrowed)

- **vs-broker visibility** is the #1 hard blocker for a faithful port (datastore +
  timing). Resolved by the accessor change above; small, mechanical.
- **Timespan semantic change** (range→duration) touches every delta computation in
  the loop and any `Timepoint::min/max`/range logic. Mitigate by centralizing delta
  construction in the runner.
- **`DataView` has no renderers** — `SaveDatastoreMiddleware` must be reworked.
- **Generic registry typing** is the real design cost: `StepSpec` payload is
  `serde_json::Value`, so configs are untyped in core and typed in the flight layer.
- **Deferred deps** (`vs-valley`, `vs-dir`) mean the first `vs-forest` build ships
  traits + runner + save-datastore middleware; validation + dir wiring is gated.

## Next steps (revised)

1. Land the two `vs-broker` accessors + confirm TCP adapter constructor.
2. Scaffold `libs/vs-forest` (workspace member, module tree, traits) and add
   `ScenarioConfig`/`GeneratedPort` + YAML `load/save`.
3. Port `WhispInstance` → `ScenarioRunner` loop and `ForestMiddleware` → `Middleware`;
   wire `save_datastore` (rendering) + `print_validation`.
4. Add the generic `StepFactory`/`ValidatorFactory` registries; move flight task /
   validator enums into `examples/` (dev-deps) and prove a file-driven YAML scenario.
5. When `vs-valley`/`vs-dir` land, plug in validation results + output-dir and add a
   validation example.

## Links

- Idea: `docs/ideas/forest.idea.md`
- Original plan: `docs/plan/forest.plan.md` (PR #33)
- Related plans: `docs/plan/valley.plan.md`, `docs/plan/victory-dir.plan.md`
- Source: `project-firefly/src_tools/forest/`, `project-firefly/src_core/wingman/`
- Targets: `libs/vs-broker`, `libs/vs-data-store`, `libs/vs-wtf`
