//! Bù chữ chương ngắn lúc chốt chương; phần chèn không bị Tìm & thay / ghép đoạn với raw đụng tới.
//! File test riêng vì phải đặt QT_AI_BASE_DIR (biến môi trường của cả tiến trình) trỏ tới cài đặt bù chữ.
use qt_ai_core::base::{BaseStore, BASE_DIR_ENV};
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::padding::{chapter_words, PaddingMode, PaddingSettings, PADDING_MARKER};
use qt_ai_core::replace::{find_in_story, replace_in_story, FindQuery};
use qt_ai_core::story_fs::{story_paths, work_file, WorkKind};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Một thư mục base chung cho cả tiến trình: bật chèn văn mẫu, ngưỡng 12 chữ.
fn base_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = tempfile::Builder::new().prefix("qt-ai-padding-base-").tempdir().unwrap().keep();
        let settings = PaddingSettings { mode: PaddingMode::Filler, threshold: 12, filler: "Hắn chèn thêm năm chữ.\n\nKết.".into() };
        BaseStore::at(&dir).save_padding(&settings).unwrap();
        std::env::set_var(BASE_DIR_ENV, &dir);
        dir
    })
}

fn accepted_story() -> tempfile::TempDir {
    base_dir();
    let dir = tempfile::Builder::new().prefix("qt-ai-padding-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::write(dir.path().join("raw").join("0001.txt"), "第一章\n\n他走了。").unwrap();
    run_init(dir.path(), "qt-ai").unwrap();
    run_next(dir.path()).unwrap();
    let paths = story_paths(dir.path());
    fs::write(work_file(&paths, "0001", WorkKind::Draft), "[[1]] Chương 1\n\n[[2]] Hắn đi rồi.").unwrap();
    run_accept(dir.path(), "0001", true).unwrap();
    dir
}

#[test]
fn chot_chuong_ngan_thi_chen_van_mau_sau_dong_ngan() {
    let dir = accepted_story();
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("Chương 1\n\nHắn đi rồi.\n\n{PADDING_MARKER}\n\nHắn chèn thêm năm chữ.\n\nHắn chèn thêm năm chữ.\n\nKết.\n\n{PADDING_MARKER}\n"));
    assert_eq!(chapter_words(&out), 3);
}

#[test]
fn tim_thay_loc_raw_chi_xet_ban_dich_khong_sua_phan_chen() {
    let dir = accepted_story();
    let paths = story_paths(dir.path());
    let query = FindQuery { find: "Hắn".into(), replace: "Y".into(), whole_word: true, raw_filter: "他".into(), ..FindQuery::default() };
    let found = find_in_story(&paths, &query, 100).unwrap();
    assert_eq!((found.total, found.skipped), (1, 0), "phần chèn không làm lệch số đoạn với raw");
    let plain = FindQuery { raw_filter: String::new(), ..query.clone() };
    assert_eq!(find_in_story(&paths, &plain, 100).unwrap().total, 1, "chữ trong phần chèn không tính");
    replace_in_story(&paths, &plain, &[], &[]).unwrap();
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.starts_with("Chương 1\n\nY đi rồi.\n") && out.contains("Hắn chèn thêm năm chữ."), "{out}");
}
