use crate::markdown;
use omd_common::{parse_front_matter, resolve_title};
use std::path::Path;

const EXPORT_CSS: &str = r#"
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  line-height: 1.7;
  padding: 2rem 1.5rem;
}
body.light { background: #f8f9fa; color: #212529; }
body.dark { background: #1a1b1e; color: #e9ecef; }
article { max-width: 48rem; margin: 0 auto; }
h1 { font-size: 1.75rem; margin: 0.5rem 0; }
h2 { font-size: 1.4rem; margin: 0.5rem 0; }
h3 { font-size: 1.2rem; margin: 0.4rem 0; }
h4, h5, h6 { margin: 0.4rem 0; }
p { margin: 0.5rem 0; }
ul, ol { margin: 0.5rem 0; padding-left: 1.5rem; }
blockquote {
  margin: 0.5rem 0;
  padding-left: 1rem;
  border-left: 3px solid #0d6efd;
}
body.dark blockquote { border-left-color: #4dabf7; color: #adb5bd; }
code {
  font-family: "SF Mono", "Fira Code", Consolas, monospace;
  font-size: 0.9em;
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
}
body.light code { background: #f1f3f5; }
body.dark code { background: #2c2e33; }
pre {
  padding: 0.75rem;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5rem 0;
}
body.light pre { background: #f1f3f5; }
body.dark pre { background: #2c2e33; }
pre code { background: none; padding: 0; }
table { border-collapse: collapse; width: 100%; margin: 0.5rem 0; font-size: 0.9rem; }
th, td { border: 1px solid #dee2e6; padding: 0.4rem 0.6rem; }
body.dark th, body.dark td { border-color: #373a40; }
body.light th { background: #e9ecef; }
body.dark th { background: #2c2e33; }
a { color: #0d6efd; }
body.dark a { color: #4dabf7; }
hr { border: none; border-top: 1px solid #dee2e6; margin: 1rem 0; }
body.dark hr { border-top-color: #373a40; }
img { max-width: 100%; height: auto; border-radius: 6px; margin: 0.5rem 0; display: block; }
.mermaid {
  margin: 0.75rem 0;
  padding: 0.75rem;
  border-radius: 6px;
  overflow-x: auto;
}
body.light .mermaid { background: #e9ecef; }
body.dark .mermaid { background: #2c2e33; }
.math-display { display: block; margin: 0.75rem 0; overflow-x: auto; }
footer {
  max-width: 48rem;
  margin: 2rem auto 0;
  padding-top: 1rem;
  font-size: 0.85rem;
  opacity: 0.65;
  text-align: center;
}
.toc {
  margin: 0.75rem 0 1rem;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  border: 1px solid #dee2e6;
}
body.dark .toc { border-color: #373a40; background: #25262b; }
body.light .toc { background: #f8f9fa; }
.toc ul { margin: 0.35rem 0 0; padding-left: 1.25rem; }
.footnotes { margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #dee2e6; font-size: 0.9rem; }
body.dark .footnotes { border-top-color: #373a40; }
"#;

const PRINT_CSS: &str = r#"
@media print {
  body { padding: 0; background: #fff !important; color: #000 !important; }
  article { max-width: none; }
  footer { display: none; }
  a { color: #000; text-decoration: none; }
}
@page { margin: 2cm; }
"#;

const EXPORT_POST_SCRIPT: &str = r#"
function omdRenderMathNodes(root) {
  if (typeof katex === 'undefined') return;
  root.querySelectorAll('.math.math-inline:not([data-katex])').forEach((el) => {
    try {
      katex.render(el.textContent, el, { throwOnError: false, displayMode: false });
      el.setAttribute('data-katex', '1');
    } catch (e) { console.warn('katex inline:', e); }
  });
  root.querySelectorAll('.math.math-display:not([data-katex])').forEach((el) => {
    try {
      katex.render(el.textContent, el, { throwOnError: false, displayMode: true });
      el.setAttribute('data-katex', '1');
    } catch (e) { console.warn('katex display:', e); }
  });
}
omdRenderMathNodes(document);
document.querySelectorAll('pre code').forEach((block) => {
  if (!block.classList.contains('language-mermaid')) hljs.highlightElement(block);
});
mermaid.run({ nodes: document.querySelectorAll('.mermaid') }).catch(console.warn);
"#;

/// Build a standalone HTML document from Markdown source.
pub fn export_html_document(markdown: &str, title: &str, dark_mode: bool) -> String {
    build_html_document(markdown, title, dark_mode, false)
}

/// HTML for print-to-PDF via the system browser (light theme, auto-print).
pub fn export_print_html_document(markdown: &str, title: &str) -> String {
    build_html_document(markdown, title, false, true)
}

fn build_html_document(markdown: &str, title: &str, dark_mode: bool, for_print: bool) -> String {
    let body = markdown::markdown_to_html(markdown);
    let theme_class = if dark_mode && !for_print {
        "dark"
    } else {
        "light"
    };
    let mermaid_theme = if dark_mode && !for_print {
        "dark"
    } else {
        "default"
    };
    let hljs_theme = if dark_mode && !for_print {
        "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/styles/github-dark.min.css"
    } else {
        "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/styles/github.min.css"
    };
    let escaped_title = html_escape_attr(title);
    let print_block = if for_print {
        format!("<style>{PRINT_CSS}</style>")
    } else {
        String::new()
    };
    let print_script = if for_print {
        r#"window.addEventListener('load', () => setTimeout(() => window.print(), 400));"#
    } else {
        ""
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>{escaped_title}</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css" />
<link rel="stylesheet" href="{hljs_theme}" />
<style>{EXPORT_CSS}</style>
{print_block}
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"></script>
<script src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/highlight.min.js"></script>
</head>
<body class="{theme_class}">
<article class="preview-content">
{body}
</article>
<footer>Exported by <a href="https://github.com/doshall/omd">omd</a></footer>
<script>
mermaid.initialize({{ startOnLoad: false, theme: '{mermaid_theme}', securityLevel: 'loose' }});
{EXPORT_POST_SCRIPT}
{print_script}
</script>
</body>
</html>
"#
    )
}

/// Derive a page title from a file path or markdown content.
pub fn export_title(file_path: Option<&Path>, markdown: &str) -> String {
    let (fm, body) = parse_front_matter(markdown);
    let filename = file_path
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str());
    resolve_title(fm.as_ref(), filename, body)
}

/// Suggest an `.html` filename from a markdown path or default name.
pub fn html_filename(file_path: Option<&Path>) -> String {
    file_path
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .map(|stem| format!("{stem}.html"))
        .unwrap_or_else(|| "document.html".to_string())
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_contains_article_and_title() {
        let html = export_html_document("# Hello\n\nWorld", "Test", false);
        assert!(html.contains("<article"));
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("katex.min.js"));
    }

    #[test]
    fn export_title_from_heading() {
        assert_eq!(
            export_title(None, "# My Doc\n\nbody"),
            "My Doc"
        );
    }

    #[test]
    fn print_html_includes_print_hook() {
        let html = export_print_html_document("# Doc", "Doc");
        assert!(html.contains("window.print"));
    }
}
