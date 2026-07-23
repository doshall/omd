#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clipboard;
mod find_replace;
mod line_gutter;
mod markdown;
mod mermaid;
mod minimap;
mod sync_scroll;
mod syntax_highlight;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("omd — Markdown Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "omd",
        options,
        Box::new(|cc| Ok(Box::new(app::OmdApp::new(cc)))),
    )
}
