//! Tiêu đề chương lặp: chốt lúc accept, khảo sát + chuẩn hoá truyện đã dịch.
use std::collections::HashMap;
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::commands::reset::run_reset;
use qt_ai_core::story_fs::{load_state, story_paths, work_file, StoryPaths, WorkKind};
use qt_ai_core::titles::{apply_titles, load_titles, survey_titles, tidy_chapter_title};
use std::fs;
use std::path::Path;

const RAW: [(&str, &str); 6] = [
    ("0001", "第1章 长风原之战（一）"),
    ("0002", "第2章 长风原之战（二）"),
    ("0003", "第3章 长风原之战（完）"),
    ("0004", "第4章 夜雨"),
    ("0005", "第5章 清宫炮灰小宫女1"),
    ("0006", "第6章 清宫炮灰小宫女2"),
];

fn story() -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-titles-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    for (id, title) in RAW {
        fs::write(dir.path().join("raw").join(format!("{id}.txt")), format!("{title}\n\n她抬起头。\n")).unwrap();
    }
    run_init(dir.path(), "qt-ai").unwrap();
    dir
}

/// Phát chương kế, ghi nháp có dòng tiêu đề `title`, chốt (force: test không quan tâm rule kiểm tra).
fn translate(root: &Path, expect_id: &str, title: &str) {
    let next = run_next(root).unwrap();
    assert_eq!(next.chapter_id, expect_id);
    let paths = story_paths(root);
    fs::write(work_file(&paths, expect_id, WorkKind::Draft), format!("[[1]] {title}\n\n[[2]] Nàng ngẩng đầu.\n")).unwrap();
    run_accept(root, expect_id, true).unwrap();
}

fn title_of(paths: &StoryPaths, id: &str) -> String {
    let text = fs::read_to_string(paths.out_dir.join(format!("{id}.txt"))).unwrap();
    assert!(text.ends_with("\n\nNàng ngẩng đầu.\n"), "thân bài giữ nguyên: {text:?}");
    text.lines().next().unwrap().to_string()
}

#[test]
fn accept_chot_ten_lan_dau_va_ep_cac_chuong_cung_ten() {
    let dir = story();
    let root = dir.path();
    let paths = story_paths(root);
    translate(root, "0001", "Chương 1: Trận chiến Trường Phong Nguyên (Một)");
    translate(root, "0002", "Chương 2: Đại chiến đồng Trường Phong (hai)");
    translate(root, "0003", "Chương 3: Trận Trường Phong Nguyên (kết)");
    translate(root, "0004", "Chương 4: Mưa đêm");
    translate(root, "0005", "Chương 5: Tiểu cung nữ pháo hôi Thanh cung 1");
    translate(root, "0006", "Chương 6: Cung nữ nhỏ bia đỡ đạn cung nhà Thanh 2");

    assert_eq!(title_of(&paths, "0001"), "Chương 1: Trận chiến Trường Phong Nguyên (1)");
    assert_eq!(title_of(&paths, "0002"), "Chương 2: Trận chiến Trường Phong Nguyên (2)");
    assert_eq!(title_of(&paths, "0003"), "Chương 3: Trận chiến Trường Phong Nguyên (hết)");
    assert_eq!(title_of(&paths, "0004"), "Chương 4: Mưa đêm");
    assert_eq!(title_of(&paths, "0005"), "Chương 5: Tiểu cung nữ pháo hôi Thanh cung 1");
    assert_eq!(title_of(&paths, "0006"), "Chương 6: Tiểu cung nữ pháo hôi Thanh cung 2");

    let book = load_titles(&paths);
    assert_eq!(
        book.titles.iter().map(|(source, target)| (source.as_str(), target.as_str())).collect::<Vec<_>>(),
        vec![("长风原之战", "Trận chiến Trường Phong Nguyên"), ("夜雨", "Mưa đêm"), ("清宫炮灰小宫女", "Tiểu cung nữ pháo hôi Thanh cung")]
    );
    assert!(!root.join("titles.json.tmp").exists());

    // Reset truyện: sổ tiêu đề của lần dịch cũ cất sang .bak.
    run_reset(root, "qt-ai").unwrap();
    assert!(!paths.titles_json.exists() && root.join("titles.json.bak").exists());
}

#[test]
fn tieu_de_mot_dinh_dang_va_bo_loi_xin_phieu() {
    assert_eq!(tidy_chapter_title("Chương 146 Hoàn thành nhiệm vụ"), "Chương 146: Hoàn thành nhiệm vụ");
    assert_eq!(tidy_chapter_title("Chương 12. Mưa đêm"), "Chương 12: Mưa đêm");
    assert_eq!(tidy_chapter_title("Chương 12 - Mưa đêm"), "Chương 12: Mưa đêm");
    assert_eq!(tidy_chapter_title("Chương 467-468：Hợp chương"), "Chương 467-468: Hợp chương");
    assert_eq!(tidy_chapter_title("Chương 99: Tên xấu xa! (Cầu xin đặt mua!!)"), "Chương 99: Tên xấu xa!");
    assert_eq!(
        tidy_chapter_title("Chương 70: Quầy hàng quỷ【Thêm chương cho “Đạo hàm bậc một”】(cầu nguyệt phiếu!)"),
        "Chương 70: Quầy hàng quỷ"
    );
    assert_eq!(tidy_chapter_title("Chương 5: (Thêm chương)"), "Chương 5");
    // Đã đúng, không có tên, ngoặc là một phần tên, không phải tiêu đề → giữ nguyên.
    for line in ["Chương 4: Mưa đêm", "Chương 7", "Chương 8: Ngoại truyện (1)", "Chương 12.5 Lỡ hẹn", "Xin nghỉ một ngày nha..."] {
        assert_eq!(tidy_chapter_title(line), line);
    }
}

#[test]
fn accept_sua_dinh_dang_tieu_de_truoc_khi_ghi_so() {
    let dir = story();
    let root = dir.path();
    let paths = story_paths(root);
    translate(root, "0001", "Chương 1 Trận chiến Trường Phong Nguyên (Một) (Cầu phiếu tháng!)");
    assert_eq!(title_of(&paths, "0001"), "Chương 1: Trận chiến Trường Phong Nguyên (1)");
    assert_eq!(load_titles(&paths).titles["长风原之战"], "Trận chiến Trường Phong Nguyên");
}

#[test]
fn khao_sat_va_chuan_hoa_truyen_da_dich_theo_da_so_hoac_ban_nguoi_dung_chot() {
    let dir = story();
    let root = dir.path();
    let paths = story_paths(root);
    translate(root, "0001", "Chương 1: Trận chiến Trường Phong Nguyên (1)");
    translate(root, "0002", "Chương 2: Trận chiến Trường Phong Nguyên (2)");
    translate(root, "0003", "Chương 3: Trận chiến Trường Phong Nguyên (hết)");
    translate(root, "0004", "Chương 4: Mưa đêm");
    translate(root, "0005", "Chương 5: Tiểu cung nữ pháo hôi Thanh cung 1");
    // Giả truyện dịch từ trước khi có sổ: bỏ sổ, viết lại vài dòng tiêu đề cho lệch nhau. 0006 chưa dịch.
    fs::remove_file(&paths.titles_json).unwrap();
    let rewrite = |id: &str, title: &str| {
        fs::write(paths.out_dir.join(format!("{id}.txt")), format!("{title}\n\nNàng ngẩng đầu.\n")).unwrap();
    };
    rewrite("0002", "Chương 2: Trận chiến Trường Phong Nguyên (hai)");
    rewrite("0003", "Chương 3: Đại chiến đồng Trường Phong (kết)");

    let state = load_state(&paths).unwrap();
    let (survey, fixes) = survey_titles(&paths, &state, &load_titles(&paths)).unwrap();
    // "夜雨" chỉ có một chương nên không thành nhóm.
    assert_eq!(survey.groups.iter().map(|g| g.source.as_str()).collect::<Vec<_>>(), vec!["长风原之战", "清宫炮灰小宫女"]);
    let battle = &survey.groups[0];
    assert_eq!((battle.chapters, battle.translated, battle.pending, battle.settled), (3, 3, 2, false));
    assert_eq!(battle.target, "Trận chiến Trường Phong Nguyên");
    assert_eq!(
        battle.variants.iter().map(|v| (v.text.as_str(), v.count)).collect::<Vec<_>>(),
        vec![("Trận chiến Trường Phong Nguyên", 2), ("Đại chiến đồng Trường Phong", 1)]
    );
    let maid = &survey.groups[1];
    assert_eq!((maid.chapters, maid.translated, maid.pending), (2, 1, 0));
    assert_eq!(fixes.iter().map(|f| f.chapter_id.as_str()).collect::<Vec<_>>(), vec!["0002", "0003"]);
    assert_eq!(title_of(&paths, "0003"), "Chương 3: Đại chiến đồng Trường Phong (kết)", "khảo sát không ghi gì");

    // Áp dụng không kèm bản chốt nào: theo đa số, đuôi về đúng luật.
    assert_eq!(apply_titles(&paths, &state, &HashMap::new()).unwrap(), 2);
    assert_eq!(title_of(&paths, "0002"), "Chương 2: Trận chiến Trường Phong Nguyên (2)");
    assert_eq!(title_of(&paths, "0003"), "Chương 3: Trận chiến Trường Phong Nguyên (hết)");
    assert_eq!(load_titles(&paths).titles["长风原之战"], "Trận chiến Trường Phong Nguyên");
    let (after, none) = survey_titles(&paths, &state, &load_titles(&paths)).unwrap();
    assert!(none.is_empty() && after.groups[0].settled && after.groups[0].variants.len() == 1);

    // Người dùng chốt bản khác (kể cả cho tên chưa dịch xong): mọi chương đã dịch đổi theo, chương sau dùng luôn.
    let mut targets = HashMap::new();
    targets.insert("长风原之战".to_string(), " Trận Trường Phong Nguyên ".to_string());
    targets.insert("清宫炮灰小宫女".to_string(), "Cung nữ pháo hôi Thanh cung".to_string());
    targets.insert("夜雨".to_string(), "  ".to_string());
    assert_eq!(apply_titles(&paths, &state, &targets).unwrap(), 4);
    assert_eq!(title_of(&paths, "0001"), "Chương 1: Trận Trường Phong Nguyên (1)");
    assert_eq!(title_of(&paths, "0003"), "Chương 3: Trận Trường Phong Nguyên (hết)");
    assert_eq!(title_of(&paths, "0005"), "Chương 5: Cung nữ pháo hôi Thanh cung 1");
    assert_eq!(title_of(&paths, "0004"), "Chương 4: Mưa đêm");
    translate(root, "0006", "Chương 6: Tiểu cung nữ pháo hôi Thanh cung 2");
    assert_eq!(title_of(&paths, "0006"), "Chương 6: Cung nữ pháo hôi Thanh cung 2");
}
