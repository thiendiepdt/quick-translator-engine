//! Template AGENTS.md + workflows cho Antigravity — nhúng từ apps/qt-ai-cli/antigravity để hai bản
//! TS/Rust chỉ có một nguồn. Cargo theo dõi file include_str! nên sửa template là rebuild.
//!
//! File render sẵn trong folder truyện chứa lệnh qt-ai và đường dẫn truyện; mang folder sang máy khác
//! (hoặc dời folder) là hai giá trị đó chết → agent đi tìm binary khắp đĩa. Vì thế mỗi lần init/mở truyện,
//! file nào chưa sửa tay thì render lại với template và giá trị hiện tại.
//!
//! "Chưa sửa tay" nhận ra bằng dòng dấu `<!-- qt-ai-template <fnv> -->` ghi ở cuối file: fnv của phần thân
//! lúc app ghi. Thân còn đúng fnv → app được phép ghi đè, kể cả khi template đã đổi lời (luật mới trong
//! AGENTS.md tới được truyện cũ). File không có dấu (app trước 1.0.6 ghi) thì so khuôn template hiện tại
//! và mọi bản cũ trong `templates/legacy/` — khớp bản nào cũng là chưa sửa tay → render lại kèm dấu.
//! Đổi template từ nay không cần thêm bản legacy nữa (file đã có dấu); folder đó chỉ để đón truyện cũ.

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

/// Bản template cũ chưa có dấu, theo tên file đích. v0 = bản pilot trước khi vào git.
const LEGACY_AGENTS: &[&str] = &[
    include_str!("../templates/legacy/AGENTS.v1.md"),
    include_str!("../templates/legacy/AGENTS.v2.md"),
];
const LEGACY_WORKFLOWS: [(&str, &[&str]); 2] = [
    (
        "setup-story.md",
        &[
            include_str!("../templates/legacy/setup-story.v0.md"),
            include_str!("../templates/legacy/setup-story.v1.md"),
        ],
    ),
    ("translate.md", &[include_str!("../templates/legacy/translate.v1.md")]),
];

fn legacy_for(name: &str) -> &'static [&'static str] {
    LEGACY_WORKFLOWS.iter().find(|(n, _)| *n == name).map(|(_, l)| *l).unwrap_or(&[])
}

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
/// nhưng còn đúng khuôn template hiện tại hay một bản trong `legacy` — mà khác bản render hiện tại thì
/// ghi đè. Trả về true nếu có ghi.
fn write_or_refresh(target: &Path, source: &str, legacy: &[&str], qt_ai_command: &str, root: &str) -> Result<bool> {
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
                    let known = matches_template(source, &existing)
                        || legacy.iter().any(|old| matches_template(old, &existing));
                    if !known {
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
    if write_or_refresh(&root.join("AGENTS.md"), AGENTS_MD, LEGACY_AGENTS, qt_ai_command, &root_text)? {
        written.push("AGENTS.md".to_string());
    }
    let workflows_dir = root.join(".agent").join("workflows");
    fs::create_dir_all(&workflows_dir).map_err(CoreError::io(&workflows_dir))?;
    for (name, source) in WORKFLOWS {
        if write_or_refresh(&workflows_dir.join(name), source, legacy_for(name), qt_ai_command, &root_text)? {
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
        assert!(write_or_refresh(&target, TEMPLATE, &[], "qt-ai", "D:\\t").unwrap());
        // Template đổi lời → khuôn cũ không còn khớp, nhưng dấu fnv nói file chưa ai sửa → ghi đè.
        assert!(write_or_refresh(&target, TEMPLATE_V2, &[], "qt-ai", "D:\\t").unwrap());
        let text = fs::read_to_string(&target).unwrap();
        assert!(text.contains("Luật mới: thử lại 3 lượt."));
        assert!(!write_or_refresh(&target, TEMPLATE_V2, &[], "qt-ai", "D:\\t").unwrap(), "cùng bản thì không ghi");
        // Sửa giữa thân (dấu vẫn còn ở cuối) → fnv lệch → giữ nguyên dù template đổi tiếp.
        let edited = text.replace("Luật mới", "Luật tôi sửa");
        fs::write(&target, &edited).unwrap();
        assert!(!write_or_refresh(&target, TEMPLATE, &[], "qt-ai", "D:\\t").unwrap());
        assert_eq!(fs::read_to_string(&target).unwrap(), edited);
    }

    #[test]
    fn file_cu_khong_dau_con_dung_khuon_thi_duoc_dong_dau_khi_lam_moi() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("AGENTS.md");
        fs::write(&target, render(TEMPLATE, "qt-ai", "D:\\t")).unwrap();
        assert!(write_or_refresh(&target, TEMPLATE, &[], "qt-ai", "D:\\t").unwrap(), "chưa có dấu → ghi lại kèm dấu");
        assert!(split_marker(&fs::read_to_string(&target).unwrap()).is_some());
        // Bản cũ không dấu mà template đã đổi lời: không có trong legacy thì coi là sửa tay, giữ nguyên;
        // khai báo trong legacy thì nhận ra là chưa sửa → render bản mới kèm dấu.
        let old = dir.path().join("translate.md");
        fs::write(&old, render(TEMPLATE, "qt-ai", "D:\\t")).unwrap();
        assert!(!write_or_refresh(&old, TEMPLATE_V2, &[], "qt-ai", "D:\\t").unwrap());
        assert!(write_or_refresh(&old, TEMPLATE_V2, &[TEMPLATE], "\"C:\\x\\qt-ai.exe\"", "C:\\b").unwrap());
        let text = fs::read_to_string(&old).unwrap();
        assert!(text.contains("Luật mới: thử lại 3 lượt.") && text.contains("C:\\b"));
        assert!(split_marker(&text).is_some());
    }

    #[test]
    fn truyen_tao_boi_app_truoc_1_0_6_duoc_lam_moi_tu_moi_ban_legacy_that() {
        // Mỗi bản legacy thật render với giá trị máy khác → copy_templates phải nhận ra và ghi đè cả 3 file.
        let cases: Vec<(&str, &str)> = LEGACY_AGENTS
            .iter()
            .map(|old| ("AGENTS.md", *old))
            .chain(LEGACY_WORKFLOWS.iter().flat_map(|(name, list)| list.iter().map(move |old| (*name, *old))))
            .collect();
        assert_eq!(cases.len(), 5);
        for (name, old) in cases {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            fs::create_dir_all(root.join(".agent").join("workflows")).unwrap();
            let target = if name == "AGENTS.md" { root.join(name) } else { root.join(".agent").join("workflows").join(name) };
            let rendered = render(old, "npm --prefix D:\\qt run -s qt-ai --", "/home/x/books/a");
            // File legacy trong repo có thể đã CRLF (autocrlf) → chuẩn hoá trước rồi mới giả CRLF của máy Windows.
            fs::write(&target, normalize(&rendered).replace('\n', "\r\n")).unwrap();
            let written = copy_templates(root, "qt-ai").unwrap();
            assert!(written.iter().any(|w| w == name), "{name}: {written:?}");
            let text = fs::read_to_string(&target).unwrap();
            assert!(!text.contains("/home/x/books/a"), "{name} vẫn giữ đường dẫn cũ");
            assert!(split_marker(&text).is_some(), "{name} chưa có dấu");
            assert!(copy_templates(root, "qt-ai").unwrap().is_empty(), "{name}: lần hai không ghi gì");
        }
    }

    #[test]
    fn legacy_khong_trung_ban_hien_tai_va_khong_trung_nhau() {
        let all: Vec<&str> = LEGACY_AGENTS.iter().copied().chain([AGENTS_MD]).collect();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(normalize(a), normalize(b), "AGENTS legacy trùng");
            }
        }
        for (name, source) in WORKFLOWS {
            let list = legacy_for(name);
            for (i, old) in list.iter().enumerate() {
                assert_ne!(normalize(old), normalize(source), "{name} legacy trùng bản hiện tại");
                for other in &list[i + 1..] {
                    assert_ne!(normalize(old), normalize(other), "{name} legacy trùng nhau");
                }
            }
        }
    }
}
