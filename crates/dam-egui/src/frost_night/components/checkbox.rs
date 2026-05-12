//! Themed checkbox component.

use egui::{Color32, CornerRadius, Response, Sense, Ui, Vec2, WidgetText};

use crate::frost_night::theme::mix;
use crate::frost_night::theme::{Theme, typography};

/// A themed checkbox with rounded square and accent checkmark.
///
/// Checked: dark navy fill with accent-colored checkmark.
/// Unchecked: border-only rounded square.
pub fn checkbox(
    ui: &mut Ui,
    theme: &Theme,
    checked: &mut bool,
    label: impl Into<WidgetText>,
) -> Response {
    checkbox_with_font(ui, theme, checked, label, 13.0)
}

/// Checkbox with a smaller label font.
pub fn checkbox_small(
    ui: &mut Ui,
    theme: &Theme,
    checked: &mut bool,
    label: impl Into<WidgetText>,
) -> Response {
    checkbox_with_font(ui, theme, checked, label, 11.0)
}

fn checkbox_with_font(
    ui: &mut Ui,
    theme: &Theme,
    checked: &mut bool,
    label: impl Into<WidgetText>,
    font_size: f32,
) -> Response {
    let box_size = 22.0;
    let spacing = theme.spacing.xs + 2.0;
    let font = typography::proportional(font_size);

    let text = label
        .into()
        .text_style(egui::TextStyle::Body)
        .color(theme.palette.foreground);
    let galley = text.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, font);

    let total_width = box_size + spacing + galley.size().x;
    let total_height = box_size.max(galley.size().y);
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::click());

    if ui.is_enabled() && response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_with_time(response.id, *checked, 0.12);

        // Outer border (same as switch track)
        let outer_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.center().y - box_size / 2.0),
            Vec2::splat(box_size),
        );
        let outer_cr = CornerRadius::same(theme.radius.lg);
        let border_color = mix(
            theme.palette.control_border,
            theme.palette.muted_foreground,
            how_on * 0.3,
        );
        ui.painter().rect_stroke(
            outer_rect,
            outer_cr,
            egui::Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        // Inner fill (gap, same as switch thumb) — only when checked
        let inner_rect = outer_rect.shrink(theme.control_gap);
        let inner_cr = CornerRadius::same(theme.radius.md);
        if how_on > 0.01 {
            let bg = mix(
                theme.palette.control_fill_off,
                theme.palette.control_fill_on,
                how_on,
            );
            let alpha = (how_on * 255.0) as u8;
            let bg = Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), alpha);
            ui.painter().rect_filled(inner_rect, inner_cr, bg);
        }

        // Checkmark (fade in)
        if how_on > 0.01 {
            let alpha = (how_on * 255.0) as u8;
            let check_color = Color32::from_rgba_unmultiplied(
                theme.palette.foreground.r(),
                theme.palette.foreground.g(),
                theme.palette.foreground.b(),
                alpha,
            );
            let center = inner_rect.center();
            let s = inner_rect.width() * 0.22;
            let p1 = center + egui::vec2(-s * 1.2, 0.0);
            let p2 = center + egui::vec2(-s * 0.2, s * 0.9);
            let p3 = center + egui::vec2(s * 1.2, -s * 0.8);
            let stroke = egui::Stroke::new(1.8, check_color);
            ui.painter().line_segment([p1, p2], stroke);
            ui.painter().line_segment([p2, p3], stroke);
        }

        // Label
        let text_pos = egui::pos2(
            outer_rect.max.x + spacing,
            rect.center().y - galley.size().y / 2.0,
        );
        ui.painter()
            .galley(text_pos, galley, theme.palette.foreground);
    }

    response
}
