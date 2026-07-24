use crate::mermaid::MermaidCache;
use crate::syntax_highlight;
use egui::{Color32, FontFamily, FontId, RichText, Sense, Stroke, Ui};
use omd_common::{collect_headings, markdown_to_html as common_html, parse_front_matter};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};

pub struct PreviewContext<'a> {
    pub dark_mode: bool,
    pub base_path: Option<&'a Path>,
    pub mermaid_cache: &'a mut MermaidCache,
    pub preview_syntax_highlight: bool,
    pub preview_font_size: f32,
    pub image_lightbox: &'a mut Option<String>,
}

struct PreviewState<'a> {
    heading_level: HeadingLevel,
    in_code_block: bool,
    code_block_lang: Option<String>,
    code_buffer: String,
    in_blockquote: bool,
    list_stack: Vec<Option<u64>>,
    link_url: Option<String>,
    emphasis: u8,
    strong: u8,
    strikethrough: bool,
    task_checked: Option<bool>,
    in_table: bool,
    in_table_head: bool,
    table_headers: Vec<String>,
    table_rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: String,
    inline_buffer: String,
    image_url: Option<String>,
    image_alt: String,
    footnote_defs: Vec<(String, String)>,
    in_footnote_def: bool,
    footnote_name: String,
    footnote_buffer: String,
    base_path: Option<PathBuf>,
    ctx: &'a mut PreviewContext<'a>,
}

impl<'a> PreviewState<'a> {
    fn new(ctx: &'a mut PreviewContext<'a>) -> Self {
        Self {
            heading_level: HeadingLevel::H1,
            in_code_block: false,
            code_block_lang: None,
            code_buffer: String::new(),
            in_blockquote: false,
            list_stack: Vec::new(),
            link_url: None,
            emphasis: 0,
            strong: 0,
            strikethrough: false,
            task_checked: None,
            in_table: false,
            in_table_head: false,
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            current_row: Vec::new(),
            current_cell: String::new(),
            inline_buffer: String::new(),
            image_url: None,
            image_alt: String::new(),
            footnote_defs: Vec::new(),
            in_footnote_def: false,
            footnote_name: String::new(),
            footnote_buffer: String::new(),
            base_path: ctx.base_path.map(Path::to_path_buf),
            ctx,
        }
    }

    fn resolve_image_uri(&self, url: &str) -> String {
        if url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("data:")
            || url.starts_with("file://")
        {
            return url.to_string();
        }

        let path = Path::new(url);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(base) = &self.base_path {
            base.join(path)
        } else {
            path.to_path_buf()
        };

        format!("file://{}", resolved.display())
    }

    fn render_image(&mut self, ui: &mut Ui, url: &str, alt: &str) {
        let uri = self.resolve_image_uri(url);
        let max_w = ui.available_width().min(480.0);

        ui.vertical(|ui| {
            let response = ui.add(
                egui::Image::new(&uri)
                    .max_width(max_w)
                    .sense(Sense::click()),
            );
            if response.clicked() {
                *self.ctx.image_lightbox = Some(uri.clone());
            }
            if !alt.is_empty() {
                ui.label(
                    RichText::new(alt)
                        .italics()
                        .size(12.0)
                        .color(ui.visuals().weak_text_color()),
                );
            }
            ui.add_space(6.0);
        });
    }

    fn flush_inline(&mut self, ui: &mut Ui) {
        if self.inline_buffer.is_empty() {
            return;
        }

        let text = std::mem::take(&mut self.inline_buffer);
        let mut rich = RichText::new(&text);
        if self.strong > 0 {
            rich = rich.strong();
        }
        if self.emphasis > 0 {
            rich = rich.italics();
        }
        if self.strikethrough {
            rich = rich.strikethrough();
        }
        if self.in_blockquote {
            rich = rich.color(ui.visuals().weak_text_color());
        }

        rich = match self.heading_level {
            HeadingLevel::H1 => rich.size(28.0).strong(),
            HeadingLevel::H2 => rich.size(24.0).strong(),
            HeadingLevel::H3 => rich.size(20.0).strong(),
            HeadingLevel::H4 => rich.size(17.0).strong(),
            HeadingLevel::H5 => rich.size(15.0).strong(),
            HeadingLevel::H6 => rich.size(14.0).strong(),
        };

        if let Some(url) = &self.link_url {
            ui.hyperlink_to(text, url.clone());
        } else {
            ui.label(rich);
        }
    }

    fn render_math(&mut self, ui: &mut Ui, tex: &str, display: bool) {
        let frame = egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(if display { 10.0 } else { 4.0 })
            .rounding(4.0);
        frame.show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            let font = FontId::new(if display { 15.0 } else { 13.0 }, FontFamily::Monospace);
            ui.label(
                RichText::new(tex)
                    .font(font)
                    .italics()
                    .color(ui.visuals().text_color()),
            );
        });
        if display {
            ui.add_space(6.0);
        }
    }

    fn render_code_block(&mut self, ui: &mut Ui, code: String, lang: Option<String>) {
        if lang.as_deref() == Some("mermaid") {
            self.ctx
                .mermaid_cache
                .show_diagram(ui, &code, self.ctx.dark_mode);
            return;
        }

        let frame = egui::Frame::none()
            .fill(ui.visuals().code_bg_color)
            .inner_margin(8.0)
            .rounding(4.0)
            .stroke(Stroke::new(
                1.0_f32,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ));
        frame.show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            if let Some(lang_name) = lang.as_deref().filter(|l| !l.is_empty()) {
                ui.label(
                    RichText::new(lang_name)
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                );
            }
            let font = FontId::new(13.0, FontFamily::Monospace);
            if self.ctx.preview_syntax_highlight {
                let lines = syntax_highlight::highlight_lines(
                    &code,
                    lang.as_deref(),
                    self.ctx.dark_mode,
                );
                for tokens in lines {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        if tokens.is_empty() {
                            ui.label(RichText::new(" ").font(font.clone()));
                        } else {
                            for (color, text) in tokens {
                                ui.label(RichText::new(text).font(font.clone()).color(color));
                            }
                        }
                    });
                }
            } else {
                ui.label(RichText::new(code).font(font));
            }
        });
        ui.add_space(4.0);
    }

    fn handle_event(&mut self, ui: &mut Ui, event: Event<'_>) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                self.heading_level = level;
            }
            Event::End(TagEnd::Heading(_)) => {
                self.flush_inline(ui);
                ui.add_space(8.0);
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if self.in_table {
                    self.current_row.push(std::mem::take(&mut self.current_cell));
                    if self.in_table_head {
                        self.table_headers = std::mem::take(&mut self.current_row);
                    } else {
                        self.table_rows.push(std::mem::take(&mut self.current_row));
                    }
                } else {
                    self.flush_inline(ui);
                    ui.add_space(6.0);
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                self.in_blockquote = true;
                ui.indent("blockquote", |_ui| {});
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.in_blockquote = false;
                ui.add_space(4.0);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                self.in_code_block = true;
                self.code_buffer.clear();
                self.code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang = lang.to_string();
                        if lang.is_empty() {
                            None
                        } else {
                            Some(lang)
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                self.in_code_block = false;
                let code = std::mem::take(&mut self.code_buffer);
                let lang = self.code_block_lang.take();
                self.render_code_block(ui, code, lang);
            }
            Event::Start(Tag::List(start)) => {
                self.list_stack.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                self.list_stack.pop();
                ui.add_space(4.0);
            }
            Event::Start(Tag::Item) => {
                let bullet = match self.list_stack.last().copied().flatten() {
                    Some(n) => format!("{n}. "),
                    None => "• ".to_string(),
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(bullet).monospace());
                });
            }
            Event::End(TagEnd::Item) => {
                self.flush_inline(ui);
            }
            Event::Start(Tag::Emphasis) => self.emphasis += 1,
            Event::End(TagEnd::Emphasis) => self.emphasis = self.emphasis.saturating_sub(1),
            Event::Start(Tag::Strong) => self.strong += 1,
            Event::End(TagEnd::Strong) => self.strong = self.strong.saturating_sub(1),
            Event::Start(Tag::Strikethrough) => self.strikethrough = true,
            Event::End(TagEnd::Strikethrough) => self.strikethrough = false,
            Event::Start(Tag::Link { dest_url, .. }) => {
                self.flush_inline(ui);
                self.link_url = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                self.link_url = None;
            }
            Event::Start(Tag::Table(_)) => {
                self.in_table = true;
                self.table_headers.clear();
                self.table_rows.clear();
            }
            Event::End(TagEnd::Table) => {
                render_table(ui, &self.table_headers, &self.table_rows);
                self.in_table = false;
                self.table_headers.clear();
                self.table_rows.clear();
                ui.add_space(8.0);
            }
            Event::Start(Tag::TableHead) => self.in_table_head = true,
            Event::End(TagEnd::TableHead) => self.in_table_head = false,
            Event::Start(Tag::TableRow) => self.current_row.clear(),
            Event::End(TagEnd::TableRow) => {
                if self.in_table_head {
                    self.table_headers = std::mem::take(&mut self.current_row);
                } else if !self.current_row.is_empty() {
                    self.table_rows.push(std::mem::take(&mut self.current_row));
                }
            }
            Event::Start(Tag::TableCell) => self.current_cell.clear(),
            Event::End(TagEnd::TableCell) => {
                self.current_row.push(std::mem::take(&mut self.current_cell));
            }
            Event::TaskListMarker(checked) => {
                self.task_checked = Some(checked);
            }
            Event::Rule => {
                self.flush_inline(ui);
                ui.separator();
                ui.add_space(4.0);
            }
            Event::SoftBreak => {
                if self.in_table {
                    self.current_cell.push(' ');
                } else if self.in_code_block {
                    self.code_buffer.push(' ');
                } else {
                    self.inline_buffer.push(' ');
                }
            }
            Event::HardBreak => {
                self.flush_inline(ui);
            }
            Event::Text(text) => {
                if self.in_footnote_def {
                    self.footnote_buffer.push_str(&text);
                } else if self.in_table {
                    self.current_cell.push_str(&text);
                } else if self.in_code_block {
                    self.code_buffer.push_str(&text);
                } else if self.image_url.is_some() {
                    self.image_alt.push_str(&text);
                } else if self.task_checked.is_some() {
                    let checked = self.task_checked == Some(true);
                    let prefix = if checked { "☑ " } else { "☐ " };
                    let mut rich = RichText::new(format!("{prefix}{text}"));
                    if checked {
                        rich = rich.strikethrough();
                    }
                    ui.label(rich);
                    self.task_checked = None;
                } else {
                    self.inline_buffer.push_str(&text);
                }
            }
            Event::Code(code) => {
                ui.label(
                    RichText::new(code.to_string())
                        .font(FontId::new(13.0, FontFamily::Monospace))
                        .background_color(ui.visuals().code_bg_color),
                );
            }
            Event::InlineMath(text) => {
                self.flush_inline(ui);
                self.render_math(ui, &text, false);
            }
            Event::DisplayMath(text) => {
                self.flush_inline(ui);
                self.render_math(ui, &text, true);
            }
            Event::Html(html) => {
                ui.label(
                    RichText::new(html.to_string())
                        .italics()
                        .color(Color32::GRAY),
                );
            }
            Event::FootnoteReference(name) => {
                self.flush_inline(ui);
                ui.label(
                    RichText::new(format!("[^{name}]"))
                        .small()
                        .color(ui.visuals().hyperlink_color),
                );
            }
            Event::Start(Tag::FootnoteDefinition(name)) => {
                self.in_footnote_def = true;
                self.footnote_name = name.to_string();
                self.footnote_buffer.clear();
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                self.footnote_defs
                    .push((self.footnote_name.clone(), self.footnote_buffer.clone()));
                self.in_footnote_def = false;
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                self.flush_inline(ui);
                self.image_url = Some(dest_url.to_string());
                self.image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                if let Some(url) = self.image_url.take() {
                    let alt = std::mem::take(&mut self.image_alt);
                    self.render_image(ui, &url, &alt);
                }
            }
            _ => {}
        }
    }
}

/// Render Markdown source into an egui scroll area.
pub fn render_preview<'a>(ui: &mut Ui, markdown: &str, ctx: &'a mut PreviewContext<'a>) {
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(ctx.preview_font_size, egui::FontFamily::Proportional),
    );
    let (front_matter, body) = parse_front_matter(markdown);
    if let Some(fm) = &front_matter {
        if let Some(title) = fm.title.as_deref().filter(|t| !t.is_empty()) {
            ui.heading(title);
        }
        if let Some(desc) = fm.description.as_deref().filter(|d| !d.is_empty()) {
            ui.label(RichText::new(desc).italics().color(ui.visuals().weak_text_color()));
            ui.add_space(4.0);
        }
    }
    let headings = collect_headings(body);
    if !headings.is_empty() {
        ui.label(RichText::new("目录").strong());
        for entry in &headings {
            let indent = "  ".repeat(entry.level.saturating_sub(1) as usize);
            ui.label(format!("{indent}• {}", entry.title));
        }
        ui.separator();
    }

    let parser = Parser::new_ext(body, markdown_options());
    let mut state = PreviewState::new(ctx);

    for event in parser {
        state.handle_event(ui, event);
    }
    state.flush_inline(ui);

    if !state.footnote_defs.is_empty() {
        ui.separator();
        ui.label(RichText::new("脚注").strong());
        for (name, text) in state.footnote_defs {
            ui.label(RichText::new(format!("[^{name}]: {text}")).small());
        }
    }
}

fn render_table(ui: &mut Ui, headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        return;
    }

    let col_count = headers.len();
    egui::Grid::new("md_table")
        .striped(true)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            for header in headers {
                ui.strong(header);
            }
            ui.end_row();

            for row in rows {
                for i in 0..col_count {
                    let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                    ui.label(cell);
                }
                ui.end_row();
            }
        });
}

/// Insert markdown formatting around the current selection or at cursor.
pub fn wrap_selection(text: &mut String, cursor_range: std::ops::Range<usize>, wrapper: &str) {
    let start = cursor_range.start.min(text.len());
    let end = cursor_range.end.min(text.len());
    let selected = text[start..end].to_string();

    let (open, close) = match wrapper {
        "**" | "__" => ("**", "**"),
        "*" | "_" => ("*", "*"),
        "~~" => ("~~", "~~"),
        "`" => ("`", "`"),
        "[]()" => ("[", "](url)"),
        other => (other, ""),
    };

    let replacement = format!("{open}{selected}{close}");
    text.replace_range(start..end, &replacement);
}

/// Insert a prefix at the beginning of each selected line.
pub fn prefix_lines(text: &mut String, cursor_range: std::ops::Range<usize>, prefix: &str) {
    let start = cursor_range.start.min(text.len());
    let end = cursor_range.end.min(text.len());

    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[end..]
        .find('\n')
        .map(|i| end + i)
        .unwrap_or(text.len());

    let block = &text[line_start..line_end];
    let prefixed: String = block
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n");

    text.replace_range(line_start..line_end, &prefixed);
}

/// Count words in markdown text (rough estimate).
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

/// Count lines in text.
pub fn line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.lines().count()
    }
}

/// Insert text at a byte cursor position.
pub fn insert_at_cursor(text: &str, cursor: usize, insertion: &str) -> String {
    let cursor = cursor.min(text.len());
    format!("{}{}{}", &text[..cursor], insertion, &text[cursor..])
}

/// Build a Markdown image snippet.
pub fn image_markdown(alt: &str, url: &str) -> String {
    format!("\n![{alt}]({url})\n")
}

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"];

/// Whether a path looks like a supported image file.
pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn markdown_options() -> Options {
    omd_common::markdown_options()
}

/// Convert Markdown to HTML for export (includes Mermaid block transform).
pub fn markdown_to_html(markdown: &str) -> String {
    let (_, body) = parse_front_matter(markdown);
    common_html(body)
}

fn transform_mermaid_blocks(html: &str) -> String {
    let marker = "<pre><code class=\"language-mermaid\">";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(marker) {
        out.push_str(&rest[..start]);
        let after = &rest[start + marker.len()..];
        if let Some(end) = after.find("</code></pre>") {
            let code = &after[..end];
            out.push_str("<div class=\"mermaid\">");
            out.push_str(code);
            out.push_str("</div>");
            rest = &after[end + "</code></pre>".len()..];
        } else {
            out.push_str(marker);
            rest = after;
            break;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod markdown_html_tests {
    use super::*;

    #[test]
    fn mermaid_html_does_not_double_escape_arrows() {
        let html_in = "<pre><code class=\"language-mermaid\">flowchart TD\n    A --&gt; B\n</code></pre>";
        let out = transform_mermaid_blocks(html_in);
        assert!(!out.contains("&amp;gt;"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_at_cursor_mid_text() {
        assert_eq!(
            insert_at_cursor("hello world", 5, "!"),
            "hello! world"
        );
    }

    #[test]
    fn image_markdown_format() {
        assert_eq!(image_markdown("alt", "/tmp/a.png"), "\n![alt](/tmp/a.png)\n");
    }

    #[test]
    fn is_image_path_checks_extension() {
        assert!(is_image_path(Path::new("photo.PNG")));
        assert!(!is_image_path(Path::new("notes.md")));
    }
}
