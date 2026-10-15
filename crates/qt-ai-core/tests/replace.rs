//! Tìm & thay trong bản dịch đã chốt.
use qt_ai_core::replace::{
    find_in_story, replace_batch_in_story, replace_in_story, undo_info, undo_last_replace, FindQuery, HitRef, ReplaceJob,
};
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

fn job(find: &str, replace: &str) -> ReplaceJob {
    ReplaceJob { query: FindQuery { case_sensitive: true, ..query(find, replace) }, excluded_chapters: vec![], excluded_hits: vec![] }
}

#[test]
fn thay_nhieu_cap_mot_lan_hoan_tac_chung() {
    let dir = story(&[
        ("0001", "a", "Con khỉ Việt Nam kêu, Việt Nam xa.\n"),
        ("0002", "b", "Giao Chỉ gần Việt Nam.\n"),
        ("0003", "c", "Không dính gì.\n"),
    ]);
    let paths = story_paths(dir.path());
    let jobs = [
        job("khỉ Việt Nam", "khỉ Thái Lan"),
        job("không có", "x"),
        job("Việt Nam", "Thái Lan"),
        job("Giao Chỉ", "Chiêm Thành"),
    ];
    let done = replace_batch_in_story(&paths, jobs.into_iter().map(Ok)).unwrap();
    assert_eq!((done.chapters, done.occurrences), (2, 4), "số chương không đếm trùng");
    assert_eq!(read(&dir, "0001"), "Con khỉ Thái Lan kêu, Thái Lan xa.\n");
    assert_eq!(read(&dir, "0002"), "Chiêm Thành gần Thái Lan.\n");
    let info = undo_info(&paths).unwrap();
    assert_eq!(
        (info.find.as_str(), info.replace.as_str(), info.chapters, info.occurrences),
        ("khỉ Việt Nam; Việt Nam; Giao Chỉ", "khỉ Thái Lan; Thái Lan; Chiêm Thành", 2, 4),
        "chỉ ghi các cặp có thay"
    );
    let undone = undo_last_replace(&paths).unwrap();
    assert_eq!((undone.restored, undone.conflicts.len()), (2, 0));
    assert_eq!(read(&dir, "0001"), "Con khỉ Việt Nam kêu, Việt Nam xa.\n", "về bản trước cả lượt");
    assert_eq!(read(&dir, "0002"), "Giao Chỉ gần Việt Nam.\n");
    assert!(undo_info(&paths).is_none());
}

#[test]
fn thay_nhieu_cap_dung_job_sau_dua_tren_ban_da_thay_cua_job_truoc() {
    let dir = story(&[("0001", "a", "khỉ Việt Nam, Việt Nam, Việt Nam.\n")]);
    let paths = story_paths(dir.path());
    let first = job("khỉ Việt Nam", "khỉ Thái Lan");
    let mut built = 0;
    let jobs = [first.query.clone(), job("Việt Nam", "Thái Lan").query].into_iter().map(|query| {
        built += 1;
        if built == 1 {
            return Ok(first.clone());
        }
        // Job 2 dựng lúc job 1 đã ghi xong: chỉ còn 2 chỗ "Việt Nam", bỏ chỗ thứ 2 (occurrence 1).
        let found = find_in_story(&paths, &query, usize::MAX)?;
        assert_eq!(found.total, 2, "thấy bản đã thay của job trước");
        let skip = HitRef { chapter: "0001".into(), paragraph: 0, occurrence: 1 };
        Ok(ReplaceJob { query, excluded_chapters: vec![], excluded_hits: vec![skip] })
    });
    let done = replace_batch_in_story(&paths, jobs).unwrap();
    assert_eq!(done.occurrences, 2);
    assert_eq!(read(&dir, "0001"), "khỉ Thái Lan, Thái Lan, Việt Nam.\n");
}

#[test]
fn thay_nhieu_cap_khong_thay_gi_thi_giu_lan_hoan_tac_truoc() {
    let dir = story(&[("0001", "a", "Một hai.\n")]);
    let paths = story_paths(dir.path());
    replace_in_story(&paths, &query("Một", "1"), &[], &[]).unwrap();
    let none = replace_batch_in_story(&paths, [job("không có", "x"), job("hai", "hai")].into_iter().map(Ok)).unwrap();
    assert_eq!((none.chapters, none.occurrences), (0, 0));
    assert_eq!(undo_info(&paths).unwrap().find, "Một");
}

#[test]
fn thay_nhieu_cap_cum_thay_rong_van_giu_dau_phan_cach() {
    let dir = story(&[("0001", "a", "Ồ, Một hai.\n")]);
    let paths = story_paths(dir.path());
    let jobs = [job("Ồ, ", ""), job("Một", "1"), job("hai", "")];
    replace_batch_in_story(&paths, jobs.into_iter().map(Ok)).unwrap();
    assert_eq!(read(&dir, "0001"), "1 .\n");
    let info = undo_info(&paths).unwrap();
    assert_eq!((info.find.as_str(), info.replace.as_str()), ("Ồ, ; Một; hai", "; 1; "), "find/replace thẳng hàng");
}
