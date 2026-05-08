//! Themed button component.

use egui::{Response, Ui, WidgetText};

use crate::frost_night::theme::{
    ControlSize, ControlVariant, Theme, to_egui_widgets, variant_tokens,
};

/// A themed button with variant styling and size control.
pub fn button(
    ui: &mut Ui,
    theme: &Theme,
    label: impl Into<WidgetText>,
    variant: ControlVariant,
    size: ControlSize,
) -> Response {
    let tokens = variant_tokens(&theme.palette, variant);
    let corner_radius = size.corner_radius(&theme.radius);
    let padding = size.padding();

    ui.scope(|ui| {
        ui.style_mut().spacing.button_padding = padding;
        ui.style_mut().visuals.widgets = to_egui_widgets(&tokens, corner_radius, 1.0);
        ui.add(egui::Button::new(label))
    })
    .inner
}
