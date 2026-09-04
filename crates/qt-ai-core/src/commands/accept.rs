use crate::commands::check::{assemble_draft, run_check};
use crate::error::{CoreError, Result};
use crate::glossary::{
    append_auto_glossary, collect_glossary_keys, resolve_auto_glossary_enabled, sanitize_extracted,
};
use crate::paragraphs::{format_translation, strip_markers};
use crate::prompt::TranslationGlossary;
use crate::story_fs::{
    load_state, load_story_config, now_ms, read_raw_chapter, read_text, resolve_root, save_state,
    save_story_config, story_paths, work_file, write_text, ChapterState, ChapterStatus, WorkKind, WORK_KINDS,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct AcceptResult {
    pub out_path: PathBuf,
    pub added_glossary: usize,
    pub warnings: Vec<String>,
}

pub fn run_accept(root: &Path, id: &str, force: bool) -> Result<AcceptResult> {
    let paths = story_paths(&resolve_root(root));
    let check = run_check(root, id)?;
    if !check.pass && !force {
        return Err(CoreError::InvalidState(format!(
            "Chương {id} chưa qua check (thiếu {} đoạn, {} vi phạm, ratio {:.2}) — sửa theo work/{id}.review.md hoặc dùng --force.",
            check.missing.len(),
            check.violations.len(),
            check.ratio
        )));
    }

    let draft = assemble_draft(root, id)?;
    let output = format_translation(&strip_markers(&draft.final_text));
    let out_path = paths.out_dir.join(format!("{id}.txt"));
    write_text(&out_path, &output)?;

    let mut story = load_story_config(&paths)?;
    let mut added_glossary = 0;
    let glossary_path = work_file(&paths, id, WorkKind::Glossary);
    if glossary_path.exists() && resolve_auto_glossary_enabled(story.auto_glossary, true) {
        // Đề xuất hỏng → bỏ qua, không chặn accept.
        let entries: Value = serde_json::from_str::<Value>(&read_text(&glossary_path)?)
            .map(|envelope| match envelope.get("entries") {
                Some(entries) => entries.clone(),
                None => envelope,
            })
            .unwrap_or(Value::Array(vec![]));
        let raw = read_raw_chapter(&paths, id)?;
        let existing = collect_glossary_keys(&TranslationGlossary::new(), &story.glossary);
        let pairs = sanitize_extracted(&entries, &raw, &output, &existing);
        if !pairs.is_empty() {
            story = append_auto_glossary(&story, &pairs, id);
            added_glossary = pairs.len();
        }
    }
    // Chỉ ghi story.json khi thực sự có glossary mới — load đã normalize (bỏ field lạ), ghi vô điều kiện
    // sẽ biến mất mát đó thành vĩnh viễn dù accept không đổi gì.
    if added_glossary > 0 {
        save_story_config(&paths, &story)?;
    }

    let mut state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .cloned()
        .ok_or_else(|| CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")))?;
    // Vấn đề còn sót (hết vòng review chỉ còn vi phạm, hoặc --force) ghi vào state để status liệt kê.
    state.chapters.insert(
        id.to_string(),
        ChapterState {
            status: ChapterStatus::Done,
            review_round: chapter.review_round,
            reason: None,
            warnings: (!check.issues.is_empty()).then(|| check.issues.clone()),
            updated_at: now_ms(),
        },
    );
    save_state(&paths, &state)?;

    for kind in WORK_KINDS {
        let _ = fs::remove_file(work_file(&paths, id, kind)); // force: true
    }
    Ok(AcceptResult { out_path, added_glossary, warnings: check.issues })
}
