//! Loaders: the anastasia pulse loader, the gradient matrix spinner, and the boot
//! splash content. All motion routes through `crate::motion` pure helpers, so
//! the math is unit-tested and these elements are testable-by-compile.
//!
//! Rendering pattern: each cell is its own `with_animation` repeating element
//! sharing one period; per-cell offsets come from [`motion::staggered_phase`],
//! so all cells stay phase-locked (they start on the same frame) without a
//! shared clock. Cells animate inside fixed-size slots — opacity and inner size
//! are paint-local and never move surrounding layout. Reduced motion snaps every
//! cell to its rest state automatically (gpui `reduce_motion`).

use gpui::{AnyElement, App, EntityId, IntoElement, ParentElement, SharedString, Styled, div, px};

use crate::motion::{self, ANASTASIA_PULSE, GRADIENT_SPIN, PULSE_STAGGER, SPLASH_OUT};
use crate::theme::Theme;

// Shared with the terminal viewport (`anastasia_proto::motion`) so both animate the
// same loaders from the same numbers.
pub use anastasia_proto::motion::{
    ANASTASIA_CELLS, MARK_CELLS, MARK_SPREAD, MATRIX_SIDE, mark_cell_stagger,
};

/// The animated anastasia mark (anastasia-loader.tsx `AnastasiaLoader`): the full logo
/// pixel grid with a light wave sweeping tail→head. Each cell rests dim
/// (opacity 0.08, scale 0.9) and flares to full as the crest passes; per-cell
/// stagger follows the flight axis. `height_px` sets the mark's height (width
/// follows the 820:940 canvas).
pub fn anastasia_mark_loader(
    _id: &'static str,
    theme: &Theme,
    height_px: f32,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    let color = theme.text;
    let scale = height_px / 940.0;
    let cell = 100.0 * scale;
    let delta = motion::pulse_delta(&ANASTASIA_PULSE, view, cx);
    div()
        .relative()
        .w(px(820.0 * scale))
        .h(px(height_px))
        .children(MARK_CELLS.iter().map(move |&(x, y)| {
            let stagger = mark_cell_stagger(x, y);
            // Fixed slot; the animated cell breathes inside it (paint-local).
            div()
                .absolute()
                .left(px(x * scale))
                .top(px(y * scale))
                .size(px(cell))
                .flex()
                .items_center()
                .justify_center()
                .child({
                    // Negative CSS delay ⇒ the cell starts mid-cycle:
                    // the stagger ADDS phase (anastasia-loader.tsx delayFor).
                    let phase = (delta + stagger).rem_euclid(1.0);
                    div()
                        .rounded(px(16.0 * scale))
                        .bg(color)
                        .opacity(motion::pulse_opacity(phase))
                        .size(px(cell * motion::pulse_scale(phase)))
                })
        }))
}

/// The anastasia wave loader: a row of cells pulsing opacity 0.08→1 / scale 0.9→1
/// over 2.4s with a 0.15s stagger per cell.
///
/// `id` scopes the per-cell animation state — give each loader instance a
/// distinct id.
pub fn anastasia_loader(
    _id: &'static str,
    theme: &Theme,
    cell_px: f32,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    let color = theme.text;
    let slot = cell_px;
    let delta = motion::pulse_delta(&ANASTASIA_PULSE, view, cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(slot / 2.0))
        .children((0..ANASTASIA_CELLS).map(move |i| {
            // Fixed slot; the animated cell breathes inside it.
            div()
                .size(px(slot))
                .flex()
                .items_center()
                .justify_center()
                .child({
                    let phase = motion::staggered_phase(delta, i, PULSE_STAGGER);
                    div()
                        .rounded(px(slot / 4.0))
                        .bg(color)
                        .opacity(motion::pulse_opacity(phase))
                        .size(px(slot * motion::pulse_scale(phase)))
                })
        }))
}

pub use anastasia_proto::motion::{GSPIN_DIM, GSPIN_ROW_TINTS};

/// The gradient matrix spinner (WorkingIndicator), ported from anastasia's
/// gradient-spin.tsx: a 3×3 grid of round cells tinted per row from the
/// sunrise gradient. Each cell pulses opacity once per 750ms period; the
/// per-cell phase follows the "arrow-up" pattern (the pulse enters at the
/// bottom edge and converges toward the top-center cell), so the wave reads
/// as travelling upward.
pub fn gradient_spinner(
    _id: &'static str,
    _theme: &Theme,
    cell_px: f32,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    let center = (MATRIX_SIDE as f32 - 1.0) / 2.0;
    let max = MATRIX_SIDE as f32 - 1.0 + center;
    let delta = motion::pulse_delta(&GRADIENT_SPIN, view, cx);
    div()
        .flex()
        .flex_col()
        .gap(px(cell_px / 2.0))
        .children((0..MATRIX_SIDE).map(move |row| {
            let tint: gpui::Hsla = gpui::rgb(GSPIN_ROW_TINTS[row]).into();
            div()
                .flex()
                .flex_row()
                .gap(px(cell_px / 2.0))
                .children((0..MATRIX_SIDE).map(move |col| {
                    // Distance of this cell from the wave origin, normalized
                    // into a phase offset (gradient-spin's `--gspin-phase`).
                    let d = MATRIX_SIDE as f32 - 1.0 - row as f32 + (col as f32 - center).abs();
                    let phase = if max == 0.0 { 0.0 } else { d / (max + 1.0) };
                    div()
                        .size(px(cell_px))
                        .rounded(px(cell_px / 2.0))
                        .bg(tint)
                        .opacity(motion::gspin_opacity(delta + phase, GSPIN_DIM))
                }))
        }))
}

/// A 2×3 miniature of [`gradient_spinner`] sized for a status-dot slot
/// (sessions-sidebar working rows): same row tints and pulse timing, but the
/// brightness SNAKES around the grid's perimeter (every cell of a 2×3 grid is
/// on the ring) instead of sweeping as a vertical wave — a tiny radial chase.
/// ~6×10px footprint at the default 2.5px cells.
pub fn mini_gradient_spinner(
    key: impl Into<SharedString>,
    cell_px: f32,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    const COLS: usize = 2;
    const ROWS: usize = 3;
    /// Clockwise ring position of each `(row, col)` cell, top-left first:
    /// (0,0) → (0,1) → (1,1) → (2,1) → (2,0) → (1,0).
    const RING: [[usize; COLS]; ROWS] = [[0, 1], [5, 2], [4, 3]];
    const RING_LEN: f32 = (COLS * ROWS) as f32;
    let _key = key.into();
    let delta = motion::pulse_delta(&GRADIENT_SPIN, view, cx);
    div()
        .flex()
        .flex_col()
        .gap(px(cell_px / 2.0))
        .children((0..ROWS).map(move |row| {
            let tint: gpui::Hsla = gpui::rgb(GSPIN_ROW_TINTS[row]).into();
            div()
                .flex()
                .flex_row()
                .gap(px(cell_px / 2.0))
                .children((0..COLS).map(move |col| {
                    let phase = RING[row][col] as f32 / RING_LEN;
                    div()
                        .size(px(cell_px))
                        .rounded(px(cell_px / 2.0))
                        .bg(tint)
                        .opacity(motion::gspin_opacity(delta + phase, GSPIN_DIM))
                }))
        }))
}

/// Full-window boot splash (anastasia App.tsx `Splash`): the animated anastasia mark
/// (`h-16`) over the app background with an uppercase tracked "Loading" line.
/// While `fading` it plays `splash-out` (150ms hold, then 0.5s fade + 6px
/// lift); the shell removes it once [`SPLASH_OUT`] has run its course.
pub fn splash_overlay(theme: &Theme, fading: bool, view: EntityId, cx: &mut App) -> AnyElement {
    let content = div()
        .absolute()
        .inset_0()
        // Frosted glass, not the opaque page tone (user request): the boot
        // overlay reads like the rest of the chrome — the frost tint over
        // the blurred window background (opaque platforms get the surface
        // tone, since `glass()` collapses to it there).
        .bg(theme.glass())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(28.0))
        .child(hero_dots(theme, view, cx))
        .child(loading_word(theme));
    if fading {
        motion::splash_out("boot-splash-out", content).into_any_element()
    } else {
        content.into_any_element()
    }
}

/// Grid pitch and dot geometry for [`hero_dots`]. The pitch sets the mark's
/// footprint (46 × 8 ≈ 368px square) — comfortably inside a narrow window.
const DOT_PITCH: f32 = 8.0;
/// The largest a dot grows, as a fraction of the pitch. Below 1.0 so even
/// solid cells keep the gaps that make the field read as a halftone.
const DOT_MAX: f32 = 0.72;
/// Fraction of the pulse cycle the light sweep occupies as it crosses the
/// grid's diagonal — the same idea as [`MARK_SPREAD`], on both axes.
const SWEEP_SPREAD: f32 = 0.55;
/// Rest opacity of a solid cell; the sweep lifts it to full.
const DOT_DIM: f32 = 0.28;

/// The Anastasia mark as a halftone dot field, with a light sweep crossing it
/// on the diagonal.
///
/// The grid comes from `assets/hero-dots.txt` (see `scripts/halftone.py`):
/// one digit per cell, `0`–`9`, sampled from the logo's ink coverage. Cell
/// density drives BOTH dot diameter and opacity, so the glyph's edges fade
/// into the surrounding field instead of ending on a staircase — and the dim
/// background cells stay visible as the field the mark sits in.
///
/// `crate::edge_fade` on all four edges dissolves the block into the frost,
/// standing in for the radial mask the landing page uses.
///
/// ponytail: 46×46 = 2116 elements, rebuilt each frame at the 60ms pulse tick.
/// That is fine for a boot overlay; halve the grid in `halftone.py` if it ever
/// shows up in a frame budget.
fn hero_dots(theme: &Theme, view: EntityId, cx: &mut App) -> AnyElement {
    const FADE_BAND: f32 = 72.0;
    let color = theme.text;
    let delta = motion::pulse_delta(&ANASTASIA_PULSE, view, cx);
    let rows: Vec<&str> = HERO_DOTS.lines().collect();
    let span = (rows.len() + rows.first().map_or(0, |r| r.len())) as f32;
    let art = div()
        .flex()
        .flex_col()
        .children(rows.iter().enumerate().map(|(y, line)| {
            div()
                .flex()
                .flex_row()
                .children(line.bytes().enumerate().map(move |(x, cell)| {
                    // Ink coverage, 0.0..=1.0.
                    let ink = f32::from(cell.saturating_sub(b'0').min(9)) / 9.0;
                    // The sweep leads at the top-left corner and trails at the
                    // bottom-right, so the light crosses the mark diagonally.
                    let phase = (x + y) as f32 / span * SWEEP_SPREAD;
                    let dot = DOT_PITCH * DOT_MAX * ink;
                    div()
                        .size(px(DOT_PITCH))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(dot))
                                .rounded(px(dot / 2.0))
                                .bg(color)
                                .opacity(ink * motion::gspin_opacity(delta + phase, DOT_DIM)),
                        )
                }))
        }));
    crate::edge_fade::edge_faded(FADE_BAND, true, true, art)
        .fade_left(true)
        .fade_right(true)
        .into_any_element()
}

/// The Anastasia mark's halftone grid, generated by `scripts/halftone.py` from
/// `anastasia-logo-rounded.png`. Regenerate it whenever the logo changes.
const HERO_DOTS: &str = include_str!("../assets/hero-dots.txt");

/// "L O A D I N G" — `text-[11px] uppercase tracking-[0.32em]
/// text-muted-foreground/70`; tracking approximated with thin spaces (gpui has
/// no letter-spacing at the pinned rev).
pub fn loading_word(theme: &Theme) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .text_color(theme.text_muted.opacity(0.7))
        .child(SharedString::from(
            "L\u{2009}O\u{2009}A\u{2009}D\u{2009}I\u{2009}N\u{2009}G",
        ))
}

// Compile-time proof the specs referenced here stay wired to the catalog.
const _: () = {
    assert!(SPLASH_OUT.delay_ms == 150);
    assert!(ANASTASIA_PULSE.duration_ms == 2400);
    assert!(GRADIENT_SPIN.duration_ms == 750);
};
