//! Compact Frost Night top toolbar for DAM actions.

use std::hash::Hash;

use egui::{CornerRadius, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::frost_night::icons::icon_font;
use crate::frost_night::theme::Theme;

#[derive(Clone, Copy, Debug)]
pub struct ToolbarAction<'a> {
    pub icon: char,
    pub label: &'a str,
    pub tooltip: &'a str,
    pub selected: bool,
    pub disabled: bool,
}

pub struct TopToolbarResponse {
    pub icon_clicked: Option<usize>,
}

fn separator(ui: &mut Ui, theme: &Theme, height: f32, margin_v: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().line_segment(
        [
            rect.center_top() + egui::vec2(0.0, margin_v),
            rect.center_bottom() - egui::vec2(0.0, margin_v),
        ],
        Stroke::new(1.0, theme.palette.border),
    );
}

pub fn top_toolbar_with_id(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl Hash,
    actions: &[ToolbarAction<'_>],
) -> TopToolbarResponse {
    let height = 36.0;
    let pad_h = theme.spacing.sm;
    let section_gap = theme.spacing.sm;
    let button_height = 28.0;
    let icon_size = 14.0;
    let icon_gap = theme.spacing.xs;
    let label_font = egui::FontId::proportional(12.0);
    let label_pad_h = theme.spacing.sm + 4.0;
    let sep_margin_v = theme.spacing.sm;
    let action_gap = theme.spacing.xs;
    let label_galleys: Vec<_> = actions
        .iter()
        .map(|action| {
            ui.painter().layout_no_wrap(
                action.label.to_owned(),
                label_font.clone(),
                theme.palette.foreground,
            )
        })
        .collect();
    let button_widths: Vec<f32> = label_galleys
        .iter()
        .map(|galley| galley.size().x + icon_size + icon_gap + label_pad_h * 2.0)
        .collect();
    let reset_separator_w = if actions.len() > 1 {
        section_gap * 2.0 + 1.0
    } else {
        0.0
    };
    let total_w = pad_h * 2.0
        + button_widths.iter().sum::<f32>()
        + action_gap * actions.len().saturating_sub(1) as f32
        + reset_separator_w;

    let (outer_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, height), Sense::hover());
    let mut result = TopToolbarResponse { icon_clicked: None };

    if ui.is_rect_visible(outer_rect) {
        let cr = CornerRadius::same(theme.radius.lg);

        ui.painter()
            .rect_filled(outer_rect, cr, theme.palette.surface_blur);
        ui.painter().rect_stroke(
            outer_rect,
            cr,
            Stroke::new(1.0, theme.palette.border),
            StrokeKind::Inside,
        );

        let inner_rect = outer_rect.shrink2(Vec2::new(pad_h, 0.0));
        let mut inner_ui =
            ui.new_child(egui::UiBuilder::new().id_salt(id_salt).max_rect(inner_rect));

        inner_ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = action_gap;
            let inner_cr = CornerRadius::same(theme.radius.md);
            for (index, action) in actions.iter().enumerate() {
                if index > 0 && index == actions.len().saturating_sub(1) {
                    ui.add_space(section_gap - action_gap);
                    separator(ui, theme, height, sep_margin_v);
                    ui.add_space(section_gap - action_gap);
                }

                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(button_widths[index], button_height),
                    Sense::click(),
                );
                let response = response.on_hover_text(action.tooltip);

                if !action.disabled && response.clicked() {
                    result.icon_clicked = Some(index);
                }

                if action.selected || (response.hovered() && !action.disabled) {
                    let inset = rect.shrink(theme.control_gap);
                    let fill = if action.selected {
                        theme.palette.control_fill_on
                    } else {
                        theme.palette.control_fill_off
                    };
                    ui.painter().rect_filled(inset, inner_cr, fill);
                }

                let text_color = if action.disabled {
                    theme.palette.muted_foreground.gamma_multiply(0.5)
                } else if action.selected || response.hovered() {
                    theme.palette.foreground
                } else {
                    theme.palette.muted_foreground
                };
                let label_galley = &label_galleys[index];
                let group_width = icon_size + icon_gap + label_galley.size().x;
                let icon_pos = egui::pos2(
                    rect.center().x - group_width / 2.0 + icon_size / 2.0,
                    rect.center().y,
                );
                ui.painter().text(
                    icon_pos,
                    egui::Align2::CENTER_CENTER,
                    action.icon.to_string(),
                    icon_font(icon_size),
                    text_color,
                );
                let text_pos = egui::pos2(
                    rect.center().x - group_width / 2.0 + icon_size + icon_gap,
                    rect.center().y - label_galley.size().y / 2.0,
                );
                ui.painter()
                    .galley(text_pos, label_galley.clone(), text_color);
            }
        });
    }

    result
}
