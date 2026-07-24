mod front_matter;
mod markdown;

pub use front_matter::{parse_front_matter, resolve_title, FrontMatter};
pub use markdown::{
    collect_headings, markdown_options, markdown_options_with, markdown_to_html,
    markdown_to_html_with_options, markdown_to_html_with_options_parts, markdown_to_html_with_toc,
    slugify, MarkdownRenderOptions, TocEntry, TocOptions,
};
