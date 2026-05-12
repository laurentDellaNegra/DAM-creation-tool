//! Draggable card with a title bar handle.
//!
//! A simplified draggable card for documentation purposes. The demo uses
//! `sidebar_card` which adds glow effects, highlight animation, and
//! toolbar integration on top of the same drag mechanics.

use egui::{Color32, CornerRadius, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::frost_night::theme::{Theme, mix, typography};

/// Persistent state for a draggable card.
#[derive(Clone, Debug)]
pub struct DragCardState {
    pub pos: Pos2,
    pub size: Vec2,
}

/// Response from a drag card.
pub struct DragCardResponse {
    /// The close button was clicked.
    pub closed: bool,
    /// The card is currently being dragged.
    pub dragging: bool,
}

/// A draggable floating card with built-in backdrop and drag-to-fade animation.
///
/// The card paints a semi-transparent `surface_blur` backdrop, a border,
/// a title bar with drag handle and close button, and a scrollable body.
/// When dragged, the entire card fades to 15% opacity.
pub fn drag_card(
    ui: &mut Ui,
    theme: &Theme,
    id: Id,
    state: &mut DragCardState,
    title: &str,
    add_contents: impl FnOnce(&mut Ui),
) -> DragCardResponse {
    let cr = CornerRadius::same(theme.radius.lg);
    let padding = theme.spacing.md;
    let dots_zone_h = theme.spacing.md; // top zone with 3 dots / grab bar
    let title_h = theme.spacing.xl; // title + close button row
    let header_gap = theme.spacing.sm; // space between dots zone and title
    let handle_h = dots_zone_h + header_gap + title_h;

    // --- Drag interaction on the full handle zone ---
    let handle_rect = Rect::from_min_size(state.pos, Vec2::new(state.size.x, handle_h));
    let drag_response = ui.interact(handle_rect, id.with("drag"), Sense::drag());
    if drag_response.dragged() {
        state.pos += drag_response.drag_delta();
    }
    // Recompute after potential drag delta
    let card_rect = Rect::from_min_size(state.pos, state.size);

    // --- Drag animation: fade + border glow ---
    let dragging = drag_response.dragged();
    let drag_t = ui
        .ctx()
        .animate_bool_with_time(id.with("drag_anim"), dragging, 0.2);

    // --- Semi-transparent backdrop ---
    ui.painter()
        .rect_filled(card_rect, cr, theme.palette.surface_blur);

    // Border glow: lerp from normal border to ring color, widen stroke
    let border_color = mix(theme.palette.border, theme.palette.ring, drag_t);
    let border_width = egui::lerp(1.0..=1.8, drag_t);
    ui.painter().rect_stroke(
        card_rect,
        cr,
        Stroke::new(border_width, border_color),
        StrokeKind::Outside,
    );

    // Outer glow halo (faint ring-colored shadow expanding outward)
    if drag_t > 0.01 {
        let glow_alpha = (drag_t * 60.0) as u8;
        let glow_color = Color32::from_rgba_unmultiplied(
            theme.palette.ring.r(),
            theme.palette.ring.g(),
            theme.palette.ring.b(),
            glow_alpha,
        );
        let glow_expand = egui::lerp(0.0..=4.0, drag_t);
        ui.painter().rect_stroke(
            card_rect.expand(glow_expand),
            cr,
            Stroke::new(2.0, glow_color),
            StrokeKind::Outside,
        );
    }

    // --- Handle indicator: 3 dots → grab bar ---
    let hovered = drag_response.hovered() || drag_response.dragged();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(id.with("handle_hover"), hovered, 0.15);

    let dots_center_y = card_rect.top() + padding + dots_zone_h / 2.0;
    let dots_center_x = card_rect.center().x;
    let dot_radius = 2.0;
    let dot_spacing = 6.0;

    // Bar dimensions (target of animation)
    let bar_half_w = theme.spacing.lg;
    let bar_h = 3.0;
    let bar_cr = CornerRadius::same(2);

    let handle_color = mix(
        theme.palette.muted_foreground,
        theme.palette.foreground,
        hover_t * 0.5,
    );

    if hover_t < 0.99 {
        // Draw 3 dots, fading out as hover_t increases
        let dot_alpha = ((1.0 - hover_t) * 255.0) as u8;
        let dot_color = Color32::from_rgba_unmultiplied(
            handle_color.r(),
            handle_color.g(),
            handle_color.b(),
            dot_alpha,
        );
        for i in [-1.0, 0.0, 1.0] {
            // Dots spread apart slightly as they fade, merging toward bar
            let spread = egui::lerp(1.0..=2.0, hover_t);
            let x = dots_center_x + i * dot_spacing * spread;
            ui.painter()
                .circle_filled(egui::pos2(x, dots_center_y), dot_radius, dot_color);
        }
    }

    if hover_t > 0.01 {
        // Draw grab bar, fading in
        let bar_alpha = (hover_t * 255.0) as u8;
        let bar_color = Color32::from_rgba_unmultiplied(
            handle_color.r(),
            handle_color.g(),
            handle_color.b(),
            bar_alpha,
        );
        // Bar width grows in from dot cluster width
        let w = egui::lerp(dot_spacing..=bar_half_w, hover_t);
        let bar_rect = Rect::from_center_size(
            egui::pos2(dots_center_x, dots_center_y),
            Vec2::new(w * 2.0, bar_h),
        );
        ui.painter().rect_filled(bar_rect, bar_cr, bar_color);
    }

    // --- Title row (below dots zone + gap) ---
    let title_center_y = card_rect.top() + padding + dots_zone_h + header_gap + title_h / 2.0;

    // Title text
    ui.painter().text(
        egui::pos2(card_rect.left() + padding, title_center_y),
        egui::Align2::LEFT_CENTER,
        title,
        typography::proportional(13.0),
        theme.palette.foreground,
    );

    // Close button (X)
    let close_size = theme.spacing.lg;
    let close_center = egui::pos2(
        card_rect.right() - padding - close_size / 2.0,
        title_center_y,
    );
    let close_rect =
        Rect::from_center_size(close_center, Vec2::splat(close_size + theme.spacing.sm));
    let close_response = ui.interact(close_rect, id.with("close"), Sense::click());

    let x_color = if close_response.hovered() {
        theme.palette.foreground
    } else {
        theme.palette.muted_foreground
    };
    let s = close_size / 2.0 - 2.0;
    ui.painter().line_segment(
        [
            close_center + egui::vec2(-s, -s),
            close_center + egui::vec2(s, s),
        ],
        Stroke::new(1.5, x_color),
    );
    ui.painter().line_segment(
        [
            close_center + egui::vec2(s, -s),
            close_center + egui::vec2(-s, s),
        ],
        Stroke::new(1.5, x_color),
    );

    // Cursor hint
    if drag_response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    if drag_response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }

    // --- Body content ---
    let body_top = card_rect.top() + padding + handle_h + theme.spacing.xs;
    let body_rect = Rect::from_min_max(
        egui::pos2(card_rect.left() + padding, body_top),
        egui::pos2(
            card_rect.right() - theme.spacing.xs,
            card_rect.bottom() - padding,
        ),
    );

    let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body_rect));
    egui::ScrollArea::vertical().show(&mut body_ui, |ui| {
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(theme.spacing.xs as i8, 2))
            .show(ui, |ui| add_contents(ui));
    });

    DragCardResponse {
        closed: close_response.clicked(),
        dragging: drag_response.dragged(),
    }
}
