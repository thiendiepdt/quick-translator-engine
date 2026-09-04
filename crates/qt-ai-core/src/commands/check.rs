use crate::check::{check_violations, Violation};
use crate::error::{CoreError, Result};
use crate::paragraphs::{labeled_repair_payload, paragraphs_of, parse_labeled_translation};
use crate::story_fs::{
    load_state, load_story_config, now_ms, read_raw_chapter, read_text, resolve_root, save_state, story_paths,
    work_file, write_text, ChapterState, ChapterStatus, WorkKind,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CheckResult {
    pub pass: bool,
    /// Nhãn 1-based của đoạn thiếu.
    pub missing: Vec<usize>,
    pub violations: Vec<Violation>,
    pub ratio: f64,
    /// Hết vòng review mà chỉ còn vi phạm rule (đủ đoạn, đủ dài): giống web, chương pass kèm cảnh báo.
    pub accepted_with_warnings: bool,
    /// Mọi vấn đề còn lại, quy về nhãn [[n]] — accept ghi vào state.warnings.
    pub issues: Vec<String>,
    pub escalated_to_error: bool,
    pub review_path: Option<PathBuf>,
}

pub struct Draft {
    pub paragraphs: Vec<String>,
    pub parsed: Vec<Option<String>>,
    pub final_text: String,
}

/// `violations[i].line` tính trên final_text (các đoạn CÒN LẠI, mỗi đoạn 1 dòng cách 1 dòng trống:
/// đoạn thứ k nằm ở dòng 2k−1). Quy về nhãn [[n]] gốc qua danh sách chỉ số đoạn không thiếu.
fn violation_label(parsed: &[Option<String>], violation: &Violation) -> usize {
    let defined: Vec<usize> = parsed.iter().enumerate().filter(|(_, p)| p.is_some()).map(|(i, _)| i).collect();
    let position = (violation.line + 1) / 2; // Math.ceil(line / 2)
    defined.get(position.saturating_sub(1)).map(|index| index + 1).unwrap_or(position)
}

fn build_violations_section(id: &str, parsed: &[Option<String>], violations: &[Violation]) -> String {
    let list = violations
        .iter()
        .map(|item| {
            format!(
                "- [[{}]] (dòng {} trong bản lắp): {} — \"{}\"",
                violation_label(parsed, item),
                item.line,
                item.message,
                item.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    [
        format!("# Vi phạm rule — sửa tối thiểu ngay trong work/{id}.draft.md"),
        String::new(),
        format!("Sửa ĐÚNG TẠI CHỖ trong work/{id}.draft.md: với mỗi vi phạm dưới đây, tìm đoạn có nhãn [[n]] tương ứng, chỉ đổi đúng từ/cụm gây lỗi, GIỮ NGUYÊN toàn bộ nhãn [[n]] hiện có và không viết lại hay chau chuốt các đoạn khác."),
        String::new(),
        list,
    ]
    .join("\n")
}

pub fn assemble_draft(root: &Path, id: &str) -> Result<Draft> {
    let paths = story_paths(&resolve_root(root));
    let draft_path = work_file(&paths, id, WorkKind::Draft);
    if !draft_path.exists() {
        return Err(CoreError::InvalidState(format!(
            "Chưa có bản dịch {} — agent phải ghi draft trước khi check.",
            draft_path.display()
        )));
    }
    let paragraphs = paragraphs_of(&read_raw_chapter(&paths, id)?);
    let parsed = parse_labeled_translation(&read_text(&draft_path)?, paragraphs.len())
        .unwrap_or_else(|| vec![None; paragraphs.len()]);
    let final_text = parsed.iter().flatten().cloned().collect::<Vec<_>>().join("\n\n");
    Ok(Draft { paragraphs, parsed, final_text })
}

fn char_count_no_ws(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

pub fn run_check(root: &Path, id: &str) -> Result<CheckResult> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .cloned()
        .ok_or_else(|| CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")))?;
    let story = load_story_config(&paths)?;
    let Draft { paragraphs, parsed, final_text } = assemble_draft(root, id)?;

    let missing: Vec<usize> =
        parsed.iter().enumerate().filter(|(_, p)| p.is_none()).map(|(i, _)| i + 1).collect();
    let violations = check_violations(&final_text, &story.check_rules, story.genre.setting);
    let raw_length = char_count_no_ws(&paragraphs.concat());
    let translated_length = char_count_no_ws(&final_text);
    let ratio = if raw_length > 0 { translated_length as f64 / raw_length as f64 } else { 1.0 };
    let too_short = ratio < state.settings.min_length_ratio;
    let clean = missing.is_empty() && violations.is_empty() && !too_short;

    let mut issues: Vec<String> = missing.iter().map(|label| format!("[[{label}]] thiếu đoạn")).collect();
    issues.extend(
        violations
            .iter()
            .map(|v| format!("[[{}]] {} — \"{}\"", violation_label(&parsed, v), v.message, v.text)),
    );
    if too_short {
        issues.push(format!(
            "Quá ngắn: tỉ lệ ký tự dịch/raw {ratio:.2} < {}",
            state.settings.min_length_ratio
        ));
    }

    let mut pass = clean;
    let mut accepted_with_warnings = false;
    let mut escalated_to_error = false;
    let mut review_path = None;

    if !clean {
        if chapter.review_round >= state.settings.max_review_rounds {
            if missing.is_empty() && !too_short {
                // Giống web: hết vòng soát mà chỉ còn vi phạm rule thì vẫn chốt, kèm cảnh báo.
                pass = true;
                accepted_with_warnings = true;
            } else {
                escalated_to_error = true;
                state.chapters.insert(
                    id.to_string(),
                    ChapterState {
                        status: ChapterStatus::Error,
                        reason: Some(format!(
                            "Quá {} vòng review vẫn chưa đạt (thiếu {} đoạn, {} vi phạm, ratio {ratio:.2}).",
                            state.settings.max_review_rounds,
                            missing.len(),
                            violations.len()
                        )),
                        updated_at: now_ms(),
                        ..chapter.clone()
                    },
                );
                save_state(&paths, &state)?;
            }
        } else {
            let mut sections: Vec<String> = Vec::new();
            if !missing.is_empty() {
                let zero_based: Vec<usize> = missing.iter().map(|label| label - 1).collect();
                sections.push(format!(
                    "# Đoạn còn thiếu — dịch bổ sung các đoạn dưới đây rồi CHÈN vào work/{id}.draft.md, giữ đúng nhãn [[n]] cho từng đoạn, không sửa các đoạn đã có\n\n{}",
                    labeled_repair_payload(&paragraphs, &zero_based)
                ));
            }
            if !violations.is_empty() {
                sections.push(build_violations_section(id, &parsed, &violations));
            }
            if too_short {
                sections.push(format!(
                    "# Bản dịch quá ngắn\n\nTỉ lệ ký tự dịch/raw = {ratio:.2} < {}. Rà từng đoạn xem có bị tóm tắt/lược ý; dịch đủ 100% nội dung.",
                    state.settings.min_length_ratio
                ));
            }
            let path = work_file(&paths, id, WorkKind::Review);
            write_text(&path, &format!("{}\n", sections.join("\n\n---\n\n")))?;
            review_path = Some(path);
            state.chapters.insert(
                id.to_string(),
                ChapterState { review_round: chapter.review_round + 1, updated_at: now_ms(), ..chapter.clone() },
            );
            save_state(&paths, &state)?;
        }
    }

    let review_round = state.chapters.get(id).map(|c| c.review_round).unwrap_or(chapter.review_round);
    let report = serde_json::json!({
        "pass": pass, "acceptedWithWarnings": accepted_with_warnings, "missing": missing,
        "violationCount": violations.len(), "ratio": ratio, "reviewRound": review_round, "checkedAt": now_ms(),
    });
    write_text(
        &work_file(&paths, id, WorkKind::Check),
        &format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )?;

    Ok(CheckResult { pass, missing, violations, ratio, accepted_with_warnings, issues, escalated_to_error, review_path })
}
