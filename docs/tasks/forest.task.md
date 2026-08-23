---
title: forest — scenario runner port
type: task
key: forest
branch: vfp/agent/plan/forest
pr: https://github.com/victoryforphil/AutonomyAge/pull/33
desc: Port project-firefly's file-driven scenario runner into a reusable vs-forest crate.
status: active
update: research continued — wingman→vs migration map, generic-registry decision, crate layout drafted
last_updated: 2026-08-23
---

## Context

project-firefly's scenario runner is the only implementation in the org. Scenarios
are file-driven (`config.yaml` commander commands trigger→action, `show.yaml` timed
show) with pluggable `ForestMiddleware` handlers. The runner is
`SILRunner::run(app) → setup → start → { tick } → stop → post_process` over one or
more `WhispInstance`s (each = a `BrokerServer<LinearBrokerCommander>` + a
`BrokerNode` of tasks + a channel adapter pair + a `SILValidator`).

Research continuation confirmed the actual target APIs by reading
`libs/vs-broker`, `libs/vs-data-store`, `libs/vs-wtf` and the wingman-* sources. The
full, method-level migration map is in the updated plan.

**Corrected a previous assumption**: `vs-broker` `Broker::tick(delta)` is **async**
(`pub async fn tick(&mut self, delta: Timespan) -> Result<(), BrokerError>`), so the
tokio runner loop survives the port. The real gaps are `Broker` field visibility,
`Timespan` semantic change, and `DataView` lacking serializers.

## Todos

- [x] Survey org for scenario runners (only project-firefly)
- [x] Draft `docs/plan/forest.plan.md`
- [x] Read wingman-* + vs-* sources; produce precise migration map + resolve decisions
- [ ] Land `vs-broker` accessors: `Broker::datastore()` + `Broker::timing()` (blocked today)
- [ ] Scaffold `libs/vs-forest` (workspace member, module tree, traits, `ScenarioConfig`/`GeneratedPort`)
- [ ] Port `WhispInstance`→`ScenarioRunner` loop; `ForestMiddleware`→`Middleware`; wire `save_datastore` + `print_validation`
- [ ] Add generic `StepFactory`/`ValidatorFactory` registries; move flight task/validator enums to `examples/`
- [ ] Wire into `vs-valley` + `vs-dir` when they land

## State

- Plan + decision draft advanced in `/tmp/opencode/vfp-research/cont/forest.plan.updated.md`.
- PR #33 open (original plan).

## Risks

- **vs-broker visibility** — `Broker`'s `datastore`/`timing` are `pub(crate)`/private;
  forest's `save_datastore` + validation can't reach them. Needs 2 public accessors.
- **Timespan semantic change** — wingman `{start,end}` range → vs single duration;
  every delta calc in the loop changes.
- **`DataView` has no `to_csv`/`to_html`/`to_pretty`** — `SaveDatastoreMiddleware` must render itself.
- **Generic registry typing** — `StepSpec` payloads become `serde_json::Value`;
  configs untyped in core, typed in the flight layer.

## Human help

- Confirm the generic `StepFactory`/`ValidatorFactory` registry (decision 1) is the
  desired direction vs. keeping a concrete enum behind a cargo feature.
- Confirm dropping `DockerComposeConfig` from the core type graph (decision 2) and
  keeping the generalized `GeneratedPort`.
- Approve adding the two `vs-broker` accessors (they are the single hard blocker).

## Followups

- Scaffold `libs/vs-forest` after the accessors land and the registry direction is
  confirmed. Depends on `vs-dir` and `vs-valley` for validation + output-dir wiring.

## Links

- Plan (revised): `/tmp/opencode/vfp-research/cont/forest.plan.updated.md`
- Plan: `docs/plan/forest.plan.md`
- Idea: `docs/ideas/forest.idea.md`
- Source: `project-firefly/src_tools/forest/`, `project-firefly/src_core/wingman/`
- Targets: `libs/vs-broker`, `libs/vs-data-store`, `libs/vs-wtf`
- Related: `docs/plan/valley.plan.md`, `docs/plan/victory-dir.plan.md`

## Open questions

- [x] Keep `ForestGeneratedPort` / `DockerComposeConfig`? → Keep+generalize `GeneratedPort`; drop `DockerComposeConfig` from core.
- [x] Generic registry or concrete enum behind a feature? → Generic `StepFactory`/`ValidatorFactory`; flight logic in `examples/`.
- [ ] Does `vs-broker` accept 2 public accessors? → Required; awaiting confirmation.
- [ ] Should `DataView` gain `to_csv`/`to_html` helpers, or should `vs-forest` render? → Lean `vs-forest` renders, keeps `vs-data-store` lean.

## Advice / lessons

- forest/valley READMEs document an API that doesn't exist; trust the code.
- Read the actual `vs-*` `lib.rs` + `Cargo.toml` before writing the port — several
  plan assumptions (e.g. "tick is sync", "DataView has with_query") were wrong.
- Centralize delta/`Timespan` construction in the runner to contain the semantic
  change; keep the payload type-erasure boundary at the registry (typed in examples).
