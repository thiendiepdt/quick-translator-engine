use crate::error::{CoreError, Result};
use crate::paragraphs::{labeled_source_payload, paragraphs_of};
use crate::prompt::{build_system_prompt, TranslationGlossary};
use crate::story::{natural_chapter_compare, StoryConfig};
use crate::story_fs::{
    load_state, load_story_config, read_raw_chapter, resolve_root, save_state, story_paths, work_file,
    write_text, ChapterState, ChapterStatus, StoryPaths, WorkKind,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct NextResult {
    pub chapter_id: String,
    pub prompt_path: PathBuf,
}

fn agent_instructions(id: &str) -> String {
    [
        "# Việc của agent sau khi dịch xong".to_string(),
        String::new(),
        format!("1. Ghi bản dịch tiếng Việt (GIỮ NGUYÊN nhãn [[n]] đầu mỗi đoạn) vào work/{id}.draft.md."),
        "2. Ghi đề xuất TÊN RIÊNG mới (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng)".to_string(),
        format!("   xuất hiện trong raw nhưng chưa có trong từ điển của prompt vào work/{id}.glossary.json, dạng:"),
        r#"   {"entries": [{"source": "chữ Hán trong raw", "target": "chép nguyên văn từ bản dịch", "category": "names|places|items|creatures|skills"}]}"#.to_string(),
        r#"   Bỏ qua từ chung, chức danh, đại từ. Không có tên mới thì ghi {"entries": []}."#.to_string(),
        format!("3. Chạy: qt-ai check {id} (xem AGENTS.md để biết lệnh đầy đủ)."),
    ]
    .join("\n")
}

/// Sinh + ghi work/<id>.prompt.md; trả về đường dẫn đã ghi.
fn write_prompt_file(paths: &StoryPaths, story: &StoryConfig, id: &str) -> Result<PathBuf> {
    let source = read_raw_chapter(paths, id)?;
    let system = build_system_prompt(&TranslationGlossary::new(), Some(story), Some(&source));
    let payload = labeled_source_payload(&paragraphs_of(&source));
    let prompt = format!("{system}\n\n---\n\n{payload}\n\n---\n\n{}\n", agent_instructions(id));
    let prompt_path = work_file(paths, id, WorkKind::Prompt);
    write_text(&prompt_path, &prompt)?;
    Ok(prompt_path)
}

pub fn run_next(root: &Path) -> Result<NextResult> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let story = load_story_config(&paths)?;

    let pending: Vec<String> = state
        .chapters
        .iter()
        .filter(|(_, chapter)| chapter.status == ChapterStatus::Translating)
        .map(|(id, _)| id.clone())
        .collect();
    if !pending.is_empty() {
        // Phiên trước chết mất work/*.prompt.md của chương đang translating → phát lại đúng chương đó,
        // không đụng reviewRound/status.
        if let Some(lost) = pending.iter().find(|id| !work_file(&paths, id, WorkKind::Prompt).exists()) {
            let prompt_path = write_prompt_file(&paths, &story, lost)?;
            return Ok(NextResult { chapter_id: lost.clone(), prompt_path });
        }
        return Err(CoreError::InvalidState(format!(
            "Chương {} đang translating chưa chốt — chạy check/accept/skip trước khi lấy chương mới.",
            pending.join(", ")
        )));
    }

    let mut queued: Vec<String> = state
        .chapters
        .iter()
        .filter(|(_, chapter)| chapter.status == ChapterStatus::Queued)
        .map(|(id, _)| id.clone())
        .collect();
    queued.sort_by(|a, b| natural_chapter_compare(a, b));
    let Some(next_id) = queued.into_iter().next() else {
        return Err(CoreError::InvalidState(
            "Không còn chương nào trong hàng đợi — chạy qt-ai status để xem tổng kết.".to_string(),
        ));
    };

    let prompt_path = write_prompt_file(&paths, &story, &next_id)?;
    state.chapters.insert(next_id.clone(), ChapterState::fresh(ChapterStatus::Translating));
    save_state(&paths, &state)?;
    Ok(NextResult { chapter_id: next_id, prompt_path })
}
