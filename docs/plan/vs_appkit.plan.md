---
title: vs_appkit
type: plan
status: todo
tags:
  - plan
  - egui
  - ui
---

# vs_appkit (Plan)

An egui-based helper / component / framework crate for desktop apps in the AutonomyAge `vs-*` Rust workspace. It provides reusable widgets, components, and app scaffolding that sit on top of `egui`. It is opinionated but composable — small pieces you opt into rather than a monolithic UI layer. It can be layered on top of `vs-wtf`, `vs-data-store`, and `vs-broker` for live-visible state in tooling.

## 1. Purpose

- Give every `vs-*` desktop tool the same visual identity and reusable building blocks without imposing one app-shaped monolith.
- Provide an **`egui`-only core** (no `eframe` dependency) so stateful/instrumentation crates can depend on widgets without pulling in a windowing backend.
- Provide an **optional `eframe` app shell** for full desktop apps, behind a cargo feature.
- Keep domain types out of the crate: widgets consume plain data structs / closures, not app-specific state.
- Let the pieces be layered: `vs_appkit::theme` → `vs_appkit::widgets` → `vs_appkit::app` (eframe feature).

## 2. Source implementations found

| Repo | Path | egui version | Date | Notes |
|------|------|--------------|------|-------|
| agentbox | `agentbox/crates/tremor-ui` | 0.34 | current | **Best base.** No `eframe` dep, composable. `lib.rs`, `options.rs` (`UiOptions` + `UiTheme` + `.apply(ctx)`), `toolbar.rs` (reusable `Toolbar` + Options menu), `fonts.rs` (`install_fonts`, `apply_text_sizes`), `assets.rs`, `icons.rs`. |
| mad-rs/cursed | `mad-rs/cursed/mad_common/src/ui/` | 0.33 | recent | Rich widget kit (mostly `bevy_egui::egui`). `theme/{palette,frames,fonts,style}.rs`, `widgets/modal_scaffold`, `widgets/debug_pane_shell.rs` (`DebugPaneContent` trait + `show_debug_pane`/`show_bento_pane`), `widgets/debug_dock_state.rs` (layout math), `widgets/sparkline/`, `widgets/stats_grid/`, `widgets/trace_overlay/`, `camera/mad_pan_camera.rs` (pure math). |
| underscore_quad | `underscore_quad/src/ui/` | 0.33 | recent | Clean `eframe` shell. `theme.rs` (`apply_theme`, brand palette, `badge()`/`section_heading()`, color thresholds), `app.rs` + `top_bar.rs` + `status_bar.rs` (state-struct → `draw(ui, &state)` panel scaffolding), `video_view.rs` (`TextureHandle` image widget), `mod.rs` (`eframe::run_native` shell). |
| rerun-interview | `rerun-interview/viewer/src/viewer.rs` | 0.31 | ref | `eframe` + `wgpu`, custom painter canvas. Reference only. |
| agentbox | `nodes/tremor-visor/src/app.rs` | 0.34 | ref | `egui_dock` docking example. Reference only. |
| cursed-kit | template | 0.28 | ref | Low quality, older API. Skip. |

## 3. Best version to port

Port **`tremor-ui` (egui 0.34)** as the base, augment with **`mad-rs/cursed` widgets** and **`underscore_quad` theming + panel scaffolding**.

- **Base crate shape:** `agentbox/crates/tremor-ui` — already egui-only, composable, one-concern-per-file.
- **Widgets:** import the pure-egui widgets from `mad_common/src/ui/`, dropping the `bevy_egui::egui` import and the Bevy-bound pieces.
- **Theme + shell:** import `underscore_quad`'s `theme.rs` palette / `apply_theme` / `badge()` / `section_heading()`, plus its top-bar / status-bar panel scaffolding, into the eframe-gated module.

**Version pinning:** choose **one** egui version — 0.34 to match `tremor-ui` (and newest code). Port for the 0.33→0.34 renames: `Rounding`→`CornerRadius`, the `Frame` builder API, `Margin`, `set_visuals`/`global_style`.

## 4. Reusable pieces to extract

| Piece | Source |
|-------|--------|
| `UiOptions` / `UiTheme` + `.apply(ctx)` | `agentbox/crates/tremor-ui/src/options.rs` |
| Font installation + text sizes | `agentbox/crates/tremor-ui/src/fonts.rs` |
| Icon font + glyph constants | `agentbox/crates/tremor-ui/src/icons.rs`, `assets.rs` |
| Reusable `Toolbar` + Options menu | `agentbox/crates/tremor-ui/src/toolbar.rs` |
| Brand palette + `apply_theme` | `underscore_quad/src/ui/theme.rs` |
| `badge()` / `section_heading()` / color thresholds | `underscore_quad/src/ui/theme.rs` |
| Panel layout framework (state-struct → `draw(ui, &state)`) | `underscore_quad/src/ui/{top_bar,status_bar,app}.rs` |
| Image/`TextureHandle` widget | `underscore_quad/src/ui/video_view.rs` |
| Modal scaffold | `mad_common/src/ui/widgets/modal_scaffold/mod.rs` |
| Debug / bento pane trait + shell | `mad_common/src/ui/widgets/debug_pane_shell.rs` |
| Debug dock layout math | `mad_common/src/ui/widgets/debug_dock_state.rs` |
| Sparkline / Stats grid | `mad_common/src/ui/widgets/{sparkline,stats_grid}/` |
| Trace overlay | `mad_common/src/ui/widgets/trace_overlay/*` |
| Pan camera (pure math) | `mad_common/src/ui/camera/mad_pan_camera.rs` |
| Paint canvas / custom painter (ref) | `rerun-interview/viewer/src/viewer.rs` |

**Truly Bevy-bound (skip, or port only the core):** `performance_overlay/*_plugin*.rs`, `draw_performance_window.rs`.

## 5. Structure / modules proposal

**`egui`-only core** (default feature set — no `eframe` dependency):

```
vs_appkit/
├── Cargo.toml        # egui only; "app" feature pulls eframe
└── src/
    ├── lib.rs
    ├── theme/        # palette, style (apply_theme), frames, fonts
    ├── options.rs    # UiOptions + UiTheme + .apply(ctx)
    ├── toolbar.rs    # Toolbar + Options menu
    ├── assets.rs     # bundled font bytes
    ├── icons.rs      # Icon enum + glyph constants
    ├── widgets/      # badge, modal, debug_pane, debug_dock, sparkline, stats_grid, trace_overlay, image
    └── camera/pan.rs # mad_pan_camera (pure math, no egui)
```

**`app` module** — gated behind the optional `eframe` feature (default-off):

```
    └── (feature = "app")
        ├── app.rs        # eframe::run_native shell
        ├── top_bar.rs    # top-bar state struct + draw()
        └── status_bar.rs # bottom-bar state struct + draw()
```

```toml
[dependencies]
egui = "0.34"

[features]
default = []
app = ["dep:eframe"]
```

## 6. Risks / open questions

- **egui version pin**: workspace spans 0.28/0.31/0.33/0.34. Confirm consumers can move to 0.34 (or consider a 0.33 base).
- **`bevy_egui` coupling**: most of `mad_common` imports via `bevy_egui::egui` (swappable); the `performance_overlay` plugins / `draw_performance_window` are genuinely Bevy-bound — skip.
- **`eframe` in the core**: keep the shell out of the default build so data/instrumentation crates get widgets without a windowing backend.
- **Keep domain types out**: widgets consume plain data or a small `DebugPaneContent`-style trait. Do not import `vs-wtf`/`vs-data-store`/`vs-broker` here.
- **`egui::Context` vs `Ui`**: decide per widget whether to take `ctx` (menus/options) or just `ui` (pure drawing).

## 7. Next steps

1. Confirm the egui version pin (0.34 preferred) with workspace consumers.
2. Port `tremor-ui` wholesale as the initial skeleton.
3. Add the `widgets/*` from `mad_common` (swap `bevy_egui::egui`→`egui`), skipping Bevy-bound pieces.
4. Add `theme/palette.rs` + `apply_theme` + `badge()`/`section_heading()` from `underscore_quad`.
5. Scaffold the optional `app` feature (`run_native` shell + top/status bars).
6. Write a small `vs_appkit` demo exercising theme + toolbar + modal + sparkline.
7. Add unit tests for the pure math (`camera/pan`), `UiOptions::apply`, and dock layout math.

## 8. Links

- Idea: [`docs/ideas/vs_appkit.idea.md`](../ideas/vs_appkit.idea.md)
- Sources: [`docs/agents/repo-index.md`](../agents/repo-index.md)
