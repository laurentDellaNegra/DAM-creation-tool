//! Semantic color palette for the Frost Night dark theme.

use egui::Color32;

/// Semantic color palette using background/foreground pairs.
///
/// Colors are sRGB hex values extracted from the Figma mockups.
/// The palette is theme-agnostic in structure — field names carry no
/// "dark" or "light" semantics, so a future light palette is just a
/// second constructor with different values.
///
/// All fields are public — construct directly or modify [`Self::dark()`]
/// for custom palettes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorPalette {
    /// Main application background.
    pub background: Color32,
    /// Primary text color.
    pub foreground: Color32,
    /// Primary accent (frost/silver).
    pub primary: Color32,
    /// Text on primary-colored surfaces.
    pub primary_foreground: Color32,
    /// Secondary fills (inactive tabs, subtle buttons).
    pub secondary: Color32,
    /// Text on secondary surfaces.
    pub secondary_foreground: Color32,
    /// Border for secondary controls (purple/violet tint).
    pub secondary_border: Color32,
    /// Muted surfaces (input backgrounds, subtle areas).
    pub muted: Color32,
    /// Muted text (placeholders, disabled).
    pub muted_foreground: Color32,
    /// Accent highlights (blue tags, info).
    pub accent: Color32,
    /// Text on accent surfaces.
    pub accent_foreground: Color32,
    /// Error/destructive state.
    pub destructive: Color32,
    /// Text on destructive surfaces.
    pub destructive_foreground: Color32,
    /// Default border color.
    pub border: Color32,
    /// Input field border color.
    pub input: Color32,
    /// Focus ring color.
    pub ring: Color32,
    /// Card/panel background.
    pub card: Color32,
    /// Text on cards.
    pub card_foreground: Color32,
    /// Popover/dialog background.
    pub popover: Color32,
    /// Text on popovers.
    pub popover_foreground: Color32,
    /// Glassmorphism tint (semi-transparent).
    pub surface_blur: Color32,
    /// Default blur radius in pixels.
    pub surface_blur_radius: f32,
    /// Outer border for interactive controls (checkbox, toggle, segmented).
    pub control_border: Color32,
    /// Inner fill color when control is OFF.
    pub control_fill_off: Color32,
    /// Inner fill color when control is ON/active.
    pub control_fill_on: Color32,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorPalette {
    /// Dark theme — extracted from Figma mockups.
    ///
    /// Background confirmed from Figma metadata (`r:0.016, g:0.027, b:0.055`).
    /// Other values approximated from screenshots.
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x04, 0x07, 0x0E),
            foreground: Color32::from_rgb(0xE0, 0xE8, 0xF0),
            primary: Color32::from_rgb(0xCC, 0xD4, 0xE2),
            primary_foreground: Color32::from_rgb(0x0E, 0x17, 0x24),
            secondary: Color32::from_rgb(0x14, 0x1E, 0x2E),
            secondary_foreground: Color32::from_rgb(0x8A, 0x96, 0xA8),
            secondary_border: Color32::from_rgb(0x3D, 0x2D, 0x55),
            muted: Color32::from_rgb(0x11, 0x1D, 0x2E),
            muted_foreground: Color32::from_rgb(0x5A, 0x6A, 0x7E),
            accent: Color32::from_rgb(0x0F, 0x1E, 0x3D),
            accent_foreground: Color32::from_rgb(0xE0, 0xE8, 0xF0),
            destructive: Color32::from_rgb(0xEF, 0x44, 0x44),
            destructive_foreground: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            border: Color32::from_rgb(0x1A, 0x2B, 0x40),
            input: Color32::from_rgb(0x1A, 0x2B, 0x40),
            ring: Color32::from_rgb(0x4A, 0x90, 0xCF),
            card: Color32::from_rgb(0x0C, 0x16, 0x22),
            card_foreground: Color32::from_rgb(0xE0, 0xE8, 0xF0),
            popover: Color32::from_rgb(0x0C, 0x16, 0x22),
            popover_foreground: Color32::from_rgb(0xE0, 0xE8, 0xF0),
            surface_blur: Color32::from_rgba_unmultiplied(0x06, 0x0C, 0x16, 0xD8),
            surface_blur_radius: 16.0,
            control_border: Color32::from_rgb(0x3C, 0x46, 0x56),
            control_fill_off: Color32::from_rgb(0x0E, 0x1A, 0x38),
            control_fill_on: Color32::from_rgb(0x16, 0x2C, 0x59),
        }
    }
}
