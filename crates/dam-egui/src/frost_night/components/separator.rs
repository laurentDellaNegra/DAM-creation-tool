//! Themed separator/divider.

use egui::{Response, Ui};

use crate::frost_night::theme::Theme;

/// A themed horizontal separator using the palette border color.
pub fn separator(ui: &mut Ui, theme: &Theme) -> Response {
    ui.scope(|ui| {
        ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, theme.palette.border);
        ui.separator()
    })
    .inner
}
