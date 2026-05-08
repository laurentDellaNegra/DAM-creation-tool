//! Vendored Frost Night egui theme.
//!
//! Source: ../frost-night-egui/ui-theme
//!
//! This is intentionally copy-pasted for now. Do not turn this into a
//! path/remote dependency yet. We may replace this vendored module with a
//! proper dependency later.

#![allow(clippy::all, dead_code, unused_imports)]

pub mod components;
pub mod composites;
pub mod containers;
pub mod effects;
#[cfg(feature = "icons")]
pub mod icons;
pub mod theme;

pub use components::FrostUiExt;
pub use effects::BlurRect;
#[cfg(feature = "icons")]
pub use icons::{add_icon_font_to, install_icon_font};
#[allow(deprecated)]
pub use theme::apply_theme;
pub use theme::{
    ColorPalette, ControlSize, ControlVariant, InstallThemeOptions, RadiusScale, SpacingScale,
    StateColors, Theme, VariantTokens, apply_visuals, install_theme,
};
