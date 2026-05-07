use dam_core::Coordinate;

pub struct PreviewOverlay {
    base_paths: Vec<Vec<Coordinate>>,
    selected_paths: Vec<Vec<Coordinate>>,
}

impl PreviewOverlay {
    pub fn new(base_paths: Vec<Vec<Coordinate>>, selected_paths: Vec<Vec<Coordinate>>) -> Self {
        Self {
            base_paths,
            selected_paths,
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
        painter.rect_filled(response.rect, 0.0, egui::Color32::from_rgb(8, 11, 14));

        paint_paths(
            painter,
            projector,
            &self.base_paths,
            egui::Stroke::new(1.4, egui::Color32::from_rgb(95, 116, 132)),
        );
        paint_paths(
            painter,
            projector,
            &self.selected_paths,
            egui::Stroke::new(2.4, egui::Color32::from_rgb(95, 200, 205)),
        );
    }
}

fn paint_paths(
    painter: &egui::Painter,
    projector: &walkers::Projector,
    paths: &[Vec<Coordinate>],
    stroke: egui::Stroke,
) {
    for path in paths {
        let points: Vec<egui::Pos2> = path
            .iter()
            .map(|coordinate| {
                let projected = projector.project(walkers::lon_lat(coordinate.lon, coordinate.lat));
                egui::pos2(projected.x, projected.y)
            })
            .collect();

        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, stroke));
        }
    }
}
