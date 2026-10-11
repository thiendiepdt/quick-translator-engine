//! Rà ngược bản dịch theo một mục glossary.
use qt_ai_core::backfill::{backfill_glossary, scan_glossary_usage, Backfill};
use qt_ai_core::story_fs::story_paths;
use std::fs;

fn story(chapters: &[(&str, &str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-backfill-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::create_dir_all(dir.path().join("out")).unwrap();
    for (id, raw, out) in chapters {
        fs::write(dir.path().join("raw").join(format!("{id}.txt")), raw).unwrap();
        fs::write(dir.path().join("out").join(format!("{id}.txt")), out).unwrap();
    }
    dir
}

#[test]
fn quet_doan_raw_co_khoa_ma_ban_dich_thieu_cach_dich_hien_tai() {
    let dir = story(&[
        ("0001", "陆云停来了。\n\n他笑了。\n\n陆云停走了。", "Lục Đình Vân tới.\n\nHắn cười.\n\nLục Vân Đình đi rồi.\n"),
        ("0002", "陆云停。\n\n多一段。", "Chỉ một đoạn.\n"),
    ]);
    let scan = scan_glossary_usage(&story_paths(dir.path()), "陆云停", "Lục Vân Đình").unwrap();
    assert_eq!(scan.skipped, 1, "0002 lệch số đoạn → bỏ qua");
    assert_eq!(scan.hits.len(), 1);
    assert_eq!((scan.hits[0].chapter.as_str(), scan.hits[0].paragraph), ("0001", 0));
    assert_eq!(scan.hits[0].text, "Lục Đình Vân tới.");
}

#[test]
fn thay_chi_trong_doan_co_khoa_xem_truoc_khong_ghi() {
    let out = "Lục Đình Vân tới.\r\n\r\nLục Đình Vân là tên khác ở đoạn không có khoá.\r\n";
    let dir = story(&[("0001", "陆云停来了。\n\n别人。", out)]);
    let paths = story_paths(dir.path());
    let preview = backfill_glossary(&paths, "陆云停", "Lục Đình Vân", "Lục Vân Đình", false).unwrap();
    assert_eq!(preview, Backfill { chapters: 1, paragraphs: 1, occurrences: 1, skipped: 0 });
    assert_eq!(fs::read_to_string(dir.path().join("out/0001.txt")).unwrap(), out, "xem trước không ghi");

    backfill_glossary(&paths, "陆云停", "Lục Đình Vân", "Lục Vân Đình", true).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("out/0001.txt")).unwrap(),
        "Lục Vân Đình tới.\r\n\r\nLục Đình Vân là tên khác ở đoạn không có khoá.\r\n",
        "giữ xuống dòng CRLF, đoạn không có khoá giữ nguyên"
    );
}

#[test]
fn dich_moi_chua_cum_cu_thi_khong_thay_chong() {
    let dir = story(&[("0001", "陆云停与云停。", "Lục Vân Đình và Vân Đình.\n")]);
    let paths = story_paths(dir.path());
    let done = backfill_glossary(&paths, "陆云停", "Vân Đình", "Lục Vân Đình", true).unwrap();
    assert_eq!(done.occurrences, 1);
    assert_eq!(fs::read_to_string(dir.path().join("out/0001.txt")).unwrap(), "Lục Vân Đình và Lục Vân Đình.\n");
    let again = backfill_glossary(&paths, "陆云停", "Vân Đình", "Lục Vân Đình", true).unwrap();
    assert_eq!(again.occurrences, 0, "chạy lại không đổi gì");
}
