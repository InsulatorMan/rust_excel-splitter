mod config;
mod error;
mod processor;
mod reader;
mod writer;
mod xml_style;
mod gui;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn"),
    )
    .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Excel 拆分工具")
            .with_inner_size([820.0, 680.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Excel 拆分工具",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::ExcelSplitterApp::new(cc)))),
    )
}
