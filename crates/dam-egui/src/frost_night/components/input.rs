//! Themed text input component.

use egui::{Response, TextBuffer, TextEdit, Ui};

use crate::frost_night::theme::{ControlSize, Theme};

/// A themed single-line text input.
pub fn text_input<S: TextBuffer>(
    ui: &mut Ui,
    theme: &Theme,
    text: &mut S,
    size: ControlSize,
) -> Response {
    let vis = theme.input(size);

    ui.scope(|ui| {
        let style = ui.style_mut();
        style.visuals.extreme_bg_color = vis.bg;
        style.visuals.widgets.inactive.bg_stroke = vis.border;
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme.palette.ring);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme.palette.ring);

        ui.add(
            TextEdit::singleline(text)
                .font(vis.font)
                .text_color(vis.text_color)
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(theme.spacing.sm as i8, 6)),
        )
    })
    .inner
}
