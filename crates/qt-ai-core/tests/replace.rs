//! Tìm & thay trong bản dịch đã chốt.
use qt_ai_core::replace::{find_in_story, replace_in_story, undo_info, undo_last_replace, FindQuery, HitRef};
use qt_ai_core::story_fs::story_paths;
use std::fs;

fn story(chapters: &[(&str, &str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-replace-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::create_dir_all(dir.path().join("out")).unwrap();
    for (id, raw, out) in chapters {
        fs::write(dir.path().join("raw").join(format!("{id}.txt")), raw).unwrap();
        fs::write(dir.path().join("out").join(format!("{id}.txt")), out).unwrap();
    }
    dir
}

fn query(find: &str, replace: &str) -> FindQuery {
    FindQuery { find: find.into(), replace: replace.into(), whole_word: true, ..FindQuery::default() }
}

fn read(dir: &tempfile::TempDir, id: &str) -> String {
    fs::read_to_string(dir.path().join("out").join(format!("{id}.txt"))).unwrap()
}

#[test]
fn tim_nguyen_tu_khong_phan_biet_hoa_thuong_kem_ngu_canh_va_raw() {
    let dir = story(&[("0001", "恩人来了。\n\n报恩。", "Ân nhân tới, thân nhân cũng tới.\n\nBáo ân.\n")]);
    let paths = story_paths(dir.path());
    let found = find_in_story(&paths, &query("ân", "x"), 100).unwrap();
    assert_eq!(found.total, 2, "không dính 'nhân', 'thân'");
    assert_eq!((found.hits[0].paragraph, found.hits[0].matched.as_str()), (0, "Ân"));
    assert_eq!(found.hits[0].after, " nhân tới, thân nhân cũng tới.");
    assert_eq!(found.hits[1].before, "Báo ");
    assert_eq!(found.hits[1].raw.as_deref(), Some("报恩。"));
    let sensitive = FindQuery { case_sensitive: true, ..query("ân", "x") };
    assert_eq!(find_in_story(&paths, &sensitive, 100).unwrap().total, 1);
    let loose = FindQuery { whole_word: false, ..query("ân", "x") };
    assert_eq!(find_in_story(&paths, &loose, 100).unwrap().total, 5, "bỏ nguyên từ thì dính cả nhân, thân");
}

#[test]
fn loc_theo_raw_pham_vi_chuong_va_cat_so_cho_tra_ve() {
    let dir = story(&[
        ("0001", "人族大军。\n\n我族之人。", "Người tộc tiến quân.\n\nNgười tộc ta.\n"),
        ("0002", "人族。\n\n多一段。", "Người tộc.\n"),
        ("0010", "人族。", "Người tộc. Người tộc.\n"),
    ]);
    let paths = story_paths(dir.path());
    let filtered = FindQuery { raw_filter: "人族".into(), ..query("người tộc", "nhân tộc") };
    let found = find_in_story(&paths, &filtered, 100).unwrap();
    assert_eq!(found.total, 3, "0001 chỉ đoạn đầu; 0010 hai chỗ");
    assert_eq!(found.skipped, 1, "0002 lệch số đoạn → bỏ qua khi lọc theo raw");
    let ranged = FindQuery { from: Some("0002".into()), to: Some("0010".into()), ..query("người tộc", "x") };
    let found = find_in_story(&paths, &ranged, 1).unwrap();
    assert_eq!((found.total, found.hits.len()), (3, 1), "tổng đủ, chỗ trả về bị cắt");
    assert_eq!(
        found.chapters.iter().map(|c| (c.chapter.as_str(), c.count)).collect::<Vec<_>>(),
        vec![("0002", 1), ("0010", 2)],
        "phạm vi theo thứ tự số: 0002 ≤ id ≤ 0010"
    );
}

#[test]
fn thay_tru_cho_bo_chon_roi_hoan_tac_bo_qua_chuong_da_sua_tiep() {
    let dir = story(&[
        ("0001", "a\n\nb", "Lục Đình Vân tới.\r\n\r\nLục Đình Vân và Lục Đình Vân.\r\n"),
        ("0002", "c", "Lục Đình Vân đi.\n"),
        ("0003", "d", "Lục Đình Vân ở lại.\n"),
    ]);
    let paths = story_paths(dir.path());
    let q = query("Lục Đình Vân", "Lục Vân Đình");
    let skip = [HitRef { chapter: "0001".into(), paragraph: 1, occurrence: 1 }];
    let done = replace_in_story(&paths, &q, &["0003".to_string()], &skip).unwrap();
    assert_eq!((done.chapters, done.occurrences), (2, 3));
    assert_eq!(read(&dir, "0001"), "Lục Vân Đình tới.\r\n\r\nLục Vân Đình và Lục Đình Vân.\r\n", "giữ CRLF, chừa chỗ bỏ chọn");
    assert_eq!(read(&dir, "0003"), "Lục Đình Vân ở lại.\n", "chương bỏ chọn giữ nguyên");
    let info = undo_info(&paths).unwrap();
    assert_eq!((info.chapters, info.occurrences, info.find.as_str()), (2, 3, "Lục Đình Vân"));

    fs::write(dir.path().join("out/0002.txt"), "Sửa tay sau đó.\n").unwrap();
    let undone = undo_last_replace(&paths).unwrap();
    assert_eq!((undone.restored, undone.conflicts.clone()), (1, vec!["0002".to_string()]));
    assert_eq!(read(&dir, "0001"), "Lục Đình Vân tới.\r\n\r\nLục Đình Vân và Lục Đình Vân.\r\n");
    assert_eq!(read(&dir, "0002"), "Sửa tay sau đó.\n", "chương đã sửa tiếp không bị đè");
    assert!(undo_info(&paths).is_none(), "hoàn tác xong thì hết");
}

#[test]
fn khong_co_cho_khop_thi_khong_ghi_va_giu_lan_hoan_tac_truoc() {
    let dir = story(&[("0001", "a", "Một hai.\n")]);
    let paths = story_paths(dir.path());
    replace_in_story(&paths, &query("Một", "1"), &[], &[]).unwrap();
    let none = replace_in_story(&paths, &query("không có", "x"), &[], &[]).unwrap();
    assert_eq!(none.occurrences, 0);
    assert_eq!(undo_info(&paths).unwrap().find, "Một");
}
