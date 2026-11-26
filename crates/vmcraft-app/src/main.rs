use eframe::egui;

mod app;
mod iso_ui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--smoke") {
        println!("vmcraft-app smoke ok");
        return Ok(());
    }
    let viewport = egui::ViewportBuilder::default()
        .with_title("vmcraft — VM workstation")
        .with_inner_size([1180.0, 680.0])
        .with_min_inner_size([720.0, 480.0]);
    let opts = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "vmcraft — VM workstation",
        opts,
        Box::new(|cc| Ok(Box::new(app::VmApp::new(cc)))),
    )
}
