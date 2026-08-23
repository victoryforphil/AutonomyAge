---
title: valley
type: plan
status: todo
tags:
  - plan
  - validation
---

# valley — Plan

Port project-firefly's validation framework into a reusable crate.

## Purpose

Extensible, scriptable validation checks with clear pass/fail results. Scriptable
checks = validation rules defined as data/scripts rather than hardcoded code. Each
check produces structured pass/fail + diagnostics, not a bare boolean. Composes with
`forest` (scenario runner) and `vs-data-store` (storing results).

## Source implementations found

| Repo | Path | Date | Notes |
|------|------|------|-------|
| project-firefly | `src_core/valley/` | Feb 2025 | The **only** implementation in the org (confirmed via `gh search code "trait Validator"` / `"ValidatorResult"`). |

No alternative exists. `README.md` describes an aspirational `ShowValidator`/
`ValidationConfig` API that does not exist — the **code is the spec**.

## The model to port

- `result.rs`
  - `enum ValidatorResult { Passing, ExitSuccess, ExitFailure, Failing }` each
    wrapping `ValidationResultInfo`.
  - `type ValidationResult<E> = Result<ValidatorResult, E>`.
  - `struct ValidationResultInfo { reason: Option<String>, data: HashMap<String,String>,
    failed_at: Option<Timecode>, label: Option<String> }` with builder helpers
    (`failed`, `passed`, `failed_at`, `with_field`, `with_label`).
  - Aggregators: `print_batch_results`, `get_passed_batch_results`,
    `get_failed_batch_results`.
- `validator/mod.rs`
  - `struct ValidatorInfo { name: String }`.
  - `pub trait Validator { fn validate(&mut self, data, time) -> ValidationResult<anyhow::Error>; fn get_info(&self) -> &ValidatorInfo; }` — sync, `&mut self`.
  - Built-ins: `pose_comparsion`, `topic_comparsion`, `validate_frequency`,
    `validate_num_datapoints`, `validate_timeout` (each has a serde `Config` +
    `new_from_config`).
- `sil_validator.rs` — `SILValidator { validators: Vec<Arc<Mutex<dyn Validator+Send+Sync>>>, current_results }`; `tick(data, time)` runs all; query helpers
  `get_failed_results`/`get_passing_results`/`get_exit_results`/`should_exit`/
  `get_results(ResultType::{Csv,Pretty,Html})`/`print_results` (rendered via
  `prettytable-rs`).

## Best version to port

Port `project-firefly/src_core/valley` (the only/newest version) and de-couple from
`wingman-*`. `SILValidator` uses `tokio::sync::Mutex` only for the handle; the
`Validator::validate` trait itself is sync — prefer `std::sync::Mutex` in the port.

## De-coupling (wingman → vs-*)

| firefly (wingman) | target (vs-*) |
|-------------------|---------------|
| `wingman_data_rs::Dataview` | `vs_data_store::DataView` |
| `Dataview::with_query(&ds, &TopicPath, DatastoreFilter)` | `DataView::new().add_query(&mut ds, &TopicKey)` |
| `data.get_datapoint(&TopicPath).value` | `view.get_datapoint(&TopicKey)` (`Option<&Datapoint>`, `.value: Primitives`) |
| `data.get_struct::<SkyPose>(&TopicPath)` | `view.get_latest::<_, SkyPose>(&Topic)` (no plain `get_struct`) |
| `data.datapoints.len()` | `data.get_all_datapoints().len()` |
| `wingman_task_rs::broker::time::BrokerTime` | `vs_broker::broker::time::BrokerTime` |
| `wingman_core_rs::time::{Timecode,Timepoint,Timespan}` | `vs_wtf::*` |
| `wingman_core_rs::transforms::SkyPose` | **no equivalent** in `vs-wtf` |

## Risks / open questions

- **`SkyPose` is a blocker.** Used by `pose_comparsion.rs` and forest samples, but
  `vs-wtf` has no `transforms` module. Fix: define a local pose type, add a
  transforms module to `vs-wtf`, or drop the pose validator.
- **Time type/serde drift.** wingman `Timecode{seconds,microseconds}` with custom
  string serde (`"5s"`, `"500ms"`, `"500us"`) vs `vs-wtf` `Timecode{secs,nanos}`
  with derived serde (`{secs,nanos}`). Config YAML and result `.csv/.html/.log`
  output will change. Constructor names differ too (`new_secs_f64` vs `new_secs`).
- **README drift** — trust the code, not the README.

## Next steps

1. Add `libs/vs-valley` depending on `vs-data-store`+`vs-broker`+`vs-wtf`.
2. Port `result.rs` / `validator/mod.rs` / the five validators / `SILValidator`.
3. Resolve the `SkyPose` dependency (see risk).
4. Add `valley` into the `forest` runner's validation middleware.

## Links

- Idea: [`docs/ideas/valley.idea.md`](../ideas/valley.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
- Related: `forest.plan.md`
