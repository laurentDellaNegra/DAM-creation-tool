//! Themed badge/tag component.

use egui::{Color32, CornerRadius, Ui};

use crate::frost_night::theme::{Theme, typography};

/// Badge visual variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Filled with primary color (gold).
    Primary,
    /// Outlined with accent color (blue).
    Accent,
    /// Outlined with border color (subtle).
    Outline,
    /// Filled with destructive color (red).
    Destructive,
}

/// A themed inline badge/tag.
pub fn badge(ui: &mut Ui, theme: &Theme, label: &str, variant: BadgeVariant) {
    let (bg, fg, border) = match variant {
        BadgeVariant::Primary => (
            theme.palette.primary,
            theme.palette.primary_foreground,
            egui::Stroke::NONE,
        ),
        BadgeVariant::Accent => (
            Color32::TRANSPARENT,
            theme.palette.ring,
            egui::Stroke::new(1.0, theme.palette.ring),
        ),
        BadgeVariant::Outline => (
            Color32::TRANSPARENT,
            theme.palette.foreground,
            egui::Stroke::new(1.0, theme.palette.border),
        ),
        BadgeVariant::Destructive => (
            theme.palette.destructive,
            theme.palette.destructive_foreground,
            egui::Stroke::NONE,
        ),
    };

    let cr = CornerRadius::same(theme.radius.sm);

    egui::Frame::new()
        .fill(bg)
        .stroke(border)
        .corner_radius(cr)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(typography::font_size(10.0))
                    .color(fg),
            );
        });
}
