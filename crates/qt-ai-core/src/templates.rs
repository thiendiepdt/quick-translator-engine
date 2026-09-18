//! Template AGENTS.md + workflows cho Antigravity — nhúng từ apps/qt-ai-cli/antigravity để hai bản
//! TS/Rust chỉ có một nguồn. Cargo theo dõi file include_str! nên sửa template là rebuild.
//!
//! File render sẵn trong folder truyện chứa lệnh qt-ai và đường dẫn truyện; mang folder sang máy khác
//! (hoặc dời folder) là hai giá trị đó chết → agent đi tìm binary khắp đĩa. Vì thế mỗi lần init/mở truyện,
//! file nào chưa sửa tay thì render lại với template và giá trị hiện tại.
//!
//! "Chưa sửa tay" nhận ra bằng dòng dấu `<!-- qt-ai-template <fnv> -->` ghi ở cuối file: fnv của phần thân
//! lúc app ghi. Thân còn đúng fnv → app được phép ghi đè, kể cả khi template đã đổi lời (luật mới trong
//! AGENTS.md tới được truyện cũ). File không có dấu (bản cũ) thì so khuôn template hiện tại như trước.

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

const MARKER_PREFIX: &str = "<!-- qt-ai-template ";
const MARKER_SUFFIX: &str = " -->";

fn fingerprint(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in normalize(text).bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Thân file + dòng dấu fnv của thân.
fn stamp(body: &str) -> String {
    let body = if body.ends_with('\n') { body.to_string() } else { format!("{body}\n") };
    format!("{body}{MARKER_PREFIX}{:016x}{MARKER_SUFFIX}\n", fingerprint(&body))
}

/// Tách (thân, fnv ghi trong dấu) nếu dòng cuối là dấu; không có dấu → None.
fn split_marker(content: &str) -> Option<(String, u64)> {
    let text = normalize(content);
    let trimmed = text.trim_end_matches('\n');
    let (body, last) = trimmed.rsplit_once('\n')?;
    let hex = last.strip_prefix(MARKER_PREFIX)?.strip_suffix(MARKER_SUFFIX)?;
    let hash = u64::from_str_radix(hex, 16).ok()?;
    Some((format!("{body}\n"), hash))
}

/// Ghi file mới (kèm dấu). File cũ chưa sửa tay — thân còn đúng fnv trong dấu, hoặc bản cũ không dấu
/// nhưng còn đúng khuôn template — mà khác bản render hiện tại thì ghi đè. Trả về true nếu có ghi.
fn write_or_refresh(target: &Path, source: &str, qt_ai_command: &str, root: &str) -> Result<bool> {
    let fresh = render(source, qt_ai_command, root);
    let stamped = stamp(&fresh);
    match fs::read_to_string(target) {
        Ok(existing) => {
            let untouched = match split_marker(&existing) {
                Some((body, hash)) => {
                    if fingerprint(&body) != hash {
                        return Ok(false);
                    }
                    normalize(&body) == normalize(&fresh)
                }
                None => {
                    if !matches_template(source, &existing) {
                        return Ok(false);
                    }
                    false
                }
            };
            if untouched {
                return Ok(false);
            }
            write_text(target, &stamped)?;
            Ok(true)
        }
        Err(_) => {
            write_text(target, &stamped)?;
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
        assert!(agents.trim_end().ends_with(" -->"), "file app ghi có dòng dấu ở cuối");
        assert_eq!(fs::read_to_string(&translate).unwrap(), hand);
    }

    const TEMPLATE_V2: &str = "Chạy `{{QT_AI}} next {{STORY_ROOT}}` rồi\nthư mục: {{STORY_ROOT}}\nLuật mới: thử lại 3 lượt.\n";

    #[test]
    fn template_doi_loi_thi_file_chua_sua_tay_duoc_lam_moi_file_sua_giua_than_thi_giu() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("AGENTS.md");
        assert!(write_or_refresh(&target, TEMPLATE, "qt-ai", "D:\\t").unwrap());
        // Template đổi lời → khuôn cũ không còn khớp, nhưng dấu fnv nói file chưa ai sửa → ghi đè.
        assert!(write_or_refresh(&target, TEMPLATE_V2, "qt-ai", "D:\\t").unwrap());
        let text = fs::read_to_string(&target).unwrap();
        assert!(text.contains("Luật mới: thử lại 3 lượt."));
        assert!(!write_or_refresh(&target, TEMPLATE_V2, "qt-ai", "D:\\t").unwrap(), "cùng bản thì không ghi");
        // Sửa giữa thân (dấu vẫn còn ở cuối) → fnv lệch → giữ nguyên dù template đổi tiếp.
        let edited = text.replace("Luật mới", "Luật tôi sửa");
        fs::write(&target, &edited).unwrap();
        assert!(!write_or_refresh(&target, TEMPLATE, "qt-ai", "D:\\t").unwrap());
        assert_eq!(fs::read_to_string(&target).unwrap(), edited);
    }

    #[test]
    fn file_cu_khong_dau_con_dung_khuon_thi_duoc_dong_dau_khi_lam_moi() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("AGENTS.md");
        fs::write(&target, render(TEMPLATE, "qt-ai", "D:\\t")).unwrap();
        assert!(write_or_refresh(&target, TEMPLATE, "qt-ai", "D:\\t").unwrap(), "chưa có dấu → ghi lại kèm dấu");
        assert!(split_marker(&fs::read_to_string(&target).unwrap()).is_some());
        // Bản cũ không dấu mà template đã đổi lời → không nhận ra là "chưa sửa", giữ nguyên (giới hạn đã biết).
        let old = dir.path().join("translate.md");
        fs::write(&old, render(TEMPLATE, "qt-ai", "D:\\t")).unwrap();
        assert!(!write_or_refresh(&old, TEMPLATE_V2, "qt-ai", "D:\\t").unwrap());
    }
}
