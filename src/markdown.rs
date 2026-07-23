use egui::{Color32, FontFamily, FontId, RichText, Stroke, Ui};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};

struct PreviewState {
    heading_level: HeadingLevel,
    in_code_block: bool,
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
    base_path: Option<PathBuf>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            heading_level: HeadingLevel::H1,
            in_code_block: false,
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
            base_path: None,
        }
    }
}

impl PreviewState {
    fn with_base_path(base_path: Option<&Path>) -> Self {
        let mut state = Self::default();
        state.base_path = base_path.map(Path::to_path_buf);
        state
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

    fn render_image(&self, ui: &mut Ui, url: &str, alt: &str) {
        let uri = self.resolve_image_uri(url);
        let max_w = ui.available_width().min(480.0);

        ui.vertical(|ui| {
            ui.add(egui::Image::new(&uri).max_width(max_w));
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
            Event::Start(Tag::CodeBlock(_)) => {
                self.in_code_block = true;
                self.code_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                self.in_code_block = false;
                let code = std::mem::take(&mut self.code_buffer);
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
                    ui.label(
                        RichText::new(code).font(FontId::new(13.0, FontFamily::Monospace)),
                    );
                });
                ui.add_space(4.0);
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
                if self.in_table {
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
            Event::Html(html) => {
                ui.label(
                    RichText::new(html.to_string())
                        .italics()
                        .color(Color32::GRAY),
                );
            }
            Event::FootnoteReference(_) | Event::Start(Tag::FootnoteDefinition(_)) => {}
            Event::End(TagEnd::FootnoteDefinition) => {}
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
pub fn render_preview(ui: &mut Ui, markdown: &str, base_path: Option<&Path>) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut state = PreviewState::with_base_path(base_path);

    for event in parser {
        state.handle_event(ui, event);
    }
    state.flush_inline(ui);
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
