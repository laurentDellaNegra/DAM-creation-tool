use crate::form::{ClickTarget, ManualMapState, NextClickInfo};
use crate::frost_night::theme::typography;
use dam_core::{
    Coordinate, ManualGeometry, ManualMap, ManualMapAttributes, ManualMapCategory, PreviewPath,
    StaticMapSymbol, StaticMapSymbolKind, TextNumberColor, TextNumberSize,
    buffered_polygon_outlines, distance_nm, expand_polygon_nodes, is_full_circle,
    pie_circle_points, strip_corners,
};

pub struct PreviewOverlay {
    base_paths: Vec<PreviewPath>,
    selected_paths: Vec<PreviewPath>,
    selected_symbols: Vec<StaticMapSymbol>,
    manual_map: Option<ManualMap>,
    level_label: Option<(Coordinate, String)>,
    next_click: Option<NextClickInfo>,
    cursor_preview: Option<(ManualMapState, ClickTarget)>,
    level_label_text: Option<String>,
    display_levels: bool,
}

impl PreviewOverlay {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_paths: Vec<PreviewPath>,
        selected_paths: Vec<PreviewPath>,
        selected_symbols: Vec<StaticMapSymbol>,
        manual_map: Option<ManualMap>,
        level_label: Option<(Coordinate, String)>,
        next_click: Option<NextClickInfo>,
        cursor_preview: Option<(ManualMapState, ClickTarget)>,
        level_label_text: Option<String>,
        display_levels: bool,
    ) -> Self {
        Self {
            base_paths,
            selected_paths,
            selected_symbols,
            manual_map,
            level_label,
            next_click,
            cursor_preview,
            level_label_text,
            display_levels,
        }
    }
}

impl walkers::Plugin for PreviewOverlay {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        response: &egui::Response,
        projector: &walkers::Projector,
        _map_memory: &walkers::MapMemory,
    ) {
        let painter = ui.painter();

        paint_preview_paths(
            painter,
            projector,
            &self.base_paths,
            1.4,
            egui::Color32::from_rgb(95, 116, 132),
        );
        paint_preview_paths(
            painter,
            projector,
            &self.selected_paths,
            2.4,
            egui::Color32::from_rgb(95, 200, 205),
        );
        paint_static_symbols(painter, projector, &self.selected_symbols);

        let hover_inside = response
            .hover_pos()
            .filter(|pos| response.rect.contains(*pos));
        let cursor_coord = hover_inside.map(|pos| {
            let projected = projector.unproject(pos.to_vec2());
            Coordinate {
                lon: projected.x(),
                lat: projected.y(),
            }
        });

        let (ghost_map, ghost_label) =
            if let (Some(coord), Some((state, target))) = (cursor_coord, &self.cursor_preview) {
                let map = state.preview_with_cursor(*target, coord);
                let label = if self.display_levels && !is_level_label_target(*target) {
                    map.label_position.zip(self.level_label_text.clone())
                } else {
                    None
                };
                (Some(map), label)
            } else {
                (None, None)
            };

        let render_map = ghost_map.as_ref().or(self.manual_map.as_ref());
        if let Some(manual_map) = render_map {
            paint_manual_map(painter, projector, manual_map);
        }

        let render_label = ghost_label.as_ref().or(self.level_label.as_ref());
        if let Some((position, label)) = render_label {
            paint_level_label(painter, projector, *position, label);
        }

        if let Some(hover_pos) = hover_inside {
            let cursor_coord = cursor_coord.expect("hover_inside implies cursor_coord");

            if let Some(next_click) = &self.next_click {
                if let Some(anchor) = next_click.anchor
                    && next_click.draw_anchor_line
                {
                    paint_preview_line(painter, projector, anchor, hover_pos);
                }
                let target_rect = if next_click.show_distance {
                    if let Some(anchor) = next_click.anchor {
                        let distance = distance_nm(anchor, cursor_coord);
                        paint_cursor_target_label(
                            painter,
                            hover_pos,
                            response.rect,
                            &format!("{} · {:.1} NM", next_click.label, distance),
                        )
                    } else {
                        paint_cursor_target_label(
                            painter,
                            hover_pos,
                            response.rect,
                            &next_click.label,
                        )
                    }
                } else {
                    paint_cursor_target_label(painter, hover_pos, response.rect, &next_click.label)
                };
                paint_cursor_readout_below(painter, response.rect, target_rect, cursor_coord);
            } else {
                paint_cursor_readout_near(painter, response.rect, hover_pos, cursor_coord);
            }
        }
    }
}

fn paint_static_symbols(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    symbols: &[StaticMapSymbol],
) {
    for symbol in symbols {
        match symbol.kind {
            StaticMapSymbolKind::Para => {
                paint_para_symbol(painter, projector, symbol.coordinate, para_symbol_color());
            }
            StaticMapSymbolKind::Fallback => {
                paint_fallback_symbol(painter, projector, symbol.coordinate);
            }
        }
    }
}

fn is_level_label_target(target: ClickTarget) -> bool {
    matches!(
        target,
        ClickTarget::PolygonLabel | ClickTarget::PieLabel | ClickTarget::StripLabel
    )
}

fn paint_preview_paths(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    paths: &[PreviewPath],
    width: f32,
    fallback_color: egui::Color32,
) {
    for path in paths {
        let color = path
            .border_color
            .map(|[r, g, b]| egui::Color32::from_rgb(r, g, b))
            .unwrap_or(fallback_color);
        paint_line(
            painter,
            projector,
            &path.coordinates,
            egui::Stroke::new(width, color),
        );
    }
}

fn paint_manual_map(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    manual_map: &ManualMap,
) {
    let color = category_color(manual_map.attributes.category);
    let buffer_nm = manual_map.attributes.lateral_buffer_nm;
    match &manual_map.geometry {
        ManualGeometry::Polygon { nodes } => {
            let points = expand_polygon_nodes(nodes);
            paint_surface_or_line(painter, projector, &points, &manual_map.attributes, true);
            if buffer_nm > 0.0 && points.len() >= 3 {
                for buffered in buffered_polygon_outlines(&points, buffer_nm) {
                    paint_buffer_outline(painter, projector, &buffered, color, true);
                }
            }
            for node in nodes {
                if let Some(coord) = node.point_coordinate() {
                    paint_point(painter, projector, coord, color);
                } else if let dam_core::PolygonNode::Arc { center, .. } = node {
                    paint_point(painter, projector, *center, color);
                }
            }
        }
        ManualGeometry::ParaSymbol { point } => {
            if let Some(point) = point {
                paint_para_symbol(painter, projector, *point, para_symbol_color());
            }
        }
        ManualGeometry::TextNumber {
            point,
            text,
            color,
            size,
        } => {
            if let Some(point) = point {
                paint_text_number(painter, projector, *point, text, *color, *size);
            }
        }
        ManualGeometry::PieCircle {
            center,
            radius_nm,
            first_angle_deg,
            last_angle_deg,
        } => {
            if let Some(center) = center {
                paint_point(painter, projector, *center, color);
                if let Some(radius_nm) = radius_nm {
                    let shape =
                        pie_circle_points(*center, *radius_nm, *first_angle_deg, *last_angle_deg);
                    paint_surface_or_line(
                        painter,
                        projector,
                        &shape,
                        &manual_map.attributes,
                        is_full_circle(*first_angle_deg, *last_angle_deg),
                    );
                    if buffer_nm > 0.0 {
                        let buffered = pie_circle_points(
                            *center,
                            *radius_nm + buffer_nm,
                            *first_angle_deg,
                            *last_angle_deg,
                        );
                        paint_buffer_outline(
                            painter,
                            projector,
                            &buffered,
                            color,
                            is_full_circle(*first_angle_deg, *last_angle_deg),
                        );
                    }
                }
            }
        }
        ManualGeometry::Strip {
            point1,
            point2,
            width_nm,
        } => {
            if let Some(point1) = point1 {
                paint_point(painter, projector, *point1, color);
            }
            if let Some(point2) = point2 {
                paint_point(painter, projector, *point2, color);
            }
            if let (Some(point1), Some(point2), Some(width_nm)) = (point1, point2, width_nm) {
                let polygon = strip_corners(*point1, *point2, *width_nm);
                paint_surface_or_line(painter, projector, &polygon, &manual_map.attributes, true);
                if buffer_nm > 0.0 {
                    for buffered in buffered_polygon_outlines(&polygon, buffer_nm) {
                        paint_buffer_outline(painter, projector, &buffered, color, true);
                    }
                }
            } else if let (Some(point1), Some(point2)) = (point1, point2) {
                paint_line(
                    painter,
                    projector,
                    &[*point1, *point2],
                    egui::Stroke::new(2.0, color),
                );
            }
        }
    }
}

fn paint_surface_or_line(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinates: &[Coordinate],
    attributes: &ManualMapAttributes,
    close: bool,
) {
    let color = category_color(attributes.category);
    let mut points = projected_points(projector, coordinates);
    if close && points.len() >= 2 {
        points.push(points[0]);
    }

    if points.len() >= 3 {
        paint_filled_polygon(painter, &points, color.linear_multiply(0.22));
    }

    if points.len() >= 2 {
        painter.add(egui::Shape::line(points, egui::Stroke::new(2.2, color)));
    } else if let Some(point) = coordinates.first() {
        paint_point(painter, projector, *point, color);
    }
}

fn paint_filled_polygon(painter: &egui::Painter, points: &[egui::Pos2], fill: egui::Color32) {
    let mut vertices: Vec<egui::Pos2> = points.to_vec();
    if vertices.first() == vertices.last() {
        vertices.pop();
    }
    if vertices.len() < 3 {
        return;
    }

    let flat: Vec<f64> = vertices
        .iter()
        .flat_map(|point| [f64::from(point.x), f64::from(point.y)])
        .collect();
    let Ok(indices) = earcutr::earcut(&flat, &[], 2) else {
        return;
    };
    if indices.is_empty() {
        return;
    }

    let mut mesh = egui::Mesh::default();
    mesh.reserve_vertices(vertices.len());
    mesh.reserve_triangles(indices.len() / 3);
    for point in vertices {
        mesh.colored_vertex(point, fill);
    }
    for triangle in indices.chunks_exact(3) {
        mesh.add_triangle(triangle[0] as u32, triangle[1] as u32, triangle[2] as u32);
    }
    painter.add(egui::Shape::from(mesh));
}

fn paint_buffer_outline(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinates: &[Coordinate],
    color: egui::Color32,
    close: bool,
) {
    let mut points = projected_points(projector, coordinates);
    if close && points.len() >= 2 {
        points.push(points[0]);
    }
    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.5, color.linear_multiply(0.55)),
        ));
    }
}

fn paint_line(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    path: &[Coordinate],
    stroke: egui::Stroke,
) {
    let points = projected_points(projector, path);
    if points.len() >= 2 {
        painter.add(egui::Shape::line(points, stroke));
    }
}

fn paint_point(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinate: Coordinate,
    color: egui::Color32,
) {
    let position = project(projector, coordinate);
    painter.circle_filled(position, 4.0, color);
    painter.circle_stroke(position, 5.5, egui::Stroke::new(1.2, egui::Color32::BLACK));
}

fn paint_para_symbol(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinate: Coordinate,
    color: egui::Color32,
) {
    let center = project(projector, coordinate);
    painter.circle_filled(center, 4.0, color);
    painter.circle_stroke(center, 12.0, egui::Stroke::new(2.0, color));
    painter.line_segment(
        [center + egui::vec2(-10.0, -3.0), center],
        egui::Stroke::new(1.4, color),
    );
    painter.line_segment(
        [center + egui::vec2(10.0, -3.0), center],
        egui::Stroke::new(1.4, color),
    );
}

fn paint_fallback_symbol(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinate: Coordinate,
) {
    let center = project(projector, coordinate);
    let color = egui::Color32::from_rgb(242, 196, 84);
    let radius = 7.0;
    let points = vec![
        center + egui::vec2(0.0, -radius),
        center + egui::vec2(radius, 0.0),
        center + egui::vec2(0.0, radius),
        center + egui::vec2(-radius, 0.0),
        center + egui::vec2(0.0, -radius),
    ];
    painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
    painter.line_segment(
        [
            center + egui::vec2(-4.0, 0.0),
            center + egui::vec2(4.0, 0.0),
        ],
        egui::Stroke::new(1.4, color),
    );
    painter.line_segment(
        [
            center + egui::vec2(0.0, -4.0),
            center + egui::vec2(0.0, 4.0),
        ],
        egui::Stroke::new(1.4, color),
    );
}

fn paint_text_number(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinate: Coordinate,
    text: &str,
    color: TextNumberColor,
    size: TextNumberSize,
) {
    let position = project(projector, coordinate);
    paint_point(painter, projector, coordinate, text_color(color));
    painter.text(
        position + egui::vec2(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        text,
        typography::proportional(text_size(size)),
        text_color(color),
    );
}

fn paint_level_label(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    coordinate: Coordinate,
    label: &str,
) {
    let position = project(projector, coordinate) + egui::vec2(10.0, 10.0);
    let galley = painter.layout_no_wrap(
        label.to_owned(),
        typography::monospace(13.0),
        egui::Color32::WHITE,
    );
    let rect = egui::Rect::from_min_size(position, galley.size() + egui::vec2(10.0, 6.0));
    painter.rect_filled(rect, 3.0, egui::Color32::from_black_alpha(190));
    painter.rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(190, 210, 220)),
        egui::StrokeKind::Outside,
    );
    painter.galley(
        position + egui::vec2(5.0, 3.0),
        galley,
        egui::Color32::WHITE,
    );
}

fn projected_points(projector: &walkers::Projector, path: &[Coordinate]) -> Vec<egui::Pos2> {
    path.iter()
        .map(|coordinate| project(projector, *coordinate))
        .collect()
}

fn project(projector: &walkers::Projector, coordinate: Coordinate) -> egui::Pos2 {
    let projected = projector.project(walkers::lon_lat(coordinate.lon, coordinate.lat));
    egui::pos2(projected.x, projected.y)
}

fn category_color(category: ManualMapCategory) -> egui::Color32 {
    match category {
        ManualMapCategory::Prohibited
        | ManualMapCategory::Danger
        | ManualMapCategory::Restricted => egui::Color32::from_rgb(235, 87, 87),
        ManualMapCategory::Glider => egui::Color32::from_rgb(76, 175, 118),
        ManualMapCategory::Ctr | ManualMapCategory::Tma | ManualMapCategory::Para => {
            egui::Color32::from_rgb(91, 153, 234)
        }
        ManualMapCategory::Cfz | ManualMapCategory::Other => egui::Color32::from_rgb(185, 194, 204),
    }
}

fn para_symbol_color() -> egui::Color32 {
    egui::Color32::from_rgb(92, 160, 255)
}

fn text_color(color: TextNumberColor) -> egui::Color32 {
    match color {
        TextNumberColor::Red => egui::Color32::from_rgb(245, 82, 82),
        TextNumberColor::Green => egui::Color32::from_rgb(86, 196, 118),
        TextNumberColor::Blue => egui::Color32::from_rgb(92, 160, 255),
        TextNumberColor::Yellow => egui::Color32::from_rgb(238, 205, 72),
        TextNumberColor::White => egui::Color32::WHITE,
    }
}

fn text_size(size: TextNumberSize) -> f32 {
    match size {
        TextNumberSize::Small => 12.0,
        TextNumberSize::Medium => 16.0,
        TextNumberSize::Large => 22.0,
    }
}

fn paint_preview_line(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    anchor: Coordinate,
    cursor_pos: egui::Pos2,
) {
    let anchor_pos = project(projector, anchor);
    let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 230, 240));
    painter.line_segment([anchor_pos, cursor_pos], stroke);
}

fn paint_cursor_target_label(
    painter: &egui::Painter,
    cursor_pos: egui::Pos2,
    bounds: egui::Rect,
    label: &str,
) -> egui::Rect {
    let position = cursor_pos + egui::vec2(14.0, -22.0);
    let galley = painter.layout_no_wrap(
        label.to_owned(),
        typography::proportional(12.0),
        egui::Color32::WHITE,
    );
    let padding = egui::vec2(6.0, 3.0);
    let size = galley.size() + padding * 2.0;
    let bg_rect = clamped_popup_rect(bounds, position, size);
    painter.rect_filled(bg_rect, 3.0, egui::Color32::from_black_alpha(200));
    painter.rect_stroke(
        bg_rect,
        3.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(170, 200, 220)),
        egui::StrokeKind::Outside,
    );
    painter.galley(bg_rect.min + padding, galley, egui::Color32::WHITE);
    bg_rect
}

fn paint_cursor_readout_below(
    painter: &egui::Painter,
    bounds: egui::Rect,
    target_rect: egui::Rect,
    coordinate: Coordinate,
) {
    let position = target_rect.left_bottom() + egui::vec2(0.0, 4.0);
    paint_cursor_readout(painter, bounds, position, coordinate);
}

fn paint_cursor_readout_near(
    painter: &egui::Painter,
    bounds: egui::Rect,
    cursor_pos: egui::Pos2,
    coordinate: Coordinate,
) {
    paint_cursor_readout(
        painter,
        bounds,
        cursor_pos + egui::vec2(14.0, 2.0),
        coordinate,
    );
}

fn paint_cursor_readout(
    painter: &egui::Painter,
    bounds: egui::Rect,
    position: egui::Pos2,
    coordinate: Coordinate,
) {
    let label = format!("{:.5} - {:.5}", coordinate.lat, coordinate.lon);
    let galley = painter.layout_no_wrap(label, typography::monospace(12.0), egui::Color32::WHITE);
    let padding = egui::vec2(8.0, 4.0);
    let size = galley.size() + padding * 2.0;
    let bg_rect = clamped_popup_rect(bounds, position, size);
    painter.rect_filled(bg_rect, 3.0, egui::Color32::from_black_alpha(200));
    painter.rect_stroke(
        bg_rect,
        3.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 170, 180)),
        egui::StrokeKind::Outside,
    );
    painter.galley(bg_rect.min + padding, galley, egui::Color32::WHITE);
}

fn clamped_popup_rect(bounds: egui::Rect, position: egui::Pos2, size: egui::Vec2) -> egui::Rect {
    let margin = 8.0;
    let min_x = bounds.left() + margin;
    let min_y = bounds.top() + margin;
    let max_x = (bounds.right() - size.x - margin).max(min_x);
    let max_y = (bounds.bottom() - size.y - margin).max(min_y);
    egui::Rect::from_min_size(
        egui::pos2(
            position.x.clamp(min_x, max_x),
            position.y.clamp(min_y, max_y),
        ),
        size,
    )
}
