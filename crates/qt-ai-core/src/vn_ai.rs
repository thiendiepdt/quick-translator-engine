//! AI sơ loại các chỗ khớp nhạy cảm về Việt Nam theo lô — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
//! Đường API dùng `TextModel::complete_json`; đường agy (GUI) ghi `payload` ra file, agent ghi kết quả ra file,
//! cả hai đọc bằng `parse_verdicts`.

use crate::api::{ApiStep, TextModel};
use crate::error::Result;
use crate::story_fs::{now_ms, StoryPaths};
use crate::vn_check::{report, set_ai, AiVerdict, HitView};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub const BATCH: usize = 20;
pub const LABELS: [&str; 4] = ["xucpham", "nhaycam", "trungtinh", "khonglienquan"];

pub const SYSTEM_PROMPT: &str = "Bạn kiểm duyệt truyện dịch Trung → Việt trước khi đăng ở Việt Nam. Mỗi mục trong `items` là một chỗ có từ liên quan Việt Nam (`matched`), kèm đoạn raw tiếng Trung (`raw`) và bản dịch (`out`): đoạn trước, đoạn chứa, đoạn sau (bản dịch có thể rỗng nếu chương chưa dịch). Xếp mỗi mục một nhãn:
- xucpham: miệt thị, chửi bới người Việt / nước Việt, hoặc tô vẽ việc xâm lược, đô hộ Việt Nam.
- nhaycam: chủ quyền biển đảo (Hoàng Sa, Trường Sa, Biển Đông coi là lãnh thổ Trung Quốc, đường chín đoạn), chiến tranh Trung - Việt, nhân vật lịch sử / chính trị Việt Nam, kể theo hướng có thể gây tranh cãi.
- trungtinh: nhắc tới Việt Nam / người Việt bình thường, không miệt thị.
- khonglienquan: chữ trùng nhưng không nói về Việt Nam (nước Việt thời Xuân Thu, Nam Hải trong truyện kỳ ảo, cát vàng...).
Kèm `reason` một câu tiếng Việt. Chỉ trả JSON: {\"verdicts\": [{\"id\": \"...\", \"label\": \"...\", \"reason\": \"...\"}]}.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyItem {
    pub id: String,
    pub matched: String,
    pub raw: Vec<String>,
    pub out: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Verdict {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOutcome {
    pub verified: usize,
    /// Chỗ không nhận được nhãn hợp lệ (lỗi gọi, JSON hỏng, thiếu id).
    pub failed: usize,
}

pub fn items(views: &[HitView], ids: &[String]) -> Vec<VerifyItem> {
    views
        .iter()
        .filter(|view| ids.contains(&view.hit.id))
        .map(|view| VerifyItem { id: view.hit.id.clone(), matched: view.hit.matched.clone(), raw: view.raw.clone(), out: view.out.clone() })
        .collect()
}

pub fn payload(items: &[VerifyItem]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({ "items": items })).expect("VerifyItem luôn serialize được")
}

/// Lấy object JSON ngoài cùng (chịu rào ```json); bỏ mục có nhãn lạ.
pub fn parse_verdicts(text: &str) -> Vec<Verdict> {
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default)]
        verdicts: Vec<Verdict>,
    }
    let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) else { return vec![] };
    if end < start {
        return vec![];
    }
    serde_json::from_str::<Envelope>(&text[start..=end])
        .map(|envelope| envelope.verdicts)
        .unwrap_or_default()
        .into_iter()
        .filter(|verdict| LABELS.contains(&verdict.label.as_str()))
        .collect()
}

pub fn save_verdicts(paths: &StoryPaths, verdicts: &[Verdict], model: &str) -> Result<usize> {
    let at = now_ms();
    let list: Vec<(String, AiVerdict)> = verdicts
        .iter()
        .map(|v| (v.id.clone(), AiVerdict { label: v.label.clone(), reason: v.reason.clone(), model: model.to_string(), at }))
        .collect();
    set_ai(paths, &list)
}

pub fn verify_with_model(paths: &StoryPaths, model: &dyn TextModel, ids: &[String], cancel: &AtomicBool) -> Result<VerifyOutcome> {
    let views = report(paths);
    let all = items(&views, ids);
    let mut outcome = VerifyOutcome::default();
    for batch in all.chunks(BATCH) {
        let verdicts = match model.complete_json(ApiStep::Glossary, SYSTEM_PROMPT, &payload(batch), cancel) {
            Ok(generated) => parse_verdicts(&generated.text),
            Err(_) => vec![],
        };
        let known: Vec<Verdict> = verdicts.into_iter().filter(|v| batch.iter().any(|item| item.id == v.id)).collect();
        outcome.failed += batch.len() - known.len().min(batch.len());
        outcome.verified += save_verdicts(paths, &known, &model.label())?;
    }
    Ok(outcome)
}

/// Lệnh cho agent agy: đọc file đầu vào, ghi kết quả JSON ra file đầu ra.
pub fn agy_prompt(input: &Path, output: &Path) -> String {
    format!(
        "{SYSTEM_PROMPT}\n\nĐọc file {} (JSON, khoá `items`). Ghi kết quả đúng dạng JSON trên vào file {} rồi dừng. Không sửa file nào khác, không hỏi lại.",
        input.display(),
        output.display()
    )
}
