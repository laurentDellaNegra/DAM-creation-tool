mod form;
mod frost_night;
mod online_tiles;
mod platform;
mod preview;
mod submission;

use crate::form::{
    ArcDraftState, ClickTarget, CoordinateFieldState, DamFormState, LevelFieldState,
    ManualGeometryType, MapMode, PeriodRowState, PieCircleDraftState, PolygonDraftState,
    PolygonNodeDraft, StripDraftState, TextNumberDraftState, geometry_supports_buffer,
};
use crate::frost_night::components::{checkbox as frost_checkbox, segmented};
use crate::frost_night::composites::{ToolbarAction, top_toolbar_with_id};
use crate::frost_night::containers::{DragCardState, drag_card, tabs_with_id};
use crate::frost_night::icons::{ICON_BOOK_OPEN, ICON_CIRCLE_X, ICON_GLOBE, ICON_PLANE};
use crate::frost_night::theme::mix;
use crate::frost_night::{
    ControlSize, ControlVariant, FrostUiExt, InstallThemeOptions, Theme, install_theme,
};
use crate::online_tiles::CartoDarkMatter;
use crate::preview::PreviewOverlay;
use crate::submission::{SubmissionEndpoint, SubmissionResult, SubmissionStatus, submit_payload};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime};
use dam_core::{
    AltitudeCorrection, BufferFilter, CatalogDiagnostic, Coordinate, MAX_PERIODS,
    MAX_POLYGON_POINTS, ManualMapCategory, ManualMapRendering, MapCatalog, PreviewGeometry,
    StaticMap, TextNumberColor, TextNumberSize, ValidationIssue, Weekday, build_aixm_payload,
    build_json_payload, bundled_catalog, switzerland_default_preview, unit_groups,
};

pub struct DamApp {
    frost_theme: Theme,
    catalog: MapCatalog,
    default_preview: PreviewGeometry,
    form: DamFormState,
    selected_period: usize,
    map_memory: walkers::MapMemory,
    map_tiles: walkers::HttpTiles,
    show_distribution: bool,
    distribution_card: DragCardState,
    show_reset_confirm: bool,
    active_date_picker: Option<DateField>,
    date_picker_month: NaiveDate,
    diagnostics_open: bool,
    submission_endpoint: Option<SubmissionEndpoint>,
    submission_status: SubmissionStatus,
    pending_click_target: Option<ClickTarget>,
    previous_active_geometry: Option<ManualGeometryType>,
}

const MANUAL_ATTRIBUTE_CATEGORIES: [ManualMapCategory; 6] = [
    ManualMapCategory::Prohibited,
    ManualMapCategory::Restricted,
    ManualMapCategory::Danger,
    ManualMapCategory::Glider,
    ManualMapCategory::Para,
    ManualMapCategory::Other,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum DateField {
    Start,
    End,
}

impl DateField {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Start date",
            Self::End => "End date",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

impl DamApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let frost_theme = Theme::dark();
        configure_visuals(&cc.egui_ctx, &frost_theme);

        let catalog = bundled_catalog();
        let default_preview = switzerland_default_preview();
        let form = DamFormState::new(&catalog);
        let mut map_memory = walkers::MapMemory::default();
        if let Some(map) = form.selected_map(&catalog) {
            center_map_on_static_map(&mut map_memory, map);
        } else {
            center_map_on_preview(&mut map_memory, &default_preview, 7.0);
        }

        Self {
            frost_theme,
            catalog,
            default_preview,
            form,
            selected_period: 0,
            map_memory,
            map_tiles: walkers::HttpTiles::new(CartoDarkMatter, cc.egui_ctx.clone()),
            show_distribution: false,
            distribution_card: DragCardState {
                pos: egui::pos2(96.0, 96.0),
                size: egui::vec2(456.0, 520.0),
            },
            show_reset_confirm: false,
            active_date_picker: None,
            date_picker_month: first_day_of_month(current_date()),
            diagnostics_open: false,
            submission_endpoint: None,
            submission_status: SubmissionStatus::Idle,
            pending_click_target: None,
            previous_active_geometry: None,
        }
    }
}

impl eframe::App for DamApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.form.sync_weekdays_from_dates();
        self.maybe_auto_focus_on_geometry_change(ui.ctx());
        self.update_click_target_from_memory(ui.ctx());

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(self.frost_theme.palette.background))
            .show_inside(ui, |ui| {
                egui::Panel::left("creation_form")
                    .resizable(true)
                    .default_size(500.0)
                    .size_range(420.0..=760.0)
                    .frame(
                        egui::Frame::new()
                            .fill(self.frost_theme.palette.background)
                            .inner_margin(egui::Margin::same(self.frost_theme.spacing.md as i8)),
                    )
                    .show_inside(ui, |ui| self.form_panel(ui));

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(self.frost_theme.palette.background))
                    .show_inside(ui, |ui| self.preview_panel(ui));
            });

        self.toolbar(ui.ctx());
        self.distribution_window(ui.ctx());
        self.reset_confirmation(ui.ctx());
        self.date_picker_window(ui.ctx());
    }
}

fn configure_visuals(ctx: &egui::Context, theme: &Theme) {
    install_theme(
        ctx,
        theme,
        InstallThemeOptions {
            install_visuals: true,
            install_fonts: true,
        },
    );

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(theme.spacing.sm, theme.spacing.sm);
    style.spacing.button_padding = ControlSize::Md.padding();
    ctx.set_global_style(style);
}

fn themed_text_edit(
    ui: &mut egui::Ui,
    theme: &Theme,
    text_edit: egui::TextEdit<'_>,
    size: ControlSize,
) -> egui::Response {
    themed_text_edit_enabled(ui, theme, true, text_edit, size)
}

fn themed_text_edit_enabled(
    ui: &mut egui::Ui,
    theme: &Theme,
    enabled: bool,
    text_edit: egui::TextEdit<'_>,
    size: ControlSize,
) -> egui::Response {
    let visuals = theme.input(size);
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.visuals.extreme_bg_color = visuals.bg;
        style.visuals.widgets.inactive.bg_stroke = visuals.border;
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme.palette.ring);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme.palette.ring);

        ui.add_enabled(
            enabled,
            text_edit
                .font(visuals.font)
                .text_color(visuals.text_color)
                .margin(egui::Margin::symmetric(theme.spacing.sm as i8, 6)),
        )
    })
    .inner
}

fn colored_segmented(
    ui: &mut egui::Ui,
    theme: &Theme,
    labels: &[&str],
    active_fills: &[egui::Color32],
    selected: &mut usize,
) -> egui::Response {
    let font = egui::FontId::proportional(12.0);
    let pad = egui::vec2(theme.spacing.lg, theme.spacing.xs + 2.0);
    let gap = theme.control_gap;

    let galleys: Vec<_> = labels
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap(label.to_string(), font.clone(), theme.palette.foreground)
        })
        .collect();
    let segment_widths: Vec<f32> = galleys
        .iter()
        .map(|galley| galley.size().x + pad.x * 2.0)
        .collect();
    let total_width = segment_widths.iter().sum();
    let height = galleys
        .iter()
        .map(|galley| galley.size().y)
        .fold(0.0_f32, f32::max)
        + pad.y * 2.0;

    let (outer_rect, mut response) =
        ui.allocate_exact_size(egui::vec2(total_width, height), egui::Sense::click());

    if !ui.is_rect_visible(outer_rect) {
        return response;
    }

    let outer_radius = egui::CornerRadius::same(theme.radius.lg);
    let inner_radius = egui::CornerRadius::same(theme.radius.md);
    ui.painter().rect_stroke(
        outer_rect,
        outer_radius,
        egui::Stroke::new(1.0, theme.palette.control_border),
        egui::StrokeKind::Inside,
    );

    let clicked_pos = if ui.is_enabled() && response.clicked() {
        response.interact_pointer_pos()
    } else {
        None
    };
    let mut x = outer_rect.left();
    for (index, galley) in galleys.into_iter().enumerate() {
        let segment_rect = egui::Rect::from_min_size(
            egui::pos2(x, outer_rect.top()),
            egui::vec2(segment_widths[index], height),
        );
        let is_active = index == *selected;
        let hovered = response.hovered()
            && response
                .hover_pos()
                .is_some_and(|pos| segment_rect.contains(pos));

        if let Some(pos) = clicked_pos
            && segment_rect.contains(pos)
            && !is_active
        {
            *selected = index;
            response.mark_changed();
        }

        let active_fill = active_fills
            .get(index)
            .copied()
            .unwrap_or(theme.palette.control_fill_on);
        if is_active {
            ui.painter()
                .rect_filled(segment_rect.shrink(gap), inner_radius, active_fill);
        }

        let text_color = if is_active {
            contrast_text_color(active_fill, theme)
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
            segment_rect.center().x - galley.size().x / 2.0,
            segment_rect.center().y - galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, galley, text_color);

        x += segment_widths[index];
    }

    response
}

fn contrast_text_color(fill: egui::Color32, theme: &Theme) -> egui::Color32 {
    let luminance = (0.2126 * f32::from(fill.r())
        + 0.7152 * f32::from(fill.g())
        + 0.0722 * f32::from(fill.b()))
        / 255.0;
    if luminance > 0.62 {
        theme.palette.background
    } else {
        egui::Color32::WHITE
    }
}

fn manual_category_color(category: ManualMapCategory) -> egui::Color32 {
    match category {
        ManualMapCategory::Prohibited
        | ManualMapCategory::Restricted
        | ManualMapCategory::Danger => egui::Color32::from_rgb(185, 47, 47),
        ManualMapCategory::Glider => egui::Color32::from_rgb(38, 128, 83),
        ManualMapCategory::Para => egui::Color32::from_rgb(48, 108, 184),
        ManualMapCategory::Other => egui::Color32::from_rgb(104, 116, 132),
        ManualMapCategory::Ctr | ManualMapCategory::Cfz | ManualMapCategory::Tma => {
            egui::Color32::from_rgb(104, 116, 132)
        }
    }
}

fn text_number_color(color: TextNumberColor) -> egui::Color32 {
    match color {
        TextNumberColor::Red => egui::Color32::from_rgb(245, 82, 82),
        TextNumberColor::Green => egui::Color32::from_rgb(86, 196, 118),
        TextNumberColor::Blue => egui::Color32::from_rgb(92, 160, 255),
        TextNumberColor::Yellow => egui::Color32::from_rgb(238, 205, 72),
        TextNumberColor::White => egui::Color32::WHITE,
    }
}

fn center_map_on_static_map(map_memory: &mut walkers::MapMemory, map: &StaticMap) {
    center_map_on_preview(map_memory, &map.preview, 8.0);
}

fn center_map_on_preview(
    map_memory: &mut walkers::MapMemory,
    preview: &PreviewGeometry,
    zoom: f64,
) {
    if let Some(bbox) = preview.bbox {
        let center = bbox.center();
        map_memory.center_at(walkers::lon_lat(center.lon, center.lat));
        let _ = map_memory.set_zoom(zoom);
    }
}

fn polygon_point_lat_id(i: usize) -> egui::Id {
    egui::Id::new(("polygon-point-lat", i))
}
fn polygon_point_lon_id(i: usize) -> egui::Id {
    egui::Id::new(("polygon-point-lon", i))
}
fn polygon_arc_center_lat_id(i: usize) -> egui::Id {
    egui::Id::new(("polygon-arc-center-lat", i))
}
fn polygon_arc_center_lon_id(i: usize) -> egui::Id {
    egui::Id::new(("polygon-arc-center-lon", i))
}
fn polygon_arc_radius_id(i: usize) -> egui::Id {
    egui::Id::new(("polygon-arc-radius", i))
}
fn polygon_label_lat_id() -> egui::Id {
    egui::Id::new("polygon-label-lat")
}
fn polygon_label_lon_id() -> egui::Id {
    egui::Id::new("polygon-label-lon")
}
fn para_lat_id() -> egui::Id {
    egui::Id::new("para-lat")
}
fn para_lon_id() -> egui::Id {
    egui::Id::new("para-lon")
}
fn text_number_lat_id() -> egui::Id {
    egui::Id::new("text-number-lat")
}
fn text_number_lon_id() -> egui::Id {
    egui::Id::new("text-number-lon")
}
fn text_number_text_id() -> egui::Id {
    egui::Id::new("text-number-text")
}
fn pie_center_lat_id() -> egui::Id {
    egui::Id::new("pie-center-lat")
}
fn pie_center_lon_id() -> egui::Id {
    egui::Id::new("pie-center-lon")
}
fn pie_radius_id() -> egui::Id {
    egui::Id::new("pie-radius")
}
fn pie_label_lat_id() -> egui::Id {
    egui::Id::new("pie-label-lat")
}
fn pie_label_lon_id() -> egui::Id {
    egui::Id::new("pie-label-lon")
}
fn strip_point1_lat_id() -> egui::Id {
    egui::Id::new("strip-point1-lat")
}
fn strip_point1_lon_id() -> egui::Id {
    egui::Id::new("strip-point1-lon")
}
fn strip_point2_lat_id() -> egui::Id {
    egui::Id::new("strip-point2-lat")
}
fn strip_point2_lon_id() -> egui::Id {
    egui::Id::new("strip-point2-lon")
}
fn strip_width_id() -> egui::Id {
    egui::Id::new("strip-width")
}
fn strip_label_lat_id() -> egui::Id {
    egui::Id::new("strip-label-lat")
}
fn strip_label_lon_id() -> egui::Id {
    egui::Id::new("strip-label-lon")
}

fn click_target_from_id(id: egui::Id, polygon: &PolygonDraftState) -> Option<ClickTarget> {
    if id == polygon_label_lat_id() || id == polygon_label_lon_id() {
        return Some(ClickTarget::PolygonLabel);
    }
    for i in 0..polygon.nodes.len() {
        match polygon.nodes[i] {
            PolygonNodeDraft::Point(_) => {
                if id == polygon_point_lat_id(i) || id == polygon_point_lon_id(i) {
                    return Some(ClickTarget::PolygonPoint(i));
                }
            }
            PolygonNodeDraft::Arc(_) => {
                if id == polygon_arc_center_lat_id(i) || id == polygon_arc_center_lon_id(i) {
                    return Some(ClickTarget::PolygonArcCenter(i));
                }
                if id == polygon_arc_radius_id(i) {
                    return Some(ClickTarget::PolygonArcRadius(i));
                }
            }
        }
    }
    if id == para_lat_id() || id == para_lon_id() {
        return Some(ClickTarget::ParaSymbolPoint);
    }
    if id == text_number_lat_id() || id == text_number_lon_id() {
        return Some(ClickTarget::TextNumberPoint);
    }
    if id == pie_center_lat_id() || id == pie_center_lon_id() {
        return Some(ClickTarget::PieCenter);
    }
    if id == pie_radius_id() {
        return Some(ClickTarget::PieRadius);
    }
    if id == pie_label_lat_id() || id == pie_label_lon_id() {
        return Some(ClickTarget::PieLabel);
    }
    if id == strip_point1_lat_id() || id == strip_point1_lon_id() {
        return Some(ClickTarget::StripPoint1);
    }
    if id == strip_point2_lat_id() || id == strip_point2_lon_id() {
        return Some(ClickTarget::StripPoint2);
    }
    if id == strip_width_id() {
        return Some(ClickTarget::StripWidth);
    }
    if id == strip_label_lat_id() || id == strip_label_lon_id() {
        return Some(ClickTarget::StripLabel);
    }
    None
}

fn click_target_widget_ids(target: ClickTarget) -> Vec<egui::Id> {
    match target {
        ClickTarget::PolygonPoint(i) => vec![polygon_point_lat_id(i), polygon_point_lon_id(i)],
        ClickTarget::PolygonArcCenter(i) => {
            vec![polygon_arc_center_lat_id(i), polygon_arc_center_lon_id(i)]
        }
        ClickTarget::PolygonArcRadius(i) => vec![polygon_arc_radius_id(i)],
        ClickTarget::PolygonLabel => vec![polygon_label_lat_id(), polygon_label_lon_id()],
        ClickTarget::ParaSymbolPoint => vec![para_lat_id(), para_lon_id()],
        ClickTarget::TextNumberPoint => vec![text_number_lat_id(), text_number_lon_id()],
        ClickTarget::PieCenter => vec![pie_center_lat_id(), pie_center_lon_id()],
        ClickTarget::PieRadius => vec![pie_radius_id()],
        ClickTarget::PieLabel => vec![pie_label_lat_id(), pie_label_lon_id()],
        ClickTarget::StripPoint1 => vec![strip_point1_lat_id(), strip_point1_lon_id()],
        ClickTarget::StripPoint2 => vec![strip_point2_lat_id(), strip_point2_lon_id()],
        ClickTarget::StripWidth => vec![strip_width_id()],
        ClickTarget::StripLabel => vec![strip_label_lat_id(), strip_label_lon_id()],
    }
}

impl DamApp {
    fn update_click_target_from_memory(&mut self, ctx: &egui::Context) {
        let focused_id = ctx.memory(|m| m.focused());
        match focused_id.and_then(|id| click_target_from_id(id, &self.form.manual.polygon)) {
            Some(target) => self.pending_click_target = Some(target),
            None => {
                if focused_id.is_some() {
                    self.pending_click_target = None;
                }
            }
        }
    }

    fn surrender_click_target_focus(&self, ctx: &egui::Context, target: ClickTarget) {
        let ids = click_target_widget_ids(target);
        ctx.memory_mut(|m| {
            for id in ids {
                m.surrender_focus(id);
            }
        });
    }

    fn maybe_auto_focus_on_geometry_change(&mut self, ctx: &egui::Context) {
        let active = if self.form.map_mode == MapMode::Manual {
            Some(self.form.manual.geometry_type)
        } else {
            None
        };
        if active == self.previous_active_geometry {
            return;
        }
        self.previous_active_geometry = active;
        let Some(geom) = active else {
            return;
        };
        let target = match geom {
            ManualGeometryType::Polygon => {
                if self.form.manual.polygon.nodes.is_empty() {
                    self.form
                        .manual
                        .polygon
                        .nodes
                        .push(PolygonNodeDraft::Point(CoordinateFieldState::default()));
                }
                ClickTarget::PolygonPoint(0)
            }
            ManualGeometryType::ParaSymbol => ClickTarget::ParaSymbolPoint,
            ManualGeometryType::TextNumber => ClickTarget::TextNumberPoint,
            ManualGeometryType::PieCircle => ClickTarget::PieCenter,
            ManualGeometryType::Strip => ClickTarget::StripPoint1,
        };
        if let Some(first_id) = click_target_widget_ids(target).into_iter().next() {
            ctx.memory_mut(|m| m.request_focus(first_id));
        }
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        let margin = self.frost_theme.spacing.md;
        egui::Area::new(egui::Id::new("dam_action_toolbar"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-margin, margin))
            .show(ctx, |ui| {
                let response = top_toolbar_with_id(
                    ui,
                    &self.frost_theme,
                    "dam-action-toolbar",
                    &[
                        ToolbarAction {
                            icon: ICON_PLANE,
                            tooltip: "Send",
                            selected: false,
                            disabled: false,
                        },
                        ToolbarAction {
                            icon: ICON_GLOBE,
                            tooltip: "Download AIXM",
                            selected: false,
                            disabled: false,
                        },
                        ToolbarAction {
                            icon: ICON_BOOK_OPEN,
                            tooltip: "Download JSON",
                            selected: false,
                            disabled: false,
                        },
                        ToolbarAction {
                            icon: ICON_CIRCLE_X,
                            tooltip: "Reset",
                            selected: false,
                            disabled: false,
                        },
                    ],
                );

                match response.icon_clicked {
                    Some(0) => self.send(),
                    Some(1) => self.download_aixm(),
                    Some(2) => self.download_json(),
                    Some(3) => self.show_reset_confirm = true,
                    _ => {}
                }
            });
    }

    fn form_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.form_section(ui, "Map", |this, ui| this.map_section(ui));
                self.form_section(ui, "Validity", |this, ui| this.date_section(ui));
                self.form_section(ui, "Today/Repetitive Periods", |this, ui| {
                    this.periods_section(ui);
                });
                self.form_section(ui, "Corrections & Buffers", |this, ui| {
                    this.corrections_section(ui);
                });
                self.form_section(ui, "Distribution", |this, ui| {
                    this.distribution_section(ui);
                });
                self.form_section(ui, "Additional Information", |this, ui| {
                    this.text_section(ui);
                });
                if !self.submission_status.is_idle() {
                    self.form_section(ui, "Status", |this, ui| this.validation_section(ui));
                }
                self.form_section(ui, "Diagnostics", |this, ui| this.diagnostics_section(ui));
            });
    }

    fn form_section(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        add_contents: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        let theme = self.frost_theme.clone();
        egui::Frame::new()
            .fill(theme.palette.surface_blur)
            .stroke(egui::Stroke::new(1.0, theme.palette.border))
            .corner_radius(egui::CornerRadius::same(theme.radius.lg))
            .inner_margin(egui::Margin::same(theme.spacing.md as i8))
            .show(ui, |ui| {
                section_heading(ui, &theme, title);
                add_contents(self, ui);
            });
        ui.add_space(theme.spacing.sm);
    }

    fn inset_panel(ui: &mut egui::Ui, theme: &Theme, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(theme.palette.surface_blur)
            .stroke(egui::Stroke::new(1.0, theme.palette.border))
            .corner_radius(egui::CornerRadius::same(theme.radius.md))
            .inner_margin(egui::Margin::same(theme.spacing.sm as i8))
            .show(ui, add_contents);
    }

    fn period_panel(ui: &mut egui::Ui, theme: &Theme, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(theme.palette.surface_blur)
            .stroke(egui::Stroke::new(
                1.0,
                mix(theme.palette.border, theme.palette.ring, 0.35),
            ))
            .corner_radius(egui::CornerRadius::same(theme.radius.lg))
            .inner_margin(egui::Margin::same(theme.spacing.md as i8))
            .show(ui, add_contents);
    }

    fn selected_map_summary(&self, ui: &mut egui::Ui) {
        Self::inset_panel(ui, &self.frost_theme, |ui| {
            match self.form.selected_map(&self.catalog) {
                Some(map) => {
                    egui::Grid::new("selected_map_summary")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("ID");
                            ui.monospace(&map.id);
                            ui.end_row();
                            ui.label("Name");
                            ui.strong(&map.name);
                            ui.end_row();
                            if let Some(description) = &map.description {
                                ui.label("Description");
                                ui.label(description);
                                ui.end_row();
                            }
                        });
                }
                None => {
                    ui.colored_label(self.frost_theme.palette.ring, "No DAM map selected.");
                }
            }
        });
    }

    fn map_section(&mut self, ui: &mut egui::Ui) {
        let modes = [MapMode::Predefined, MapMode::Manual];
        let labels = modes.map(MapMode::label);
        let mut selected = modes
            .iter()
            .position(|mode| *mode == self.form.map_mode)
            .unwrap_or(0);
        tabs_with_id(
            ui,
            &self.frost_theme,
            "map-mode-tabs",
            &mut selected,
            &labels,
        );
        self.form.map_mode = modes[selected];

        ui.add_space(8.0);

        match self.form.map_mode {
            MapMode::Predefined => self.predefined_map_section(ui),
            MapMode::Manual => self.manual_map_section(ui),
        }
    }

    fn predefined_map_section(&mut self, ui: &mut egui::Ui) {
        if self.catalog.maps.is_empty() {
            ui.colored_label(
                self.frost_theme.palette.destructive,
                "No bundled GeoJSON maps were found in assets/maps.",
            );
            return;
        }

        ui.horizontal(|ui| {
            ui.strong("Search static maps");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} available", self.catalog.maps.len()));
            });
        });
        themed_text_edit(
            ui,
            &self.frost_theme,
            egui::TextEdit::singleline(&mut self.form.map_search)
                .hint_text("Search by id, name, or description")
                .desired_width(f32::INFINITY),
            ControlSize::Md,
        );

        let filter = self.form.map_search.to_lowercase();
        let filtered: Vec<&StaticMap> = self
            .catalog
            .maps
            .iter()
            .filter(|map| {
                filter.is_empty()
                    || map.id.to_lowercase().contains(&filter)
                    || map.name.to_lowercase().contains(&filter)
                    || map
                        .description
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&filter)
            })
            .collect();

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.strong("Results");
            ui.label(format!("{} match(es)", filtered.len()));
        });

        let theme = self.frost_theme.clone();
        let mut selected_after = None;
        Self::inset_panel(ui, &theme, |ui| {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.label("No matching maps.");
                    }

                    for map in &filtered {
                        let selected =
                            self.form.selected_map_id.as_deref() == Some(map.id.as_str());
                        let fill = if selected {
                            mix(theme.palette.surface_blur, theme.palette.ring, 0.18)
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let stroke = if selected {
                            egui::Stroke::new(1.0, theme.palette.ring)
                        } else {
                            egui::Stroke::new(1.0, egui::Color32::TRANSPARENT)
                        };
                        let label = if selected {
                            egui::RichText::new(map.label())
                                .strong()
                                .color(theme.palette.ring)
                        } else {
                            egui::RichText::new(map.label()).color(theme.palette.foreground)
                        };

                        let item = egui::Frame::new()
                            .fill(fill)
                            .stroke(stroke)
                            .corner_radius(egui::CornerRadius::same(theme.radius.md))
                            .inner_margin(egui::Margin::same(theme.spacing.sm as i8))
                            .show(ui, |ui| {
                                ui.label(label);
                                if let Some(description) = &map.description {
                                    ui.small(description);
                                }
                            });
                        let item_response = ui
                            .interact(
                                item.response.rect,
                                ui.make_persistent_id(("static-map-result", map.id.as_str())),
                                egui::Sense::click(),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if item_response.clicked() {
                            selected_after = Some(map.id.clone());
                        }
                        ui.add_space(4.0);
                    }
                });
        });

        if let Some(id) = selected_after {
            self.form.selected_map_id = Some(id);
            if let Some(map) = self.form.selected_map(&self.catalog) {
                center_map_on_static_map(&mut self.map_memory, map);
                let defaults = map.defaults.clone();
                self.form.apply_defaults(&defaults);
            }
        }

        ui.add_space(6.0);
        ui.strong("Selected DAM map");
        self.selected_map_summary(ui);
    }

    fn manual_map_section(&mut self, ui: &mut egui::Ui) {
        let theme = self.frost_theme.clone();
        ui.label("Map name");
        themed_text_edit(
            ui,
            &theme,
            egui::TextEdit::singleline(&mut self.form.manual.name).desired_width(f32::INFINITY),
            ControlSize::Md,
        );

        ui.add_space(theme.spacing.sm);
        ui.label("Geometry type");
        let labels = ManualGeometryType::ALL.map(ManualGeometryType::label);
        let mut selected = ManualGeometryType::ALL
            .iter()
            .position(|geometry_type| *geometry_type == self.form.manual.geometry_type)
            .unwrap_or(0);
        tabs_with_id(
            ui,
            &theme,
            "manual-geometry-type-tabs",
            &mut selected,
            &labels,
        );
        self.form.manual.geometry_type = ManualGeometryType::ALL[selected];

        ui.add_space(theme.spacing.sm);
        Self::inset_panel(ui, &theme, |ui| match self.form.manual.geometry_type {
            ManualGeometryType::Polygon => {
                manual_polygon_ui(ui, &theme, &mut self.form.manual.polygon);
            }
            ManualGeometryType::ParaSymbol => {
                ui.strong("Para symbol point");
                coordinate_field_ui_with_ids(
                    ui,
                    &theme,
                    "Position",
                    &mut self.form.manual.para_symbol.point,
                    para_lat_id(),
                    para_lon_id(),
                );
            }
            ManualGeometryType::TextNumber => {
                manual_text_number_ui(ui, &theme, &mut self.form.manual.text_number);
            }
            ManualGeometryType::PieCircle => {
                manual_pie_circle_ui(ui, &theme, &mut self.form.manual.pie_circle);
            }
            ManualGeometryType::Strip => {
                manual_strip_ui(ui, &theme, &mut self.form.manual.strip);
            }
        });

        let geometry_type = self.form.manual.geometry_type;
        if geometry_supports_buffer(geometry_type) {
            ui.add_space(theme.spacing.sm);
            ui.strong("Map attributes");
            Self::inset_panel(ui, &theme, |ui| {
                ui.label("Map category");
                if !MANUAL_ATTRIBUTE_CATEGORIES.contains(&self.form.manual.attributes.category) {
                    self.form.manual.attributes.category = ManualMapCategory::Other;
                }
                let category_labels = MANUAL_ATTRIBUTE_CATEGORIES.map(ManualMapCategory::label);
                let category_colors = MANUAL_ATTRIBUTE_CATEGORIES.map(manual_category_color);
                let mut selected_category = MANUAL_ATTRIBUTE_CATEGORIES
                    .iter()
                    .position(|category| *category == self.form.manual.attributes.category)
                    .unwrap_or(0);
                if colored_segmented(
                    ui,
                    &theme,
                    &category_labels,
                    &category_colors,
                    &mut selected_category,
                )
                .changed()
                {
                    self.form.manual.attributes.category =
                        MANUAL_ATTRIBUTE_CATEGORIES[selected_category];
                }

                ui.label("Rendering");
                let rendering_labels = ManualMapRendering::ALL.map(ManualMapRendering::label);
                let mut selected_rendering = ManualMapRendering::ALL
                    .iter()
                    .position(|rendering| *rendering == self.form.manual.attributes.rendering)
                    .unwrap_or(0);
                if segmented(ui, &theme, &rendering_labels, &mut selected_rendering).changed() {
                    self.form.manual.attributes.rendering =
                        ManualMapRendering::ALL[selected_rendering];
                }

                ui.label("Lateral buffer (NM)");
                themed_text_edit(
                    ui,
                    &theme,
                    egui::TextEdit::singleline(&mut self.form.manual.attributes.lateral_buffer_nm)
                        .desired_width(96.0),
                    ControlSize::Md,
                );
            });
        }
    }

    fn date_section(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.date_field_ui(ui, DateField::Start);
            self.date_field_ui(ui, DateField::End);
        });

        if self.form.is_repetitive_range() {
            ui.label("Active weekdays");
            ui.horizontal_wrapped(|ui| {
                for weekday in Weekday::ALL {
                    let possible = self.form.possible_weekdays.contains(&weekday);
                    let mut selected = self.form.active_weekdays.contains(&weekday);
                    let response = ui
                        .add_enabled_ui(possible, |ui| {
                            frost_checkbox(
                                ui,
                                &self.frost_theme,
                                &mut selected,
                                weekday.to_string(),
                            )
                        })
                        .inner;
                    if response.changed() {
                        if selected {
                            self.form.active_weekdays.insert(weekday);
                        } else {
                            self.form.active_weekdays.remove(&weekday);
                        }
                    }
                }
            });
        }
    }

    fn date_field_ui(&mut self, ui: &mut egui::Ui, field: DateField) {
        let active = self.active_date_picker == Some(field);
        let mut open_picker = false;

        ui.vertical(|ui| {
            ui.label(field.label());
            ui.horizontal(|ui| {
                match field {
                    DateField::Start => {
                        themed_text_edit(
                            ui,
                            &self.frost_theme,
                            egui::TextEdit::singleline(&mut self.form.start_date)
                                .desired_width(112.0),
                            ControlSize::Md,
                        );
                    }
                    DateField::End => {
                        themed_text_edit(
                            ui,
                            &self.frost_theme,
                            egui::TextEdit::singleline(&mut self.form.end_date)
                                .desired_width(112.0),
                            ControlSize::Md,
                        );
                    }
                }

                let variant = if active {
                    ControlVariant::Primary
                } else {
                    ControlVariant::Secondary
                };
                open_picker = ui
                    .frost_button(&self.frost_theme, "Pick", variant, ControlSize::Sm)
                    .on_hover_text("Open calendar")
                    .clicked();
            });
        });

        if open_picker {
            self.open_date_picker(field);
        }
    }

    fn open_date_picker(&mut self, field: DateField) {
        let date = self.form_date(field).unwrap_or_else(current_date);
        self.date_picker_month = first_day_of_month(date);
        self.active_date_picker = Some(field);
    }

    fn form_date(&self, field: DateField) -> Option<NaiveDate> {
        let value = match field {
            DateField::Start => &self.form.start_date,
            DateField::End => &self.form.end_date,
        };
        parse_date_text(value)
    }

    fn set_form_date(&mut self, field: DateField, date: NaiveDate) {
        let value = date.format("%Y-%m-%d").to_string();
        match field {
            DateField::Start => self.form.start_date = value,
            DateField::End => self.form.end_date = value,
        }
        self.form.sync_weekdays_from_dates();
    }

    fn periods_section(&mut self, ui: &mut egui::Ui) {
        let theme = self.frost_theme.clone();
        frost_checkbox(ui, &theme, &mut self.form.display_levels, "Display levels");
        ui.label(format!(
            "{} / {MAX_PERIODS} activation period(s)",
            self.form.periods.len()
        ));

        let mut remove_index = None;
        let mut add_after = None;

        for index in 0..self.form.periods.len() {
            Self::period_panel(ui, &theme, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(format!("Period {}", index + 1));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let add_enabled = self.form.periods.len() < MAX_PERIODS;
                        if ui
                            .add_enabled(
                                add_enabled,
                                egui::Button::new("+").min_size([28.0, 28.0].into()),
                            )
                            .clicked()
                        {
                            add_after = Some(index);
                        }
                        let mut remove_clicked = false;
                        ui.add_enabled_ui(self.form.periods.len() > 1, |ui| {
                            remove_clicked =
                                ui.add_sized([28.0, 28.0], egui::Button::new("-")).clicked();
                        });
                        if remove_clicked {
                            remove_index = Some(index);
                        }
                    });
                });

                period_row_ui(ui, &theme, index, &mut self.form.periods[index]);
            });
            ui.add_space(theme.spacing.sm);
        }

        if let Some(index) = add_after {
            self.form
                .periods
                .insert(index + 1, PeriodRowState::default());
            self.selected_period = index + 1;
        }

        if let Some(index) = remove_index {
            self.form.periods.remove(index);
            self.selected_period = self.selected_period.min(self.form.periods.len() - 1);
        }
    }

    fn corrections_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Altitude correction");
        let altitude_options = [
            AltitudeCorrection::None,
            AltitudeCorrection::QnhCorr,
            AltitudeCorrection::FlCorr,
        ];
        let altitude_labels = ["None", "QNH Corr", "FL Corr"];
        let mut selected_altitude = altitude_options
            .iter()
            .position(|option| *option == self.form.altitude_correction)
            .unwrap_or(0);
        if segmented(
            ui,
            &self.frost_theme,
            &altitude_labels,
            &mut selected_altitude,
        )
        .changed()
        {
            self.form.altitude_correction = altitude_options[selected_altitude];
        }

        ui.label("Upper buffer");
        let buffer_options = [
            BufferFilter::Default,
            BufferFilter::Half,
            BufferFilter::NoBuffer,
        ];
        let upper_buffer_labels = ["Default", "UL half", "UL no buffer"];
        let mut selected_upper_buffer = buffer_options
            .iter()
            .position(|option| *option == self.form.upper_buffer)
            .unwrap_or(0);
        if segmented(
            ui,
            &self.frost_theme,
            &upper_buffer_labels,
            &mut selected_upper_buffer,
        )
        .changed()
        {
            self.form.upper_buffer = buffer_options[selected_upper_buffer];
        }

        ui.label("Lower buffer");
        let lower_buffer_labels = ["Default", "LL half", "LL no buffer"];
        let mut selected_lower_buffer = buffer_options
            .iter()
            .position(|option| *option == self.form.lower_buffer)
            .unwrap_or(0);
        if segmented(
            ui,
            &self.frost_theme,
            &lower_buffer_labels,
            &mut selected_lower_buffer,
        )
        .changed()
        {
            self.form.lower_buffer = buffer_options[selected_lower_buffer];
        }
    }

    fn distribution_section(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .frost_button(
                    &self.frost_theme,
                    "Unit / Sector",
                    ControlVariant::Secondary,
                    ControlSize::Md,
                )
                .clicked()
            {
                self.show_distribution = true;
            }
            ui.label(format!(
                "{} sector(s) selected",
                self.form.distribution.sectors.len()
            ));
        });
    }

    fn text_section(&mut self, ui: &mut egui::Ui) {
        frost_checkbox(
            ui,
            &self.frost_theme,
            &mut self.form.display_text,
            "Display Text",
        );
        ui.label(format!("Text ({} / 250)", self.form.text.chars().count()));
        themed_text_edit(
            ui,
            &self.frost_theme,
            egui::TextEdit::multiline(&mut self.form.text)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
            ControlSize::Md,
        );
        ui.label("DABS Info");
        let mut dabs_info = String::new();
        themed_text_edit_enabled(
            ui,
            &self.frost_theme,
            false,
            egui::TextEdit::multiline(&mut dabs_info)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
            ControlSize::Md,
        );
    }

    fn validation_section(&mut self, ui: &mut egui::Ui) {
        match &self.submission_status {
            SubmissionStatus::Idle => {}
            SubmissionStatus::Invalid(issues) => {
                ui.colored_label(
                    self.frost_theme.palette.destructive,
                    "Blocked by validation errors.",
                );
                render_validation_issues(ui, &self.frost_theme, issues);
            }
            SubmissionStatus::Building => {
                ui.label("Building submission payload...");
            }
            SubmissionStatus::Ready { message } => {
                ui.label(message);
            }
            SubmissionStatus::Submitting => {
                ui.label("Submitting payload...");
            }
            SubmissionStatus::Sent { message } => {
                ui.label(message);
            }
            SubmissionStatus::Failed { message } => {
                ui.colored_label(self.frost_theme.palette.destructive, message);
            }
        }
    }

    fn diagnostics_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Developer diagnostics")
            .default_open(self.diagnostics_open)
            .show(ui, |ui| {
                diagnostics(ui, &self.catalog.diagnostics);
                ui.separator();
                ui.label("Tiles: online CARTO Dark Matter raster tiles.");
                ui.label(format!("Build: {}", env!("CARGO_PKG_VERSION")));
            });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        let selected_map = if self.form.map_mode == MapMode::Predefined {
            self.form.selected_map(&self.catalog)
        } else {
            None
        };
        let manual_map = if self.form.map_mode == MapMode::Manual {
            Some(self.form.manual.preview_manual_map())
        } else {
            None
        };

        let selected_paths = selected_map
            .map(|map| map.preview.paths.clone())
            .unwrap_or_default();
        let center = selected_map
            .and_then(|map| map.preview.bbox)
            .or(self.default_preview.bbox)
            .map(|bbox| bbox.center())
            .map(|center| walkers::lon_lat(center.lon, center.lat))
            .unwrap_or_else(|| walkers::lon_lat(8.22, 46.8));
        let level_label = if self.form.display_levels {
            let label_text = self.current_level_label();
            manual_map
                .as_ref()
                .and_then(|map| map.label_position)
                .or_else(|| selected_map.and_then(|map| map.defaults.label_coordinate))
                .or_else(|| {
                    selected_map
                        .and_then(|map| map.preview.bbox)
                        .map(|bbox| bbox.center())
                })
                .map(|coordinate| (coordinate, label_text))
        } else {
            None
        };

        let manual_mode = self.form.map_mode == MapMode::Manual;
        let click_target = self.pending_click_target;
        let level_label_text = self.current_level_label();
        let next_click = if manual_mode {
            click_target.map(|target| self.form.manual.next_click_info(target, &level_label_text))
        } else {
            None
        };

        let cursor_preview = if manual_mode {
            click_target.map(|target| (self.form.manual.clone(), target))
        } else {
            None
        };

        let overlay = PreviewOverlay::new(
            self.default_preview.paths.clone(),
            selected_paths,
            manual_map,
            level_label,
            next_click,
            cursor_preview,
            Some(level_label_text.clone()),
            self.form.display_levels,
        );
        let mut clicked_coordinate = None;
        walkers::Map::new(Some(&mut self.map_tiles), &mut self.map_memory, center)
            .zoom_with_ctrl(false)
            .double_click_to_zoom(true)
            .with_plugin(overlay)
            .show(ui, |_ui, response, projector, _memory| {
                if manual_mode
                    && click_target.is_some()
                    && response.clicked_by(egui::PointerButton::Primary)
                    && let Some(pointer_position) = response.interact_pointer_pos()
                {
                    let position = projector.unproject(pointer_position.to_vec2());
                    clicked_coordinate = Some(Coordinate {
                        lon: position.x(),
                        lat: position.y(),
                    });
                }
            });

        if let (Some(coord), Some(target)) = (clicked_coordinate, click_target) {
            self.form.manual.apply_click_target(target, coord);
            match self.form.manual.next_click_target_after(target) {
                Some(next_target) => {
                    self.pending_click_target = Some(next_target);
                    if let Some(first_id) = click_target_widget_ids(next_target).into_iter().next()
                    {
                        ui.ctx().memory_mut(|m| m.request_focus(first_id));
                    }
                }
                None => {
                    self.pending_click_target = None;
                    self.surrender_click_target_focus(ui.ctx(), target);
                    if target == ClickTarget::TextNumberPoint {
                        ui.ctx()
                            .memory_mut(|m| m.request_focus(text_number_text_id()));
                    }
                }
            }
        }
    }

    fn current_level_label(&self) -> String {
        self.form
            .periods
            .get(self.selected_period)
            .map(|period| {
                format!(
                    "{}/{}",
                    period.lower.value.trim(),
                    period.upper.value.trim()
                )
            })
            .unwrap_or_else(|| "000/999".to_owned())
    }

    fn date_picker_window(&mut self, ctx: &egui::Context) {
        let Some(field) = self.active_date_picker else {
            return;
        };

        let mut open = true;
        let mut picked = None;
        let selected_date = self.form_date(field);
        let theme = self.frost_theme.clone();
        egui::Window::new(format!("Select {}", field.label()))
            .id(egui::Id::new(("date_picker", field.id())))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                date_picker_contents(
                    ui,
                    &theme,
                    &mut self.date_picker_month,
                    selected_date,
                    &mut picked,
                );
            });

        if let Some(date) = picked {
            self.set_form_date(field, date);
            self.active_date_picker = None;
        } else if !open {
            self.active_date_picker = None;
        }
    }

    fn distribution_window(&mut self, ctx: &egui::Context) {
        if !self.show_distribution {
            return;
        }

        let mut open = self.show_distribution;
        let content_rect = ctx.content_rect();
        let preferred_card_width: f32 = 456.0;
        let max_card_size = egui::vec2(
            (content_rect.width() - 32.0).max(320.0),
            (content_rect.height() - 32.0).max(280.0),
        );
        self.distribution_card.size = egui::vec2(
            preferred_card_width.min(max_card_size.x),
            self.distribution_card.size.y.min(max_card_size.y),
        );

        egui::Area::new(egui::Id::new("distribution_card_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(content_rect.min)
            .show(ctx, |ui| {
                ui.set_min_size(content_rect.size());
                let response = drag_card(
                    ui,
                    &self.frost_theme,
                    egui::Id::new("distribution_card"),
                    &mut self.distribution_card,
                    "Unit / Sector",
                    |ui| {
                        for region in ["Geneva", "Zurich"] {
                            ui.heading(region);
                            for group in unit_groups().iter().filter(|group| group.region == region)
                            {
                                Self::inset_panel(ui, &self.frost_theme, |ui| {
                                    let mut unit_selected = group.sectors.iter().all(|sector| {
                                        self.form.distribution.sectors.contains(sector.id)
                                    });
                                    let unit_response = frost_checkbox(
                                        ui,
                                        &self.frost_theme,
                                        &mut unit_selected,
                                        group.label,
                                    );
                                    if unit_response.changed() {
                                        for sector in group.sectors {
                                            if unit_selected {
                                                self.form
                                                    .distribution
                                                    .sectors
                                                    .insert(sector.id.to_owned());
                                            } else {
                                                self.form.distribution.sectors.remove(sector.id);
                                            }
                                        }
                                    }

                                    ui.horizontal_wrapped(|ui| {
                                        for sector in group.sectors {
                                            let mut selected =
                                                self.form.distribution.sectors.contains(sector.id);
                                            if frost_checkbox(
                                                ui,
                                                &self.frost_theme,
                                                &mut selected,
                                                sector.label,
                                            )
                                            .changed()
                                            {
                                                if selected {
                                                    self.form
                                                        .distribution
                                                        .sectors
                                                        .insert(sector.id.to_owned());
                                                } else {
                                                    self.form
                                                        .distribution
                                                        .sectors
                                                        .remove(sector.id);
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    },
                );
                if response.closed {
                    open = false;
                }
            });
        self.show_distribution = open;
    }

    fn reset_confirmation(&mut self, ctx: &egui::Context) {
        if !self.show_reset_confirm {
            return;
        }

        let mut open = self.show_reset_confirm;
        let mut reset = false;
        let mut cancel = false;
        egui::Window::new("Reset form")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Reset all current creation inputs?");
                ui.horizontal(|ui| {
                    if ui
                        .frost_button(
                            &self.frost_theme,
                            "Reset",
                            ControlVariant::Destructive,
                            ControlSize::Md,
                        )
                        .clicked()
                    {
                        reset = true;
                    }
                    if ui
                        .frost_button(
                            &self.frost_theme,
                            "Cancel",
                            ControlVariant::Outline,
                            ControlSize::Md,
                        )
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });

        if reset {
            self.form = DamFormState::new(&self.catalog);
            self.submission_status = SubmissionStatus::Idle;
            self.selected_period = 0;
            if let Some(map) = self.form.selected_map(&self.catalog) {
                center_map_on_static_map(&mut self.map_memory, map);
            } else {
                center_map_on_preview(&mut self.map_memory, &self.default_preview, 7.0);
            }
            open = false;
        }
        if cancel {
            open = false;
        }
        self.show_reset_confirm = open;
    }

    fn send(&mut self) {
        self.submission_status = SubmissionStatus::Building;
        let payload = match self.build_aixm_payload_from_form() {
            Ok(payload) => payload,
            Err(status) => {
                self.submission_status = status;
                return;
            }
        };

        self.submission_status = SubmissionStatus::Submitting;
        self.submission_status = match submit_payload(self.submission_endpoint.as_ref(), &payload) {
            SubmissionResult::Sent(message) => SubmissionStatus::Sent { message },
            SubmissionResult::Failed(message) => SubmissionStatus::Failed { message },
        };
    }

    fn download_aixm(&mut self) {
        self.submission_status = SubmissionStatus::Building;
        let payload = match self.build_aixm_payload_from_form() {
            Ok(payload) => payload,
            Err(status) => {
                self.submission_status = status;
                return;
            }
        };

        self.download_payload(payload);
    }

    fn download_json(&mut self) {
        self.submission_status = SubmissionStatus::Building;
        let payload = match self.build_json_payload_from_form() {
            Ok(payload) => payload,
            Err(status) => {
                self.submission_status = status;
                return;
            }
        };

        self.download_payload(payload);
    }

    fn download_payload(&mut self, payload: dam_core::SubmissionPayload) {
        self.submission_status = match platform::download_payload(&payload) {
            Ok(path) => SubmissionStatus::Ready {
                message: format!("Exported {path}"),
            },
            Err(message) => SubmissionStatus::Failed {
                message: format!("Export failed: {message}"),
            },
        };
    }

    fn build_aixm_payload_from_form(
        &self,
    ) -> Result<dam_core::SubmissionPayload, SubmissionStatus> {
        let creation = self
            .form
            .to_creation(&self.catalog)
            .map_err(SubmissionStatus::Invalid)?;

        build_aixm_payload(&creation).map_err(status_from_export_error)
    }

    fn build_json_payload_from_form(
        &self,
    ) -> Result<dam_core::SubmissionPayload, SubmissionStatus> {
        let creation = self
            .form
            .to_creation(&self.catalog)
            .map_err(SubmissionStatus::Invalid)?;

        build_json_payload(&creation).map_err(status_from_export_error)
    }
}

fn render_validation_issues(ui: &mut egui::Ui, theme: &Theme, issues: &[ValidationIssue]) {
    if issues.is_empty() {
        return;
    }

    ui.colored_label(theme.palette.destructive, "Validation issues");
    for issue in issues {
        ui.label(format!("{}: {}", issue.field, issue.message));
    }
}

fn status_from_export_error(error: dam_core::ExportError) -> SubmissionStatus {
    match error {
        dam_core::ExportError::Validation(error) => SubmissionStatus::Invalid(error.issues),
        error => SubmissionStatus::Failed {
            message: format!("Payload build failed: {error}"),
        },
    }
}

fn period_row_ui(ui: &mut egui::Ui, theme: &Theme, index: usize, row: &mut PeriodRowState) {
    ui.horizontal_wrapped(|ui| {
        frost_checkbox(ui, theme, &mut row.start_indication, "Start indication");
        frost_checkbox(ui, theme, &mut row.end_indication, "End indication");
    });

    ui.horizontal(|ui| {
        let start_id = ui.make_persistent_id(format!("period-{index}-start"));
        let end_id = ui.make_persistent_id(format!("period-{index}-end"));
        ui.vertical(|ui| {
            ui.label("Start time");
            let response = themed_text_edit(
                ui,
                theme,
                egui::TextEdit::singleline(&mut row.start_time)
                    .id(start_id)
                    .desired_width(72.0),
                ControlSize::Md,
            );
            if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::N)) {
                row.start_time = current_time_text();
                ui.memory_mut(|memory| memory.request_focus(end_id));
            }
        });
        ui.vertical(|ui| {
            ui.label("End time");
            let response = themed_text_edit(
                ui,
                theme,
                egui::TextEdit::singleline(&mut row.end_time)
                    .id(end_id)
                    .desired_width(72.0),
                ControlSize::Md,
            );
            if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::E)) {
                row.end_time = "23:59".to_owned();
            }
        });
    });

    ui.horizontal(|ui| {
        level_field_ui(ui, theme, "Low level", &mut row.lower);
        level_field_ui(ui, theme, "High level", &mut row.upper);
    });
}

fn level_field_ui(ui: &mut egui::Ui, theme: &Theme, label: &str, level: &mut LevelFieldState) {
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            themed_text_edit(
                ui,
                theme,
                egui::TextEdit::singleline(&mut level.value).desired_width(78.0),
                ControlSize::Md,
            );
            let locked = level.is_forced_feet();
            let effective = level.effective_unit();
            ui.add_enabled_ui(!locked, |ui| {
                let units = [dam_core::LevelUnit::FlightLevel, dam_core::LevelUnit::Feet];
                let labels = units.map(dam_core::LevelUnit::label);
                let mut selected = units
                    .iter()
                    .position(|unit| *unit == effective)
                    .unwrap_or(0);
                if segmented(ui, theme, &labels, &mut selected).changed() {
                    level.explicit_unit = units[selected];
                }
            });
            if locked {
                ui.label("4+ digits -> ft");
            }
        });
    });
}

fn manual_polygon_ui(ui: &mut egui::Ui, theme: &Theme, polygon: &mut PolygonDraftState) {
    ui.horizontal(|ui| {
        ui.strong(format!(
            "Point list ({} / {MAX_POLYGON_POINTS})",
            polygon.nodes.len()
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let add_enabled = polygon.nodes.len() < MAX_POLYGON_POINTS;
            if ui
                .add_enabled(add_enabled, egui::Button::new("Add Arc"))
                .clicked()
            {
                polygon
                    .nodes
                    .push(PolygonNodeDraft::Arc(ArcDraftState::default()));
                let new_index = polygon.nodes.len() - 1;
                ui.memory_mut(|m| m.request_focus(polygon_arc_center_lat_id(new_index)));
            }
            if ui
                .add_enabled(add_enabled, egui::Button::new("Add Point"))
                .clicked()
            {
                polygon
                    .nodes
                    .push(PolygonNodeDraft::Point(CoordinateFieldState::default()));
                let new_index = polygon.nodes.len() - 1;
                ui.memory_mut(|m| m.request_focus(polygon_point_lat_id(new_index)));
            }
        });
    });

    if polygon.nodes.is_empty() {
        ui.colored_label(
            egui::Color32::LIGHT_YELLOW,
            "Use \"Add Point\" or \"Add Arc\", then click the map to place.",
        );
    }

    let mut remove_index = None;
    let mut insert_point_after = None;
    let mut insert_arc_after = None;
    let mut point_seen = 0usize;
    let mut arc_seen = 0usize;

    for index in 0..polygon.nodes.len() {
        ui.separator();
        let is_arc = matches!(polygon.nodes[index], PolygonNodeDraft::Arc(_));
        let row_label = if is_arc {
            arc_seen += 1;
            format!("Center {arc_seen}")
        } else {
            point_seen += 1;
            format!("Point {point_seen}")
        };

        ui.horizontal(|ui| {
            ui.strong(row_label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Remove").clicked() {
                    remove_index = Some(index);
                }
                let insert_enabled = polygon.nodes.len() < MAX_POLYGON_POINTS;
                if ui
                    .add_enabled(insert_enabled, egui::Button::new("Insert Arc"))
                    .clicked()
                {
                    insert_arc_after = Some(index);
                }
                if ui
                    .add_enabled(insert_enabled, egui::Button::new("Insert Point"))
                    .clicked()
                {
                    insert_point_after = Some(index);
                }
            });
        });
        match &mut polygon.nodes[index] {
            PolygonNodeDraft::Point(field) => {
                coordinate_field_ui_with_ids(
                    ui,
                    theme,
                    "",
                    field,
                    polygon_point_lat_id(index),
                    polygon_point_lon_id(index),
                );
            }
            PolygonNodeDraft::Arc(arc) => {
                coordinate_field_ui_with_ids(
                    ui,
                    theme,
                    "Center",
                    &mut arc.center,
                    polygon_arc_center_lat_id(index),
                    polygon_arc_center_lon_id(index),
                );
                ui.label("Radius (NM)");
                numeric_field_ui_with_id(
                    ui,
                    theme,
                    &mut arc.radius_nm,
                    polygon_arc_radius_id(index),
                    96.0,
                );
            }
        }
    }

    if let Some(index) = insert_point_after {
        polygon.nodes.insert(
            index + 1,
            PolygonNodeDraft::Point(CoordinateFieldState::default()),
        );
        ui.memory_mut(|m| m.request_focus(polygon_point_lat_id(index + 1)));
    }
    if let Some(index) = insert_arc_after {
        polygon
            .nodes
            .insert(index + 1, PolygonNodeDraft::Arc(ArcDraftState::default()));
        ui.memory_mut(|m| m.request_focus(polygon_arc_center_lat_id(index + 1)));
    }
    if let Some(index) = remove_index {
        polygon.nodes.remove(index);
    }

    ui.separator();
    ui.label("Label position (optional)");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "",
        &mut polygon.label,
        polygon_label_lat_id(),
        polygon_label_lon_id(),
    );
}

fn manual_text_number_ui(ui: &mut egui::Ui, theme: &Theme, text_number: &mut TextNumberDraftState) {
    ui.strong("Text and number point");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "Position",
        &mut text_number.point,
        text_number_lat_id(),
        text_number_lon_id(),
    );
    ui.label("Text / number");
    themed_text_edit(
        ui,
        theme,
        egui::TextEdit::singleline(&mut text_number.text)
            .id(text_number_text_id())
            .desired_width(f32::INFINITY),
        ControlSize::Md,
    );

    ui.label("Color");
    let colors = TextNumberColor::ALL;
    let color_labels = colors.map(TextNumberColor::label);
    let color_fills = colors.map(text_number_color);
    let mut selected_color = colors
        .iter()
        .position(|color| *color == text_number.color)
        .unwrap_or(0);
    if colored_segmented(ui, theme, &color_labels, &color_fills, &mut selected_color).changed() {
        text_number.color = colors[selected_color];
    }

    ui.label("Size");
    let sizes = TextNumberSize::ALL;
    let size_labels = sizes.map(TextNumberSize::label);
    let mut selected_size = sizes
        .iter()
        .position(|size| *size == text_number.size)
        .unwrap_or(0);
    if segmented(ui, theme, &size_labels, &mut selected_size).changed() {
        text_number.size = sizes[selected_size];
    }
}

fn manual_pie_circle_ui(ui: &mut egui::Ui, theme: &Theme, pie: &mut PieCircleDraftState) {
    ui.strong("Pie / circle");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "Center",
        &mut pie.center,
        pie_center_lat_id(),
        pie_center_lon_id(),
    );
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Radius (NM)");
            numeric_field_ui_with_id(ui, theme, &mut pie.radius_nm, pie_radius_id(), 96.0);
        });
        ui.vertical(|ui| {
            ui.label("First angle");
            themed_text_edit(
                ui,
                theme,
                egui::TextEdit::singleline(&mut pie.first_angle_deg).desired_width(78.0),
                ControlSize::Md,
            );
        });
        ui.vertical(|ui| {
            ui.label("Last angle");
            themed_text_edit(
                ui,
                theme,
                egui::TextEdit::singleline(&mut pie.last_angle_deg).desired_width(78.0),
                ControlSize::Md,
            );
        });
    });

    ui.separator();
    ui.label("Label position (optional)");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "",
        &mut pie.label,
        pie_label_lat_id(),
        pie_label_lon_id(),
    );
}

fn manual_strip_ui(ui: &mut egui::Ui, theme: &Theme, strip: &mut StripDraftState) {
    ui.strong("Strip corridor");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "Point 1",
        &mut strip.point1,
        strip_point1_lat_id(),
        strip_point1_lon_id(),
    );
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "Point 2",
        &mut strip.point2,
        strip_point2_lat_id(),
        strip_point2_lon_id(),
    );
    ui.label("Width (NM)");
    numeric_field_ui_with_id(ui, theme, &mut strip.width_nm, strip_width_id(), 96.0);

    ui.separator();
    ui.label("Label position (optional)");
    coordinate_field_ui_with_ids(
        ui,
        theme,
        "",
        &mut strip.label,
        strip_label_lat_id(),
        strip_label_lon_id(),
    );
}

const FOCUS_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(255, 200, 80);

fn coordinate_field_ui_with_ids(
    ui: &mut egui::Ui,
    theme: &Theme,
    label: &str,
    coordinate: &mut CoordinateFieldState,
    lat_id: egui::Id,
    lon_id: egui::Id,
) {
    if !label.is_empty() {
        ui.label(label);
    }
    let (any_focused, combined_rect) = ui
        .horizontal(|ui| {
            let mut focused = false;
            let mut combined: Option<egui::Rect> = None;
            ui.vertical(|ui| {
                ui.label("Latitude");
                let r = themed_text_edit(
                    ui,
                    theme,
                    egui::TextEdit::singleline(&mut coordinate.lat)
                        .id(lat_id)
                        .desired_width(104.0),
                    ControlSize::Md,
                );
                if r.has_focus() {
                    focused = true;
                }
                combined = Some(r.rect);
            });
            ui.vertical(|ui| {
                ui.label("Longitude");
                let r = themed_text_edit(
                    ui,
                    theme,
                    egui::TextEdit::singleline(&mut coordinate.lon)
                        .id(lon_id)
                        .desired_width(104.0),
                    ControlSize::Md,
                );
                if r.has_focus() {
                    focused = true;
                }
                combined = Some(match combined {
                    Some(prev) => prev.union(r.rect),
                    None => r.rect,
                });
            });
            (focused, combined)
        })
        .inner;
    if let (true, Some(rect)) = (any_focused, combined_rect) {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            4.0,
            egui::Stroke::new(2.0, FOCUS_HIGHLIGHT),
            egui::StrokeKind::Outside,
        );
    }
}

fn numeric_field_ui_with_id(
    ui: &mut egui::Ui,
    theme: &Theme,
    value: &mut String,
    id: egui::Id,
    width: f32,
) {
    let response = themed_text_edit(
        ui,
        theme,
        egui::TextEdit::singleline(value)
            .id(id)
            .desired_width(width),
        ControlSize::Md,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(3.0),
            4.0,
            egui::Stroke::new(2.0, FOCUS_HIGHLIGHT),
            egui::StrokeKind::Outside,
        );
    }
}

fn date_picker_contents(
    ui: &mut egui::Ui,
    theme: &Theme,
    visible_month: &mut NaiveDate,
    selected_date: Option<NaiveDate>,
    picked: &mut Option<NaiveDate>,
) {
    ui.horizontal(|ui| {
        if ui.add_sized([30.0, 28.0], egui::Button::new("<")).clicked() {
            *visible_month = shift_month(*visible_month, -1);
        }
        ui.add_space(8.0);
        ui.strong(visible_month.format("%B %Y").to_string());
        ui.add_space(8.0);
        if ui.add_sized([30.0, 28.0], egui::Button::new(">")).clicked() {
            *visible_month = shift_month(*visible_month, 1);
        }
    });
    ui.add_space(8.0);

    let first = first_day_of_month(*visible_month);
    let leading_empty_days = first.weekday().num_days_from_monday() as usize;
    let days = days_in_month(first.year(), first.month());
    let today = current_date();
    let day_size = egui::vec2(34.0, 28.0);

    egui::Grid::new("date_picker_calendar")
        .num_columns(7)
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            for weekday in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                ui.strong(weekday);
            }
            ui.end_row();

            let mut column = 0;
            for _ in 0..leading_empty_days {
                ui.label("");
                column += 1;
            }

            for day in 1..=days {
                let date = NaiveDate::from_ymd_opt(first.year(), first.month(), day).unwrap();
                let text = if selected_date == Some(date) {
                    egui::RichText::new(day.to_string()).strong()
                } else {
                    egui::RichText::new(day.to_string())
                };
                let mut button = egui::Button::new(text).min_size(day_size);
                if selected_date == Some(date) {
                    button = button.fill(mix(theme.palette.card, theme.palette.ring, 0.32));
                } else if date == today {
                    button = button.stroke(egui::Stroke::new(1.0, theme.palette.ring));
                }

                if ui.add(button).clicked() {
                    *picked = Some(date);
                }

                column += 1;
                if column % 7 == 0 {
                    ui.end_row();
                }
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .frost_button(theme, "Today", ControlVariant::Secondary, ControlSize::Sm)
            .clicked()
        {
            let today = current_date();
            *visible_month = first_day_of_month(today);
            *picked = Some(today);
        }
    });
}

fn shift_month(month: NaiveDate, delta: i32) -> NaiveDate {
    let mut year = month.year();
    let mut month_number = month.month() as i32 + delta;

    while month_number < 1 {
        month_number += 12;
        year -= 1;
    }
    while month_number > 12 {
        month_number -= 12;
        year += 1;
    }

    NaiveDate::from_ymd_opt(year, month_number as u32, 1).unwrap()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    (next_month - Duration::days(1)).day()
}

fn first_day_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
}

fn parse_date_text(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn section_heading(ui: &mut egui::Ui, theme: &Theme, title: &str) {
    ui.horizontal(|ui| {
        let rect = ui
            .allocate_exact_size(egui::vec2(3.0, 18.0), egui::Sense::hover())
            .0;
        ui.painter().rect_filled(rect, 1.5, theme.palette.ring);
        ui.heading(egui::RichText::new(title).color(theme.palette.foreground));
    });
    ui.add_space(theme.spacing.xs);
    ui.separator();
    ui.add_space(theme.spacing.sm);
}

fn diagnostics(ui: &mut egui::Ui, diagnostics: &[CatalogDiagnostic]) {
    if diagnostics.is_empty() {
        ui.label("Catalog diagnostics: none");
    } else {
        ui.colored_label(egui::Color32::YELLOW, "Catalog diagnostics");
        for diagnostic in diagnostics {
            ui.label(format!("{}: {}", diagnostic.path, diagnostic.message));
        }
    }
}

fn current_time_text() -> String {
    Local::now().time().format("%H:%M").to_string()
}

fn parse_date(value: &str, field: &str, issues: &mut Vec<ValidationIssue>) -> Option<NaiveDate> {
    match NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d") {
        Ok(date) => Some(date),
        Err(_) => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Date must use YYYY-MM-DD.".to_owned(),
            });
            None
        }
    }
}

fn parse_time(value: &str, field: &str, issues: &mut Vec<ValidationIssue>) -> Option<NaiveTime> {
    match NaiveTime::parse_from_str(value.trim(), "%H:%M") {
        Ok(time) => Some(time),
        Err(_) => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Time must use HH:MM.".to_owned(),
            });
            None
        }
    }
}

fn parse_level(
    value: &str,
    unit: dam_core::LevelUnit,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) -> Option<dam_core::Level> {
    let value = value.trim();
    if value.is_empty() {
        issues.push(ValidationIssue {
            field: field.to_owned(),
            message: "Level is required.".to_owned(),
        });
        return None;
    }
    if !value.chars().all(|c| c.is_ascii_digit()) {
        issues.push(ValidationIssue {
            field: field.to_owned(),
            message: "Level must contain digits only.".to_owned(),
        });
        return None;
    }

    let parsed = match value.parse::<u32>() {
        Ok(parsed) => parsed,
        Err(_) => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Level is too large.".to_owned(),
            });
            return None;
        }
    };

    if unit == dam_core::LevelUnit::FlightLevel && value.len() >= 4 {
        issues.push(ValidationIssue {
            field: field.to_owned(),
            message: "Four or more digits force feet; select ft or shorten the value.".to_owned(),
        });
    }

    Some(dam_core::Level::new(parsed, unit))
}

fn current_date() -> NaiveDate {
    Local::now().date_naive()
}

fn current_date_text() -> String {
    current_date().format("%Y-%m-%d").to_string()
}
