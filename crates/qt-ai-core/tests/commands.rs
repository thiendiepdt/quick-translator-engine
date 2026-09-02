//! State machine trên tempdir — tương đương bộ vitest của apps/qt-ai-cli.
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::check::run_check;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::story::StoryConfig;
use qt_ai_core::story_fs::*;
use qt_ai_core::CoreError;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Dựng folder truyện tạm với raw/ cho trước (như helpers.ts).
pub fn make_story_dir(chapters: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-test-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    for (id, text) in chapters {
        fs::write(dir.path().join("raw").join(format!("{id}.txt")), text).unwrap();
    }
    dir
}

#[test]
fn story_fs_liet_ke_chuong_theo_thu_tu_tu_nhien() {
    let dir = make_story_dir(&[("10", "十"), ("2", "二"), ("1", "一")]);
    fs::write(dir.path().join("raw").join("ghi-chu.md"), "bỏ qua").unwrap();
    let ids = list_raw_chapter_ids(&story_paths(dir.path())).unwrap();
    assert_eq!(ids, vec!["1", "2", "10"]);
}

#[test]
fn story_fs_save_load_story_atomic_kem_bak() {
    let dir = make_story_dir(&[]);
    let paths = story_paths(dir.path());
    let mut story = StoryConfig::empty();
    story.name = "A".into();
    save_story_config(&paths, &story).unwrap();
    assert!(!paths.story_json.with_extension("json.bak").exists());
    story.name = "B".into();
    save_story_config(&paths, &story).unwrap();
    assert!(paths.story_json.with_extension("json.bak").exists());
    assert!(fs::read_to_string(paths.story_json.with_extension("json.bak")).unwrap().contains("\"A\""));
    assert_eq!(load_story_config(&paths).unwrap().name, "B");
    assert!(!paths.root.join("story.json.tmp").exists());
    fs::write(&paths.story_json, "[]").unwrap();
    assert!(matches!(load_story_config(&paths), Err(qt_ai_core::CoreError::InvalidStory(_))));
    fs::write(&paths.story_json, "{ hỏng").unwrap();
    assert!(matches!(load_story_config(&paths), Err(qt_ai_core::CoreError::InvalidStory(_))));
}

#[test]
fn story_fs_state_round_trip_va_validate() {
    let dir = make_story_dir(&[]);
    let paths = story_paths(dir.path());
    let mut state = StoryState::new();
    state.chapters.insert(
        "1".into(),
        ChapterState { status: ChapterStatus::Queued, review_round: 0, reason: None, warnings: None, updated_at: 1 },
    );
    state.chapters.insert(
        "2".into(),
        ChapterState {
            status: ChapterStatus::Done,
            review_round: 2,
            reason: None,
            warnings: Some(vec!["[[1]] x".into()]),
            updated_at: 2,
        },
    );
    save_state(&paths, &state).unwrap();
    let text = fs::read_to_string(&paths.state_json).unwrap();
    assert!(
        text.contains("\"minLengthRatio\": 0.75")
            && text.contains("\"maxReviewRounds\": 3")
            && text.contains("\"reviewRound\": 2")
    );
    assert!(!text.contains("\"reason\""));
    assert_eq!(load_state(&paths).unwrap(), state);
    // settings thiếu field → fallback từng field; chương sai schema bị bỏ
    fs::write(
        &paths.state_json,
        r#"{"version":1,"settings":{"chaptersPerSession":5},"chapters":{"a":{"status":"done","reviewRound":0,"updatedAt":1},"b":{"status":"lạ","reviewRound":0,"updatedAt":1},"c":"rác"}}"#,
    )
    .unwrap();
    let loaded = load_state(&paths).unwrap();
    assert_eq!(
        loaded.settings,
        HarnessSettings { min_length_ratio: 0.75, max_review_rounds: 3, chapters_per_session: 5 }
    );
    assert_eq!(loaded.chapters.keys().collect::<Vec<_>>(), vec!["a"]);
    fs::write(&paths.state_json, "[]").unwrap();
    assert!(matches!(load_state(&paths), Err(qt_ai_core::CoreError::InvalidState(_))));
    fs::remove_file(&paths.state_json).unwrap();
    assert!(matches!(load_state(&paths), Err(qt_ai_core::CoreError::StoryNotFound(_))));
}

#[test]
fn story_fs_work_file_va_resolve_root() {
    let paths = story_paths(Path::new("x"));
    assert_eq!(work_file(&paths, "0001", WorkKind::Prompt), Path::new("x").join("work").join("0001.prompt.md"));
    assert_eq!(
        work_file(&paths, "0001", WorkKind::Glossary),
        Path::new("x").join("work").join("0001.glossary.json")
    );
    let abs = std::env::current_dir().unwrap().join("abs");
    assert_eq!(resolve_root(&abs), abs);
    assert_eq!(resolve_root(Path::new("rel")), std::env::current_dir().unwrap().join("rel"));
    let _: PathBuf = resolve_root(Path::new("."));
}

const RAW: &str = "赵静文抬头。\n\n方寸之间。";

#[test]
fn init_dung_story_state_copy_template_idempotent() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章")]);
    let root = dir.path();
    let message = run_init(root, "C:\\bin\\qt-ai.exe").unwrap();
    assert!(message.contains("2 chương (2 mới thêm vào hàng đợi)"));
    let paths = story_paths(root);
    assert!(load_story_config(&paths).unwrap().glossary["names"].is_empty());
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters.keys().collect::<Vec<_>>(), vec!["0001", "0002"]);
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Queued);
    assert_eq!(state.settings.max_review_rounds, 3);
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(
        !agents.contains("{{QT_AI}}")
            && agents.contains("C:\\bin\\qt-ai.exe")
            && agents.contains(&root.display().to_string())
    );
    let translate = fs::read_to_string(root.join(".agent").join("workflows").join("translate.md")).unwrap();
    assert!(translate.starts_with("---") && translate.contains("description:"));
    assert!(root.join(".agent").join("workflows").join("setup-story.md").exists());

    // idempotent: giữ state cũ, thêm chương mới, không đè AGENTS.md
    let mut state = load_state(&paths).unwrap();
    state.chapters.get_mut("0001").unwrap().status = ChapterStatus::Done;
    save_state(&paths, &state).unwrap();
    fs::write(root.join("raw").join("0003.txt"), "第三章").unwrap();
    fs::write(root.join("AGENTS.md"), "tự sửa").unwrap();
    run_init(root, "qt-ai").unwrap();
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert_eq!(state.chapters["0003"].status, ChapterStatus::Queued);
    assert_eq!(fs::read_to_string(root.join("AGENTS.md")).unwrap(), "tự sửa");
}

#[test]
fn next_phat_chuong_dau_prompt_du_3_phan_state_translating() {
    let dir = make_story_dir(&[("0001", RAW), ("0002", "第二章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut config = load_story_config(&paths).unwrap();
    config.glossary.get_mut("names").unwrap().insert("赵静文".into(), "Triệu Tĩnh Văn".into());
    config.glossary.get_mut("names").unwrap().insert("不出现".into(), "Không Xuất Hiện".into());
    save_story_config(&paths, &config).unwrap();

    let result = run_next(root).unwrap();
    assert_eq!(result.chapter_id, "0001");
    let prompt = fs::read_to_string(&result.prompt_path).unwrap();
    assert!(prompt.contains("dịch giả tiểu thuyết Trung Quốc"));
    assert!(prompt.contains("Triệu Tĩnh Văn"));
    assert!(!prompt.contains("Không Xuất Hiện"));
    assert!(prompt.contains("[[1]] 赵静文抬头。"));
    assert!(prompt.contains("0001.draft.md") && prompt.contains("0001.glossary.json"));
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Translating);

    // từ chối phát chương mới khi còn translating
    let err = run_next(root).unwrap_err();
    assert!(matches!(err, CoreError::InvalidState(ref m) if m.contains("0001") && m.contains("translating")));

    // mất work/prompt.md (phiên chết) → phát lại đúng chương đó, không đổi state
    fs::remove_file(&result.prompt_path).unwrap();
    let again = run_next(root).unwrap();
    assert_eq!(again.chapter_id, "0001");
    assert!(again.prompt_path.exists());
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].review_round, 0);
}

#[test]
fn next_het_hang_doi_thi_bao() {
    let dir = make_story_dir(&[]);
    run_init(dir.path(), "qt-ai").unwrap();
    let err = run_next(dir.path()).unwrap_err();
    assert!(matches!(err, CoreError::InvalidState(ref m) if m.to_lowercase().contains("không còn chương")));
}

const RAW2: &str = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const GOOD_DRAFT: &str =
    "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.";

/// init + next + ghi draft (và glossary.json nếu có) cho chương 0001.
fn story_with_draft(draft: &str, glossary_json: Option<&str>) -> TempDir {
    let dir = make_story_dir(&[("0001", RAW2)]);
    run_init(dir.path(), "qt-ai").unwrap();
    run_next(dir.path()).unwrap();
    let paths = story_paths(dir.path());
    fs::write(work_file(&paths, "0001", WorkKind::Draft), draft).unwrap();
    if let Some(json) = glossary_json {
        fs::write(work_file(&paths, "0001", WorkKind::Glossary), json).unwrap();
    }
    dir
}

#[test]
fn check_pass_khi_du_doan_sach_rule_du_dai() {
    let dir = story_with_draft(GOOD_DRAFT, None);
    let paths = story_paths(dir.path());
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.pass && result.missing.is_empty() && result.violations.is_empty() && !result.accepted_with_warnings);
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Translating);
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(work_file(&paths, "0001", WorkKind::Check)).unwrap()).unwrap();
    assert_eq!(report["pass"], true);
}

#[test]
fn check_bat_thieu_doan_va_vi_pham_sinh_review_tang_round() {
    let dir = story_with_draft("[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔 nơi xa.", None);
    let paths = story_paths(dir.path());
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(!result.pass);
    assert_eq!(result.missing, vec![2]);
    assert!(result.violations.iter().any(|v| v.message.contains("CJK")));
    let review = fs::read_to_string(result.review_path.as_ref().unwrap()).unwrap();
    assert!(review.contains("[[2]] 她沉默了很久没有说话。"));
    assert!(review.contains("CJK") && review.contains("0001.draft.md"));
    assert!(regex::Regex::new(r"\[\[1\]\][^\n]*CJK").unwrap().is_match(&review));
    assert!(review.contains("GIỮ NGUYÊN toàn bộ nhãn"));
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].review_round, 1);
    assert_eq!(result.issues[0], "[[2]] thiếu đoạn");
    assert!(result.issues[1].starts_with("[[1]] CJK còn sót"));
}

#[test]
fn check_qua_ngan_fail_theo_min_ratio() {
    let dir = story_with_draft("[[1]] Nàng nhìn.\n\n[[2]] Nàng im.", None);
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(!result.pass && result.ratio < 0.75);
    assert!(result.issues.iter().any(|i| i.starts_with("Quá ngắn")));
}

#[test]
fn check_het_vong_con_thieu_doan_thi_error() {
    let dir = story_with_draft("[[1]] 高塔", None);
    for _ in 0..3 {
        assert!(!run_check(dir.path(), "0001").unwrap().pass);
    }
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.escalated_to_error && !result.pass);
    let chapter = &load_state(&story_paths(dir.path())).unwrap().chapters["0001"];
    assert_eq!(chapter.status, ChapterStatus::Error);
    assert!(chapter.reason.as_deref().unwrap().starts_with("Quá 3 vòng review vẫn chưa đạt"));
}

#[test]
fn check_het_vong_chi_con_vi_pham_thi_pass_kem_canh_bao_accept_ghi_warnings() {
    let dir = story_with_draft(
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía 高塔 nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
        None,
    );
    let paths = story_paths(dir.path());
    for _ in 0..3 {
        assert!(!run_check(dir.path(), "0001").unwrap().pass);
    }
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.pass && result.accepted_with_warnings && !result.escalated_to_error);
    assert_eq!(result.issues.len(), 1);
    assert!(result.issues[0].starts_with("[[1]] CJK"));
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Translating);
    let accepted = run_accept(dir.path(), "0001", false).unwrap();
    assert_eq!(accepted.warnings, result.issues);
    let chapter = &load_state(&paths).unwrap().chapters["0001"];
    assert_eq!(chapter.status, ChapterStatus::Done);
    assert_eq!(chapter.warnings.as_ref().unwrap(), &result.issues);
}

#[test]
fn check_draft_mat_sach_nhan_coi_nhu_thieu_toan_bo() {
    let dir = story_with_draft("Bản dịch không có nhãn nào cả.", None);
    assert_eq!(run_check(dir.path(), "0001").unwrap().missing, vec![1, 2]);
}

#[test]
fn accept_ghi_out_sach_nhan_merge_glossary_don_work_state_done() {
    let glossary = r#"{"entries":[{"source":"赵静文","target":"Triệu Tĩnh Văn","category":"names"},{"source":"不在raw","target":"Bịa","category":"names"},{"source":"高塔","target":"không có trong dịch","category":"places"}]}"#;
    let dir = story_with_draft(GOOD_DRAFT, Some(glossary));
    let paths = story_paths(dir.path());
    let result = run_accept(dir.path(), "0001", false).unwrap();
    assert!(result.out_path.ends_with("0001.txt"));
    let out = fs::read_to_string(&result.out_path).unwrap();
    assert!(out.contains("Triệu Tĩnh Văn ngẩng đầu") && !out.contains("[["));
    assert!(out.ends_with(".\n"));
    assert_eq!(result.added_glossary, 1);
    assert!(result.warnings.is_empty());
    let story = load_story_config(&paths).unwrap();
    assert_eq!(story.glossary["names"]["赵静文"], "Triệu Tĩnh Văn");
    assert_eq!(story.auto_glossary_log.len(), 1);
    assert_eq!(story.auto_glossary_log[0].chapter, "0001");
    assert!(paths.story_json.with_extension("json.bak").exists());
    let chapter = &load_state(&paths).unwrap().chapters["0001"];
    assert_eq!(chapter.status, ChapterStatus::Done);
    assert!(chapter.warnings.is_none());
    for kind in WORK_KINDS {
        assert!(!work_file(&paths, "0001", kind).exists());
    }
}

#[test]
fn accept_check_fail_thi_tu_choi_force_thi_qua_va_ghi_warnings() {
    let dir = story_with_draft("[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔.", None);
    let err = run_accept(dir.path(), "0001", false).unwrap_err();
    assert!(matches!(err, CoreError::InvalidState(ref m) if m.contains("chưa qua check") && m.contains("--force")));
    let forced = run_accept(dir.path(), "0001", true).unwrap();
    assert!(forced.out_path.exists());
    assert!(forced.warnings.iter().any(|w| w == "[[2]] thiếu đoạn"));
}

#[test]
fn accept_khong_glossary_moi_thi_khong_ghi_de_story_json() {
    let dir = story_with_draft(GOOD_DRAFT, None);
    let paths = story_paths(dir.path());
    // story.json có field lạ do người dùng thêm tay; accept không có glossary mới thì không được xoá nó
    let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&paths.story_json).unwrap()).unwrap();
    value["ghiChuRieng"] = serde_json::json!("giữ nguyên");
    fs::write(&paths.story_json, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    run_accept(dir.path(), "0001", false).unwrap();
    assert!(fs::read_to_string(&paths.story_json).unwrap().contains("ghiChuRieng"));
}

#[test]
fn accept_auto_glossary_off_khong_merge_nhung_van_accept() {
    let glossary = r#"{"entries":[{"source":"赵静文","target":"Triệu Tĩnh Văn","category":"names"}]}"#;
    let dir = story_with_draft(GOOD_DRAFT, Some(glossary));
    let paths = story_paths(dir.path());
    let mut config = load_story_config(&paths).unwrap();
    config.auto_glossary = qt_ai_core::story::AutoGlossarySetting::Off;
    save_story_config(&paths, &config).unwrap();
    let result = run_accept(dir.path(), "0001", false).unwrap();
    assert_eq!(result.added_glossary, 0);
    assert!(load_story_config(&paths).unwrap().glossary["names"].is_empty());
}

#[test]
fn e2e_hai_chuong_glossary_hoc_tu_chuong_1_lot_vao_prompt_chuong_2() {
    let dir = make_story_dir(&[("0001", RAW2), ("0002", "第二天早上，赵静文和他们一起出发了。")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let drafts = [
        ("0001", GOOD_DRAFT),
        ("0002", "[[1]] Sáng sớm hôm sau, Triệu Tĩnh Văn cùng bọn họ lên đường xuất phát."),
    ];
    for (id, draft) in drafts {
        let next = run_next(root).unwrap();
        assert_eq!(next.chapter_id, id);
        if id == "0002" {
            assert!(fs::read_to_string(&next.prompt_path).unwrap().contains("Triệu Tĩnh Văn"));
        }
        fs::write(work_file(&paths, id, WorkKind::Draft), draft).unwrap();
        if id == "0001" {
            fs::write(
                work_file(&paths, id, WorkKind::Glossary),
                r#"{"entries":[{"source":"赵静文","target":"Triệu Tĩnh Văn","category":"names"}]}"#,
            )
            .unwrap();
        }
        assert!(run_check(root, id).unwrap().pass);
        run_accept(root, id, false).unwrap();
    }
    assert!(fs::read_to_string(paths.out_dir.join("0001.txt")).unwrap().contains("Triệu Tĩnh Văn"));
    assert!(matches!(run_next(root), Err(CoreError::InvalidState(_))));
}
