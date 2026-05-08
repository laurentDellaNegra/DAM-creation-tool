//! Thin component wrappers that use the theme for styling.

mod badge;
mod button;
mod checkbox;
mod input;
mod segmented;
mod separator;
mod toggle;

pub use badge::{BadgeVariant, badge};
pub use button::button;
pub use checkbox::{checkbox, checkbox_small};
pub use input::text_input;
pub use segmented::segmented;
pub use separator::separator;
pub use toggle::toggle;

use crate::frost_night::theme::{ControlSize, ControlVariant, Theme};

/// Ergonomic extension methods for adding Frost Night controls to an `egui::Ui`.
pub trait FrostUiExt {
    fn frost_button(
        &mut self,
        theme: &Theme,
        label: impl Into<egui::WidgetText>,
        variant: ControlVariant,
        size: ControlSize,
    ) -> egui::Response;

    fn frost_text_input<S: egui::TextBuffer>(
        &mut self,
        theme: &Theme,
        text: &mut S,
        size: ControlSize,
    ) -> egui::Response;
}

impl FrostUiExt for egui::Ui {
    fn frost_button(
        &mut self,
        theme: &Theme,
        label: impl Into<egui::WidgetText>,
        variant: ControlVariant,
        size: ControlSize,
    ) -> egui::Response {
        button(self, theme, label, variant, size)
    }

    fn frost_text_input<S: egui::TextBuffer>(
        &mut self,
        theme: &Theme,
        text: &mut S,
        size: ControlSize,
    ) -> egui::Response {
        text_input(self, theme, text, size)
    }
}
