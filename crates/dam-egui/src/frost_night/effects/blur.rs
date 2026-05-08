//! Glassmorphism / frosted glass blur support.
//!
//! Currently implements a **fallback** that draws a semi-transparent tint
//! without actual backdrop blur. The API is designed so that a future
//! wgpu-based Gaussian blur implementation can be swapped in without
//! changing component code.

use egui::{Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind, Ui};

/// Configuration for a blurred background region.
#[derive(Clone, Debug)]
pub struct BlurRect {
    /// Screen-space rectangle to blur.
    pub rect: Rect,
    /// Blur radius in pixels (used by future wgpu implementation).
    pub radius: f32,
    /// Semi-transparent overlay tint color.
    pub tint: Color32,
    /// Corner radius for the blurred region.
    pub corner_radius: CornerRadius,
}

/// Paint a blurred rect as an egui [`Shape`].
///
/// **Current implementation**: draws the tint color as a semi-transparent
/// filled rectangle. When a wgpu blur backend is added, this will use
/// a Gaussian blur shader instead.
pub fn blur_shape(blur: &BlurRect) -> Shape {
    Shape::rect_filled(blur.rect, blur.corner_radius, blur.tint)
}

/// Paint a blur backdrop with a subtle border (glassmorphism style).
pub fn blur_shape_with_border(blur: &BlurRect, border_color: Color32) -> Vec<Shape> {
    vec![
        Shape::rect_filled(blur.rect, blur.corner_radius, blur.tint),
        Shape::rect_stroke(
            blur.rect,
            blur.corner_radius,
            Stroke::new(1.0, border_color),
            StrokeKind::Outside,
        ),
    ]
}

/// Convenience: paint a blur backdrop into a [`Ui`].
pub fn paint_blur(ui: &Ui, blur: &BlurRect) {
    ui.painter().add(blur_shape(blur));
}
