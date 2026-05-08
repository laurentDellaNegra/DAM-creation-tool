//! Frost Night theme tokens and egui integration.

use egui::CornerRadius;

pub mod oklch;
pub mod palette;
pub mod scale;
pub mod style;
pub mod tokens;
pub mod typography;

pub use palette::ColorPalette;
pub use scale::{ControlSize, RadiusScale, SpacingScale};
#[allow(deprecated)]
pub use style::apply_theme;
pub(crate) use style::to_egui_widgets;
pub use style::{InstallThemeOptions, apply_visuals, install_theme};
pub(crate) use tokens::variant_tokens;
pub use tokens::{ControlVariant, StateColors, VariantTokens, mix};

/// Resolved visual properties for rendering a control.
#[derive(Clone, Debug)]
pub struct ControlVisuals {
    pub tokens: VariantTokens,
    pub corner_radius: CornerRadius,
    pub font: egui::FontId,
    pub padding: egui::Vec2,
}

/// Resolved visual properties for rendering a text input.
#[derive(Clone, Debug)]
pub struct InputVisuals {
    pub bg: egui::Color32,
    pub border: egui::Stroke,
    pub corner_radius: CornerRadius,
    pub font: egui::FontId,
    pub text_color: egui::Color32,
    pub placeholder_color: egui::Color32,
}

/// The main theme, combining palette and scales.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Theme {
    pub palette: ColorPalette,
    pub radius: RadiusScale,
    pub spacing: SpacingScale,
    /// Gap between outer border and inner fill in controls (checkbox, toggle, segmented).
    pub control_gap: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme.
    pub fn dark() -> Self {
        Self {
            palette: ColorPalette::dark(),
            radius: RadiusScale::default(),
            spacing: SpacingScale::default(),
            control_gap: 3.0,
        }
    }

    /// Resolve widget visuals for a variant + size combination.
    pub fn control(&self, variant: ControlVariant, size: ControlSize) -> ControlVisuals {
        ControlVisuals {
            tokens: variant_tokens(&self.palette, variant),
            corner_radius: size.corner_radius(&self.radius),
            font: size.font(),
            padding: size.padding(),
        }
    }

    /// Resolve input visuals for a given size.
    pub fn input(&self, size: ControlSize) -> InputVisuals {
        InputVisuals {
            bg: self.palette.muted,
            border: egui::Stroke::new(1.0, self.palette.input),
            corner_radius: size.corner_radius(&self.radius),
            font: size.font(),
            text_color: self.palette.foreground,
            placeholder_color: self.palette.muted_foreground,
        }
    }

    /// Create a [`BlurRect`](crate::frost_night::effects::BlurRect) for a glassmorphism surface.
    pub fn surface_blur(
        &self,
        rect: egui::Rect,
        corner_radius: CornerRadius,
    ) -> crate::frost_night::effects::BlurRect {
        crate::frost_night::effects::BlurRect {
            rect,
            radius: self.palette.surface_blur_radius,
            tint: self.palette.surface_blur,
            corner_radius,
        }
    }
}
