//! Template AGENTS.md + workflows cho Antigravity — nhúng từ apps/qt-ai-cli/antigravity để hai bản
//! TS/Rust chỉ có một nguồn. Cargo theo dõi file include_str! nên sửa template là rebuild.
//!
//! File render sẵn trong folder truyện chứa lệnh qt-ai và đường dẫn truyện; mang folder sang máy khác
//! (hoặc dời folder) là hai giá trị đó chết → agent đi tìm binary khắp đĩa. Vì thế mỗi lần init/mở truyện,
//! file nào vẫn đúng khuôn template (chưa sửa tay) thì render lại với giá trị hiện tại.

use crate::error::{CoreError, Result};
use crate::story_fs::write_text;
use regex::Regex;
use std::fs;
use std::path::Path;

pub const AGENTS_MD: &str = include_str!("../../../apps/qt-ai-cli/antigravity/AGENTS.md");
pub const WORKFLOWS: [(&str, &str); 2] = [
    ("setup-story.md", include_str!("../../../apps/qt-ai-cli/antigravity/workflows/setup-story.md")),
    ("translate.md", include_str!("../../../apps/qt-ai-cli/antigravity/workflows/translate.md")),
];
const PLACEHOLDERS: [&str; 2] = ["{{QT_AI}}", "{{STORY_ROOT}}"];

pub fn render(source: &str, qt_ai_command: &str, root: &str) -> String {
    source.replace("{{QT_AI}}", qt_ai_command).replace("{{STORY_ROOT}}", root)
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// `content` có phải là `template` render với giá trị nào đó (bất kỳ) cho hai placeholder — tức chưa ai sửa tay?
pub fn matches_template(template: &str, content: &str) -> bool {
    let mut pattern = String::from("^");
    let mut rest = normalize(template);
    loop {
        let next = PLACEHOLDERS
            .iter()
            .filter_map(|p| rest.find(p).map(|i| (i, p.len())))
            .min();
        match next {
            Some((index, len)) => {
                pattern.push_str(&regex::escape(&rest[..index]));
                pattern.push_str("[^\n]*?");
                rest = rest[index + len..].to_string();
            }
            None => {
                pattern.push_str(&regex::escape(&rest));
                break;
            }
        }
    }
    pattern.push('$');
    Regex::new(&pattern).map(|re| re.is_match(&normalize(content))).unwrap_or(false)
}

/// Ghi file mới; file cũ còn đúng khuôn template mà nội dung khác bản render hiện tại thì ghi đè.
/// Trả về true nếu có ghi.
fn write_or_refresh(target: &Path, source: &str, qt_ai_command: &str, root: &str) -> Result<bool> {
    let fresh = render(source, qt_ai_command, root);
    match fs::read_to_string(target) {
        Ok(existing) => {
            if normalize(&existing) == normalize(&fresh) || !matches_template(source, &existing) {
                return Ok(false);
            }
            write_text(target, &fresh)?;
            Ok(true)
        }
        Err(_) => {
            write_text(target, &fresh)?;
            Ok(true)
        }
    }
}

/// Copy template vào folder truyện. File đã sửa tay thì giữ nguyên; file còn đúng khuôn thì cập nhật
/// lệnh qt-ai/đường dẫn. Trả về tên các file đã ghi.
pub fn copy_templates(root: &Path, qt_ai_command: &str) -> Result<Vec<String>> {
    let root_text = root.display().to_string();
    let mut written = Vec::new();
    if write_or_refresh(&root.join("AGENTS.md"), AGENTS_MD, qt_ai_command, &root_text)? {
        written.push("AGENTS.md".to_string());
    }
    let workflows_dir = root.join(".agent").join("workflows");
    fs::create_dir_all(&workflows_dir).map_err(CoreError::io(&workflows_dir))?;
    for (name, source) in WORKFLOWS {
        if write_or_refresh(&workflows_dir.join(name), source, qt_ai_command, &root_text)? {
            written.push(name.to_string());
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE: &str = "Chạy `{{QT_AI}} next {{STORY_ROOT}}` rồi\nthư mục: {{STORY_ROOT}}\n";

    #[test]
    fn render_tu_may_khac_van_khop_khuon() {
        let other = render(TEMPLATE, "\"C:\\Users\\x\\qt-ai.exe\"", "C:\\truyen\\a");
        assert!(matches_template(TEMPLATE, &other));
        let npm = render(TEMPLATE, "npm --prefix D:\\qt run -s qt-ai --", "D:\\t");
        assert!(matches_template(TEMPLATE, &npm));
        assert!(matches_template(TEMPLATE, &other.replace('\n', "\r\n")));
    }

    #[test]
    fn sua_tay_thi_khong_khop() {
        let edited = render(TEMPLATE, "qt-ai", "D:\\t") + "Luật thêm của tôi\n";
        assert!(!matches_template(TEMPLATE, &edited));
        assert!(!matches_template(TEMPLATE, "khác hẳn"));
    }

    #[test]
    fn copy_templates_va_lai_file_cu_dung_khuon_giu_file_sua_tay() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let first = copy_templates(root, "\"C:\\may-khac\\qt-ai.exe\"").unwrap();
        assert_eq!(first, vec!["AGENTS.md", "setup-story.md", "translate.md"]);
        // Lần hai cùng giá trị: không ghi gì.
        assert!(copy_templates(root, "\"C:\\may-khac\\qt-ai.exe\"").unwrap().is_empty());
        // Sửa tay translate.md rồi đổi lệnh: AGENTS.md + setup-story.md vá, translate.md giữ nguyên.
        let translate = root.join(".agent").join("workflows").join("translate.md");
        let hand = fs::read_to_string(&translate).unwrap() + "\nGhi chú riêng.\n";
        fs::write(&translate, &hand).unwrap();
        let refreshed = copy_templates(root, "qt-ai").unwrap();
        assert_eq!(refreshed, vec!["AGENTS.md", "setup-story.md"]);
        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("    qt-ai <lệnh> "));
        assert!(!agents.contains("may-khac"));
        assert_eq!(fs::read_to_string(&translate).unwrap(), hand);
    }
}
