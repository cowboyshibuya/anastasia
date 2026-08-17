use gpui::{App, Global, Hsla, Window, WindowAppearance, hsla, rgb, transparent_black};

pub use waku_client::theme::ThemePreference;

fn resolves_to_dark(preference: ThemePreference, system_appearance: WindowAppearance) -> bool {
    match preference {
        ThemePreference::System => matches!(
            system_appearance,
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        ),
        ThemePreference::Light => false,
        ThemePreference::Dark => true,
    }
}

fn native_override(preference: ThemePreference) -> Option<bool> {
    match preference {
        ThemePreference::System => None,
        ThemePreference::Light => Some(false),
        ThemePreference::Dark => Some(true),
    }
}

/// Anastasia's visual language, take two: neutral graphite surfaces in the spirit
/// of Cursor — color is reserved for meaning. On macOS the sidebar's semantic
/// tint is installed as a native layer above Sidebar vibrancy; keeping this
/// GPUI surface clear avoids incorrectly accumulating the alpha of nested Metal
/// backgrounds. Selected, hovered, and pressed rows remain a 6% neutral layer.
#[derive(Clone, Copy)]
pub struct Theme {
    pub is_dark: bool,
    pub canvas: Hsla,
    pub sidebar: Hsla,
    pub sidebar_drag_background: Hsla,
    pub sidebar_item_background: Hsla,
    pub surface: Hsla,
    pub raised: Hsla,
    pub composer: Hsla,
    pub inset: Hsla,
    /// Terminal screen surface: paper-white in light mode, near-black in dark.
    pub terminal: Hsla,
    pub overlay: Hsla,
    pub overlay_strong: Hsla,

    pub border: Hsla,
    pub border_strong: Hsla,
    pub sidebar_border: Hsla,

    pub text: Hsla,
    pub text_secondary: Hsla,
    pub text_tertiary: Hsla,
    pub text_ghost: Hsla,

    /// Brand coral. Logo, caret, live-activity pulses — nothing structural.
    pub accent: Hsla,
    pub resize_handle: Hsla,
    /// Meter fills in the usage panel. Quota-meter blue by convention;
    /// warning/danger take over as a lane fills.
    pub gauge: Hsla,

    /// Text-selection wash. Painted *under* the glyphs, so it stays
    /// translucent and deliberately reads as the familiar browser blue rather
    /// than as brand color.
    pub selection: Hsla,
    /// Inline `code` foreground and its rounded wash.
    pub code_text: Hsla,
    pub code_wash: Hsla,

    /// Light fill for primary buttons (send, allow), dark glyph on top.
    pub inverse: Hsla,
    pub on_inverse: Hsla,

    pub warning: Hsla,
    pub success: Hsla,
    pub favorite: Hsla,
    pub danger: Hsla,
    pub danger_soft: Hsla,
}

impl Theme {
    pub fn current(cx: &App) -> Self {
        if cx.has_global::<ActiveWakuTheme>() {
            cx.global::<ActiveWakuTheme>().0
        } else {
            Self::dark()
        }
    }

    /// Dark is the default appearance, and the one the identity is drawn for:
    /// a near-black plane the way the halftone marks are set on black, with the
    /// chrome sitting a few points above it rather than a charcoal grey the
    /// content has to fight.
    ///
    /// Elevation reads bottom to top as inset < sidebar < surface < composer <
    /// raised, in luminance order — depth comes from that ladder, not from
    /// borders, which stay hairline.
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            canvas: rgb(0x0E0E11).into(),
            sidebar: if cfg!(target_os = "macos") {
                transparent_black()
            } else {
                rgb(0x0B0B0E).into()
            },
            sidebar_drag_background: rgb(0x0B0B0E).into(),
            sidebar_item_background: hsla(0.0, 0.0, 0.941, 0.06),
            surface: rgb(0x0E0E11).into(),
            raised: rgb(0x18181D).into(),
            composer: rgb(0x151519).into(),
            inset: rgb(0x090A0C).into(),
            terminal: rgb(0x090A0C).into(),
            overlay: hsla(240.0 / 360.0, 0.10, 0.90, 0.05),
            overlay_strong: hsla(240.0 / 360.0, 0.10, 0.90, 0.09),

            border: hsla(240.0 / 360.0, 0.10, 0.90, 0.08),
            border_strong: hsla(240.0 / 360.0, 0.10, 0.90, 0.16),
            sidebar_border: hsla(240.0 / 360.0, 0.10, 0.90, 0.08),

            text: rgb(0xEDEDF0).into(),
            text_secondary: rgb(0xA8A8B2).into(),
            text_tertiary: rgb(0x82828C).into(),
            // Was #575757 — 2.4:1 on the old plane, under the 3:1 a disabled
            // control still owes the reader.
            text_ghost: rgb(0x63636D).into(),

            // One accent, one meaning. The gauge and the resize handle used to
            // paint themselves a different blue from the accent, so two colors
            // competed for "this is the active thing".
            accent: rgb(0x6E8BFF).into(),
            resize_handle: rgb(0x6E8BFF).into(),
            gauge: rgb(0x6E8BFF).into(),

            selection: hsla(228.0 / 360.0, 1.0, 0.62, 0.38),
            code_text: rgb(0xC9C4E8).into(),
            code_wash: hsla(240.0 / 360.0, 0.10, 0.90, 0.08),

            inverse: rgb(0xEDEDF0).into(),
            on_inverse: rgb(0x0E0E11).into(),

            warning: rgb(0xE0B36A).into(),
            success: rgb(0x62C987).into(),
            favorite: rgb(0xEAB308).into(),
            danger: rgb(0xE2726A).into(),
            danger_soft: hsla(4.0 / 360.0, 0.55, 0.63, 0.10),
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            canvas: rgb(0xF6F5F6).into(),
            sidebar: if cfg!(target_os = "macos") {
                transparent_black()
            } else {
                rgb(0xF3F3F3).into()
            },
            sidebar_drag_background: rgb(0xF3F3F3).into(),
            sidebar_item_background: hsla(0.0, 0.0, 0.078, 0.06),
            surface: rgb(0xF6F5F6).into(),
            raised: rgb(0xECECEC).into(),
            composer: rgb(0xFFFFFF).into(),
            inset: rgb(0xE6E6E6).into(),
            terminal: rgb(0xFFFFFF).into(),
            overlay: hsla(220.0 / 360.0, 0.10, 0.12, 0.05),
            overlay_strong: hsla(220.0 / 360.0, 0.10, 0.12, 0.09),

            border: hsla(220.0 / 360.0, 0.10, 0.12, 0.08),
            border_strong: hsla(220.0 / 360.0, 0.10, 0.12, 0.15),
            sidebar_border: hsla(0.0, 0.0, 0.078, 0.12),

            text: rgb(0x242424).into(),
            text_secondary: rgb(0x666666).into(),
            // Both were under threshold on the #F6F5F6 canvas (3.4:1 and
            // 2.3:1); nudged down to clear body text and the 3:1 a disabled
            // control still owes the reader.
            text_tertiary: rgb(0x707070).into(),
            text_ghost: rgb(0x8E8E8E).into(),

            accent: rgb(0x3B5BDB).into(),
            resize_handle: rgb(0x3B5BDB).into(),
            gauge: rgb(0x3B5BDB).into(),

            selection: hsla(211.0 / 360.0, 1.0, 0.50, 0.35),
            code_text: rgb(0x9A5528).into(),
            code_wash: hsla(220.0 / 360.0, 0.10, 0.12, 0.07),

            inverse: rgb(0x202227).into(),
            on_inverse: rgb(0xF8F8F9).into(),

            warning: rgb(0xA66B20).into(),
            success: rgb(0x2F8F52).into(),
            favorite: rgb(0xCA8A04).into(),
            danger: rgb(0xC14840).into(),
            danger_soft: hsla(4.0 / 360.0, 0.55, 0.52, 0.10),
        }
    }
}

#[derive(Clone, Copy)]
struct ActiveWakuTheme(Theme);

impl Global for ActiveWakuTheme {}

/// Publish the resolved palette. [`Theme::current`] reads it back from the
/// global, which is how every view gets its colors.
fn set_active_theme(theme: Theme, cx: &mut App) {
    cx.set_global(ActiveWakuTheme(theme));
}

/// Resolve and publish the startup palette, before any window exists.
pub fn init(cx: &mut App) {
    let system_appearance = cx.window_appearance();
    let theme = if resolves_to_dark(ThemePreference::System, system_appearance) {
        Theme::dark()
    } else {
        Theme::light()
    };
    set_active_theme(theme, cx);
}

pub fn apply_theme_preference(preference: ThemePreference, window: &mut Window, cx: &mut App) {
    crate::platform::set_window_appearance(window, native_override(preference));
    let is_dark = resolves_to_dark(preference, cx.window_appearance());
    set_active_theme(
        if is_dark {
            Theme::dark()
        } else {
            Theme::light()
        },
        cx,
    );
    crate::platform::configure_sidebar_material(window, is_dark);
    window.refresh();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG 2 relative luminance of an opaque theme color.
    fn luminance(color: Hsla) -> f32 {
        let rgba = gpui::Rgba::from(color);
        let channel = |c: f32| {
            if c <= 0.040_45 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(rgba.r) + 0.7152 * channel(rgba.g) + 0.0722 * channel(rgba.b)
    }

    fn contrast(foreground: Hsla, background: Hsla) -> f32 {
        let (a, b) = (luminance(foreground), luminance(background));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// Text and controls have to stay legible on the surface they sit on.
    /// Body-weight text owes 4.5:1; a disabled or decorative control still owes
    /// 3:1, which is where `text_ghost` used to fail in both appearances.
    #[test]
    fn every_foreground_clears_its_contrast_floor() {
        for theme in [Theme::dark(), Theme::light()] {
            let name = if theme.is_dark { "dark" } else { "light" };
            let body = [
                ("text", theme.text),
                ("text_secondary", theme.text_secondary),
                ("text_tertiary", theme.text_tertiary),
                ("danger", theme.danger),
            ];
            for (label, color) in body {
                let ratio = contrast(color, theme.surface);
                assert!(ratio >= 4.5, "{name}.{label} is {ratio:.2}:1, under 4.5:1");
            }
            let non_body = [
                ("text_ghost", theme.text_ghost),
                ("accent", theme.accent),
                ("warning", theme.warning),
                ("success", theme.success),
            ];
            for (label, color) in non_body {
                let ratio = contrast(color, theme.surface);
                assert!(ratio >= 3.0, "{name}.{label} is {ratio:.2}:1, under 3.0:1");
            }
        }
    }

    /// Depth is carried by the elevation ladder rather than by borders, so the
    /// steps have to stay ordered — a raised surface that is not actually
    /// lighter than the plane it sits on reads as flat.
    #[test]
    fn the_elevation_ladder_stays_ordered() {
        let dark = Theme::dark();
        assert!(luminance(dark.inset) < luminance(dark.surface));
        assert!(luminance(dark.surface) < luminance(dark.composer));
        assert!(luminance(dark.composer) < luminance(dark.raised));

        let light = Theme::light();
        assert!(luminance(light.inset) < luminance(light.canvas));
        assert!(luminance(light.canvas) < luminance(light.composer));
    }

    /// One color, one meaning: the gauge and the resize handle used to paint a
    /// different blue from the accent, so two colors competed for "active".
    #[test]
    fn the_accent_is_the_only_accent() {
        for theme in [Theme::dark(), Theme::light()] {
            assert_eq!(theme.gauge, theme.accent);
            assert_eq!(theme.resize_handle, theme.accent);
        }
    }
}
