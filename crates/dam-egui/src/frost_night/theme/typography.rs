//! Typography scale helpers.

use egui::{FontId, Style, TextStyle};

const APP_FONT_SIZE_BUMP: f32 = 1.0;

pub fn font_size(size: f32) -> f32 {
    size + APP_FONT_SIZE_BUMP
}

pub fn proportional(size: f32) -> FontId {
    FontId::proportional(font_size(size))
}

pub fn monospace(size: f32) -> FontId {
    FontId::monospace(font_size(size))
}

pub fn apply_app_text_styles(style: &mut Style) {
    style
        .text_styles
        .insert(TextStyle::Small, proportional(9.0));
    style
        .text_styles
        .insert(TextStyle::Body, proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Button, proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Heading, proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, monospace(13.0));
}
