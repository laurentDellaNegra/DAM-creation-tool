//! egui integration helpers.

use egui::CornerRadius;
use egui::style::WidgetVisuals;

use super::Theme;
use super::tokens::{StateColors, VariantTokens, filled_tokens, with_alpha};

/// Convert a [`StateColors`] into an egui [`WidgetVisuals`].
fn to_widget_visuals(
    state: &StateColors,
    corner_radius: CornerRadius,
    expansion: f32,
) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: state.bg_fill,
        weak_bg_fill: state.bg_fill,
        bg_stroke: state.border,
        corner_radius,
        fg_stroke: state.fg_stroke,
        expansion,
    }
}

/// Convert [`VariantTokens`] to egui's native [`Widgets`](egui::style::Widgets) struct.
pub(crate) fn to_egui_widgets(
    tokens: &VariantTokens,
    corner_radius: CornerRadius,
    expansion: f32,
) -> egui::style::Widgets {
    egui::style::Widgets {
        noninteractive: to_widget_visuals(&tokens.idle, corner_radius, 0.0),
        inactive: to_widget_visuals(&tokens.idle, corner_radius, 0.0),
        hovered: to_widget_visuals(&tokens.hovered, corner_radius, expansion),
        active: to_widget_visuals(&tokens.active, corner_radius, expansion),
        open: to_widget_visuals(&tokens.active, corner_radius, 0.0),
    }
}

/// Options for installing Frost Night into an egui context.
#[derive(Clone, Copy, Debug)]
pub struct InstallThemeOptions {
    pub install_visuals: bool,
    pub install_fonts: bool,
}

impl Default for InstallThemeOptions {
    fn default() -> Self {
        Self {
            install_visuals: true,
            install_fonts: true,
        }
    }
}

/// Apply Frost Night visuals globally to an egui context.
///
/// Sets dark mode as the base, then overrides colors from the palette.
/// Call once at app startup, or whenever the theme changes. This function does
/// not install fonts.
pub fn apply_visuals(ctx: &egui::Context, theme: &Theme) {
    let p = &theme.palette;
    let mut visuals = egui::Visuals::dark();

    // Base surface colors
    visuals.window_fill = p.card;
    visuals.panel_fill = p.card;
    visuals.extreme_bg_color = p.background;
    visuals.faint_bg_color = p.muted;
    visuals.window_stroke = egui::Stroke::new(1.0, p.border);

    // Selection
    visuals.selection.bg_fill = with_alpha(p.primary, 80);
    visuals.selection.stroke = egui::Stroke::new(1.0, p.primary);

    // Hyperlinks
    visuals.hyperlink_color = p.accent;

    // Widget visuals — neutral filled defaults (no special border)
    let default_tokens = filled_tokens(p.secondary, p.secondary_foreground, p);
    let corner_radius = CornerRadius::same(theme.radius.md);
    visuals.widgets = to_egui_widgets(&default_tokens, corner_radius, 1.0);

    ctx.set_visuals(visuals);
}

/// Install Frost Night visuals and, when the `icons` feature is enabled, the
/// bundled Lucide icon font according to `options`.
pub fn install_theme(ctx: &egui::Context, theme: &Theme, options: InstallThemeOptions) {
    if options.install_visuals {
        apply_visuals(ctx, theme);
    }

    if options.install_fonts {
        #[cfg(feature = "icons")]
        crate::frost_night::icons::install_icon_font(ctx);
    }
}

/// Apply the full Frost Night theme.
#[deprecated(note = "Use install_theme or apply_visuals + install_icon_font instead")]
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    install_theme(ctx, theme, InstallThemeOptions::default());
}
