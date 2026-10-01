//! State machine trên tempdir — tương đương bộ vitest của apps/qt-ai-cli.
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::check::run_check;
use qt_ai_core::commands::delete::run_delete;
use qt_ai_core::commands::export::{run_export, ExportOptions};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::commands::retry::{retry_backup_path, run_retry, run_retry_ids, run_retry_range};
use qt_ai_core::commands::skip::run_skip;
use qt_ai_core::commands::status::run_status;
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
fn retry_range_dich_lai_khoang_bo_qua_queued_giu_bak_va_all() {
    let dir = make_story_dir(&[("0001", "一"), ("0002", "二"), ("0003", "三"), ("0004", "四")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    for (id, status) in [("0001", ChapterStatus::Done), ("0002", ChapterStatus::Error), ("0004", ChapterStatus::Done)] {
        state.chapters.get_mut(id).unwrap().status = status;
    }
    save_state(&paths, &state).unwrap();
    fs::create_dir_all(&paths.out_dir).unwrap();
    fs::write(paths.out_dir.join("0001.txt"), "dịch 1").unwrap();
    fs::write(paths.out_dir.join("0004.txt"), "dịch 4").unwrap();

    let outcome = run_retry_range(root, Some("0001"), Some("0003")).unwrap();
    assert_eq!(outcome.retried, vec!["0001", "0002"]);
    assert_eq!(outcome.backed_up, vec!["0001"]);
    assert_eq!(outcome.already_queued, vec!["0003"]);
    let state = load_state(&paths).unwrap();
    assert!(["0001", "0002", "0003"].iter().all(|id| state.chapters[*id].status == ChapterStatus::Queued));
    assert_eq!(state.chapters["0004"].status, ChapterStatus::Done, "ngoài khoảng giữ nguyên");
    assert_eq!(fs::read_to_string(retry_backup_path(&paths, "0001")).unwrap(), "dịch 1");
    assert!(!paths.out_dir.join("0001.txt").exists());

    // Không from/to = toàn bộ: chỉ còn 0004 chưa queued.
    let all = run_retry_range(root, None, None).unwrap();
    assert_eq!(all.retried, vec!["0004"]);
    assert_eq!(all.already_queued.len(), 3);
    assert!(matches!(run_retry_range(root, Some("0003"), Some("0001")), Err(CoreError::InvalidState(ref m)) if m.contains("ngược")));
    assert!(matches!(run_retry_range(root, Some("9999"), None), Err(CoreError::StoryNotFound(_))));
}

#[test]
fn retry_ids_dich_lai_dung_cac_chuong_hong_bo_qua_queued_khong_dung_done_xen_giua() {
    let dir = make_story_dir(&[("0001", "一"), ("0002", "二"), ("0003", "三"), ("0004", "四")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    for (id, status) in [("0001", ChapterStatus::Done), ("0002", ChapterStatus::Error), ("0004", ChapterStatus::Skipped)] {
        state.chapters.get_mut(id).unwrap().status = status;
    }
    save_state(&paths, &state).unwrap();
    fs::create_dir_all(&paths.out_dir).unwrap();
    fs::write(paths.out_dir.join("0001.txt"), "dịch 1").unwrap();

    // Id lạ → không đổi gì.
    let ids = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(matches!(run_retry_ids(root, &ids(&["0002", "9999"])), Err(CoreError::StoryNotFound(_))));
    assert_eq!(load_state(&paths).unwrap().chapters["0002"].status, ChapterStatus::Error);

    // Thứ tự lộn xộn + trùng: kết quả theo thứ tự tự nhiên, queued bỏ qua, done xen giữa giữ nguyên.
    let outcome = run_retry_ids(root, &ids(&["0004", "0002", "0003", "0004"])).unwrap();
    assert_eq!(outcome.retried, vec!["0002", "0004"]);
    assert_eq!(outcome.already_queued, vec!["0003"]);
    assert!(outcome.backed_up.is_empty());
    let state = load_state(&paths).unwrap();
    assert!(["0002", "0003", "0004"].iter().all(|id| state.chapters[*id].status == ChapterStatus::Queued));
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(paths.out_dir.join("0001.txt").exists(), "chương done không được động vào");
}

#[test]
fn init_go_chuong_raw_da_mat_ke_ca_done_giu_out_va_don_work() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章"), ("0003", "第三章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    state.chapters.get_mut("0001").unwrap().status = ChapterStatus::Done;
    state.chapters.get_mut("0002").unwrap().status = ChapterStatus::Error;
    save_state(&paths, &state).unwrap();
    fs::write(paths.out_dir.join("0001.txt"), "dịch 1").unwrap();
    fs::create_dir_all(&paths.work_dir).unwrap();
    fs::write(work_file(&paths, "0002", WorkKind::Draft), "nháp").unwrap();
    // Người dùng xoá/đổi tên file raw: 0001 (done)/0002/0003 mất, 0004 mới xuất hiện.
    for id in ["0001", "0002", "0003"] {
        fs::remove_file(root.join("raw").join(format!("{id}.txt"))).unwrap();
    }
    fs::write(root.join("raw").join("0004.txt"), "第四章").unwrap();

    let message = run_init(root, "qt-ai").unwrap();
    assert!(message.contains("1 chương (1 mới thêm vào hàng đợi, 3 gỡ vì raw đã mất"), "{message}");
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters.keys().collect::<Vec<_>>(), vec!["0004"], "chương done raw mất cũng gỡ");
    assert_eq!(fs::read_to_string(paths.out_dir.join("0001.txt")).unwrap(), "dịch 1", "out/ giữ nguyên");
    assert!(!work_file(&paths, "0002", WorkKind::Draft).exists(), "dọn work/ của chương bị gỡ");
}

#[test]
fn delete_xoa_raw_work_va_state_giu_out_kiem_du_id_truoc() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章"), ("0003", "第三章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    state.chapters.get_mut("0001").unwrap().status = ChapterStatus::Done;
    save_state(&paths, &state).unwrap();
    fs::write(paths.out_dir.join("0001.txt"), "dịch 1").unwrap();
    fs::write(work_file(&paths, "0002", WorkKind::Draft), "nháp").unwrap();

    // Thiếu một id → không xoá gì.
    let err = run_delete(root, &["0002".to_string(), "9999".to_string()]).unwrap_err();
    assert!(matches!(err, CoreError::StoryNotFound(ref m) if m.contains("9999")));
    assert!(root.join("raw").join("0002.txt").exists());
    assert!(matches!(run_delete(root, &[]), Err(CoreError::InvalidState(_))));

    let outcome = run_delete(root, &["0002".to_string(), "0001".to_string()]).unwrap();
    assert_eq!(outcome.removed, vec!["0001", "0002"], "theo thứ tự tự nhiên");
    assert_eq!(outcome.kept_outputs, vec!["0001"]);
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters.keys().collect::<Vec<_>>(), vec!["0003"]);
    assert!(!root.join("raw").join("0001.txt").exists());
    assert!(!root.join("raw").join("0002.txt").exists());
    assert!(!work_file(&paths, "0002", WorkKind::Draft).exists());
    assert_eq!(fs::read_to_string(paths.out_dir.join("0001.txt")).unwrap(), "dịch 1", "out/ giữ nguyên");
    // Quét lại không mọc lại chương đã xoá vì raw đã mất.
    run_init(root, "qt-ai").unwrap();
    assert_eq!(load_state(&paths).unwrap().chapters.keys().collect::<Vec<_>>(), vec!["0003"]);
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
    assert!(prompt.contains("phải phiên cùng âm"), "chỉ dẫn glossary: tên mới theo âm entry sẵn có");
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
fn next_prompt_co_muc_xung_ho_theo_cast_json() {
    let dir = make_story_dir(&[("0001", "贺静昭点头。贺老师很擅长包饺子。")]);
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("贺静昭".into(), "Hạ Tĩnh Chiêu".into());
    save_story_config(&paths, &story).unwrap();
    let mut cast = qt_ai_core::cast::Cast::default();
    cast.characters.insert(
        "贺静昭".into(),
        qt_ai_core::cast::Character { gender: Some(qt_ai_core::cast::Gender::Female), ..Default::default() },
    );
    qt_ai_core::cast::save_cast(&paths, &cast).unwrap();
    let next = run_next(dir.path()).unwrap();
    let prompt = fs::read_to_string(next.prompt_path).unwrap();
    assert!(prompt.contains("# Xưng hô") && prompt.contains("- 贺静昭 (Hạ Tĩnh Chiêu): nữ"), "{prompt}");
    assert!(prompt.contains("\"gender\": \"nam\" hoặc \"nữ\""), "chỉ dẫn agent khai giới");
    assert!(prompt.contains("quan hệ hai người đã khác lúc chốt cặp"), "chỉ dẫn agent khai đổi cặp theo trạng thái quan hệ");
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
        "[[1]] Triệu Tĩnh Văn ngẩng đầu, trong não hải hiện lên tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
        None,
    );
    let paths = story_paths(dir.path());
    for _ in 0..3 {
        assert!(!run_check(dir.path(), "0001").unwrap().pass);
    }
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.pass && result.accepted_with_warnings && !result.escalated_to_error);
    assert_eq!(result.issues.len(), 1);
    assert!(result.issues[0].starts_with("[[1]]") && result.issues[0].contains("não hải"), "{:?}", result.issues);
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Translating);
    let accepted = run_accept(dir.path(), "0001", false).unwrap();
    assert_eq!(accepted.warnings, result.issues);
    let chapter = &load_state(&paths).unwrap().chapters["0001"];
    assert_eq!(chapter.status, ChapterStatus::Done);
    assert_eq!(chapter.warnings.as_ref().unwrap(), &result.issues);
}

#[test]
fn check_bao_doan_con_han_tu_theo_nhan() {
    let dir = story_with_draft(
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu, tai濡目染 không nói lời nào.",
        None,
    );
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(!result.pass);
    assert_eq!(result.untranslated, vec![2]);
    assert!(result.issues.iter().any(|i| i.starts_with("[[2]] CJK còn sót")), "{:?}", result.issues);
}

#[test]
fn check_ngoac_kep_goc_trung_khong_tinh_la_han_tu() {
    let dir = story_with_draft(
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn tòa tháp cao nơi xa, tay cầm 《Kiếm Phổ》.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
        None,
    );
    assert!(run_check(dir.path(), "0001").unwrap().untranslated.is_empty());
}

#[test]
fn check_het_vong_con_han_tu_thi_error_khong_chot_kem_canh_bao() {
    let dir = story_with_draft(
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu, chạy得 không nói lời nào.",
        None,
    );
    for _ in 0..3 {
        assert!(!run_check(dir.path(), "0001").unwrap().pass);
    }
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.escalated_to_error && !result.pass && !result.accepted_with_warnings);
    let chapter = &load_state(&story_paths(dir.path())).unwrap().chapters["0001"];
    assert_eq!(chapter.status, ChapterStatus::Error);
    assert!(chapter.reason.as_deref().unwrap().contains("còn chữ Hán ở [[2]]"), "{:?}", chapter.reason);
}

#[test]
fn accept_khong_force_tu_choi_khi_con_han_tu() {
    let dir = story_with_draft(
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu, chạy得 không nói lời nào.",
        None,
    );
    let err = run_accept(dir.path(), "0001", false).unwrap_err();
    assert!(matches!(err, CoreError::InvalidState(ref m) if m.contains("chưa qua check")), "{err:?}");
    assert!(!story_paths(dir.path()).out_dir.join("0001.txt").exists());
}

const RAW_DOAN: &str = "段锋带着人进了城。\n\n李贤和李显并肩走来。\n\n段锋点了点头。";

/// Chương 0001 với raw cho trước, glossary truyện nhóm names, draft đã ghi.
fn story_with_glossary(raw: &str, names: &[(&str, &str)], draft: &str) -> TempDir {
    let dir = make_story_dir(&[("0001", raw)]);
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    let group = story.glossary.entry("names".to_string()).or_default();
    for (source, target) in names {
        group.insert(source.to_string(), target.to_string());
    }
    save_story_config(&paths, &story).unwrap();
    run_next(dir.path()).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Draft), draft).unwrap();
    dir
}

#[test]
fn check_bat_ten_lech_dau_so_voi_glossary_theo_doan() {
    let dir = story_with_glossary(
        RAW_DOAN,
        &[("段锋", "Đoàn Phong")],
        "[[1]] Đoàn Phong dẫn người vào thành.\n\n[[2]] Lý Hiền và Lý Hiển sóng vai đi tới.\n\n[[3]] Đoạn Phong gật gật đầu.",
    );
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(!result.pass);
    assert_eq!(result.issues, vec!["[[3]] Tên lệch glossary: 段锋 là `Đoàn Phong`, bản dịch viết `Đoạn Phong` — \"Đoạn Phong gật gật đầu.\""]);
}

#[test]
fn check_khong_bat_ten_lech_khi_la_target_cua_entry_khac_trong_doan() {
    // 李贤 Lý Hiền và 李显 Lý Hiển là hai người: "Lý Hiển" đúng là 李显, không phải viết sai 李贤.
    let dir = story_with_glossary(
        RAW_DOAN,
        &[("段锋", "Đoàn Phong"), ("李贤", "Lý Hiền"), ("李显", "Lý Hiển")],
        "[[1]] Đoàn Phong dẫn người vào thành.\n\n[[2]] Lý Hiển và Lý Hiền sóng vai đi tới.\n\n[[3]] Đoàn Phong gật gật đầu.",
    );
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(result.pass, "{:?}", result.issues);
}

#[test]
fn check_khong_bat_ten_lech_khi_key_khong_nam_trong_raw_cua_doan_do() {
    // Đoạn 2 không có 段锋 trong raw → "Đoạn" ở đó không bị so với entry.
    let dir = story_with_glossary(
        "段锋带着人进了城。\n\n这一段路很长。",
        &[("段锋", "Đoàn Phong")],
        "[[1]] Đoàn Phong dẫn người vào thành.\n\n[[2]] Đoạn Phong cảnh đường này rất dài.",
    );
    assert!(run_check(dir.path(), "0001").unwrap().pass);
}

#[test]
fn accept_khong_hoc_ten_moi_trai_am_ho_da_chot() {
    let raw = "段誉看向段延庆。\n\n段正淳也来了。";
    let glossary = r#"{"entries":[{"source":"段延庆","target":"Đoạn Duyên Khánh","category":"names"},{"source":"段正淳","target":"Đoàn Chính Thuần","category":"names"}]}"#;
    let dir = story_with_glossary(
        raw,
        &[("段誉", "Đoàn Dự")],
        "[[1]] Đoàn Dự nhìn về phía Đoạn Duyên Khánh.\n\n[[2]] Đoàn Chính Thuần cũng đã tới.",
    );
    let paths = story_paths(dir.path());
    fs::write(work_file(&paths, "0001", WorkKind::Glossary), glossary).unwrap();
    let result = run_accept(dir.path(), "0001", false).unwrap();
    assert_eq!(result.added_glossary, 1);
    let story = load_story_config(&paths).unwrap();
    assert_eq!(story.glossary["names"]["段正淳"], "Đoàn Chính Thuần");
    assert!(!story.glossary["names"].contains_key("段延庆"), "段 đã chốt Đoàn, không học Đoạn");
}

#[test]
fn check_bat_sai_gioi_va_loi_goi_tho_theo_nhan() {
    let dir = story_with_glossary(
        "贺静昭帮忙洗菜。\n\n“贺老师很擅长包饺子。”\n\n“学弟，你不要吓学姐好不好！”",
        &[("贺静昭", "Hạ Tĩnh Chiêu")],
        "[[1]] Hạ Tĩnh Chiêu giúp rửa rau.\n\n[[2]] “Thầy Hạ rất khéo gói sủi cảo.”\n\n[[3]] “Đàn em, cậu đừng dọa đàn chị có được không!”",
    );
    let paths = story_paths(dir.path());
    let mut cast = qt_ai_core::cast::Cast::default();
    cast.characters.insert(
        "贺静昭".into(),
        qt_ai_core::cast::Character { gender: Some(qt_ai_core::cast::Gender::Female), ..Default::default() },
    );
    qt_ai_core::cast::save_cast(&paths, &cast).unwrap();
    let result = run_check(dir.path(), "0001").unwrap();
    assert!(!result.pass);
    assert_eq!(result.issues.len(), 2, "{:?}", result.issues);
    assert!(result.issues[0].starts_with("[[2]] Sai giới: 贺老师 là 贺静昭 (nữ) → `cô Hạ`, bản dịch viết `Thầy Hạ`"), "{:?}", result.issues);
    assert!(result.issues[1].starts_with("[[3]] Danh xưng thô trong thoại: `Đàn em`"), "{:?}", result.issues);
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

// ---- accept: học giới, siết cặp xưng hô mới, đổi cặp theo mốc chương ----

use qt_ai_core::cast::{load_cast, save_cast, AddressChange, AddressTimeline, Cast, Character, EntrySource, Gender};

const RAW_CAST: &str = "许浪看着苏雨。她笑了。\n\n保安走过来。他点头。";
const DRAFT_CAST: &str = "[[1]] Hứa Lãng nhìn Tô Vũ. Cô cười.\n\n[[2]] Bảo vệ đi tới. Hắn gật đầu.";

/// Chương `id` với glossary truyện (names + addressing) cho trước, draft sạch, glossary.json đề xuất.
fn story_for_accept(names: &[(&str, &str)], addressing: &[(&str, &str)], entries: &str) -> TempDir {
    let dir = story_with_glossary(RAW_CAST, names, DRAFT_CAST);
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    let group = story.glossary.entry("addressing".to_string()).or_default();
    for (source, target) in addressing {
        group.insert(source.to_string(), target.to_string());
    }
    save_story_config(&paths, &story).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Glossary), format!("{{\"entries\":{entries}}}")).unwrap();
    dir
}

#[test]
fn accept_hoc_gioi_tinh_khi_raw_co_can_cu() {
    let entries = r#"[
        {"source":"苏雨","target":"Tô Vũ","category":"names","gender":"nữ"},
        {"source":"许浪","category":"names","gender":"nam"},
        {"source":"不在","category":"names","gender":"nữ"}
    ]"#;
    let dir = story_for_accept(&[("许浪", "Hứa Lãng")], &[], entries);
    run_accept(dir.path(), "0001", false).unwrap();
    let cast = load_cast(&story_paths(dir.path()));
    assert_eq!(cast.gender_of("苏雨"), Some(Gender::Female));
    assert_eq!(cast.characters["苏雨"].chapter, "0001");
    assert_eq!(cast.gender_of("许浪"), Some(Gender::Male), "nhân vật đã có trong từ điển được khai lại giới");
    assert!(!cast.characters.contains_key("不在"), "tên không có trong raw/từ điển thì bỏ");
}

#[test]
fn accept_khong_hoc_gioi_thieu_can_cu_va_ghi_tranh_chap_khi_khai_nguoc() {
    // Raw không có 她/女… nào → khai "nữ" cho 许浪 bị bỏ (chưa có giới) hoặc ghi tranh chấp (đã có giới).
    let dir = make_story_dir(&[("0001", "许浪点头。他笑了。\n\n许浪走了。")]);
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("许浪".into(), "Hứa Lãng".into());
    save_story_config(&paths, &story).unwrap();
    run_next(dir.path()).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Draft), "[[1]] Hứa Lãng gật đầu. Hắn cười.\n\n[[2]] Hứa Lãng đi rồi.").unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Glossary), r#"{"entries":[{"source":"许浪","category":"names","gender":"nữ"}]}"#).unwrap();
    run_accept(dir.path(), "0001", false).unwrap();
    assert_eq!(load_cast(&paths).gender_of("许浪"), None, "không có 她/女 trong raw → không nhận nữ");
}

#[test]
fn accept_khai_gioi_nguoc_voi_bang_thi_giu_cu_va_ghi_disputed() {
    let entries = r#"[{"source":"苏雨","category":"names","gender":"nam"}]"#;
    let dir = story_for_accept(&[("许浪", "Hứa Lãng"), ("苏雨", "Tô Vũ")], &[], entries);
    let paths = story_paths(dir.path());
    let mut cast = Cast::default();
    cast.characters.insert("苏雨".into(), Character { gender: Some(Gender::Female), ..Default::default() });
    save_cast(&paths, &cast).unwrap();
    let result = run_accept(dir.path(), "0001", false).unwrap();
    let cast = load_cast(&paths);
    assert_eq!(cast.gender_of("苏雨"), Some(Gender::Female));
    assert_eq!(cast.characters["苏雨"].disputed, vec!["0001"]);
    assert!(result.cast_notes.iter().any(|n| n.contains("苏雨") && n.contains("khai giới ngược")), "{:?}", result.cast_notes);
}

#[test]
fn accept_chi_hoc_cap_giua_nhan_vat_co_ten_hop_gioi_va_dao_cap_nguoc() {
    let entries = r#"[
        {"source":"苏雨","target":"Tô Vũ","category":"names","gender":"nữ"},
        {"source":"保安→许浪","target":"tôi–cậu","category":"addressing"},
        {"source":"许浪→苏雨","target":"tôi–anh","category":"addressing"},
        {"source":"苏雨→许浪","target":"cậu–tớ","category":"addressing"}
    ]"#;
    let dir = story_for_accept(&[("许浪", "Hứa Lãng")], &[], entries);
    let paths = story_paths(dir.path());
    run_accept(dir.path(), "0001", false).unwrap();
    let story = load_story_config(&paths).unwrap();
    let addressing = &story.glossary["addressing"];
    assert!(!addressing.contains_key("保安→许浪"), "保安 không phải nhân vật có tên");
    assert!(!addressing.contains_key("许浪→苏雨"), "gọi nữ là anh → trái giới vừa học, bỏ");
    assert_eq!(addressing["苏雨→许浪"], "tớ–cậu", "cặp viết ngược được đảo");
    assert!(story.auto_glossary_log.iter().all(|e| e.source != "保安→许浪" && e.source != "许浪→苏雨"));
}

#[test]
fn accept_nhan_doi_cap_co_ly_do_ghi_moc_chuong_va_dich_lai_thi_thay_moc_cu() {
    let entries = r#"[
        {"source":"许浪→苏雨","target":"anh–em","category":"addressing","note":"thành người yêu"},
        {"source":"苏雨→许浪","target":"em–anh","category":"addressing"}
    ]"#;
    let dir = story_for_accept(
        &[("许浪", "Hứa Lãng"), ("苏雨", "Tô Vũ")],
        &[("许浪→苏雨", "tôi–cô"), ("苏雨→许浪", "tôi–anh")],
        entries,
    );
    let paths = story_paths(dir.path());
    let result = run_accept(dir.path(), "0001", false).unwrap();
    let cast = load_cast(&paths);
    assert_eq!(
        cast.addressing["许浪→苏雨"].changes,
        vec![AddressChange { from: "0001".into(), target: "anh–em".into(), note: "thành người yêu".into(), source: EntrySource::Auto }]
    );
    assert!(!cast.addressing.contains_key("苏雨→许浪"), "đổi cặp không có note thì không nhận");
    assert_eq!(load_story_config(&paths).unwrap().glossary["addressing"]["许浪→苏雨"], "tôi–cô", "story.json giữ cặp gốc");
    assert!(result.cast_notes.iter().any(|n| n == "đổi xưng hô 许浪→苏雨: tôi–cô → anh–em (thành người yêu)"), "{:?}", result.cast_notes);

    // Dịch lại cùng chương, model khai mốc khác → thay mục cùng `from`, không chồng thêm.
    run_retry(dir.path(), "0001").unwrap();
    run_next(dir.path()).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Draft), DRAFT_CAST).unwrap();
    fs::write(
        work_file(&paths, "0001", WorkKind::Glossary),
        r#"{"entries":[{"source":"许浪→苏雨","target":"anh–bà xã","category":"addressing","note":"cưới"}]}"#,
    )
    .unwrap();
    run_accept(dir.path(), "0001", false).unwrap();
    let changes = &load_cast(&paths).addressing["许浪→苏雨"].changes;
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].target, "anh–bà xã");
}

#[test]
fn accept_khong_doi_cap_da_ghim_trai_gioi_hoac_dao_ve_gia_tri_truoc() {
    let entries = r#"[
        {"source":"许浪→苏雨","target":"tôi–cô","category":"addressing","note":"cãi nhau"},
        {"source":"苏雨→许浪","target":"em–chị","category":"addressing","note":"thân"},
        {"source":"许浪→保安","target":"anh–em","category":"addressing","note":"thân"}
    ]"#;
    let dir = story_for_accept(
        &[("许浪", "Hứa Lãng"), ("苏雨", "Tô Vũ")],
        &[("许浪→苏雨", "tôi–cô"), ("苏雨→许浪", "tôi–anh"), ("许浪→保安", "tôi–chú")],
        entries,
    );
    let paths = story_paths(dir.path());
    let mut cast = Cast::default();
    cast.characters.insert("许浪".into(), Character { gender: Some(Gender::Male), ..Default::default() });
    // 许浪→苏雨 đã đổi anh–em từ chương trước; giờ model đòi về tôi–cô (giá trị ngay trước) → bỏ.
    cast.addressing.insert(
        "许浪→苏雨".into(),
        AddressTimeline { pinned: false, changes: vec![AddressChange { from: "0000".into(), target: "anh–em".into(), note: "yêu".into(), source: EntrySource::Auto }] },
    );
    cast.addressing.insert("许浪→保安".into(), AddressTimeline { pinned: true, changes: vec![] });
    save_cast(&paths, &cast).unwrap();
    run_accept(dir.path(), "0001", false).unwrap();
    let cast = load_cast(&paths);
    assert_eq!(cast.addressing["许浪→苏雨"].changes.len(), 1, "không đảo về giá trị ngay trước");
    assert!(!cast.addressing.contains_key("苏雨→许浪"), "gọi 许浪 (nam) là chị → trái giới");
    assert!(cast.addressing["许浪→保安"].changes.is_empty(), "cặp ghim không tự đổi");
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

#[test]
fn skip_chuong_translating_kem_ly_do_next_di_tiep() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    run_next(root).unwrap();
    run_skip(root, "0001", "model từ chối nội dung").unwrap();
    let state = load_state(&story_paths(root)).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Skipped);
    assert_eq!(state.chapters["0001"].reason.as_deref(), Some("model từ chối nội dung"));
    assert!(!work_file(&story_paths(root), "0001", WorkKind::Prompt).exists());
    assert_eq!(run_next(root).unwrap().chapter_id, "0002");
    assert!(matches!(run_skip(root, "0002", "  "), Err(CoreError::InvalidState(ref m)) if m.contains("--reason")));
}

#[test]
fn retry_dua_error_skipped_ve_queued_va_chan_case_vo_nghia() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    state.chapters.insert(
        "0001".into(),
        ChapterState {
            status: ChapterStatus::Error,
            review_round: 3,
            reason: Some("Quá 3 vòng".into()),
            warnings: None,
            updated_at: 1,
        },
    );
    save_state(&paths, &state).unwrap();
    fs::create_dir_all(&paths.work_dir).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Draft), "[[1]] nháp cũ").unwrap();
    run_retry(root, "0001").unwrap();
    let after = &load_state(&paths).unwrap().chapters["0001"];
    assert_eq!(after.status, ChapterStatus::Queued);
    assert_eq!(after.review_round, 0);
    assert!(after.reason.is_none());
    assert!(!work_file(&paths, "0001", WorkKind::Draft).exists());
    assert_eq!(run_next(root).unwrap().chapter_id, "0001");

    // translating (phiên chết, chương kẹt) cũng dịch lại được — caller tự chắc không có phiên đang chạy.
    run_retry(root, "0001").unwrap();
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Queued);
    assert_eq!(run_next(root).unwrap().chapter_id, "0001");
    assert!(matches!(run_retry(root, "0002"), Err(CoreError::InvalidState(ref m)) if m.contains("queued sẵn")));
    assert!(matches!(run_retry(root, "9999"), Err(CoreError::StoryNotFound(_))));
    run_skip(root, "0001", "thử").unwrap();
    run_retry(root, "0001").unwrap();
    assert_eq!(load_state(&paths).unwrap().chapters["0001"].status, ChapterStatus::Queued);
}

#[test]
fn retry_chuong_done_doi_out_thanh_bak_roi_ve_queued() {
    let dir = make_story_dir(&[("0001", "第一章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    state.chapters.insert(
        "0001".into(),
        ChapterState { status: ChapterStatus::Done, review_round: 1, reason: None, warnings: None, updated_at: 1 },
    );
    save_state(&paths, &state).unwrap();
    fs::create_dir_all(&paths.out_dir).unwrap();
    let out = paths.out_dir.join("0001.txt");
    fs::write(&out, "bản dịch cũ").unwrap();
    fs::create_dir_all(&paths.work_dir).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Review), "review cũ").unwrap();

    run_retry(root, "0001").unwrap();
    let after = &load_state(&paths).unwrap().chapters["0001"];
    assert_eq!(after.status, ChapterStatus::Queued);
    assert_eq!(after.review_round, 0);
    assert!(!out.exists(), "out/<id>.txt phải được dọn khỏi out/ để accept sau này không đè nhầm");
    assert_eq!(fs::read_to_string(retry_backup_path(&paths, "0001")).unwrap(), "bản dịch cũ");
    assert!(!work_file(&paths, "0001", WorkKind::Review).exists());
    assert_eq!(run_next(root).unwrap().chapter_id, "0001");

    // Dịch lại lần nữa: bak cũ bị đè bằng bản mới nhất, không tích luỹ file.
    let mut state = load_state(&paths).unwrap();
    state.chapters.get_mut("0001").unwrap().status = ChapterStatus::Done;
    save_state(&paths, &state).unwrap();
    fs::write(&out, "bản dịch mới").unwrap();
    run_retry(root, "0001").unwrap();
    assert_eq!(fs::read_to_string(retry_backup_path(&paths, "0001")).unwrap(), "bản dịch mới");
    assert_eq!(fs::read_dir(&paths.out_dir).unwrap().count(), 1);
}

#[test]
fn status_tong_hop_du_trang_thai_va_canh_bao() {
    let dir = make_story_dir(&[("0001", "第一章"), ("0002", "第二章"), ("0003", "第三章")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    run_next(root).unwrap();
    run_skip(root, "0001", "thử").unwrap();
    let mut state = load_state(&paths).unwrap();
    state.chapters.insert(
        "0003".into(),
        ChapterState {
            status: ChapterStatus::Done,
            review_round: 3,
            reason: None,
            warnings: Some(vec!["[[1]] CJK còn sót".into()]),
            updated_at: 1,
        },
    );
    save_state(&paths, &state).unwrap();
    let report = run_status(root).unwrap();
    assert!(report.starts_with(
        "Tổng 3 chương — done: 1, queued: 1, translating: 0, error: 0, skipped: 1, done kèm cảnh báo: 1"
    ));
    assert!(report.contains("  0001 [skipped] thử"));
    assert!(report.contains("  0003 [done, 1 cảnh báo] [[1]] CJK còn sót"));
    assert!(report.ends_with("Giới hạn phiên: dịch tối đa 10 chương/phiên rồi nghỉ."));
}

/// 4 chương: 1, 2, 4 done có out/; 3 skipped.
fn story_with_outputs() -> TempDir {
    let dir = make_story_dir(&[("0001", "一"), ("0002", "二"), ("0003", "三"), ("0004", "四")]);
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut state = load_state(&paths).unwrap();
    for (id, text) in [("0001", "Chương một.\n"), ("0002", "Chương hai.\n\n"), ("0004", "Chương bốn.")] {
        fs::write(paths.out_dir.join(format!("{id}.txt")), text).unwrap();
        state.chapters.insert(
            id.into(),
            ChapterState { status: ChapterStatus::Done, review_round: 0, reason: None, warnings: None, updated_at: 1 },
        );
    }
    state.chapters.insert(
        "0003".into(),
        ChapterState {
            status: ChapterStatus::Skipped,
            review_round: 0,
            reason: Some("thử".into()),
            warnings: None,
            updated_at: 1,
        },
    );
    save_state(&paths, &state).unwrap();
    dir
}

#[test]
fn export_gop_chuong_done_cach_1_dong_trong_bao_hong() {
    let dir = story_with_outputs();
    let result = run_export(dir.path(), &ExportOptions::default()).unwrap();
    assert_eq!(result.ids, vec!["0001", "0002", "0004"]);
    assert_eq!(result.gaps, vec!["0003"]);
    assert_eq!(result.out_path, dir.path().join("export").join("0001-0004.txt"));
    assert_eq!(fs::read_to_string(&result.out_path).unwrap(), "Chương một.\n\nChương hai.\n\nChương bốn.\n");

    let out = dir.path().join("custom").join("tap1.txt");
    let result = run_export(
        dir.path(),
        &ExportOptions { from: Some("0002".into()), to: Some("0004".into()), out: Some(out.clone()) },
    )
    .unwrap();
    assert_eq!(result.ids, vec!["0002", "0004"]);
    assert_eq!(fs::read_to_string(&out).unwrap(), "Chương hai.\n\nChương bốn.\n");

    assert!(matches!(
        run_export(dir.path(), &ExportOptions { from: Some("0004".into()), to: Some("0001".into()), out: None }),
        Err(CoreError::InvalidState(ref m)) if m.contains("ngược")
    ));
    assert!(matches!(
        run_export(dir.path(), &ExportOptions { from: Some("9999".into()), to: None, out: None }),
        Err(CoreError::StoryNotFound(_))
    ));
    let empty = make_story_dir(&[("0001", "一")]);
    run_init(empty.path(), "qt-ai").unwrap();
    assert!(matches!(
        run_export(empty.path(), &ExportOptions::default()),
        Err(CoreError::InvalidState(ref m)) if m.contains("done")
    ));
}
