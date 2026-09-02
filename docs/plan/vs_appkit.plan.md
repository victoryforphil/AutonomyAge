---
title: vs_appkit
type: plan
status: todo
tags:
  - plan
  - egui
  - ui
---

# vs_appkit (Plan) — UPDATED (research track)

An egui-based helper / component / framework crate for desktop apps in the AutonomyAge `vs-*` Rust workspace. Provides reusable widgets, components, and app scaffolding that sit on top of `egui`. Opinionated but composable — small pieces you opt into, not a monolithic UI layer. Layers on top of `vs-wtf`, `vs-data-store`, and `vs-broker` for live-visible state in tooling.

This revision **resolves the egui version pin**, the `eframe` feature split, and produces a **concrete file/module mapping** derived from the source repos. Supersedes `docs/plan/vs_appkit.plan.md`.

## 1. Purpose

- Give every `vs-*` desktop tool the same visual identity and reusable building blocks without imposing one app-shaped monolith.
- Provide an **`egui`-only core** (no `eframe` dependency) so stateful/instrumentation crates can depend on widgets without pulling a windowing backend.
- Provide an **optional `eframe` app shell** for full desktop apps, behind a cargo feature.
- Keep domain types out of the crate: widgets consume plain data structs / closures, not app-specific state.
- Layering: `vs_appkit::theme` → `vs_appkit::widgets` → `vs_appkit::app` (eframe feature).

## 2. Source repos & egui versions (verified)

| Repo | Path | egui/eframe | Verified how |
|------|------|-------------|--------------|
| agentbox | `crates/tremor-ui/` | **egui 0.34** | `Cargo.toml` (`egui = "0.34"`); uses `ctx.global_style()`/`set_global_style()` |
| mad-rs/cursed | `mad_common/src/ui/` | **egui 0.33** | `Cargo.toml` `bevy_egui = "0.39.1"`; docs.rs confirms `bevy_egui 0.39.1 → egui ^0.33`. Already uses `CornerRadius`, `Margin`, `Frame::window()`, `painter.rect(.., StrokeKind::Outside)` |
| underscore_quad | `src/ui/` | **egui/eframe 0.33** | `Cargo.toml` (`eframe = "0.33"`, `egui_extras = "0.33"`); uses `ctx.set_style()`, `TopBottomPanel`/`SidePanel`/`CentralPanel`, `eframe::App::update`, `CornerRadius`, `Margin` |
| (ref) rerun-interview | `viewer/src/viewer.rs` | 0.31 | reference custom painter. Skip. |
| (ref) agentbox | `nodes/tremor-visor/src/app.rs` | 0.34 | `egui_dock` docking example. Reference only. |
| (ref) cursed-kit template | — | 0.28 | low quality. Skip. |

**Key correction vs. earlier plan:** mad-rs is **0.33**, not 0.32; and underscore_quad is 0.33. The `Rounding`→`CornerRadius` rename already happened in **egui 0.32** (before both 0.33 and 0.34), so it is NOT a porting task. See §3.

## 3. VERSION DECISION — pin `egui = "0.34"` (and `eframe = "0.34"` behind `app`)

**Recommendation: one version, `0.34`.** Rationale:
- `tremor-ui` (the base we port **wholesale**) is already on 0.34 and uses the 0.34 API surface (`global_style`/`set_global_style`). Matching it is costless.
- It is the newest version among all source repos we are actively porting.
- **No workspace conflict exists**: the current workspace members (`vs-wtf`, `vs-data-store`, `vs-broker`) depend on **no** egui/eframe, so introducing 0.34 cannot clash with existing pins. The only "consumer" concern is future `vs-*` tooling opting into egui — since the core is egui-only, they get whichever version the resolved graph picks; standardize on 0.34.
- MSRV: egui 0.34 raises MSRV to **Rust 1.92** (0.33 = 1.88). Acceptable for a fresh crate; note it.
- `eframe 0.34` makes **wgpu the default renderer** (heavy). Fine for standalone desktop tooling under the optional `app` feature.

### 3.1 Concrete 0.33 → 0.34 API renames actually required (from mad_common / underscore_quad)

The earlier plan's assumption about `Rounding`→`CornerRadius` and `Frame` builder renames is **moot** — those already exist in the 0.33 sources. The renames that ARE needed:

| # | 0.33 source code | 0.34 required | Where |
|---|------------------|----------------|-------|
| 1 | `ctx.set_style(s)` | `ctx.set_global_style(s)` | `mad_common/.../theme/style.rs::apply_theme`; `underscore_quad/.../theme.rs::apply_theme` |
| 2 | `ctx.style()` | `ctx.global_style()` (returns `Arc<Style>`, deref, `.as_ref()` to get `&Style`) | `mad_common/.../widgets/modal_scaffold/mod.rs` (`ctx.style().as_ref()`) |
| 3 | `eframe::App::update(&mut self, ctx, frame)` | split into `fn logic(&mut self, ctx: &Context)` + `fn ui(&mut self, ui: &mut egui::Ui)` (deprecated in 0.34, **removed in 0.35**) | `underscore_quad/.../app.rs` |
| 4 | `egui::TopBottomPanel::top/bottom(..).show(ctx, ..)`; `egui::SidePanel::left(..).show(ctx, ..)`; `egui::CentralPanel::default().show(ctx, ..)` | deprecated in 0.34 (still compiles, warns); **removed in 0.35**. Preferred: unified `Panel::top/bottom/left(..).show_inside(ui, ..)` on a `Ui` | `underscore_quad/.../app.rs` |
| 5 | `use bevy_egui::egui;` | `use egui;` | every `mad_common` widget/theme file |
| 6 | — | `visuals.clip_rect_margin` is still valid in 0.34 (set in `mad_common style.rs`). **Remove if we later bump to 0.36** (removed there). | `mad_common/.../theme/style.rs` |

**No change needed (identical across 0.33↔0.34):** `CornerRadius` (`::same`, `::ZERO`, struct fields `nw/ne/sw/se`), `Margin` (`::same`, `::symmetric`, i8 fields), `Frame` builder (`::window(style)`, `::new()`, `.fill/.stroke/.inner_margin/.corner_radius/.shadow`), `StrokeKind::Outside`, `painter.rect/line/text`, `ctx.set_visuals`, `ctx.set_pixels_per_point`, `DragValue::range`, `Rect::rect_filled`, `ScrollArea::show_rows`, `Area::new(..).fixed_pos`, `eframe::run_native`, `egui::ViewportBuilder`, `ctx.load_texture` / `TextureHandle` / `ui.image((id, size))`.

## 4. MODULE TREE + file mapping (concrete)

Place crate at **`libs/vs_appkit`** (add `"libs/vs_appkit"` to workspace `members`). Note: sibling crate names are kebab (`vs-data-store`); if you prefer consistency use `libs/vs-appkit`/`vs-appkit`. Underscore (`vs_appkit`) matches the plan/idea naming throughout; either is fine — pick one and use everywhere.

### `egui`-only core (default features, NO `eframe`)

```
libs/vs_appkit/
├── Cargo.toml                 # egui = "0.34"; [features] default=[], app=["dep:eframe"]
└── src/
    ├── lib.rs                 # crate docs + pub mod / pub use re-exports
    ├── assets.rs              # bundled font bytes  ← tremor-ui/.../assets.rs (DejaVuSans, NotoSansSymbols2)
    ├── icons.rs               # Icon glyph constants  ← tremor-ui/.../icons.rs
    ├── options.rs             # UiOptions + UiTheme + .apply(ctx)  ← tremor-ui/.../options.rs
    ├── toolbar.rs             # Toolbar + Options menu  ← tremor-ui/.../toolbar.rs
    ├── theme/
    │   ├── mod.rs
    │   ├── palette.rs         # merged brand palette (see §4.1)  ← underscore_quad theme.rs colors + mad_common palette.rs semantic/trace constants
    │   ├── style.rs           # apply_theme(ctx), build_style(), badge(), section_heading(), latency_color(), fps_color()  ← underscore_quad theme.rs (+ port mad_common style.rs defaults, drop runtime theme loader lookups)
    │   ├── frames.rs          # overlay()/overlay_unlocked()/modal()/console_overlay()  ← mad_common theme/frames.rs
    │   └── fonts.rs           # install_fonts()/apply_text_sizes()  ← tremor-ui fonts.rs (NOT mad_common fonts.rs — Bevy/system-font/filesystem-bound; skip)
    └── widgets/
        ├── mod.rs
        ├── modal.rs           # ModalOptions + show_modal()  ← mad_common widgets/modal_scaffold/mod.rs
        ├── debug_pane.rs      # DebugPaneContent trait + DebugPaneSpec + show_debug_pane/show_bento_pane  ← mad_common widgets/debug_pane_shell.rs
        ├── debug_dock.rs      # panel_x_slot0..3, DebugDockState, DockPanelState, recompute_key_slots  ← mad_common widgets/debug_dock_state.rs (DROP `bevy::prelude::Resource` + `#[derive(Resource)]`)
        ├── sparkline.rs       # SparklineOptions + draw_sparkline()  ← mad_common widgets/sparkline/mod.rs
        ├── stats_grid.rs      # StatsGridOptions + draw_stars_grid()  ← mad_common widgets/stats_grid/mod.rs
        ├── trace_overlay/
        │   ├── mod.rs
        │   ├── timeline.rs
        │   ├── top_callers.rs
        │   ├── trace_types.rs
        │   └── trace_ui_state.rs   ← mad_common widgets/trace_overlay/{mod,timeline,top_callers,trace_types,trace_ui_state}.rs
        └── image.rs           # ImageView: TextureHandle image widget  ← underscore_quad video_view.rs (generalize: consume egui::ColorImage, drop camera FrameMessage coupling)
```

### optional `app` module — gated behind `feature = "app"` (default off)

```
libs/vs_appkit/src/
└── (feature = "app")
    ├── app.rs                 # eframe::run_native shell + `App` impl (use 0.34 `fn logic` + `fn ui`)  ← underscore_quad src/ui/mod.rs + app.rs
    ├── top_bar.rs             # TopBarState + draw(ui, &state)  ← underscore_quad src/ui/top_bar.rs
    └── status_bar.rs          # StatusBarState + draw(ui, &state)  ← underscore_quad src/ui/status_bar.rs
```

### 4.1 Palette merge decision

Two source palettes conflict:
- **underscore_quad**: teal/cyan accent (`ACCENT = #00bcd4`), blue-black surfaces (`BG_DARK` #12121 8, `BG_PANEL` #181820), `GREEN/YELLOW/RED/ORANGE`, `TEXT_*`, `CODE_BG`. Has a self-contained `apply_theme` + `badge()`/`section_heading()`/`latency_color()`.
- **mad_common**: orange accent (`ACCENT = #ff8700`), near-black surfaces (`SURFACE_0..3`, `BASE`, `FAINT_BG`), `BORDER_*`, `TEXT_PRIMARY/SECONDARY/MUTED`, `INFO/OK/WARN/ERROR`, `TRACE_0..5`, `LAYER_*`.

The mad_common widgets reference `palette::*` constants (`ACCENT`, `SURFACE_1`, `BORDER_SUBTLE`, `BASE`, `INFO`, `OK`, `WARN`, `ERROR`, `TEXT_SECONDARY`, `TEXT_MUTED`, `TRACE_*`, …).

**Recommendation:** build **one** `theme/palette.rs` that takes the **underscore_quad brand** as the base identity (teal accent, blue-black surfaces), then *additionally define* the mad_common constant names (`SURFACE_0..3`, `BASE`, `FAINT_BG`, `BORDER_SUBTLE/MEDIUM/STRONG`, `TEXT_MUTED`, `INFO/OK/WARN/ERROR`, `TRACE_0..5`, `LAYER_CURRENT/NEW`) aliased onto the brand palette. Update each imported widget's `crate::theme::palette::X` path (no value changes). This gives one coherent identity and keeps the widget ports mechanical. (Alternative, if you want to keep the orange identity: do the inverse — adopt mad_common as base and add underscore_quad's badge/section_heading/latency_color helpers. Decide once; record in the task.)

### 4.2 Bevy-bound / skip list (do NOT port, or port only the core)

| File | Reason |
|------|--------|
| `mad_common/.../theme/fonts.rs` | Bevy-agnostic filesystem theme loader (`.madrs/themes/*.toml`, SF Pro / Inter system font paths, `MAD_UI_THEME_PATH` env). Port **none** of it; use `tremor-ui`'s `fonts.rs` + bundled assets instead. |
| `mad_common/.../theme/style.rs` `theme_color()`/`theme_f32()` | reads runtime values from the filesystem theme loader. Port the **defaults** into `theme/style.rs` as fixed constants; drop the `fonts::active_theme_value` lookups. |
| `mad_common/.../widgets/performance_overlay/*` | genuinely Bevy-bound (plugins, `draw_performance_window.rs`, `metric_stats.rs`, `performance_snapshot.rs` reference Bevy ECS/timing). Skip entirely. |
| `mad_common/.../widgets/debug_dock_state.rs` | Only the `#[derive(Resource)]` + `use bevy::prelude::Resource` is Bevy-bound. Drop both; the state structs + `panel_x_slot*` math port as-is. |
| `underscore_quad/.../video_view.rs` `FrameMessage` coupling | camera-specific. Generalize `update()` to accept `egui::ColorImage` (or `&[u8]` + dims) so the widget is reusable. |

## 5. Cargo.toml features

```toml
[package]
name = "vs_appkit"          # or vs-appkit
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.34"

[features]
default = []
app = ["dep:eframe"]

[dependencies.eframe]
version = "0.34"
optional = true

# app feature pulls eframe's default (wgpu) renderer — heavy, so default-off.
```

Constraints:
- **Core must compile with `--no-default-features` / `default = []` and no `eframe`.** Everything in `src/` except the `#[cfg(feature = "app")]` module must be `egui`-only.
- `assets.rs` uses `include_bytes!` — no extra dep.
- Domain crates (`vs-wtf`, `vs-data-store`, `vs-broker`) are **not** dependencies of `vs_appkit`; the widgets consume plain data / the `DebugPaneContent`-like trait.

## 6. Risks / open questions

- **0.34 panel deprecation.** `underscore_quad`'s shell uses `TopBottomPanel`/`SidePanel`/`CentralPanel` and `eframe::App::update`. On 0.34 these compile with deprecation warnings; they are removed in 0.35. **Decision needed:** port the shell to the new unified `Panel` + `App::ui/logic` now (cleaner, more work), or port as-is and accept 0.34-only (must bump to 0.35+ before panels/update land). Recommend the new API for a fresh crate.
- **Palette identity** (orange vs teal brand) — pick one (see §4.1). Blocking a coherent `apply_theme`.
- **`visuals.clip_rect_margin`** is 0.34-valid but removed in 0.36; harmless now, flag for later bump.
- **MSRV 1.92** for egui 0.34.
- **`egui_extras`?** underscore_quad uses it only for SVG/image loaders; not required by the widget kit. Skip unless an image widget needs it.

## 7. Next steps

1. Confirm egui 0.34 pin + palette identity + panel-API approach (see open questions).
2. Port `tremor-ui` wholesale (`options`, `toolbar`, `fonts`, `icons`, `assets`) as the skeleton. `rename` nothing — it's already 0.34.
3. Add `theme/palette.rs` (merged), `theme/style.rs` (apply_theme + badge/section_heading/latency_color + ported build_style defaults), `theme/frames.rs`.
4. Add `widgets/*` from `mad_common`, swapping `bevy_egui::egui` → `egui`, applying `set_style`→`set_global_style`, and dropping the Bevy `Resource` derive and the filesystem theme loader lookups.
5. Add `camera/pan.rs` (pure math, no changes).
6. Scaffold `#[cfg(feature = "app")]` shell using the **0.34** `App::ui`/`logic` + unified `Panel` (or the deprecated API if decision #1 leans that way).
7. Write a small `vs_appkit` demo exercising theme + toolbar + modal + sparkline + a debug_pane.
8. Add unit tests for `camera/pan`, `UiOptions::apply` (round-trip via a mock context—use `egui_kittest` if available), and `debug_dock` `panel_x_slot*` / `recompute_key_slots`.

## 8. Links

- Idea: `docs/ideas/vs_appkit.idea.md`
- Sources: `agentbox/crates/tremor-ui/` (0.34), `mad-rs/cursed/mad_common/src/ui/` (0.33), `underscore_quad/src/ui/` (0.33)
- egui changelog: `emilk/egui` `CHANGELOG.md` + `crates/eframe/CHANGELOG.md` (0.33/0.34 sections)
- bevy_egui→egui mapping: docs.rs `bevy_egui 0.39.1` (dep `egui ^0.33`)
