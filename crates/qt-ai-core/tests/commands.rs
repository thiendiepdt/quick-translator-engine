//! State machine trên tempdir — tương đương bộ vitest của apps/qt-ai-cli.
use qt_ai_core::story::StoryConfig;
use qt_ai_core::story_fs::*;
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
