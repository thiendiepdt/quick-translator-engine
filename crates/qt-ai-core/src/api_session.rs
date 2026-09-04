//! Vòng dịch bằng API key, không cần agent: next → gọi model → draft có nhãn → glossary → check →
//! (bổ sung / soát / dịch lại) → accept. Port logic vòng dịch của qt-web
//! (ai-translation-workspace.tsx) lên trên cùng state machine `commands::*` mà agy dùng, nên hai
//! động cơ dùng chung folder truyện và đổi qua lại giữa chừng được.

use crate::api::{ApiError, TextModel};
use crate::check::check_violations;
use crate::commands::accept::run_accept;
use crate::commands::check::run_check;
use crate::commands::next::run_next;
use crate::commands::skip::run_skip;
use crate::error::{CoreError, Result};
use crate::glossary::collect_glossary_keys;
use crate::paragraphs::{labeled_repair_payload, labeled_source_payload, paragraphs_of, parse_labeled_translation};
use crate::prompt::{build_system_prompt, TranslationGlossary};
use crate::session::{read_progress, spawn_runner, LogStream, SessionEvent, SessionHandle, Sink, StopReason};
use crate::story::{natural_chapter_compare, StoryConfig};
use crate::story_fs::{
    load_state, load_story_config, read_raw_chapter, resolve_root, story_paths, work_file, write_text,
    ChapterStatus, StoryPaths, WorkKind,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Port GLOSSARY_EXTRACT_SYSTEM_PROMPT của qt-web/src/lib/ai-client.ts.
const GLOSSARY_EXTRACT_SYSTEM_PROMPT: &str = "Bạn nhận raw tiếng Trung và bản dịch tiếng Việt của cùng một chương truyện. \
Liệt kê các TÊN RIÊNG (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng) \
xuất hiện trong raw nhưng CHƯA có trong danh sách loại trừ, kèm đúng cách bản dịch đã phiên âm chúng. \
target phải chép nguyên văn từ bản dịch, không tự nghĩ phương án khác. \
category chỉ được là một trong: \"names\", \"places\", \"items\", \"creatures\", \"skills\". \
Bỏ qua từ chung, chức danh, đại từ. Không có tên mới thì trả entries rỗng. \
Chỉ xuất JSON dạng {\"entries\": [{\"source\": \"...\", \"target\": \"...\", \"category\": \"...\"}]}.";

/// Port system prompt của `buildAiTranslationReviewPrompt`, thêm yêu cầu giữ nhãn.
const REVIEW_SYSTEM_PROMPT: &str = "Đây là tác vụ soát tối thiểu một bản dịch tiểu thuyết hư cấu do người dùng cung cấp. \
Chỉ sửa đúng các vi phạm được liệt kê; tuyệt đối không đổi văn phong, thêm nội dung hoặc chỉnh phần khác. \
Bản dịch gồm các đoạn mở đầu bằng nhãn [[n]]; GIỮ NGUYÊN mọi nhãn, không gộp, không tách, không bỏ đoạn. \
Chỉ xuất toàn bộ bản dịch đã soát (kèm nhãn), không giải thích và không markdown.";

/// Khoảng cách tối thiểu giữa hai dòng log "đã nhận … ký tự" khi đang stream.
const STREAM_LOG_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct ApiSessionConfig {
    pub root: PathBuf,
    /// Đợi trước khi thử lại một chương sau lỗi API tạm thời.
    pub retry_delay: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChapterOutcome {
    Accepted { review_rounds: u32, warnings: usize, added_glossary: usize },
    /// check escalate sang error (quá vòng soát mà vẫn thiếu đoạn/quá ngắn) — state đã ghi.
    Errored { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ChapterFailure {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(transparent)]
    Core(#[from] CoreError),
}

type Log<'a> = &'a dyn Fn(String);

/// Ngữ cảnh một chương: mọi bước dịch/bổ sung/soát đều cần đúng bộ này.
struct Chapter<'a> {
    model: &'a dyn TextModel,
    id: &'a str,
    system: &'a str,
    paragraphs: &'a [String],
    story: &'a StoryConfig,
    cancel: &'a AtomicBool,
    log: Log<'a>,
}

fn format_chars(count: usize) -> String {
    if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

/// Gọi model và log tiến độ nhận stream thưa thớt.
fn generate(chapter: &Chapter, step: &str, system: &str, user: &str) -> std::result::Result<String, ApiError> {
    let (id, log) = (chapter.id, chapter.log);
    log(format!("{id}: {step} ({})…", chapter.model.label()));
    let mut last_log = Instant::now();
    let mut on_progress = |received: usize| {
        if last_log.elapsed() >= STREAM_LOG_INTERVAL {
            last_log = Instant::now();
            log(format!("{id}: đã nhận {} ký tự", format_chars(received)));
        }
    };
    chapter.model.generate(system, user, chapter.cancel, &mut on_progress)
}

/// Bản draft có nhãn ghi ra work/<id>.draft.md — đúng dạng `assemble_draft` đọc.
pub fn labeled_draft(draft: &[String]) -> String {
    let body = draft.iter().enumerate().map(|(index, text)| format!("[[{}]] {}", index + 1, text)).collect::<Vec<_>>().join("\n\n");
    format!("{body}\n")
}

fn write_draft(paths: &StoryPaths, id: &str, draft: &[String]) -> Result<()> {
    write_text(&work_file(paths, id, WorkKind::Draft), &labeled_draft(draft))
}

fn final_text(draft: &[String]) -> String {
    draft.join("\n\n")
}

fn chars_no_ws(draft: &[String]) -> usize {
    draft.iter().flat_map(|p| p.chars()).filter(|c| !c.is_whitespace()).count()
}

/// Dịch bổ sung đúng các đoạn `missing` (0-based) và ghi đè vào `draft` khi model trả về.
fn repair_missing(chapter: &Chapter, draft: &mut [String], missing: &[usize]) -> std::result::Result<(), ApiError> {
    if missing.is_empty() {
        return Ok(());
    }
    let output = generate(
        chapter,
        &format!("dịch bổ sung {} đoạn", missing.len()),
        chapter.system,
        &labeled_repair_payload(chapter.paragraphs, missing),
    )?;
    if let Some(parsed) = parse_labeled_translation(&output, chapter.paragraphs.len()) {
        for index in missing {
            if let Some(Some(text)) = parsed.get(*index) {
                draft[*index] = text.clone();
            }
        }
    }
    Ok(())
}

/// Lượt dịch chính: model trả đủ nhãn → draft; thiếu đoạn → dịch bổ sung một lần; vẫn thiếu → giữ
/// nguyên văn Hán (rule CJK bắt ở check, người đọc thấy chỗ hổng thay vì mất đoạn).
fn translate_full(chapter: &Chapter) -> std::result::Result<Vec<String>, ApiError> {
    let paragraphs = chapter.paragraphs;
    let payload = labeled_source_payload(paragraphs);
    let mut output = generate(chapter, "dịch", chapter.system, &payload)?;
    let mut parsed = parse_labeled_translation(&output, paragraphs.len());
    if parsed.is_none() {
        // Model bỏ hết nhãn — thử lại một lần rồi mới bó tay.
        (chapter.log)(format!("{}: model bỏ nhãn [[n]], dịch lại", chapter.id));
        output = generate(chapter, "dịch lại", chapter.system, &payload)?;
        parsed = parse_labeled_translation(&output, paragraphs.len());
    }
    let Some(parsed) = parsed else {
        return Err(ApiError::BadOutput("không có nhãn [[n]] nào trong bản dịch".to_string()));
    };
    let mut draft: Vec<String> = parsed.iter().zip(paragraphs).map(|(p, raw)| p.clone().unwrap_or_else(|| raw.clone())).collect();
    let missing: Vec<usize> = parsed.iter().enumerate().filter(|(_, p)| p.is_none()).map(|(i, _)| i).collect();
    repair_missing(chapter, &mut draft, &missing)?;
    Ok(draft)
}

/// Trích tên riêng mới → work/<id>.glossary.json. Bước phụ: mọi lỗi nuốt, ghi entries rỗng.
fn harvest_glossary(chapter: &Chapter, paths: &StoryPaths, raw: &str, draft: &[String]) -> Result<()> {
    let (id, log) = (chapter.id, chapter.log);
    let mut exclude: Vec<String> =
        collect_glossary_keys(&TranslationGlossary::new(), &chapter.story.glossary).into_iter().collect();
    exclude.sort();
    let user = json!({ "exclude": exclude, "raw": raw, "translation": final_text(draft) }).to_string();
    let entries = match chapter.model.complete_json(GLOSSARY_EXTRACT_SYSTEM_PROMPT, &user) {
        Ok(text) => serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|value| value.get("entries").cloned().or(Some(value)))
            .filter(Value::is_array)
            .unwrap_or_else(|| Value::Array(vec![])),
        Err(error) => {
            log(format!("{id}: bỏ qua trích glossary — {error}"));
            Value::Array(vec![])
        }
    };
    write_text(&work_file(paths, id, WorkKind::Glossary), &format!("{}\n", json!({ "entries": entries })))
}

/// Soát tối thiểu theo danh sách vấn đề của check; chỉ nhận bản soát khi còn đủ nhãn và ít vi phạm hơn.
fn review(chapter: &Chapter, round: u32, draft: &mut [String], issues: &[String]) -> std::result::Result<(), ApiError> {
    let (id, log, story) = (chapter.id, chapter.log, chapter.story);
    let list = issues.iter().map(|issue| format!("- {issue}")).collect::<Vec<_>>().join("\n");
    let user = format!(
        "Bản dịch dưới đây có các vi phạm được phát hiện tự động (nhãn [[n]] chỉ đoạn):\n\n{list}\n\n\
Kiểm tra từng vi phạm. Nếu thực sự sai, chỉ thay từ hoặc cụm gây lỗi bằng phương án ngắn nhất. Nếu đúng trong ngữ cảnh, giữ nguyên. \
Giữ nguyên toàn bộ chữ, thứ tự câu, dấu câu và ngắt đoạn không liên quan. Không chau chuốt, viết lại, thêm từ nối hoặc thêm miêu tả. \
Đoạn nào còn nguyên chữ Hán thì dịch đoạn đó theo đúng quy tắc.\n\n---\n\n{}",
        labeled_draft(draft)
    );
    let output = generate(chapter, &format!("soát lần {round}"), REVIEW_SYSTEM_PROMPT, &user)?;
    let Some(reviewed) = parse_labeled_translation(&output, draft.len()) else {
        log(format!("{id}: bản soát mất nhãn — bỏ"));
        return Ok(());
    };
    if reviewed.iter().any(Option::is_none) {
        log(format!("{id}: bản soát thiếu đoạn — bỏ"));
        return Ok(());
    }
    let reviewed: Vec<String> = reviewed.into_iter().flatten().collect();
    let before = check_violations(&final_text(draft), &story.check_rules).len();
    let after = check_violations(&final_text(&reviewed), &story.check_rules).len();
    if after >= before {
        log(format!("{id}: bản soát không giảm vi phạm ({before} → {after}) — bỏ"));
        return Ok(());
    }
    draft.clone_from_slice(&reviewed);
    Ok(())
}

/// Dịch một chương đang ở trạng thái translating tới khi accept/error. Lỗi API trả về nguyên để
/// vòng ngoài quyết định thử lại/skip; lỗi core (IO, state) là lỗi nội bộ.
pub fn translate_chapter(
    root: &Path,
    id: &str,
    model: &dyn TextModel,
    cancel: &AtomicBool,
    log: Log,
) -> std::result::Result<ChapterOutcome, ChapterFailure> {
    let paths = story_paths(&resolve_root(root));
    let raw = read_raw_chapter(&paths, id)?;
    let paragraphs = paragraphs_of(&raw);
    let story = load_story_config(&paths)?;
    let system = build_system_prompt(&TranslationGlossary::new(), Some(&story), Some(&raw));
    let min_ratio = load_state(&paths)?.settings.min_length_ratio;
    let chapter = Chapter { model, id, system: &system, paragraphs: &paragraphs, story: &story, cancel, log };

    let mut draft = translate_full(&chapter)?;
    write_draft(&paths, id, &draft)?;
    harvest_glossary(&chapter, &paths, &raw, &draft)?;

    let mut rounds = 0;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(ApiError::Cancelled.into());
        }
        let check = run_check(root, id)?;
        if check.pass {
            let accepted = run_accept(root, id, false)?;
            return Ok(ChapterOutcome::Accepted {
                review_rounds: rounds,
                warnings: accepted.warnings.len(),
                added_glossary: accepted.added_glossary,
            });
        }
        if check.escalated_to_error {
            let reason = load_state(&paths)?.chapters.get(id).and_then(|c| c.reason.clone()).unwrap_or_default();
            return Ok(ChapterOutcome::Errored { reason });
        }
        rounds += 1;
        let missing: Vec<usize> = check.missing.iter().map(|label| label - 1).collect();
        if !missing.is_empty() {
            repair_missing(&chapter, &mut draft, &missing)?;
        } else if check.ratio < min_ratio {
            // Quá ngắn (tóm tắt/lược ý): dịch lại cả chương, giữ bản dài hơn — như qt-web.
            let again = translate_full(&chapter)?;
            if chars_no_ws(&again) > chars_no_ws(&draft) {
                draft = again;
            }
        } else if !check.violations.is_empty() {
            review(&chapter, rounds, &mut draft, &check.issues)?;
        }
        write_draft(&paths, id, &draft)?;
    }
}

/// Chương translating dở (phiên trước bị huỷ) được làm tiếp trước; hết thì phát chương mới qua `next`.
fn pick_chapter(root: &Path) -> Result<Option<String>> {
    let paths = story_paths(&resolve_root(root));
    let state = load_state(&paths)?;
    let mut pending: Vec<&String> =
        state.chapters.iter().filter(|(_, c)| c.status == ChapterStatus::Translating).map(|(id, _)| id).collect();
    pending.sort_by(|a, b| natural_chapter_compare(a, b));
    if let Some(id) = pending.first() {
        return Ok(Some((*id).clone()));
    }
    if !state.chapters.values().any(|c| c.status == ChapterStatus::Queued) {
        return Ok(None);
    }
    Ok(Some(run_next(root)?.chapter_id))
}

fn sleep_unless_cancelled(delay: Duration, cancel: &AtomicBool) {
    let deadline = Instant::now() + delay;
    while Instant::now() < deadline && !cancel.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(50).min(deadline - Instant::now()));
    }
}

fn emit_progress(root: &Path, sink: &Sink) {
    if let Ok(progress) = read_progress(root) {
        sink(SessionEvent::Progress(progress));
    }
}

pub fn api_loop(config: &ApiSessionConfig, model: &dyn TextModel, sink: &Sink, cancel: &AtomicBool) -> StopReason {
    let root = config.root.as_path();
    let log = |line: String| sink(SessionEvent::AgyLog { line, stream: LogStream::Stdout });
    sink(SessionEvent::Started { session_no: 1 });
    log(format!("Động cơ API: {}", model.label()));
    let mut failed_chapters = 0;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return StopReason::UserCancelled;
        }
        let id = match pick_chapter(root) {
            Ok(Some(id)) => id,
            Ok(None) => return StopReason::Finished,
            Err(error) => return StopReason::Internal { message: error.to_string() },
        };
        emit_progress(root, sink);
        let mut attempt = 0;
        let outcome = loop {
            attempt += 1;
            match translate_chapter(root, &id, model, cancel, &log) {
                Ok(outcome) => break Ok(outcome),
                Err(ChapterFailure::Api(ApiError::Cancelled)) => return StopReason::UserCancelled,
                Err(ChapterFailure::Api(ApiError::Blocked(reason))) => break Err(format!("model từ chối: {reason}")),
                Err(ChapterFailure::Api(error)) if attempt < 2 => {
                    log(format!("{id}: {error} — thử lại sau {} giây", config.retry_delay.as_secs()));
                    sleep_unless_cancelled(config.retry_delay, cancel);
                    if cancel.load(Ordering::SeqCst) {
                        return StopReason::UserCancelled;
                    }
                }
                Err(ChapterFailure::Api(error)) => {
                    failed_chapters += 1;
                    if failed_chapters >= 2 {
                        return StopReason::ApiFailed { message: error.to_string() };
                    }
                    break Err(format!("lỗi API: {error}"));
                }
                Err(ChapterFailure::Core(error)) => return StopReason::Internal { message: error.to_string() },
            }
        };
        match outcome {
            Ok(ChapterOutcome::Accepted { review_rounds, warnings, added_glossary }) => {
                failed_chapters = 0;
                log(format!("{id}: chốt (soát {review_rounds} lần, {warnings} cảnh báo, +{added_glossary} glossary)"));
            }
            Ok(ChapterOutcome::Errored { reason }) => {
                failed_chapters = 0;
                log(format!("{id}: lỗi — {reason}"));
            }
            Err(reason) => {
                if let Err(error) = run_skip(root, &id, &reason) {
                    return StopReason::Internal { message: error.to_string() };
                }
                log(format!("{id}: bỏ qua — {reason}"));
            }
        }
        emit_progress(root, sink);
    }
}

/// Bắt đầu vòng API trên thread riêng, cùng lock/handle/event với vòng agy.
pub fn start_api_session(config: ApiSessionConfig, model: Arc<dyn TextModel>, sink: Sink) -> Result<SessionHandle> {
    let root = config.root.clone();
    let loop_sink = sink.clone();
    spawn_runner(root, sink, move |cancel| api_loop(&config, model.as_ref(), &loop_sink, &cancel))
}
