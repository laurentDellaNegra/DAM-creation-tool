//! Themed segmented control (toggle buttons).

use egui::{CornerRadius, Response, Sense, Ui, Vec2};

use crate::frost_night::theme::Theme;
use crate::frost_night::theme::mix;

/// A horizontal segmented control. Returns the newly selected index if changed.
///
/// Same outer border, gap, and inner radius as checkbox/toggle.
/// Active segment has a navy-filled inset rect; inactive segments are transparent.
pub fn segmented(ui: &mut Ui, theme: &Theme, labels: &[&str], selected: &mut usize) -> Response {
    let font = egui::FontId::proportional(12.0);
    let pad = Vec2::new(theme.spacing.lg, theme.spacing.xs + 2.0);
    let gap = theme.control_gap;

    // Measure total width
    let galleys: Vec<_> = labels
        .iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap(l.to_string(), font.clone(), theme.palette.foreground)
        })
        .collect();
    let segment_widths: Vec<f32> = galleys.iter().map(|g| g.size().x + pad.x * 2.0).collect();
    let total_width: f32 = segment_widths.iter().sum();
    let height = galleys.iter().map(|g| g.size().y).fold(0.0_f32, f32::max) + pad.y * 2.0;

    let (outer_rect, mut response) =
        ui.allocate_exact_size(Vec2::new(total_width, height), Sense::click());

    if ui.is_rect_visible(outer_rect) {
        let outer_cr = CornerRadius::same(theme.radius.lg);
        let inner_cr = CornerRadius::same(theme.radius.md);

        // Outer border (same as checkbox/toggle)
        ui.painter().rect_stroke(
            outer_rect,
            outer_cr,
            egui::Stroke::new(1.0, theme.palette.control_border),
            egui::StrokeKind::Inside,
        );

        let mut x = outer_rect.left();
        for (i, galley) in galleys.into_iter().enumerate() {
            let seg_rect = egui::Rect::from_min_size(
                egui::pos2(x, outer_rect.top()),
                Vec2::new(segment_widths[i], height),
            );

            let is_active = i == *selected;
            let hovered =
                response.hovered() && seg_rect.contains(response.hover_pos().unwrap_or_default());

            // Handle click on this segment
            if ui.is_enabled() && response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if seg_rect.contains(pos) && !is_active {
                        *selected = i;
                        response.mark_changed();
                    }
                }
            }

            // Active segment: inset filled rect (3px gap, md radius)
            if is_active {
                let inset = seg_rect.shrink(gap);
                ui.painter()
                    .rect_filled(inset, inner_cr, theme.palette.control_fill_on);
            }

            // Text
            let text_color = if is_active {
                theme.palette.foreground
            } else if hovered {
                mix(
                    theme.palette.muted_foreground,
                    theme.palette.foreground,
                    0.3,
                )
            } else {
                theme.palette.muted_foreground
            };

            let text_pos = egui::pos2(
                seg_rect.center().x - galley.size().x / 2.0,
                seg_rect.center().y - galley.size().y / 2.0,
            );
            ui.painter().galley(text_pos, galley, text_color);

            x += segment_widths[i];
        }
    }

    response
}
