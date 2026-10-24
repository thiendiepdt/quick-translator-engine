//! Vòng dịch bằng API key, không cần agent: next → gọi model → draft có nhãn → glossary → check →
//! (bổ sung / soát / dịch lại) → accept. Port logic vòng dịch của qt-web
//! (ai-translation-workspace.tsx) lên trên cùng state machine `commands::*` mà agy dùng, nên hai
//! động cơ dùng chung folder truyện và đổi qua lại giữa chừng được.

use crate::api::{format_tokens, ApiError, ApiStep, Generated, TextModel, Usage};
use crate::check::{check_violations, contains_han, glossary_near_misses, GLOSSARY_NAME_ISSUE};
use crate::commands::accept::run_accept;
use crate::commands::check::run_check;
use crate::commands::next::run_next;
use crate::commands::retry::run_retry;
use crate::commands::skip::run_skip;
use crate::error::{CoreError, Result};
use crate::glossary::{collect_glossary_keys, glossary_key_touches_source};
use crate::paragraphs::{labeled_repair_payload, labeled_source_payload, paragraphs_of, parse_labeled_translation};
use crate::prompt::{build_chapter_prompt, merge_story_glossary};
use crate::session::{read_progress, spawn_runner, LogStream, SessionEvent, SessionHandle, Sink, StopReason};
use crate::story::{natural_chapter_compare, Glossary, StoryConfig};
use crate::story_fs::{
    load_state, load_story_config, now_ms, read_raw_chapter, resolve_root, save_state, story_paths, work_file, write_text,
    ChapterStatus, StoryPaths, WorkKind,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Port GLOSSARY_EXTRACT_SYSTEM_PROMPT của qt-web/src/lib/ai-client.ts — lượt trích riêng (fallback khi
/// model không kèm khối glossary trong lượt dịch).
const GLOSSARY_EXTRACT_SYSTEM_PROMPT: &str = "Bạn nhận raw tiếng Trung và bản dịch tiếng Việt của cùng một chương truyện. \
Liệt kê các TÊN RIÊNG (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng) \
xuất hiện trong raw nhưng CHƯA có trong danh sách loại trừ, kèm đúng cách bản dịch đã phiên âm chúng. \
target phải chép nguyên văn từ bản dịch, không tự nghĩ phương án khác. \
category chỉ được là một trong: \"names\", \"places\", \"items\", \"creatures\", \"skills\", \"common\". \
Ngoài tên riêng, khai category \"common\" cho THUẬT NGỮ RIÊNG CỦA TRUYỆN lặp lại nhiều chương: tên chỉ số, bảng, âm báo của hệ thống (面板, 好感度, 积分, 检测到), khái niệm của thế giới truyện (末世, 丧尸, 大比, 成长体), tên tổ chức, chương trình (督察局, 春晚); target chép nguyên văn cách bạn vừa dịch. KHÔNG khai (kể cả vào names) từ thường, thành ngữ, lóng/meme, biệt danh trêu chọc (小辣椒), danh xưng (老X, X阿姨, X女士). \
Bỏ qua từ thông thường không riêng của truyện, chức danh, đại từ.\
 Chữ Hán nào đã có âm Hán-Việt trong từ điển sẵn có thì tên mới chứa chữ đó phải phiên cùng âm (họ 段 đã là Đoàn thì không đề xuất Đoạn). \
Tên NGƯỜI (category \"names\") thêm \"gender\": \"nam\" hoặc \"nữ\" khi raw có căn cứ (她/他, 女/男, 姐/哥…), hoặc \"biến thân\" khi nhân vật đổi giới (xuyên vào / biến thành thân xác khác giới, raw gọi lúc 他 lúc 她); không chắc thì bỏ trường này. \
Thêm các CẶP XƯNG HÔ mới trong thoại với category \"addressing\": source là \"甲→乙\" (hai TÊN NHÂN VẬT như trong raw), \
target là \"X–Y\" với X là cách 甲 tự xưng và Y là cách 甲 gọi 乙 trong bản dịch (ví dụ \"anh–em\", \"ta–ngươi\", \"tôi–cậu\"); \
mỗi chiều một mục, chỉ ghi cặp chưa có trong danh sách loại trừ. \
Không có gì mới thì trả entries rỗng. \
Chỉ xuất JSON dạng {\"entries\": [{\"source\": \"...\", \"target\": \"...\", \"category\": \"...\"}]}.";

/// Dấu mở khối glossary cuối lượt dịch. Không khớp regex nhãn `[[n]]` (chỉ nhận chữ số).
pub const GLOSSARY_MARKER: &str = "[[glossary]]";

/// Đuôi user message lượt dịch: model trả glossary ngay trong cùng lượt (như luồng agy ghi glossary.json),
/// tiết kiệm trọn lượt trích riêng (raw + bản dịch gửi lại lần hai ≈ 2× chương).
const GLOSSARY_INLINE_INSTRUCTION: &str = "Sau đoạn dịch cuối cùng, xuống dòng và thêm đúng một dòng `[[glossary]]`, rồi một JSON object \
{\"entries\": [{\"source\": \"...\", \"target\": \"...\", \"category\": \"...\"}]} liệt kê các TÊN RIÊNG \
(nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng) xuất hiện trong raw mà CHƯA có trong \"Từ điển riêng của truyện\"; \
target chép nguyên văn cách bạn vừa dịch; category chỉ được là \"names\", \"places\", \"items\", \"creatures\", \"skills\", \"common\". \
Ngoài tên riêng, khai category \"common\" cho THUẬT NGỮ RIÊNG CỦA TRUYỆN lặp lại nhiều chương: tên chỉ số, bảng, âm báo của hệ thống (面板, 好感度, 积分, 检测到), khái niệm của thế giới truyện (末世, 丧尸, 大比, 成长体), tên tổ chức, chương trình (督察局, 春晚); target chép nguyên văn cách bạn vừa dịch. KHÔNG khai (kể cả vào names) từ thường, thành ngữ, lóng/meme, biệt danh trêu chọc (小辣椒), danh xưng (老X, X阿姨, X女士). \
Bỏ qua từ thông thường không riêng của truyện, chức danh, đại từ.\
 Chữ Hán nào đã có âm Hán-Việt trong từ điển sẵn có thì tên mới chứa chữ đó phải phiên cùng âm (họ 段 đã là Đoàn thì không đề xuất Đoạn). Tên NGƯỜI (category \"names\") thêm \"gender\": \"nam\" hoặc \"nữ\" khi raw có căn cứ (她/他, 女/男, 姐/哥…), hoặc \"biến thân\" khi nhân vật đổi giới (xuyên vào / biến thành thân xác khác giới, raw gọi lúc 他 lúc 她); không chắc thì bỏ trường này. \
Nhân vật đã có trong từ điển nhưng chưa nằm ở mục \"Nhân vật có mặt trong chương\" mà chương này lộ giới: khai {\"source\": \"tên Hán\", \"category\": \"names\", \"gender\": \"...\"}. \
Cặp xưng hô mới trong thoại ghi category \"addressing\": source \"甲→乙\" (hai TÊN NHÂN VẬT như trong raw, không dùng chức danh chung như 保安/助理), \
target \"X–Y\" với X là cách 甲 tự xưng và Y là cách 甲 gọi 乙 trong bản dịch; mỗi chiều một mục. \
Cặp ĐÃ CÓ trong mục \"Cặp xưng hô đang dùng\" mà chương này cho thấy quan hệ hai người đã khác lúc chốt cặp \
(đã là người yêu, đính hôn, vợ chồng, thầy trò, chủ tớ, nhận người thân, kết nghĩa, trở mặt thành thù) nên bạn đã dịch bằng cặp khác: \
khai lại cặp đó với target đúng như vừa dịch và \"note\": \"lý do ngắn\", cả hai chiều; giận dỗi nhất thời, đóng kịch, nói đùa thì không khai. \
Cặp xưng hô khai cả khi không có tên mới; chỉ trả {\"entries\": []} khi không có cả tên mới lẫn cặp xưng hô mới. \
Khối này nằm ngoài bản dịch, không có nhãn [[n]].";

/// Tách khối `[[glossary]]` cuối output lượt dịch: (phần bản dịch, mảng entries nếu parse được).
/// Chịu rào ```json quanh JSON. Không có dấu → trả nguyên văn.
pub fn split_glossary_block(output: &str) -> (&str, Option<Value>) {
    let Some(at) = output.rfind(GLOSSARY_MARKER) else { return (output, None) };
    let text = output[..at].trim_end();
    let entries = crate::api::extract_json_object(&output[at + GLOSSARY_MARKER.len()..])
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .and_then(|value| value.get("entries").cloned())
        .filter(Value::is_array);
    (text, entries)
}

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
    /// Lần chờ đầu trước khi gọi lại sau lỗi API tạm thời (mạng, 408/429/5xx, stream đứt); mỗi lần sau gấp đôi.
    /// Cũng là khoảng chờ trước khi dịch lại chương khi model trả hỏng (không nhãn, rỗng).
    pub retry_delay: Duration,
    /// Trần thời gian chờ: chạm trần thì cứ mỗi `retry_max` gọi lại một lần, tới khi được hoặc người dùng dừng.
    pub retry_max: Duration,
}

/// Chờ trước lần gọi lại thứ `attempt` (từ 1): `base`, 2·`base`, 4·`base`… tối đa `max`.
pub fn retry_wait(base: Duration, max: Duration, attempt: u32) -> Duration {
    base.saturating_mul(1u32 << attempt.saturating_sub(1).min(16)).min(max)
}

fn format_wait(wait: Duration) -> String {
    if wait >= Duration::from_secs(1) {
        format!("{} giây", wait.as_secs())
    } else {
        format!("{} ms", wait.as_millis())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChapterOutcome {
    Accepted { review_rounds: u32, warnings: usize, added_glossary: usize, usage: Usage },
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
    /// Glossary app + truyện đã gộp — check dùng để bắt tên lệch, soát dùng để chấm bản soát.
    glossary: &'a Glossary,
    cast: &'a crate::cast::Cast,
    /// Dòng nêu nhân vật có mặt chưa có cặp xưng hô (rỗng nếu không có) — nối vào lời dặn khối glossary.
    unpaired: &'a str,
    cancel: &'a AtomicBool,
    log: Log<'a>,
    /// Token cộng dồn mọi lượt gọi của chương (provider không báo thì giữ 0).
    usage: Mutex<Usage>,
    retry_delay: Duration,
    retry_max: Duration,
}

impl Chapter<'_> {
    /// Ghi nhận một lượt gọi xong: cộng token, log token; hub trả chữ lỗi mã hoá đã được sửa thì log để người dùng biết.
    fn record(&self, label: &str, generated: &Generated) {
        if generated.repaired_encoding {
            (self.log)(format!("{}: phản hồi API bị lỗi mã hoá — đã tự sửa", self.id));
        }
        if let Some(usage) = generated.usage {
            *self.usage.lock().unwrap() += usage;
            (self.log)(format!("{}: {label} — {}", self.id, usage.summary()));
        }
    }

    fn usage(&self) -> Usage {
        *self.usage.lock().unwrap()
    }
}

fn format_chars(count: usize) -> String {
    if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

/// Số lượt gọi tối đa cho một bước khi hub cứ trả chữ lỗi mã hoá không sửa được.
const MOJIBAKE_MAX_ATTEMPTS: u32 = 3;

/// Gọi lại đúng lượt API này khi lỗi tạm thời (provider quá tải, hết hạn mức theo phút, mạng, stream đứt): chờ tăng
/// dần tới trần rồi cứ trần một lần, không giới hạn số lần — provider chết vài phút là chuyện thường, bỏ chương thì
/// người dùng mất chương, dịch lại cả chương thì mất lượt đã trả. Bấm Dừng thì trả `Cancelled`; lỗi khác trả ngay.
/// Riêng chữ lỗi mã hoá không sửa được (`Mojibake`) có ngân sách riêng `MOJIBAKE_MAX_ATTEMPTS` lượt: hub vẫn trả lời
/// (không phải đang chết) mà hỏng mãi thì chờ thêm không ích gì — trả `MojibakeExhausted` để chương chuyển lỗi.
fn with_retry<T>(
    chapter: &Chapter,
    label: &str,
    mut call: impl FnMut() -> std::result::Result<T, ApiError>,
) -> std::result::Result<T, ApiError> {
    let mut attempt = 0;
    let mut mojibake = 0;
    loop {
        match call() {
            Err(ApiError::Mojibake) if mojibake + 1 >= MOJIBAKE_MAX_ATTEMPTS => {
                (chapter.log)(format!("{}: {label}: {} — {MOJIBAKE_MAX_ATTEMPTS} lượt liền, thôi", chapter.id, ApiError::Mojibake));
                return Err(ApiError::MojibakeExhausted);
            }
            Err(error) if error.is_transient() => {
                if error == ApiError::Mojibake {
                    mojibake += 1;
                }
                attempt += 1;
                let wait = retry_wait(chapter.retry_delay, chapter.retry_max, attempt);
                (chapter.log)(format!("{}: {label}: {error} — thử lại lần {attempt} sau {}", chapter.id, format_wait(wait)));
                sleep_unless_cancelled(wait, chapter.cancel);
                if chapter.cancel.load(Ordering::SeqCst) {
                    return Err(ApiError::Cancelled);
                }
            }
            other => return other,
        }
    }
}

/// Gọi model (mức nghĩ theo `step`) và log tiến độ nhận stream thưa thớt; `label` là tên bước trong log.
/// Xong thì log token của lượt (nếu provider báo).
fn generate(chapter: &Chapter, step: ApiStep, label: &str, system: &str, user: &str) -> std::result::Result<String, ApiError> {
    let (id, log) = (chapter.id, chapter.log);
    log(format!("{id}: {label} ({})…", chapter.model.label()));
    let mut last_log = Instant::now();
    let mut on_progress = |received: usize| {
        if last_log.elapsed() >= STREAM_LOG_INTERVAL {
            last_log = Instant::now();
            log(format!("{id}: đã nhận {} ký tự", format_chars(received)));
        }
    };
    let generated = with_retry(chapter, label, || chapter.model.generate(step, system, user, chapter.cancel, &mut on_progress))?;
    chapter.record(label, &generated);
    Ok(generated.text)
}

/// `limit` ký tự đầu của output, xuống dòng gộp thành khoảng trắng, bọc «…» để lọt vào log/lý do skip.
fn output_head(text: &str, limit: usize) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut head: String = flat.chars().take(limit).collect();
    if flat.chars().count() > limit {
        head.push('…');
    }
    format!("«{head}»")
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
    // Đoạn thiếu bị chặn riêng: giữ nguyên văn Hán (check thấy đoạn còn Hán, dịch lại tiếp), không vứt cả chương.
    retranslate(chapter, draft, missing, "dịch bổ sung", "giữ nguyên văn Hán đoạn thiếu")
}

/// Dịch lại từ raw các đoạn `indices` (0-based) còn chữ Hán. Bị chặn thì giữ bản cũ; hết vòng mà vẫn còn
/// Hán thì check chuyển chương sang error, không bao giờ chốt kèm cảnh báo.
fn repair_untranslated(chapter: &Chapter, draft: &mut [String], indices: &[usize]) -> std::result::Result<(), ApiError> {
    (chapter.log)(format!("{}: còn chữ Hán ở {} đoạn — dịch lại", chapter.id, indices.len()));
    retranslate(chapter, draft, indices, "dịch lại", "giữ bản cũ")
}

fn retranslate(
    chapter: &Chapter,
    draft: &mut [String],
    indices: &[usize],
    action: &str,
    when_blocked: &str,
) -> std::result::Result<(), ApiError> {
    if indices.is_empty() {
        return Ok(());
    }
    let output = match generate(
        chapter,
        ApiStep::Translate,
        &format!("{action} {} đoạn", indices.len()),
        chapter.system,
        &labeled_repair_payload(chapter.paragraphs, indices),
    ) {
        Ok(output) => output,
        Err(ApiError::Blocked(reason)) => {
            (chapter.log)(format!("{}: {action} bị từ chối ({reason}) — {when_blocked}", chapter.id));
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    if let Some(parsed) = parse_labeled_translation(split_glossary_block(&output).0, chapter.paragraphs.len()) {
        for index in indices {
            if let Some(Some(text)) = parsed.get(*index) {
                draft[*index] = text.clone();
            }
        }
    }
    Ok(())
}

/// Lượt dịch chính: model trả đủ nhãn → draft; thiếu đoạn → dịch bổ sung một lần; vẫn thiếu → giữ
/// nguyên văn Hán (rule CJK bắt ở check, người đọc thấy chỗ hổng thay vì mất đoạn). Kèm entries glossary
/// nếu model trả khối `[[glossary]]` cuối output.
fn translate_full(chapter: &Chapter) -> std::result::Result<(Vec<String>, Option<Value>), ApiError> {
    let paragraphs = chapter.paragraphs;
    let unpaired = if chapter.unpaired.is_empty() { String::new() } else { format!("\n{}", chapter.unpaired) };
    let payload = format!("{}\n\n{GLOSSARY_INLINE_INSTRUCTION}{unpaired}", labeled_source_payload(paragraphs));
    let mut output = generate(chapter, ApiStep::Translate, "dịch", chapter.system, &payload)?;
    let (mut text, mut glossary) = split_glossary_block(&output);
    let mut parsed = parse_labeled_translation(text, paragraphs.len());
    if parsed.is_none() {
        // Model bỏ hết nhãn — thử lại một lần rồi mới bó tay.
        (chapter.log)(format!("{}: model bỏ nhãn [[n]], dịch lại", chapter.id));
        output = generate(chapter, ApiStep::Translate, "dịch lại", chapter.system, &payload)?;
        (text, glossary) = split_glossary_block(&output);
        parsed = parse_labeled_translation(text, paragraphs.len());
    }
    let Some(parsed) = parsed else {
        // Không có gì để ghép → không thể biết model từ chối hay dịch mà bỏ nhãn; ghi đầu output ra log
        // và kèm vào lý do để người dùng thấy ngay.
        let head = output_head(text, 200);
        (chapter.log)(format!("{}: model trả không nhãn, đầu output: {head}", chapter.id));
        return Err(ApiError::BadOutput(format!("không có nhãn [[n]] nào trong bản dịch — model trả: {}", output_head(text, 80))));
    };
    let mut draft: Vec<String> = parsed.iter().zip(paragraphs).map(|(p, raw)| p.clone().unwrap_or_else(|| raw.clone())).collect();
    let missing: Vec<usize> = parsed.iter().enumerate().filter(|(_, p)| p.is_none()).map(|(i, _)| i).collect();
    repair_missing(chapter, &mut draft, &missing)?;
    Ok((draft, glossary))
}

/// Trích tên riêng mới → work/<id>.glossary.json. Có khối glossary từ lượt dịch thì dùng luôn; không thì
/// gọi lượt trích riêng. Bước phụ: mọi lỗi nuốt, ghi entries rỗng.
fn harvest_glossary(
    chapter: &Chapter,
    paths: &StoryPaths,
    raw: &str,
    draft: &[String],
    inline: Option<Value>,
) -> Result<()> {
    let (id, log) = (chapter.id, chapter.log);
    if let Some(entries) = inline {
        log(format!("{id}: glossary kèm lượt dịch — {} đề xuất", entries.as_array().map_or(0, Vec::len)));
        return write_text(&work_file(paths, id, WorkKind::Glossary), &format!("{}\n", json!({ "entries": entries })));
    }
    // Chỉ gửi key chương này chạm tới — sanitize vốn chặn đề xuất không có trong raw, gửi cả glossary là phí token.
    let base_glossary = crate::base::BaseStore::from_env().glossary(chapter.story.genre.setting);
    let mut exclude: Vec<String> = collect_glossary_keys(&base_glossary, &chapter.story.glossary)
        .into_iter()
        .filter(|key| glossary_key_touches_source(key, raw))
        .collect();
    exclude.sort();
    let user = json!({ "exclude": exclude, "raw": raw, "translation": final_text(draft) }).to_string();
    let extract = || chapter.model.complete_json(ApiStep::Glossary, GLOSSARY_EXTRACT_SYSTEM_PROMPT, &user, chapter.cancel);
    let entries = match with_retry(chapter, "trích glossary", extract) {
        Ok(generated) => {
            chapter.record("trích glossary", &generated);
            serde_json::from_str::<Value>(&generated.text)
                .ok()
                .and_then(|value| value.get("entries").cloned().or(Some(value)))
                .filter(Value::is_array)
                .unwrap_or_else(|| Value::Array(vec![]))
        }
        Err(error) => {
            log(format!("{id}: bỏ qua trích glossary — {error}"));
            Value::Array(vec![])
        }
    };
    write_text(&work_file(paths, id, WorkKind::Glossary), &format!("{}\n", json!({ "entries": entries })))
}

/// Kết quả một vòng soát: bản nháp đã đổi (hoặc bản soát bị bỏ) / model xét xong và giữ nguyên toàn bộ.
#[derive(Debug, PartialEq, Eq)]
enum ReviewOutcome {
    Changed,
    Kept,
}

/// Soát tối thiểu theo danh sách vấn đề của check; chỉ nhận bản soát khi còn đủ nhãn và ít vi phạm hơn
/// (đoạn còn chữ Hán xét trước, rồi mới tới tổng vi phạm rule + tên lệch glossary).
/// Model trả y hệt bản cũ nghĩa là đã xét từng vi phạm và thấy đúng ngữ cảnh — lặp thêm chỉ tốn tiền.
fn review(chapter: &Chapter, round: u32, draft: &mut [String], issues: &[String]) -> std::result::Result<ReviewOutcome, ApiError> {
    let (id, log, story) = (chapter.id, chapter.log, chapter.story);
    let (names, others): (Vec<&String>, Vec<&String>) = issues.iter().partition(|issue| issue.contains(GLOSSARY_NAME_ISSUE));
    let bullets = |items: &[&String]| items.iter().map(|issue| format!("- {issue}")).collect::<Vec<_>>().join("\n");
    let names_block = if names.is_empty() {
        String::new()
    } else {
        format!(
            "Tên theo từ điển — BẮT BUỘC sửa: thay đúng cụm sai bằng tên trong từ điển, không giữ nguyên, không tự đổi sang âm khác.\n\n{}\n\n",
            bullets(&names)
        )
    };
    let list = bullets(&others);
    let user = format!(
        "Bản dịch dưới đây có các vi phạm được phát hiện tự động (nhãn [[n]] chỉ đoạn):\n\n{names_block}{list}\n\n\
Kiểm tra từng vi phạm. Nếu thực sự sai, chỉ thay từ hoặc cụm gây lỗi bằng phương án ngắn nhất. Nếu đúng trong ngữ cảnh, giữ nguyên. \
Giữ nguyên toàn bộ chữ, thứ tự câu, dấu câu và ngắt đoạn không liên quan. Không chau chuốt, viết lại, thêm từ nối hoặc thêm miêu tả. \
Đoạn nào còn nguyên chữ Hán thì dịch đoạn đó theo đúng quy tắc.\n\n---\n\n{}",
        labeled_draft(draft)
    );
    let output = match generate(chapter, ApiStep::Review, &format!("soát lần {round}"), REVIEW_SYSTEM_PROMPT, &user) {
        Ok(output) => output,
        // Bản dịch đã có; bước soát bị chặn thì chốt kèm cảnh báo như model xét và giữ nguyên — đốt thêm
        // vòng cũng bị chặn tiếp, còn skip là mất chương dịch xong.
        Err(ApiError::Blocked(reason)) => {
            log(format!("{id}: model từ chối soát ({reason}) — chốt kèm cảnh báo"));
            return Ok(ReviewOutcome::Kept);
        }
        Err(error) => return Err(error),
    };
    let Some(reviewed) = parse_labeled_translation(split_glossary_block(&output).0, draft.len()) else {
        log(format!("{id}: bản soát mất nhãn — bỏ"));
        return Ok(ReviewOutcome::Changed);
    };
    if reviewed.iter().any(Option::is_none) {
        log(format!("{id}: bản soát thiếu đoạn — bỏ"));
        return Ok(ReviewOutcome::Changed);
    }
    let reviewed: Vec<String> = reviewed.into_iter().flatten().collect();
    if reviewed.iter().map(|p| p.trim()).eq(draft.iter().map(|p| p.trim())) {
        if !names.is_empty() {
            // Tên theo từ điển không được "đúng ngữ cảnh": soát tiếp; hết vòng thì check chốt kèm cảnh báo.
            log(format!("{id}: model giữ nguyên tên sai so với từ điển — soát tiếp"));
            return Ok(ReviewOutcome::Changed);
        }
        log(format!("{id}: model giữ nguyên bản dịch (vi phạm đúng ngữ cảnh) — chốt kèm cảnh báo"));
        return Ok(ReviewOutcome::Kept);
    }
    let score = |text: &[String]| {
        let han = text.iter().filter(|p| contains_han(p) || crate::mojibake::mojibake_score(p) > 0).count();
        let parsed: Vec<Option<String>> = text.iter().cloned().map(Some).collect();
        let no_names = crate::story::StringMap::new();
        let names = chapter.glossary.get("names").unwrap_or(&no_names);
        let addressing = crate::cast::addressing_issues(chapter.paragraphs, &parsed, names, chapter.cast).len();
        let near = glossary_near_misses(chapter.paragraphs, &parsed, chapter.glossary).len();
        let syllables = text.iter().filter(|p| !crate::vn_syllable::suspect_syllables(p).is_empty()).count();
        (han, check_violations(&final_text(text), &story.check_rules, story.genre.setting).len() + addressing + near + syllables)
    };
    let (before_han, before) = score(draft);
    let (after_han, after) = score(&reviewed);
    if (after_han, after) >= (before_han, before) {
        log(format!("{id}: bản soát không giảm vi phạm ({before} → {after}) — bỏ"));
        return Ok(ReviewOutcome::Changed);
    }
    draft.clone_from_slice(&reviewed);
    Ok(ReviewOutcome::Changed)
}

/// Đổi xưng hô theo mốc chương, khai giới ngược, tên tự sửa theo glossary — người dùng cần thấy ngay
/// trong log để hoàn tác nếu sai.
fn log_cast_notes(chapter: &Chapter, accepted: &crate::commands::accept::AcceptResult) {
    for note in &accepted.cast_notes {
        (chapter.log)(format!("{}: {note}", chapter.id));
    }
    if !accepted.glossary_fixes.is_empty() {
        (chapter.log)(format!("{}: tự sửa theo glossary: {}", chapter.id, accepted.glossary_fixes.join(", ")));
    }
}

/// Dịch một chương đang ở trạng thái translating tới khi accept/error. Lỗi API tạm thời được gọi lại ngay tại
/// lượt đó (`with_retry`, theo `retry_delay`/`retry_max` của `config`); lỗi API khác trả về nguyên để vòng ngoài
/// quyết định thử lại/skip; lỗi core (IO, state) là lỗi nội bộ. Hub trả chữ lỗi mã hoá liên tục thì chương chuyển
/// error (như check hết vòng) — phiên đi tiếp chương sau.
pub fn translate_chapter(
    config: &ApiSessionConfig,
    id: &str,
    model: &dyn TextModel,
    cancel: &AtomicBool,
    log: Log,
) -> std::result::Result<ChapterOutcome, ChapterFailure> {
    match run_chapter(config, id, model, cancel, log) {
        Err(ChapterFailure::Api(error @ ApiError::MojibakeExhausted)) => {
            let reason = error.to_string();
            mark_error(&config.root, id, &reason)?;
            Ok(ChapterOutcome::Errored { reason })
        }
        other => other,
    }
}

fn mark_error(root: &Path, id: &str, reason: &str) -> Result<()> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    if let Some(chapter) = state.chapters.get_mut(id) {
        chapter.status = ChapterStatus::Error;
        chapter.reason = Some(reason.to_string());
        chapter.updated_at = now_ms();
    }
    save_state(&paths, &state)
}

fn run_chapter(
    config: &ApiSessionConfig,
    id: &str,
    model: &dyn TextModel,
    cancel: &AtomicBool,
    log: Log,
) -> std::result::Result<ChapterOutcome, ChapterFailure> {
    let root = config.root.as_path();
    let paths = story_paths(&resolve_root(root));
    let raw = read_raw_chapter(&paths, id)?;
    let paragraphs = paragraphs_of(&raw);
    let story = load_story_config(&paths)?;
    let base_glossary = crate::base::BaseStore::from_env().glossary(story.genre.setting);
    let cast = crate::cast::load_cast(&paths);
    let system = build_chapter_prompt(&base_glossary, &story, &cast, id, &raw);
    let glossary = merge_story_glossary(&base_glossary, Some(&story));
    let unpaired =
        crate::prompt::unpaired_line(&crate::prompt::chapter_unpaired(&base_glossary, &story, &cast, id, &raw));
    let min_ratio = load_state(&paths)?.settings.min_length_ratio;
    let chapter = Chapter {
        model,
        id,
        system: &system,
        paragraphs: &paragraphs,
        story: &story,
        glossary: &glossary,
        cast: &cast,
        unpaired: &unpaired,
        cancel,
        log,
        usage: Mutex::new(Usage::default()),
        retry_delay: config.retry_delay,
        retry_max: config.retry_max,
    };

    let (mut draft, inline_glossary) = translate_full(&chapter)?;
    write_draft(&paths, id, &draft)?;
    harvest_glossary(&chapter, &paths, &raw, &draft, inline_glossary)?;

    let mut rounds = 0;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(ApiError::Cancelled.into());
        }
        let check = run_check(root, id)?;
        if check.pass {
            let accepted = run_accept(root, id, false)?;
            log_cast_notes(&chapter, &accepted);
            return Ok(ChapterOutcome::Accepted {
                review_rounds: rounds,
                warnings: accepted.warnings.len(),
                added_glossary: accepted.added_glossary,
                usage: chapter.usage(),
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
            let (again, _) = translate_full(&chapter)?;
            if chars_no_ws(&again) > chars_no_ws(&draft) {
                draft = again;
            }
        } else if !check.garbled.is_empty() {
            // Chữ lỗi mã hoá (agent/hub ghi sai bảng mã): model soát không đọc nổi chữ hỏng — dịch lại từ raw đúng đoạn đó.
            let indices: Vec<usize> = check.garbled.iter().map(|label| label - 1).collect();
            log(format!("{id}: chữ lỗi mã hoá ở {} đoạn — dịch lại", indices.len()));
            retranslate(&chapter, &mut draft, &indices, "dịch lại", "giữ bản cũ")?;
        } else if !check.untranslated.is_empty() {
            // Còn chữ Hán: dịch lại từ raw đúng đoạn đó. Không đưa vào soát — model soát hay giữ nguyên
            // ("đúng ngữ cảnh") và chương bị chốt kèm cảnh báo với Hán tự lọt ra out.
            let indices: Vec<usize> = check.untranslated.iter().map(|label| label - 1).collect();
            repair_untranslated(&chapter, &mut draft, &indices)?;
        } else if review(&chapter, rounds, &mut draft, &check.issues)? == ReviewOutcome::Kept {
            // Đủ đoạn, đủ dài, chỉ còn vi phạm rule mà model đã xét và giữ → chốt kèm cảnh báo ngay,
            // không đốt thêm vòng soát y hệt.
            let accepted = run_accept(root, id, true)?;
            log_cast_notes(&chapter, &accepted);
            return Ok(ChapterOutcome::Accepted {
                review_rounds: rounds,
                warnings: accepted.warnings.len(),
                added_glossary: accepted.added_glossary,
                usage: chapter.usage(),
            });
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

/// Lỗi cấu hình (400 model không có, 401/403 key sai, 404 URL sai…): áp dụng cho mọi chương nên thử lại
/// hay skip đều vô nghĩa — dừng ngay. Lỗi mạng/429/5xx/stream đứt đã được gọi lại tại lượt đó (`with_retry`) tới
/// khi được; tới đây chỉ còn model trả hỏng (rỗng, không nhãn) — dịch lại chương một lần rồi skip.
fn is_config_error(error: &ApiError) -> bool {
    matches!(error, ApiError::Http { .. }) && !error.is_transient()
}

/// Phiên dừng giữa chừng thì chương đang dịch trả về hàng đợi, không để kẹt "translating" khi không
/// có phiên nào chạy (huỷ tay thì giữ translating để phiên sau làm tiếp — xem `run_next`).
fn requeue(root: &Path, id: &str, log: &dyn Fn(String)) {
    match run_retry(root, id) {
        Ok(()) => log(format!("{id}: trả về hàng đợi")),
        Err(error) => log(format!("{id}: không trả về hàng đợi được — {error}")),
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
            match translate_chapter(config, &id, model, cancel, &log) {
                Ok(outcome) => break Ok(outcome),
                Err(ChapterFailure::Api(ApiError::Cancelled)) => return StopReason::UserCancelled,
                Err(ChapterFailure::Api(ApiError::Blocked(reason))) => break Err(format!("model từ chối: {reason}")),
                Err(ChapterFailure::Api(error)) if is_config_error(&error) => {
                    log(format!("{id}: {error} — lỗi cấu hình API, dừng phiên"));
                    requeue(root, &id, &log);
                    return StopReason::ApiFailed { message: error.to_string() };
                }
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
                        log(format!("{id}: {error} — hai chương lỗi liên tiếp, dừng phiên"));
                        requeue(root, &id, &log);
                        return StopReason::ApiFailed { message: error.to_string() };
                    }
                    break Err(format!("lỗi API: {error}"));
                }
                Err(ChapterFailure::Core(error)) => return StopReason::Internal { message: error.to_string() },
            }
        };
        match outcome {
            Ok(ChapterOutcome::Accepted { review_rounds, warnings, added_glossary, usage }) => {
                failed_chapters = 0;
                let tokens = if usage.is_empty() { String::new() } else { format!(" · {} token", format_tokens(usage.total())) };
                log(format!("{id}: chốt (soát {review_rounds} lần, {warnings} cảnh báo, +{added_glossary} glossary){tokens}"));
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
