use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    Zh,
    En,
}

/// Translate a UI string key for the given locale.
pub fn t<'a>(locale: Locale, key: &'a str) -> Cow<'a, str> {
    match (locale, key) {
        // Navigation & actions
        (Locale::Zh, "new") => Cow::Borrowed("新建"),
        (Locale::En, "new") => Cow::Borrowed("New"),
        (Locale::Zh, "open") => Cow::Borrowed("打开"),
        (Locale::En, "open") => Cow::Borrowed("Open"),
        (Locale::Zh, "folder") => Cow::Borrowed("文件夹"),
        (Locale::En, "folder") => Cow::Borrowed("Folder"),
        (Locale::Zh, "download") => Cow::Borrowed("下载"),
        (Locale::En, "download") => Cow::Borrowed("Download"),
        (Locale::Zh, "export_html") => Cow::Borrowed("导出 HTML"),
        (Locale::En, "export_html") => Cow::Borrowed("Export HTML"),
        (Locale::Zh, "export_pdf") => Cow::Borrowed("导出 PDF"),
        (Locale::En, "export_pdf") => Cow::Borrowed("Export PDF"),
        (Locale::Zh, "settings") => Cow::Borrowed("设置"),
        (Locale::En, "settings") => Cow::Borrowed("Settings"),
        (Locale::Zh, "project_sidebar") => Cow::Borrowed("项目侧边栏"),
        (Locale::En, "project_sidebar") => Cow::Borrowed("Project sidebar"),
        (Locale::Zh, "dark_mode") => Cow::Borrowed("深色模式"),
        (Locale::En, "dark_mode") => Cow::Borrowed("Dark mode"),
        (Locale::Zh, "editor") => Cow::Borrowed("编辑区"),
        (Locale::En, "editor") => Cow::Borrowed("Editor"),
        (Locale::Zh, "preview") => Cow::Borrowed("预览"),
        (Locale::En, "preview") => Cow::Borrowed("Preview"),
        (Locale::Zh, "done") => Cow::Borrowed("完成"),
        (Locale::En, "done") => Cow::Borrowed("Done"),
        (Locale::Zh, "close") => Cow::Borrowed("关闭"),
        (Locale::En, "close") => Cow::Borrowed("Close"),
        (Locale::Zh, "loading") => Cow::Borrowed("正在加载文档…"),
        (Locale::En, "loading") => Cow::Borrowed("Loading document…"),
        (Locale::Zh, "editor_placeholder") => Cow::Borrowed("在此输入 Markdown，可粘贴或拖入图片…"),
        (Locale::En, "editor_placeholder") => Cow::Borrowed("Write Markdown here — paste or drop images…"),

        // Settings sections
        (Locale::Zh, "settings_title") => Cow::Borrowed("编辑器设置"),
        (Locale::En, "settings_title") => Cow::Borrowed("Editor settings"),
        (Locale::Zh, "section_editor") => Cow::Borrowed("编辑区"),
        (Locale::En, "section_editor") => Cow::Borrowed("Editor"),
        (Locale::Zh, "section_preview") => Cow::Borrowed("预览"),
        (Locale::En, "section_preview") => Cow::Borrowed("Preview"),
        (Locale::Zh, "section_images") => Cow::Borrowed("图片"),
        (Locale::En, "section_images") => Cow::Borrowed("Images"),
        (Locale::Zh, "section_focus") => Cow::Borrowed("专注与提示"),
        (Locale::En, "section_focus") => Cow::Borrowed("Focus & hints"),
        (Locale::Zh, "section_appearance") => Cow::Borrowed("外观"),
        (Locale::En, "section_appearance") => Cow::Borrowed("Appearance"),
        (Locale::Zh, "section_accessibility") => Cow::Borrowed("无障碍"),
        (Locale::En, "section_accessibility") => Cow::Borrowed("Accessibility"),

        // Settings labels
        (Locale::Zh, "show_line_numbers") => Cow::Borrowed("显示行号"),
        (Locale::En, "show_line_numbers") => Cow::Borrowed("Show line numbers"),
        (Locale::Zh, "highlight_current_line") => Cow::Borrowed("高亮当前行"),
        (Locale::En, "highlight_current_line") => Cow::Borrowed("Highlight current line"),
        (Locale::Zh, "show_minimap") => Cow::Borrowed("显示 Minimap"),
        (Locale::En, "show_minimap") => Cow::Borrowed("Show minimap"),
        (Locale::Zh, "font_size") => Cow::Borrowed("字号"),
        (Locale::En, "font_size") => Cow::Borrowed("Font size"),
        (Locale::Zh, "line_height") => Cow::Borrowed("行高"),
        (Locale::En, "line_height") => Cow::Borrowed("Line height"),
        (Locale::Zh, "editor_syntax_highlight") => Cow::Borrowed("编辑区语法高亮"),
        (Locale::En, "editor_syntax_highlight") => Cow::Borrowed("Syntax highlight in editor"),
        (Locale::Zh, "sync_scroll") => Cow::Borrowed("同步滚动"),
        (Locale::En, "sync_scroll") => Cow::Borrowed("Sync scroll"),
        (Locale::Zh, "preview_syntax_highlight") => Cow::Borrowed("代码块语法高亮"),
        (Locale::En, "preview_syntax_highlight") => Cow::Borrowed("Syntax highlight code blocks"),
        (Locale::Zh, "preview_font_size") => Cow::Borrowed("预览字号"),
        (Locale::En, "preview_font_size") => Cow::Borrowed("Preview font size"),
        (Locale::Zh, "show_toc") => Cow::Borrowed("显示目录（TOC）"),
        (Locale::En, "show_toc") => Cow::Borrowed("Show table of contents"),
        (Locale::Zh, "enable_footnotes") => Cow::Borrowed("启用脚注"),
        (Locale::En, "enable_footnotes") => Cow::Borrowed("Enable footnotes"),
        (Locale::Zh, "compress_images") => Cow::Borrowed("粘贴/上传时压缩图片"),
        (Locale::En, "compress_images") => Cow::Borrowed("Compress pasted images"),
        (Locale::Zh, "max_image_width") => Cow::Borrowed("最大宽度（px）"),
        (Locale::En, "max_image_width") => Cow::Borrowed("Max width (px)"),
        (Locale::Zh, "image_quality") => Cow::Borrowed("JPEG 质量"),
        (Locale::En, "image_quality") => Cow::Borrowed("JPEG quality"),
        (Locale::Zh, "focus_mode") => Cow::Borrowed("专注模式"),
        (Locale::En, "focus_mode") => Cow::Borrowed("Focus mode"),
        (Locale::Zh, "focus_mode_hint") => Cow::Borrowed("快捷键：F11 切换，Esc 退出"),
        (Locale::En, "focus_mode_hint") => Cow::Borrowed("Shortcut: F11 toggle, Esc exit"),
        (Locale::Zh, "undo_redo_hint") => Cow::Borrowed("撤销/重做提示"),
        (Locale::En, "undo_redo_hint") => Cow::Borrowed("Undo / redo hints"),
        (Locale::Zh, "keybinding_mode") => Cow::Borrowed("键位模式"),
        (Locale::En, "keybinding_mode") => Cow::Borrowed("Keybinding mode"),
        (Locale::Zh, "keybinding_standard") => Cow::Borrowed("标准"),
        (Locale::En, "keybinding_standard") => Cow::Borrowed("Standard"),
        (Locale::Zh, "vim_block_highlight") => Cow::Borrowed("Visual Block 高亮"),
        (Locale::En, "vim_block_highlight") => Cow::Borrowed("Highlight Visual Block selection"),
        (Locale::Zh, "vim_system_clipboard") => Cow::Borrowed("系统剪贴板寄存器 (+/*)"),
        (Locale::En, "vim_system_clipboard") => Cow::Borrowed("Use system clipboard for \"+ / \"* registers"),
        (Locale::Zh, "locale") => Cow::Borrowed("界面语言"),
        (Locale::En, "locale") => Cow::Borrowed("Interface language"),
        (Locale::Zh, "locale_zh") => Cow::Borrowed("中文"),
        (Locale::En, "locale_zh") => Cow::Borrowed("中文"),
        (Locale::Zh, "locale_en") => Cow::Borrowed("English"),
        (Locale::En, "locale_en") => Cow::Borrowed("English"),
        (Locale::Zh, "spell_check") => Cow::Borrowed("拼写检查"),
        (Locale::En, "spell_check") => Cow::Borrowed("Spell check"),
        (Locale::Zh, "custom_preview_css") => Cow::Borrowed("自定义预览 CSS"),
        (Locale::En, "custom_preview_css") => Cow::Borrowed("Custom preview CSS"),
        (Locale::Zh, "custom_preview_css_hint") => Cow::Borrowed("注入到预览与导出 HTML，例如：body { font-family: serif; }"),
        (Locale::En, "custom_preview_css_hint") => Cow::Borrowed("Injected into preview and exported HTML, e.g. body { font-family: serif; }"),

        // Desktop settings
        (Locale::Zh, "section_files") => Cow::Borrowed("文件"),
        (Locale::En, "section_files") => Cow::Borrowed("Files"),
        (Locale::Zh, "auto_save") => Cow::Borrowed("自动保存到磁盘"),
        (Locale::En, "auto_save") => Cow::Borrowed("Auto-save to disk"),
        (Locale::Zh, "auto_save_delay") => Cow::Borrowed("自动保存延迟（秒）"),
        (Locale::En, "auto_save_delay") => Cow::Borrowed("Auto-save delay (seconds)"),
        (Locale::Zh, "auto_save_hint") => Cow::Borrowed("仅对已保存路径的文件生效。"),
        (Locale::En, "auto_save_hint") => Cow::Borrowed("Only applies to files with a saved path on disk."),
        (Locale::Zh, "section_desktop") => Cow::Borrowed("桌面"),
        (Locale::En, "section_desktop") => Cow::Borrowed("Desktop"),
        (Locale::Zh, "minimize_to_tray") => Cow::Borrowed("关闭窗口时最小化到托盘"),
        (Locale::En, "minimize_to_tray") => Cow::Borrowed("Minimize to tray when closing window"),
        (Locale::Zh, "global_shortcuts") => Cow::Borrowed("全局快捷键（Ctrl+Shift+O 显示，Ctrl+Shift+N 新建）"),
        (Locale::En, "global_shortcuts") => Cow::Borrowed("Global shortcuts (Ctrl+Shift+O show, Ctrl+Shift+N new)"),
        (Locale::Zh, "tray_hint") => Cow::Borrowed("托盘图标：右键菜单，双击显示窗口。"),
        (Locale::En, "tray_hint") => Cow::Borrowed("Tray icon: right-click menu, double-click to show."),

        // TOC
        (Locale::Zh, "toc") => Cow::Borrowed("目录"),
        (Locale::En, "toc") => Cow::Borrowed("Contents"),

        _ => Cow::Borrowed(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_and_en_differ() {
        assert_ne!(t(Locale::Zh, "new"), t(Locale::En, "new"));
    }

    #[test]
    fn unknown_key_returns_key() {
        assert_eq!(t(Locale::En, "unknown_key_xyz"), "unknown_key_xyz");
    }
}
