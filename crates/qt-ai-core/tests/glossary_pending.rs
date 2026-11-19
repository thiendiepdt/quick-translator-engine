//! Hàng chờ "nghi trùng" của glossary: người dùng chọn dùng tên cũ / tên mới / bỏ qua.

use qt_ai_core::commands::init::run_init;
use qt_ai_core::glossary_pending::{load_pending, resolve_pending, save_pending, PendingChoice, PendingEntry, SimilarKind};
use qt_ai_core::story_fs::{load_story_config, save_story_config, story_paths};
use std::fs;

fn story_with_twins() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::write(dir.path().join("raw").join("0001.txt"), "萧诧笑了。").unwrap();
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    let names = story.glossary.entry("names".to_string()).or_default();
    names.insert("萧诧".into(), "Tiêu Sá".into());
    names.insert("萧姹".into(), "Tiêu Xá".into());
    save_story_config(&paths, &story).unwrap();
    save_pending(
        &paths,
        &[PendingEntry {
            source: "萧姹".into(),
            target: "Tiêu Xá".into(),
            category: "names".into(),
            chapter: "0002".into(),
            similar_source: "萧诧".into(),
            similar_target: "Tiêu Sá".into(),
            similar_category: "names".into(),
            kind: SimilarKind::Homophone,
            source_count: 7,
            similar_count: 955,
        }],
    )
    .unwrap();
    dir
}

#[test]
fn dung_ten_cu_doi_muc_moi_va_goi_y_ra_chuong_cu() {
    let dir = story_with_twins();
    let paths = story_paths(dir.path());
    let hint = resolve_pending(&paths, "萧姹", PendingChoice::KeepOld).unwrap().unwrap();
    assert_eq!((hint.key.as_str(), hint.from.as_str(), hint.to.as_str()), ("萧姹", "Tiêu Xá", "Tiêu Sá"));
    let story = load_story_config(&paths).unwrap();
    assert_eq!(story.glossary["names"]["萧姹"], "Tiêu Sá");
    assert_eq!(story.glossary["names"]["萧诧"], "Tiêu Sá");
    assert!(load_pending(&paths).is_empty());
    assert!(!dir.path().join("glossary-pending.json").exists(), "hết câu hỏi thì xoá file");
}

#[test]
fn dung_ten_moi_doi_muc_cu() {
    let dir = story_with_twins();
    let paths = story_paths(dir.path());
    let hint = resolve_pending(&paths, "萧姹", PendingChoice::UseNew).unwrap().unwrap();
    assert_eq!((hint.key.as_str(), hint.to.as_str()), ("萧诧", "Tiêu Xá"));
    assert_eq!(load_story_config(&paths).unwrap().glossary["names"]["萧诧"], "Tiêu Xá");
}

#[test]
fn hai_thu_khac_nhau_chi_bo_cau_hoi() {
    let dir = story_with_twins();
    let paths = story_paths(dir.path());
    assert!(resolve_pending(&paths, "萧姹", PendingChoice::Dismiss).unwrap().is_none());
    let story = load_story_config(&paths).unwrap();
    assert_eq!(story.glossary["names"]["萧姹"], "Tiêu Xá");
    assert!(load_pending(&paths).is_empty());
    assert!(resolve_pending(&paths, "萧姹", PendingChoice::Dismiss).is_err(), "đã trả lời rồi thì báo lỗi");
}
