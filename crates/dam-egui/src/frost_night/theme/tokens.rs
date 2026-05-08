//! Interaction state tokens and color mixing utilities.

use egui::{Color32, Stroke};

use super::palette::ColorPalette;

/// Atomic visual state for one interaction state of a widget.
#[derive(Clone, Copy, Debug)]
pub struct StateColors {
    /// Background fill color.
    pub bg_fill: Color32,
    /// Foreground (text/icon) stroke.
    pub fg_stroke: Stroke,
    /// Border stroke.
    pub border: Stroke,
}

/// All interaction states for a control variant.
#[derive(Clone, Copy, Debug)]
pub struct VariantTokens {
    pub idle: StateColors,
    pub hovered: StateColors,
    pub active: StateColors,
    pub disabled: StateColors,
}

/// Control style variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlVariant {
    Primary,
    Secondary,
    Ghost,
    Outline,
    Destructive,
    Link,
}

/// Linear interpolation between two colors in premultiplied sRGB space.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let [ar, ag, ab, aa] = a.to_array();
    let [br, bg, bb, ba] = b.to_array();
    Color32::from_rgba_premultiplied(
        (ar as f32 * inv + br as f32 * t + 0.5) as u8,
        (ag as f32 * inv + bg as f32 * t + 0.5) as u8,
        (ab as f32 * inv + bb as f32 * t + 0.5) as u8,
        (aa as f32 * inv + ba as f32 * t + 0.5) as u8,
    )
}

/// Return a color with modified alpha (input color should be opaque).
pub fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = c.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

const WHITE: Color32 = Color32::WHITE;

/// Derive all interaction state colors for a given control variant.
pub fn variant_tokens(palette: &ColorPalette, variant: ControlVariant) -> VariantTokens {
    match variant {
        ControlVariant::Primary => {
            filled_tokens(palette.primary, palette.primary_foreground, palette)
        }
        ControlVariant::Secondary => bordered_tokens(
            palette.secondary,
            palette.secondary_foreground,
            Stroke::new(1.0, palette.secondary_border),
            palette,
        ),
        ControlVariant::Ghost => ghost_tokens(palette.foreground, Stroke::NONE, palette),
        ControlVariant::Outline => ghost_tokens(
            palette.foreground,
            Stroke::new(1.0, palette.border),
            palette,
        ),
        ControlVariant::Destructive => {
            filled_tokens(palette.destructive, palette.destructive_foreground, palette)
        }
        ControlVariant::Link => VariantTokens {
            idle: StateColors {
                bg_fill: Color32::TRANSPARENT,
                fg_stroke: Stroke::new(1.0, palette.ring),
                border: Stroke::NONE,
            },
            hovered: StateColors {
                bg_fill: Color32::TRANSPARENT,
                fg_stroke: Stroke::new(1.0, mix(palette.ring, WHITE, 0.15)),
                border: Stroke::NONE,
            },
            active: StateColors {
                bg_fill: Color32::TRANSPARENT,
                fg_stroke: Stroke::new(1.0, mix(palette.ring, WHITE, 0.25)),
                border: Stroke::NONE,
            },
            disabled: StateColors {
                bg_fill: Color32::TRANSPARENT,
                fg_stroke: Stroke::new(1.0, with_alpha(palette.muted_foreground, 128)),
                border: Stroke::NONE,
            },
        },
    }
}

/// Tokens for a filled (opaque background) control.
pub(crate) fn filled_tokens(bg: Color32, fg: Color32, palette: &ColorPalette) -> VariantTokens {
    VariantTokens {
        idle: StateColors {
            bg_fill: bg,
            fg_stroke: Stroke::new(1.0, fg),
            border: Stroke::NONE,
        },
        hovered: StateColors {
            bg_fill: mix(bg, WHITE, 0.06),
            fg_stroke: Stroke::new(1.0, fg),
            border: Stroke::NONE,
        },
        active: StateColors {
            bg_fill: mix(bg, WHITE, 0.10),
            fg_stroke: Stroke::new(1.0, fg),
            border: Stroke::NONE,
        },
        disabled: StateColors {
            bg_fill: mix(palette.muted, palette.background, 0.6),
            fg_stroke: Stroke::new(1.0, with_alpha(fg, 128)),
            border: Stroke::NONE,
        },
    }
}

/// Tokens for a bordered control (opaque bg with visible border).
fn bordered_tokens(
    bg: Color32,
    fg: Color32,
    border: Stroke,
    palette: &ColorPalette,
) -> VariantTokens {
    VariantTokens {
        idle: StateColors {
            bg_fill: bg,
            fg_stroke: Stroke::new(1.0, fg),
            border,
        },
        hovered: StateColors {
            bg_fill: mix(bg, WHITE, 0.04),
            fg_stroke: Stroke::new(1.0, mix(fg, WHITE, 0.1)),
            border: Stroke::new(border.width, mix(border.color, WHITE, 0.15)),
        },
        active: StateColors {
            bg_fill: mix(bg, WHITE, 0.08),
            fg_stroke: Stroke::new(1.0, mix(fg, WHITE, 0.15)),
            border: Stroke::new(border.width, mix(border.color, WHITE, 0.25)),
        },
        disabled: StateColors {
            bg_fill: mix(palette.muted, palette.background, 0.6),
            fg_stroke: Stroke::new(1.0, with_alpha(fg, 128)),
            border: Stroke::new(border.width, with_alpha(border.color, 64)),
        },
    }
}

/// Tokens for a ghost/outline (transparent background) control.
fn ghost_tokens(fg: Color32, border: Stroke, palette: &ColorPalette) -> VariantTokens {
    VariantTokens {
        idle: StateColors {
            bg_fill: Color32::TRANSPARENT,
            fg_stroke: Stroke::new(1.0, fg),
            border,
        },
        hovered: StateColors {
            bg_fill: with_alpha(palette.secondary, 128),
            fg_stroke: Stroke::new(1.0, fg),
            border,
        },
        active: StateColors {
            bg_fill: with_alpha(palette.secondary, 180),
            fg_stroke: Stroke::new(1.0, fg),
            border,
        },
        disabled: StateColors {
            bg_fill: Color32::TRANSPARENT,
            fg_stroke: Stroke::new(1.0, with_alpha(fg, 128)),
            border: Stroke::new(border.width, with_alpha(border.color, 64)),
        },
    }
}
