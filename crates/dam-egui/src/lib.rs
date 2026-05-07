mod form;
mod platform;
mod preview;

use crate::form::{
    CoordinateFieldState, DamFormState, LevelFieldState, ManualGeometryType, MapMode,
    PeriodRowState, PieCircleDraftState, PieClickTarget, PolygonDraftState, StripClickTarget,
    StripDraftState, TextNumberDraftState,
};
use crate::preview::PreviewOverlay;
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime};
use dam_core::{
    AltitudeCorrection, BufferFilter, CatalogDiagnostic, Coordinate, DamCreation, MAX_PERIODS,
    MAX_POLYGON_POINTS, ManualMapCategory, ManualMapRendering, MapCatalog, PreviewGeometry,
    StaticMap, TextNumberColor, TextNumberSize, ValidationIssue, Weekday, bundled_catalog,
    switzerland_border_preview, to_pretty_json, unit_groups,
};

const APP_BG: egui::Color32 = egui::Color32::from_rgb(11, 15, 19);
const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(17, 23, 30);
const SECTION_BG: egui::Color32 = egui::Color32::from_rgb(19, 27, 35);
const SECTION_STROKE: egui::Color32 = egui::Color32::from_rgb(45, 59, 74);
const SUBSECTION_BG: egui::Color32 = egui::Color32::from_rgb(14, 20, 27);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(95, 200, 205);

pub struct DamApp {
    catalog: MapCatalog,
    default_preview: PreviewGeometry,
    form: DamFormState,
    selected_period: usize,
    map_memory: walkers::MapMemory,
    show_distribution: bool,
    show_reset_confirm: bool,
    active_date_picker: Option<DateField>,
    date_picker_month: NaiveDate,
    diagnostics_open: bool,
    validation_issues: Vec<ValidationIssue>,
    status: Option<String>,
}

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
        configure_visuals(&cc.egui_ctx);

        let catalog = bundled_catalog();
        let default_preview = switzerland_border_preview();
        let form = DamFormState::new(&catalog);
        let mut map_memory = walkers::MapMemory::default();
        if let Some(map) = form.selected_map(&catalog) {
            center_map_on_static_map(&mut map_memory, map);
        } else {
            center_map_on_preview(&mut map_memory, &default_preview, 7.0);
        }

        Self {
            catalog,
            default_preview,
            form,
            selected_period: 0,
            map_memory,
            show_distribution: false,
            show_reset_confirm: false,
            active_date_picker: None,
            date_picker_month: first_day_of_month(current_date()),
            diagnostics_open: false,
            validation_issues: Vec::new(),
            status: None,
        }
    }
}

impl eframe::App for DamApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.form.sync_weekdays_from_dates();

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(APP_BG))
            .show_inside(ui, |ui| {
                egui::Panel::top("toolbar")
                    .frame(egui::Frame::new().fill(PANEL_BG))
                    .show_inside(ui, |ui| self.toolbar(ui));

                egui::Panel::left("creation_form")
                    .resizable(true)
                    .default_size(500.0)
                    .size_range(420.0..=760.0)
                    .frame(egui::Frame::new().fill(PANEL_BG).inner_margin(12))
                    .show_inside(ui, |ui| self.form_panel(ui));

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(APP_BG).inner_margin(12))
                    .show_inside(ui, |ui| self.preview_panel(ui));
            });

        self.distribution_window(ui.ctx());
        self.reset_confirmation(ui.ctx());
        self.date_picker_window(ui.ctx());
    }
}

fn configure_visuals(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_global_style(style);
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

impl DamApp {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("DAM Creation Tool");
            ui.separator();
            ui.label("Map creation");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Send").clicked() {
                    self.export();
                }
                if ui.button("Reset").clicked() {
                    self.show_reset_confirm = true;
                }
            });
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
                if self.status.is_some() || !self.validation_issues.is_empty() {
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
        egui::Frame::new()
            .fill(SECTION_BG)
            .stroke(egui::Stroke::new(1.0, SECTION_STROKE))
            .inner_margin(12)
            .show(ui, |ui| {
                section_heading(ui, title);
                add_contents(self, ui);
            });
        ui.add_space(10.0);
    }

    fn inset_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(SUBSECTION_BG)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(31, 43, 55)))
            .inner_margin(10)
            .show(ui, add_contents);
    }

    fn selected_map_summary(&self, ui: &mut egui::Ui) {
        Self::inset_panel(ui, |ui| match self.form.selected_map(&self.catalog) {
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
                ui.colored_label(egui::Color32::LIGHT_YELLOW, "No DAM map selected.");
            }
        });
    }

    fn map_section(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            selectable_enum(
                ui,
                &mut self.form.map_mode,
                MapMode::Predefined,
                MapMode::Predefined.label(),
            );
            selectable_enum(
                ui,
                &mut self.form.map_mode,
                MapMode::Manual,
                MapMode::Manual.label(),
            );
        });

        ui.add_space(8.0);

        match self.form.map_mode {
            MapMode::Predefined => self.predefined_map_section(ui),
            MapMode::Manual => self.manual_map_section(ui),
        }
    }

    fn predefined_map_section(&mut self, ui: &mut egui::Ui) {
        if self.catalog.maps.is_empty() {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
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
        ui.add(
            egui::TextEdit::singleline(&mut self.form.map_search)
                .hint_text("Search by id, name, or description")
                .desired_width(f32::INFINITY),
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

        let mut selected_after = None;
        Self::inset_panel(ui, |ui| {
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
                        let label = if selected {
                            egui::RichText::new(map.label()).strong().color(ACCENT)
                        } else {
                            egui::RichText::new(map.label())
                        };
                        if ui.selectable_label(selected, label).clicked() {
                            selected_after = Some(map.id.clone());
                        }
                        if let Some(description) = &map.description {
                            ui.small(description);
                        }
                        ui.separator();
                    }
                });
        });

        if let Some(id) = selected_after {
            self.form.selected_map_id = Some(id);
            if let Some(map) = self.form.selected_map(&self.catalog) {
                center_map_on_static_map(&mut self.map_memory, map);
            }
        }

        ui.add_space(6.0);
        ui.strong("Selected DAM map");
        self.selected_map_summary(ui);
    }

    fn manual_map_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Map name");
        ui.add(egui::TextEdit::singleline(&mut self.form.manual.name).desired_width(f32::INFINITY));

        ui.add_space(8.0);
        ui.label("Geometry type");
        ui.horizontal_wrapped(|ui| {
            for geometry_type in ManualGeometryType::ALL {
                selectable_enum(
                    ui,
                    &mut self.form.manual.geometry_type,
                    geometry_type,
                    geometry_type.label(),
                );
            }
        });

        ui.add_space(8.0);
        Self::inset_panel(ui, |ui| match self.form.manual.geometry_type {
            ManualGeometryType::Polygon => manual_polygon_ui(ui, &mut self.form.manual.polygon),
            ManualGeometryType::ParaSymbol => {
                ui.strong("Para symbol point");
                coordinate_field_ui(ui, "Position", &mut self.form.manual.para_symbol.point);
            }
            ManualGeometryType::TextNumber => {
                manual_text_number_ui(ui, &mut self.form.manual.text_number);
            }
            ManualGeometryType::PieCircle => {
                manual_pie_circle_ui(ui, &mut self.form.manual.pie_circle);
            }
            ManualGeometryType::Strip => {
                manual_strip_ui(ui, &mut self.form.manual.strip);
            }
        });

        ui.add_space(8.0);
        ui.strong("Map attributes");
        Self::inset_panel(ui, |ui| {
            ui.label("Map category");
            ui.horizontal_wrapped(|ui| {
                for category in ManualMapCategory::ALL {
                    selectable_enum(
                        ui,
                        &mut self.form.manual.attributes.category,
                        category,
                        category.label(),
                    );
                }
            });

            ui.label("Rendering");
            ui.horizontal(|ui| {
                for rendering in ManualMapRendering::ALL {
                    selectable_enum(
                        ui,
                        &mut self.form.manual.attributes.rendering,
                        rendering,
                        rendering.label(),
                    );
                }
            });

            ui.label("Lateral buffer (NM)");
            ui.add(
                egui::TextEdit::singleline(&mut self.form.manual.attributes.lateral_buffer_nm)
                    .desired_width(96.0),
            );
        });
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
                    let response = ui.add_enabled(
                        possible,
                        egui::Checkbox::new(&mut selected, weekday.to_string()),
                    );
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
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.start_date)
                                .desired_width(112.0),
                        );
                    }
                    DateField::End => {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.end_date)
                                .desired_width(112.0),
                        );
                    }
                }

                let button = egui::Button::new("Pick").selected(active);
                open_picker = ui
                    .add_sized([46.0, 24.0], button)
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
        ui.checkbox(&mut self.form.display_levels, "Display levels");
        ui.label(format!(
            "{} / {MAX_PERIODS} activation period(s)",
            self.form.periods.len()
        ));

        let mut remove_index = None;
        let mut add_after = None;

        for index in 0..self.form.periods.len() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let selected = self.selected_period == index;
                    if ui
                        .selectable_label(selected, format!("Period {}", index + 1))
                        .clicked()
                    {
                        self.selected_period = index;
                    }
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

                period_row_ui(ui, index, &mut self.form.periods[index]);
            });
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
        ui.horizontal(|ui| {
            selectable_enum(
                ui,
                &mut self.form.altitude_correction,
                AltitudeCorrection::None,
                "None",
            );
            selectable_enum(
                ui,
                &mut self.form.altitude_correction,
                AltitudeCorrection::QnhCorr,
                "QNH Corr",
            );
            selectable_enum(
                ui,
                &mut self.form.altitude_correction,
                AltitudeCorrection::FlCorr,
                "FL Corr",
            );
        });

        ui.label("Upper buffer");
        ui.horizontal(|ui| {
            selectable_enum(
                ui,
                &mut self.form.upper_buffer,
                BufferFilter::Default,
                "Default",
            );
            selectable_enum(
                ui,
                &mut self.form.upper_buffer,
                BufferFilter::Half,
                "UL half",
            );
            selectable_enum(
                ui,
                &mut self.form.upper_buffer,
                BufferFilter::NoBuffer,
                "UL no buffer",
            );
        });

        ui.label("Lower buffer");
        ui.horizontal(|ui| {
            selectable_enum(
                ui,
                &mut self.form.lower_buffer,
                BufferFilter::Default,
                "Default",
            );
            selectable_enum(
                ui,
                &mut self.form.lower_buffer,
                BufferFilter::Half,
                "LL half",
            );
            selectable_enum(
                ui,
                &mut self.form.lower_buffer,
                BufferFilter::NoBuffer,
                "LL no buffer",
            );
        });
    }

    fn distribution_section(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Unit / Sector").clicked() {
                self.show_distribution = true;
            }
            ui.label(format!(
                "{} sector(s) selected",
                self.form.distribution.sectors.len()
            ));
        });
    }

    fn text_section(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.form.display_text, "Display Text");
        ui.label(format!("Text ({} / 250)", self.form.text.chars().count()));
        ui.add(
            egui::TextEdit::multiline(&mut self.form.text)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        ui.label("DABS Info");
        let mut dabs_info = String::new();
        ui.add_enabled(
            false,
            egui::TextEdit::multiline(&mut dabs_info)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );
    }

    fn validation_section(&mut self, ui: &mut egui::Ui) {
        if let Some(status) = &self.status {
            ui.label(status);
        }

        if self.validation_issues.is_empty() {
            return;
        }

        ui.colored_label(egui::Color32::LIGHT_RED, "Validation issues");
        for issue in &self.validation_issues {
            ui.label(format!("{}: {}", issue.field, issue.message));
        }
    }

    fn diagnostics_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Developer diagnostics")
            .default_open(self.diagnostics_open)
            .show(ui, |ui| {
                diagnostics(ui, &self.catalog.diagnostics);
                ui.separator();
                ui.label("PMTiles: no runtime PMTiles file configured; using dark fallback.");
                ui.label(format!("Build: {}", env!("CARGO_PKG_VERSION")));
            });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Map Preview");
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

        if let Some(map) = selected_map {
            ui.strong(map.label());
            if let Some(description) = &map.description {
                ui.label(description);
            }
        } else if let Some(manual_map) = &manual_map {
            ui.strong(if manual_map.name.trim().is_empty() {
                "Manual DAM map"
            } else {
                manual_map.name.as_str()
            });
            ui.label("Click the preview to edit the active manual geometry.");
        } else {
            ui.strong("Switzerland country border");
            ui.label("No DAM map selected.");
        }

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
                .or_else(|| {
                    selected_map
                        .and_then(|map| map.preview.bbox)
                        .map(|bbox| bbox.center())
                })
                .map(|coordinate| (coordinate, label_text))
        } else {
            None
        };

        let overlay = PreviewOverlay::new(
            self.default_preview.paths.clone(),
            selected_paths,
            manual_map,
            level_label,
        );
        let mut clicked_coordinate = None;
        let manual_mode = self.form.map_mode == MapMode::Manual;
        walkers::Map::new(None, &mut self.map_memory, center)
            .zoom_with_ctrl(false)
            .double_click_to_zoom(true)
            .with_plugin(overlay)
            .show(ui, |_ui, response, projector, _memory| {
                if manual_mode
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

        if let Some(coordinate) = clicked_coordinate
            && !self.form.manual.apply_click(coordinate)
        {
            self.status = Some(format!(
                "Polygon point limit reached ({MAX_POLYGON_POINTS} points)."
            ));
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
        egui::Window::new(format!("Select {}", field.label()))
            .id(egui::Id::new(("date_picker", field.id())))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                date_picker_contents(ui, &mut self.date_picker_month, selected_date, &mut picked);
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
        egui::Window::new("Unit / Sector")
            .open(&mut open)
            .resizable(true)
            .default_size([620.0, 520.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for region in ["Geneva", "Zurich"] {
                        ui.heading(region);
                        for group in unit_groups().iter().filter(|group| group.region == region) {
                            ui.group(|ui| {
                                let mut unit_selected = group.sectors.iter().all(|sector| {
                                    self.form.distribution.sectors.contains(sector.id)
                                });
                                let unit_response = ui.checkbox(&mut unit_selected, group.label);
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
                                        if ui.checkbox(&mut selected, sector.label).changed() {
                                            if selected {
                                                self.form
                                                    .distribution
                                                    .sectors
                                                    .insert(sector.id.to_owned());
                                            } else {
                                                self.form.distribution.sectors.remove(sector.id);
                                            }
                                        }
                                    }
                                });
                            });
                        }
                    }
                });
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
                    if ui.button("Reset").clicked() {
                        reset = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if reset {
            self.form = DamFormState::new(&self.catalog);
            self.validation_issues.clear();
            self.status = None;
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

    fn export(&mut self) {
        self.validation_issues.clear();
        match self.form.to_creation(&self.catalog) {
            Ok(creation) => match export_creation(&creation) {
                Ok(message) => self.status = Some(message),
                Err(ExportFailure::Validation(issues)) => {
                    self.status = Some("Export blocked by validation errors.".to_owned());
                    self.validation_issues = issues;
                }
                Err(ExportFailure::Io(message)) => {
                    self.status = Some(format!("Export failed: {message}"));
                }
            },
            Err(issues) => {
                self.status = Some("Export blocked by validation errors.".to_owned());
                self.validation_issues = issues;
            }
        }
    }
}

fn period_row_ui(ui: &mut egui::Ui, index: usize, row: &mut PeriodRowState) {
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut row.start_indication, "Start indication");
        ui.checkbox(&mut row.end_indication, "End indication");
    });

    ui.horizontal(|ui| {
        let start_id = ui.make_persistent_id(format!("period-{index}-start"));
        let end_id = ui.make_persistent_id(format!("period-{index}-end"));
        ui.vertical(|ui| {
            ui.label("Start time");
            let response = ui.add(
                egui::TextEdit::singleline(&mut row.start_time)
                    .id(start_id)
                    .desired_width(72.0),
            );
            if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::N)) {
                row.start_time = current_time_text();
                ui.memory_mut(|memory| memory.request_focus(end_id));
            }
        });
        ui.vertical(|ui| {
            ui.label("End time");
            let response = ui.add(
                egui::TextEdit::singleline(&mut row.end_time)
                    .id(end_id)
                    .desired_width(72.0),
            );
            if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::E)) {
                row.end_time = "23:59".to_owned();
            }
        });
    });

    ui.horizontal(|ui| {
        level_field_ui(ui, "Low level", &mut row.lower);
        level_field_ui(ui, "High level", &mut row.upper);
    });
}

fn level_field_ui(ui: &mut egui::Ui, label: &str, level: &mut LevelFieldState) {
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut level.value).desired_width(78.0));
            let locked = level.is_forced_feet();
            let mut effective = level.effective_unit();
            ui.add_enabled_ui(!locked, |ui| {
                if ui
                    .selectable_label(effective == dam_core::LevelUnit::FlightLevel, "FL")
                    .clicked()
                {
                    level.explicit_unit = dam_core::LevelUnit::FlightLevel;
                    effective = dam_core::LevelUnit::FlightLevel;
                }
                if ui
                    .selectable_label(effective == dam_core::LevelUnit::Feet, "ft")
                    .clicked()
                {
                    level.explicit_unit = dam_core::LevelUnit::Feet;
                }
            });
            if locked {
                ui.label("4+ digits -> ft");
            }
        });
    });
}

fn manual_polygon_ui(ui: &mut egui::Ui, polygon: &mut PolygonDraftState) {
    ui.horizontal(|ui| {
        ui.strong(format!(
            "Point list ({} / {MAX_POLYGON_POINTS})",
            polygon.points.len()
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(false, egui::Button::new("Add Arc"))
                .on_disabled_hover_text("Arc rows are deferred.");
            let add_enabled = polygon.points.len() < MAX_POLYGON_POINTS;
            if ui
                .add_enabled(add_enabled, egui::Button::new("Add Point"))
                .clicked()
            {
                polygon.points.push(CoordinateFieldState::default());
            }
        });
    });

    if polygon.points.is_empty() {
        ui.colored_label(
            egui::Color32::LIGHT_YELLOW,
            "Click the preview or add point rows.",
        );
    }

    let mut remove_index = None;
    let mut insert_after = None;
    for index in 0..polygon.points.len() {
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong(format!("Point {}", index + 1));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Remove").clicked() {
                    remove_index = Some(index);
                }
                let insert_enabled = polygon.points.len() < MAX_POLYGON_POINTS;
                if ui
                    .add_enabled(insert_enabled, egui::Button::new("Insert after"))
                    .clicked()
                {
                    insert_after = Some(index);
                }
                ui.add_enabled(false, egui::Button::new("Insert Arc"))
                    .on_disabled_hover_text("Arc rows are deferred.");
            });
        });
        coordinate_field_ui(ui, "", &mut polygon.points[index]);
    }

    if let Some(index) = insert_after {
        polygon
            .points
            .insert(index + 1, CoordinateFieldState::default());
    }
    if let Some(index) = remove_index {
        polygon.points.remove(index);
    }
}

fn manual_text_number_ui(ui: &mut egui::Ui, text_number: &mut TextNumberDraftState) {
    ui.strong("Text and number point");
    coordinate_field_ui(ui, "Position", &mut text_number.point);
    ui.label(format!("Text ({} / 25)", text_number.text.chars().count()));
    ui.add(egui::TextEdit::singleline(&mut text_number.text).desired_width(f32::INFINITY));

    ui.label("Color");
    ui.horizontal_wrapped(|ui| {
        for color in TextNumberColor::ALL {
            selectable_enum(ui, &mut text_number.color, color, color.label());
        }
    });

    ui.label("Size");
    ui.horizontal(|ui| {
        for size in TextNumberSize::ALL {
            selectable_enum(ui, &mut text_number.size, size, size.label());
        }
    });
}

fn manual_pie_circle_ui(ui: &mut egui::Ui, pie: &mut PieCircleDraftState) {
    ui.strong("Pie / circle");
    coordinate_field_ui(ui, "Center", &mut pie.center);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Radius (NM)");
            ui.add(egui::TextEdit::singleline(&mut pie.radius_nm).desired_width(96.0));
        });
        ui.vertical(|ui| {
            ui.label("First angle");
            ui.add(egui::TextEdit::singleline(&mut pie.first_angle_deg).desired_width(78.0));
        });
        ui.vertical(|ui| {
            ui.label("Last angle");
            ui.add(egui::TextEdit::singleline(&mut pie.last_angle_deg).desired_width(78.0));
        });
    });

    ui.label("Preview click target");
    ui.horizontal(|ui| {
        for target in [PieClickTarget::Center, PieClickTarget::Radius] {
            selectable_enum(ui, &mut pie.click_target, target, target.label());
        }
    });
}

fn manual_strip_ui(ui: &mut egui::Ui, strip: &mut StripDraftState) {
    ui.strong("Strip corridor");
    coordinate_field_ui(ui, "Point 1", &mut strip.point1);
    coordinate_field_ui(ui, "Point 2", &mut strip.point2);
    ui.label("Width (NM)");
    ui.add(egui::TextEdit::singleline(&mut strip.width_nm).desired_width(96.0));

    ui.label("Preview click target");
    ui.horizontal(|ui| {
        for target in [StripClickTarget::Point1, StripClickTarget::Point2] {
            selectable_enum(ui, &mut strip.click_target, target, target.label());
        }
    });
}

fn coordinate_field_ui(ui: &mut egui::Ui, label: &str, coordinate: &mut CoordinateFieldState) {
    if !label.is_empty() {
        ui.label(label);
    }
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Latitude");
            ui.add(egui::TextEdit::singleline(&mut coordinate.lat).desired_width(104.0));
        });
        ui.vertical(|ui| {
            ui.label("Longitude");
            ui.add(egui::TextEdit::singleline(&mut coordinate.lon).desired_width(104.0));
        });
    });
}

fn selectable_enum<T: Copy + PartialEq>(ui: &mut egui::Ui, value: &mut T, option: T, label: &str) {
    if ui.selectable_label(*value == option, label).clicked() {
        *value = option;
    }
}

fn date_picker_contents(
    ui: &mut egui::Ui,
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
                    button = button.fill(egui::Color32::from_rgb(35, 94, 98));
                } else if date == today {
                    button = button.stroke(egui::Stroke::new(1.0, ACCENT));
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
        if ui.button("Today").clicked() {
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

fn section_heading(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        let rect = ui
            .allocate_exact_size(egui::vec2(3.0, 18.0), egui::Sense::hover())
            .0;
        ui.painter().rect_filled(rect, 1.5, ACCENT);
        ui.heading(title);
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);
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

enum ExportFailure {
    Validation(Vec<ValidationIssue>),
    Io(String),
}

fn export_creation(creation: &DamCreation) -> Result<String, ExportFailure> {
    match to_pretty_json(creation) {
        Ok(json) => platform::export_json("dam-export.json", &json)
            .map(|path| format!("Exported {path}"))
            .map_err(ExportFailure::Io),
        Err(dam_core::ExportError::Validation(error)) => {
            Err(ExportFailure::Validation(error.issues))
        }
        Err(error) => Err(ExportFailure::Io(error.to_string())),
    }
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
