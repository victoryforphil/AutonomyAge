---
title: vs_appkit — egui desktop app kit plan
type: task
key: vs-appkit
branch: vfp/agent/plan/vs-appkit
pr: https://github.com/victoryforphil/AutonomyAge/pull/40
desc: Port an egui desktop app helper/component framework into a vs-* crate (plan).
status: active
update: Research track done — egui version pinned to 0.34; concrete module/file map produced.
last_updated: 2026-08-23
---

## Context

Base is `agentbox/crates/tremor-ui` (plain egui 0.34, composable, no eframe dep).
Augment with `mad-rs` `mad_common/src/ui/` widgets (theme, modal, debug pane, sparkline,
stats grid, trace overlay, pan camera) and `underscore_quad` theming + panel scaffolding.
egui-only core + optional `eframe` feature.

Research track resolved the open questions:

- **egui version = 0.34** (pin both `egui` and, behind `app`, `eframe` to 0.34). mad_common is
  actually **0.33** (via `bevy_egui 0.39.1 → egui ^0.33`, confirmed on docs.rs), and underscore_quad
  is 0.33. No workspace conflict: `vs-wtf`/`vs-data-store`/`vs-broker` depend on NO egui/eframe.
- **Correction:** `Rounding`→`CornerRadius` and `Frame` builder renames are NOT needed (baked in
  since egui 0.32). The real 0.33→0.34 renames are `ctx.set_style`→`set_global_style`,
  `ctx.style`→`global_style`, `eframe::App::update`→`App::ui`+`App::logic`, and
  `TopBottomPanel`/`SidePanel`/`CentralPanel` → unified `Panel` (+ `show_inside`).
- **eframe stays out of the default build** (recommended): core = egui-only, `app = ["dep:eframe"]`.

See `docs/plan/vs_appkit.plan.md` (existing) and the updated plan at
`/tmp/opencode/vfp-research/cont/vs-appkit.plan.updated.md`.

## Todos

- [x] Survey egui app scaffolds across org
- [x] Draft `docs/plan/vs_appkit.plan.md`
- [x] Deepen research: pin version, map modules, resolve eframe split (this track)
- [ ] Implement `libs/vs_appkit` (core + `app` feature)

## State

- Plan drafted; PR #40 open.
- Research track complete — version decision (0.34), concrete file/module mapping, bevy-bound skip
  list, and the real API renames documented in the updated plan.

## Risks

- **0.34 panel/App-API deprecation:** underscore_quad shell uses `TopBottomPanel`/`SidePanel`/
  `CentralPanel` + `eframe::App::update` (removed in 0.35). Decide: port to new `Panel`/`App::ui`
  now (cleaner) vs accept 0.34-only. Recommend new API.
- **Palette identity conflict:** mad_common (orange `#ff8700`) vs underscore_quad (teal `#00bcd4`)
  brand. Need one merged `theme/palette.rs`; blocking coherent `apply_theme`.
- `visuals.clip_rect_margin` is 0.34-valid but removed in 0.36 (flag for later bump).
- egui 0.34 MSRV = Rust 1.92.

## Human help

- Confirm the egui version pin (**0.34**) with any future `vs-*`/tooling consumers.
- Pick the **palette identity** (teal vs orange brand) — one decision.
- Approve the **panel/App API** approach (new unified `Panel` + `App::ui` vs deprecated 0.34 API).

## Followups

- Implement `libs/vs_appkit`; domain types stay out (widgets consume plain data).
- Port `tremor-ui` wholesale (already 0.34), then `mad_common` widgets (swap `bevy_egui::egui`→`egui`,
  `set_style`→`set_global_style`, drop Bevy `Resource` derive + filesystem theme loader lookups),
  then `underscore_quad` theme/shell under `app`.

## Links

- Plan: `docs/plan/vs_appkit.plan.md`
- Updated plan: `/tmp/opencode/vfp-research/cont/vs-appkit.plan.updated.md`
- Idea: `docs/ideas/vs_appkit.idea.md`
- Sources: `agentbox/crates/tremor-ui` (0.34), `mad-rs/cursed/mad_common/src/ui/` (0.33),
  `underscore_quad/src/ui/` (0.33)

## Open questions

- Palette identity: teal (underscore_quad) vs orange (mad_common) as the `vs_appkit` brand?
- Port the app shell to the new unified `Panel` + `App::ui/logic` (0.34) now, or keep deprecated
  `TopBottomPanel`/`SidePanel`/`CentralPanel` + `App::update` (compiles on 0.34, removed on 0.35)?
- Keep `eframe` out of the default build? **Yes** (recommended, no windowing backend in core).

## Advice / lessons

- Widgets must not import `vs-wtf`/`vs-data-store`/`vs-broker` types.
- Skip mad_common `theme/fonts.rs` (filesystem `.madrs/themes/*.toml` + SF Pro/Inter system-font
  loader) wholesale; use tremor-ui `fonts.rs` + bundled assets.
- Skip `performance_overlay/*` (genuinely Bevy-bound). `debug_dock_state.rs` only needs the Bevy
  `Resource` derive removed (the state structs + `panel_x_slot*` math are egui-free).
- `camera/pan.rs` is pure math — ports with zero changes.
- Generalize underscore_quad `video_view.rs` to consume `egui::ColorImage`, not the camera
  `FrameMessage`, so the image widget is reusable.
