//! Themed accordion — collapsible sections with animated open/close.

use std::hash::Hash;

use egui::{Response, Sense, Ui, Vec2};

use crate::frost_night::theme::{Theme, mix, typography};

/// Render an accordion group.
///
/// - `items`: title strings for each section header.
/// - `open`: mutable slice tracking which sections are expanded.
///   Must have the same length as `items`.
/// - `exclusive`: when `true`, opening a section closes all others.
/// - `add_body`: called for each open section with `(ui, index)`.
pub fn accordion(
    ui: &mut Ui,
    theme: &Theme,
    items: &[&str],
    open: &mut [bool],
    exclusive: bool,
    add_body: impl FnMut(&mut Ui, usize),
) -> Response {
    accordion_with_id(ui, theme, "accordion", items, open, exclusive, add_body)
}

/// Render an accordion group with a caller-provided ID salt.
///
/// `items` and `open` should have the same length. In debug builds this is
/// asserted. In release builds the accordion renders the shared prefix instead
/// of panicking during UI rendering.
pub fn accordion_with_id(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl Hash,
    items: &[&str],
    open: &mut [bool],
    exclusive: bool,
    mut add_body: impl FnMut(&mut Ui, usize),
) -> Response {
    debug_assert_eq!(
        items.len(),
        open.len(),
        "items and open must have same length"
    );
    let len = items.len().min(open.len());

    let anim_duration = 0.15;
    // Use the parent scope's stable ID (not auto_id which depends on widget counter).
    let accordion_id = ui.id().with(id_salt);

    egui::Frame::new()
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            for (i, &title) in items.iter().take(len).enumerate() {
                let item_id = accordion_id.with(i);

                // Header
                let header_height = theme.spacing.sm * 2.0 + 16.0;
                let width = ui.available_width();

                let response = ui.push_id(item_id, |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(width, header_height), Sense::click());
                    (rect, response)
                });
                let (header_rect, header_response) = response.inner;

                // Handle click
                if header_response.clicked() {
                    if exclusive {
                        let was_open = open[i];
                        for o in open.iter_mut() {
                            *o = false;
                        }
                        open[i] = !was_open;
                    } else {
                        open[i] = !open[i];
                    }
                }

                // Animate open/close
                let open_t =
                    ui.ctx()
                        .animate_bool_with_time(item_id.with("anim"), open[i], anim_duration);

                // Remember body height from previous frame for smooth animation
                let height_id = item_id.with("body_h");
                let prev_body_height: f32 =
                    ui.ctx().data_mut(|d| d.get_temp(height_id).unwrap_or(0.0));

                // Paint header
                if ui.is_rect_visible(header_rect) {
                    if header_response.hovered() {
                        let clip = ui.clip_rect();
                        let hover_rect = header_rect.intersect(clip);
                        if hover_rect.is_positive() {
                            ui.painter().rect_filled(
                                hover_rect,
                                egui::CornerRadius::same(theme.radius.md),
                                theme.palette.muted,
                            );
                        }
                    }

                    // Animated triangle (rotates right → down)
                    let tri_size = 6.0;
                    let tri_x = header_rect.left() + theme.spacing.md;
                    let tri_cy = header_rect.center().y;

                    let indicator_color =
                        mix(theme.palette.muted_foreground, theme.palette.ring, open_t);

                    let angle = open_t * std::f32::consts::FRAC_PI_2;
                    let (sin_a, cos_a) = angle.sin_cos();

                    let base = [
                        (-0.35 * tri_size, -0.5 * tri_size),
                        (0.45 * tri_size, 0.0),
                        (-0.35 * tri_size, 0.5 * tri_size),
                    ];
                    let points: Vec<egui::Pos2> = base
                        .iter()
                        .map(|&(px, py)| {
                            egui::pos2(
                                tri_x + px * cos_a - py * sin_a,
                                tri_cy + px * sin_a + py * cos_a,
                            )
                        })
                        .collect();

                    ui.painter().add(egui::Shape::convex_polygon(
                        points,
                        indicator_color,
                        egui::Stroke::NONE,
                    ));

                    // Title
                    let text_x = tri_x + tri_size + theme.spacing.sm;
                    let title_color = mix(
                        theme.palette.muted_foreground,
                        theme.palette.foreground,
                        open_t,
                    );
                    let galley = ui.painter().layout_no_wrap(
                        title.to_string(),
                        typography::proportional(13.0),
                        title_color,
                    );
                    ui.painter().galley(
                        egui::pos2(text_x, header_rect.center().y - galley.size().y / 2.0),
                        galley,
                        title_color,
                    );
                }

                // Body with height animation
                if open_t > 0.0 {
                    let animated_height = prev_body_height * open_t;

                    // Clip region for the body
                    let body_max_rect = egui::Rect::from_min_size(
                        ui.available_rect_before_wrap().min,
                        Vec2::new(width, animated_height),
                    );

                    // Allocate the animated height in the layout
                    ui.allocate_exact_size(Vec2::new(width, animated_height), Sense::hover());

                    // Render body content clipped to the animated rect
                    let prev_opacity = ui.opacity();
                    ui.set_opacity(prev_opacity * open_t);

                    let mut body_ui =
                        ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                            body_max_rect.min,
                            Vec2::new(width, f32::INFINITY),
                        )));
                    body_ui.set_clip_rect(body_max_rect.intersect(ui.clip_rect()));

                    let body_margin =
                        egui::Margin::symmetric(theme.spacing.md as i8, theme.spacing.sm as i8);
                    egui::Frame::new()
                        .inner_margin(body_margin)
                        .show(&mut body_ui, |ui| {
                            add_body(ui, i);
                        });

                    // Store actual body height for next frame
                    let actual_height = body_ui.min_size().y;
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(height_id, actual_height));

                    ui.set_opacity(prev_opacity);
                } else {
                    // Even when closed, render invisibly to measure height
                    // so the opening animation knows the target height.
                    // Only needed if we don't have a stored height yet.
                    if prev_body_height == 0.0 {
                        let measure_rect = egui::Rect::from_min_size(
                            ui.available_rect_before_wrap().min,
                            Vec2::new(width, 0.0),
                        );
                        let mut measure_ui = ui.new_child(egui::UiBuilder::new().max_rect(
                            egui::Rect::from_min_size(
                                measure_rect.min,
                                Vec2::new(width, f32::INFINITY),
                            ),
                        ));
                        measure_ui.set_clip_rect(measure_rect);
                        measure_ui.set_invisible();

                        let body_margin =
                            egui::Margin::symmetric(theme.spacing.md as i8, theme.spacing.sm as i8);
                        egui::Frame::new()
                            .inner_margin(body_margin)
                            .show(&mut measure_ui, |ui| {
                                add_body(ui, i);
                            });

                        let actual_height = measure_ui.min_size().y;
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(height_id, actual_height));
                    }
                }
            }
        })
        .response
}
