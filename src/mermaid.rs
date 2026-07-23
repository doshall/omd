use egui::{ColorImage, TextureHandle, Ui, Vec2};
use mermaid_rs_renderer::{render_with_options, RenderOptions, Theme};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

struct CacheEntry {
    texture: TextureHandle,
    size: Vec2,
}

/// Caches rendered Mermaid diagrams as egui textures.
pub struct MermaidCache {
    entries: HashMap<u64, CacheEntry>,
    dark_mode: bool,
}

impl Default for MermaidCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            dark_mode: true,
        }
    }
}

impl MermaidCache {
    pub fn sync_theme(&mut self, dark_mode: bool) {
        if self.dark_mode != dark_mode {
            self.entries.clear();
            self.dark_mode = dark_mode;
        }
    }

    pub fn show_diagram(&mut self, ui: &mut Ui, source: &str, dark_mode: bool) {
        self.sync_theme(dark_mode);
        let key = cache_key(source, dark_mode);

        if !self.entries.contains_key(&key) {
            if let Some((color_image, size)) = render_mermaid_to_image(source, dark_mode, ui.available_width()) {
                let texture = ui.ctx().load_texture(
                    format!("mermaid_{key}"),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.entries.insert(
                    key,
                    CacheEntry {
                        texture,
                        size,
                    },
                );
            }
        }

        if let Some(entry) = self.entries.get(&key) {
            let max_w = ui.available_width();
            let scale = (max_w / entry.size.x).min(1.0);
            let display_size = entry.size * scale;
            ui.image((entry.texture.id(), display_size));
            ui.add_space(6.0);
        } else {
            let frame = egui::Frame::none()
                .fill(ui.visuals().widgets.inactive.bg_fill)
                .inner_margin(8.0)
                .rounding(4.0);
            frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Mermaid render failed")
                        .color(ui.visuals().error_fg_color),
                );
                ui.label(
                    egui::RichText::new(source)
                        .font(egui::FontId::monospace(12.0))
                        .color(ui.visuals().weak_text_color()),
                );
            });
            ui.add_space(4.0);
        }
    }
}

fn cache_key(source: &str, dark_mode: bool) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    dark_mode.hash(&mut hasher);
    hasher.finish()
}

fn render_mermaid_to_image(
    source: &str,
    dark_mode: bool,
    max_width: f32,
) -> Option<(ColorImage, Vec2)> {
    let mut options = RenderOptions::modern();
    options.theme = if dark_mode {
        Theme::dark()
    } else {
        Theme::mermaid_default()
    };

    let svg = render_with_options(source, options).ok()?;
    svg_to_color_image(&svg, max_width)
}

fn svg_to_color_image(svg: &str, max_width: f32) -> Option<(ColorImage, Vec2)> {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &options).ok()?;
    let size = tree.size();

    let w = size.width();
    let h = size.height();
    let scale = if w > max_width { max_width / w } else { 1.0 };
    let width = (w * scale).ceil().max(1.0) as u32;
    let height = (h * scale).ceil().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    let transform = tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let pixels: Vec<egui::Color32> = pixmap
        .pixels()
        .iter()
        .map(|p| {
            egui::Color32::from_rgba_unmultiplied(p.red(), p.green(), p.blue(), p.alpha())
        })
        .collect();

    let color_image = ColorImage {
        size: [width as usize, height as usize],
        pixels,
    };
    Some((color_image, Vec2::new(width as f32, height as f32)))
}
