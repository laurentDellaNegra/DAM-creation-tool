//! Underline-style tab bar matching the Figma card mockup.

use std::hash::Hash;

use egui::{Response, Sense, Stroke, Ui, Vec2};

use crate::frost_night::theme::{Theme, mix, typography};

type IconGalley = Option<(char, std::sync::Arc<egui::Galley>)>;
type TabMetric = (IconGalley, std::sync::Arc<egui::Galley>, f32);

/// A horizontal tab bar with underline indicator.
pub fn tabs(ui: &mut Ui, theme: &Theme, selected: &mut usize, labels: &[&str]) -> Response {
    tabs_with_id(ui, theme, "tabs", selected, labels)
}

/// A horizontal tab bar with a caller-provided ID salt.
///
/// Use this when rendering multiple tab bars under the same parent `Ui`.
pub fn tabs_with_id(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl Hash,
    selected: &mut usize,
    labels: &[&str],
) -> Response {
    tabs_impl(ui, theme, id_salt, selected, labels, None)
}

/// A horizontal tab bar with optional per-tab icons.
///
/// `icons` can be shorter than `labels` — missing entries get no icon.
#[cfg(feature = "icons")]
pub fn tabs_with_icons(
    ui: &mut Ui,
    theme: &Theme,
    selected: &mut usize,
    labels: &[&str],
    icons: &[Option<char>],
) -> Response {
    tabs_with_icons_with_id(ui, theme, "tabs", selected, labels, icons)
}

/// A horizontal tab bar with optional icons and a caller-provided ID salt.
#[cfg(feature = "icons")]
pub fn tabs_with_icons_with_id(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl Hash,
    selected: &mut usize,
    labels: &[&str],
    icons: &[Option<char>],
) -> Response {
    tabs_impl(ui, theme, id_salt, selected, labels, Some(icons))
}

fn tabs_impl(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl Hash,
    selected: &mut usize,
    labels: &[&str],
    icons: Option<&[Option<char>]>,
) -> Response {
    let tab_height = theme.spacing.sm * 2.0 + 14.0;
    let underline_thickness = 1.5;
    let anim_duration = 0.12;
    let tab_bar_id = ui.id().with(id_salt);
    let icon_gap = theme.spacing.sm;

    let current = *selected;

    let width = ui.available_width();
    let (bar_rect, bar_response) = ui.allocate_exact_size(
        Vec2::new(width, tab_height + underline_thickness),
        Sense::hover(),
    );

    if !ui.is_rect_visible(bar_rect) {
        return bar_response;
    }

    ui.painter().hline(
        bar_rect.left()..=bar_rect.right(),
        bar_rect.bottom(),
        Stroke::new(1.0, theme.palette.border),
    );

    let tab_padding_h = theme.spacing.md;
    let tab_gap = theme.spacing.xs;
    #[cfg(feature = "icons")]
    let icon_font_size = typography::font_size(13.0);

    // Measure tab widths (icon + gap + text)
    let tab_metrics: Vec<TabMetric> = labels
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let icon_galley: IconGalley = {
                #[cfg(feature = "icons")]
                {
                    icons
                        .and_then(|icons| icons.get(i).and_then(|opt| *opt))
                        .map(|ch| {
                            (
                                ch,
                                ui.painter().layout_no_wrap(
                                    ch.to_string(),
                                    crate::frost_night::icons::icon_font(icon_font_size),
                                    theme.palette.foreground,
                                ),
                            )
                        })
                }

                #[cfg(not(feature = "icons"))]
                {
                    let _ = i;
                    let _ = icons;
                    None
                }
            };
            let text_galley = ui.painter().layout_no_wrap(
                label.to_string(),
                typography::proportional(13.0),
                theme.palette.foreground,
            );
            let icon_w = icon_galley
                .as_ref()
                .map_or(0.0, |(_, g)| g.size().x + icon_gap);
            let total_w = tab_padding_h + icon_w + text_galley.size().x + tab_padding_h;
            (icon_galley, text_galley, total_w)
        })
        .collect();

    let mut x = bar_rect.left();
    let mut clicked_tab: Option<usize> = None;

    for (i, (_icon_galley, _text_galley, tab_w)) in tab_metrics.iter().enumerate() {
        let tab_id = tab_bar_id.with(i);
        let is_selected = current == i;

        let tab_rect = egui::Rect::from_min_size(
            egui::pos2(x, bar_rect.top()),
            Vec2::new(*tab_w, tab_height + underline_thickness),
        );

        let tab_response = ui.interact(tab_rect, tab_id, Sense::click());
        if tab_response.clicked() {
            clicked_tab = Some(i);
        }

        let sel_t = ui
            .ctx()
            .animate_bool_with_time(tab_id.with("sel"), is_selected, anim_duration);

        let hovered = tab_response.hovered() && !is_selected;

        let text_color = if hovered {
            mix(
                theme.palette.muted_foreground,
                theme.palette.foreground,
                0.5,
            )
        } else {
            mix(
                theme.palette.muted_foreground,
                theme.palette.foreground,
                sel_t,
            )
        };

        let text_x = tab_rect.left() + tab_padding_h;
        let center_y = bar_rect.top() + tab_height / 2.0;

        // Icon (if any)
        #[cfg(feature = "icons")]
        let text_x = {
            let mut cx = text_x;
            if let Some((icon, ig)) = _icon_galley {
                let icon_color = if hovered {
                    mix(theme.palette.muted_foreground, theme.palette.ring, 0.5)
                } else {
                    mix(theme.palette.muted_foreground, theme.palette.ring, sel_t)
                };
                let icon_g = ui.painter().layout_no_wrap(
                    icon.to_string(),
                    crate::frost_night::icons::icon_font(icon_font_size),
                    icon_color,
                );
                ui.painter().galley(
                    egui::pos2(cx, center_y - icon_g.size().y / 2.0),
                    icon_g.clone(),
                    icon_color,
                );
                cx += ig.size().x + icon_gap;
            }
            cx
        };

        // Text
        let text_galley = ui.painter().layout_no_wrap(
            labels[i].to_string(),
            typography::proportional(13.0),
            text_color,
        );
        ui.painter().galley(
            egui::pos2(text_x, center_y - text_galley.size().y / 2.0),
            text_galley,
            text_color,
        );

        // Underline
        if sel_t > 0.0 {
            let underline_color = mix(egui::Color32::TRANSPARENT, theme.palette.ring, sel_t);
            let underline_y = bar_rect.bottom() - underline_thickness / 2.0;
            let center_x = tab_rect.center().x;
            let half_w = (*tab_w * sel_t) / 2.0;
            ui.painter().hline(
                (center_x - half_w)..=(center_x + half_w),
                underline_y,
                Stroke::new(underline_thickness, underline_color),
            );
        }

        x += *tab_w + tab_gap;
    }

    if let Some(i) = clicked_tab {
        *selected = i;
    }

    bar_response
}
