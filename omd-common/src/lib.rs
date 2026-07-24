mod front_matter;
mod markdown;
mod tasks;

pub use front_matter::{parse_front_matter, resolve_title, FrontMatter};
pub use markdown::{
    collect_headings, make_task_lists_interactive, markdown_options, markdown_options_with,
    markdown_to_html, markdown_to_html_with_options, markdown_to_html_with_options_parts,
    markdown_to_html_with_toc, slugify, MarkdownRenderOptions, TocEntry, TocOptions,
};
pub use tasks::{task_line_count, toggle_task_by_index};
