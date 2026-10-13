use crate::error::{CoreError, Result};
use crate::paragraphs::{labeled_source_payload, paragraphs_of};
use crate::prompt::build_chapter_prompt;
use crate::story::{natural_chapter_compare, StoryConfig};
use crate::story_fs::{
    load_state, load_story_config, read_raw_chapter, resolve_root, save_state, story_paths, work_file,
    write_atomic, ChapterState, ChapterStatus, StoryPaths, WorkKind,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct NextResult {
    pub chapter_id: String,
    pub prompt_path: PathBuf,
}

fn agent_instructions(id: &str, unpaired: &[(String, String)]) -> String {
    let mut lines = vec![
        "# Việc của agent sau khi dịch xong".to_string(),
        String::new(),
        format!("1. Ghi bản dịch tiếng Việt (GIỮ NGUYÊN nhãn [[n]] đầu mỗi đoạn) vào work/{id}.draft.md."),
        "2. Ghi đề xuất TÊN RIÊNG mới (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng)".to_string(),
        format!("   xuất hiện trong raw nhưng chưa có trong từ điển của prompt vào work/{id}.glossary.json, dạng:"),
        r#"   {"entries": [{"source": "chữ Hán trong raw", "target": "chép nguyên văn từ bản dịch", "category": "names|places|items|creatures|skills|common"}]}"#.to_string(),
        r#"   Tên NGƯỜI (category names) thêm "gender": "nam" hoặc "nữ" khi raw có căn cứ (她/他, 女/男, 姐/哥…), hoặc "biến thân" khi nhân vật đổi giới (xuyên vào / biến thành thân xác khác giới, raw gọi lúc 他 lúc 她); không chắc thì bỏ trường này."#.to_string(),
        r#"   Nhân vật đã có trong từ điển nhưng chưa nằm ở mục "Nhân vật có mặt trong chương" mà chương này lộ giới: khai {"source": "tên Hán", "category": "names", "gender": "nam|nữ|biến thân"}."#.to_string(),
        "   Chữ Hán nào đã có âm Hán-Việt trong từ điển của prompt thì tên mới chứa chữ đó phải phiên cùng âm (họ 段 đã là Đoàn thì không đề xuất Đoạn).".to_string(),
        r#"   Ngoài tên riêng, khai category "common" cho THUẬT NGỮ RIÊNG CỦA TRUYỆN lặp lại nhiều chương: tên chỉ số, bảng, âm báo của hệ thống (面板, 好感度, 积分, 检测到), khái niệm của thế giới truyện (末世, 丧尸, 大比, 成长体), tên tổ chức, chương trình (督察局, 春晚); target chép nguyên văn cách bạn vừa dịch. KHÔNG khai (kể cả vào names) từ thường, thành ngữ, lóng/meme, biệt danh trêu chọc (小辣椒), danh xưng (老X, X阿姨, X女士)."#.to_string(),
        "   Bỏ qua từ thông thường không riêng của truyện, chức danh, đại từ.".to_string(),
        format!("3. Cặp xưng hô — làm cả khi không có tên mới: ghi thêm vào cùng work/{id}.glossary.json."),
        r#"   Mỗi đôi nhân vật nói chuyện trực tiếp trong chương mà chưa có ở mục "Cặp xưng hô đang dùng": {"source": "甲→乙", "target": "X–Y", "category": "addressing"} (甲, 乙 là tên nhân vật; 甲 tự xưng X, gọi 乙 là Y, đúng như bạn vừa dịch; mỗi chiều một mục)."#.to_string(),
    ];
    let line = crate::prompt::unpaired_line(unpaired);
    if !line.is_empty() {
        lines.extend(line.lines().map(|text| format!("   {text}")));
    }
    lines.extend([
        r#"   Cặp ĐÃ CÓ mà chương này cho thấy quan hệ hai người đã khác lúc chốt cặp (đã là người yêu, đính hôn, vợ chồng, thầy trò, chủ tớ, nhận người thân, kết nghĩa, trở mặt) nên đã dịch bằng cặp khác: khai lại cặp đó với target đúng như vừa dịch và "note": "lý do ngắn", cả hai chiều."#.to_string(),
        r#"   Chỉ ghi {"entries": []} khi không có cả tên mới lẫn cặp xưng hô mới."#.to_string(),
        format!("4. Chạy: qt-ai check {id} (xem AGENTS.md để biết lệnh đầy đủ)."),
    ]);
    lines.join("\n")
}

/// Sinh + ghi work/<id>.prompt.md; trả về đường dẫn đã ghi.
fn write_prompt_file(paths: &StoryPaths, story: &StoryConfig, id: &str) -> Result<PathBuf> {
    let source = read_raw_chapter(paths, id)?;
    let base_glossary = crate::base::BaseStore::from_env().glossary(story.genre.setting);
    let cast = crate::cast::load_cast(paths);
    let system = build_chapter_prompt(&base_glossary, story, &cast, id, &source);
    let unpaired = crate::prompt::chapter_unpaired(&base_glossary, story, &cast, id, &source);
    let payload = labeled_source_payload(&paragraphs_of(&source));
    let prompt = format!("{system}\n\n---\n\n{payload}\n\n---\n\n{}\n", agent_instructions(id, &unpaired));
    let prompt_path = work_file(paths, id, WorkKind::Prompt);
    // Atomic + ép xuống đĩa: chương chỉ sang translating khi prompt đã đọc được (xem `run_next`).
    write_atomic(&prompt_path, &prompt)?;
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
