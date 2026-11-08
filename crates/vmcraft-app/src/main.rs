mod app;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--smoke") {
        println!("vmcraft-app smoke ok");
        return Ok(());
    }
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "vmcraft — VM workstation",
        opts,
        Box::new(|cc| Ok(Box::new(app::VmApp::new(cc)))),
    )
}
