---
title: vs_appkit — egui desktop app kit plan
type: task
key: vs-appkit
branch: vfp/agent/plan/vs-appkit
pr: https://github.com/victoryforphil/AutonomyAge/pull/40
desc: Port an egui desktop app helper/component framework into a vs-* crate (plan).
status: PR open — plan drafted
last_updated: 2026-08-22
---

## Context

Best base is `agentbox/crates/tremor-ui` (plain egui 0.34, composable, no eframe dep).
Augment with `mad-rs` `mad_common/src/ui/` widgets (theme, modal, debug pane, sparkline,
stats grid, pan camera — swap `bevy_egui::egui`→`egui`, skip Bevy-bound pieces) and
`underscore_quad` theming + panel scaffolding. egui-only core + optional `eframe` feature.

## Todos

- [x] Survey egui app scaffolds across org
- [x] Draft `docs/plan/vs_appkit.plan.md`
- [ ] Implement `libs/vs_appkit` (core + `app` feature)

## State

- Plan drafted; PR #40 open.

## Risks

- egui version split 0.28/0.31/0.33/0.34; pin 0.34 and port (`Rounding`→`CornerRadius`,
  `Frame` builder API).
- `bevy_egui` coupling in mad_common; `performance_overlay` plugins genuinely Bevy-bound.

## Human help

- Confirm the egui version pin (0.34 preferred) with workspace consumers.

## Followups

- Implement `libs/vs_appkit`; domain types stay out (widgets consume plain data).

## Links

- Plan: `docs/plan/vs_appkit.plan.md`
- Idea: `docs/ideas/vs_appkit.idea.md`
- Sources: `agentbox/crates/tremor-ui`, `mad-rs/cursed/mad_common/src/ui/`, `underscore_quad/src/ui/`

## Open questions

- Keep `eframe` out of the default build (recommended)?

## Advice / lessons

- Widgets must not import `vs-wtf`/`vs-data-store`/`vs-broker` types.
