//! State machine trên tempdir — tương đương bộ vitest của apps/qt-ai-cli.
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
