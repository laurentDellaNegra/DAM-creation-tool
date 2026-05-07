fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DAM Creation Tool",
        options,
        Box::new(|cc| Ok(Box::new(dam_egui::DamApp::new(cc)))),
    )
}
