//! Basic themed card container.
//!
//! A simplified card for documentation purposes. The demo uses
//! `sidebar_card` which adds drag, glow, and toolbar integration.

use egui::{Response, Ui};

use crate::frost_night::theme::Theme;

/// A themed card with border and rounded corners.
pub fn card(ui: &mut Ui, theme: &Theme, add_contents: impl FnOnce(&mut Ui)) -> Response {
    let cr = egui::CornerRadius::same(theme.radius.lg);

    egui::Frame::new()
        .fill(theme.palette.card)
        .stroke(egui::Stroke::new(1.0, theme.palette.border))
        .corner_radius(cr)
        .inner_margin(egui::Margin::same(theme.spacing.md as i8))
        .show(ui, add_contents)
        .response
}
