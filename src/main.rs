#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clipboard;
mod editor_highlight;
mod export;
mod find_replace;
mod image_compress;
mod keybindings;
mod vim_ex;
mod line_gutter;
mod markdown;
mod mermaid;
mod minimap;
mod project;
mod settings;
mod sync_scroll;
mod syntax_highlight;
mod tabs;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("omd — Markdown Editor"),
        ..Default::default()
    };

    let cli_open = parse_cli_open_path();

    eframe::run_native(
        "omd",
        options,
        Box::new(move |cc| Ok(Box::new(app::OmdApp::new(cc, cli_open.clone())))),
    )
}

/// `omd path/to/file.md` — open that file on startup.
fn parse_cli_open_path() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    let first = args.next()?;
    if first == "--help" || first == "-h" {
        eprintln!(
            "OMD Markdown Editor\n\nUsage:\n  omd              Open the editor\n  omd <file.md>    Open the editor with a file"
        );
        std::process::exit(0);
    }
    if first.starts_with('-') {
        return None;
    }
    Some(PathBuf::from(first))
}
