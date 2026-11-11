//! Lời đáp của các lệnh vòng dịch (next/check/accept/skip/retry/status) — chữ và mã thoát dùng chung cho CLI
//! `qt-ai` (mã 0 in stdout, khác 0 in stderr) và tool MCP (`crate::mcp`: mã khác 0 thành `isError`).

use crate::commands::accept::run_accept;
use crate::commands::check::run_check;
use crate::commands::next::{run_next_via, Via};
use crate::commands::retry::run_retry;
use crate::commands::skip::run_skip;
use crate::commands::status::run_status;
use crate::error::Result;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// Mã thoát của CLI: 0 ổn, 1 check FAIL còn lượt sửa, 2 check quá số vòng review.
    pub code: i32,
    pub text: String,
}

impl Reply {
    fn ok(text: impl Into<String>) -> Reply {
        Reply { code: 0, text: text.into() }
    }
}

pub fn next(root: &Path, via: Via) -> Result<Reply> {
    let result = run_next_via(root, via)?;
    let mut text = format!("Chương {} → đọc prompt tại: {}", result.chapter_id, result.prompt_path.display());
    if via == Via::Mcp {
        // Id là tên file raw, có khi dài ("Chương 0039 - 第39章 …"): đưa sẵn tham số để model chép nguyên văn.
        let args = serde_json::json!({"id": result.chapter_id});
        text.push_str(&format!("\nTham số cho qt_check / qt_accept: {args}"));
    }
    Ok(Reply::ok(text))
}

pub fn check(root: &Path, id: &str) -> Result<Reply> {
    let result = run_check(root, id)?;
    if result.pass {
        return Ok(Reply::ok(if result.accepted_with_warnings {
            format!("Chương {id} hết vòng review, còn {} vi phạm rule → chốt kèm cảnh báo. Chạy accept.", result.violations.len())
        } else {
            format!("Chương {id} PASS (ratio {:.2}) — chạy accept.", result.ratio)
        }));
    }
    if result.escalated_to_error {
        return Ok(Reply { code: 2, text: format!("Chương {id} quá số vòng review → error. Xem qt-ai status.") });
    }
    Ok(Reply {
        code: 1,
        text: format!(
            "Chương {id} FAIL: thiếu {} đoạn, {} vi phạm, ratio {:.2}.\nSửa theo: {}",
            result.missing.len(),
            result.violations.len(),
            result.ratio,
            result.review_path.map(|p| p.display().to_string()).unwrap_or_default()
        ),
    })
}

pub fn accept(root: &Path, id: &str, force: bool) -> Result<Reply> {
    let result = run_accept(root, id, force)?;
    let mut lines = vec![format!("Đã chốt {} (+{} glossary mới).", result.out_path.display(), result.added_glossary)];
    lines.extend(result.cast_notes.iter().map(|note| format!("Bảng nhân vật: {note}")));
    if !result.glossary_fixes.is_empty() {
        lines.push(format!("Tự sửa theo glossary: {}", result.glossary_fixes.join(", ")));
    }
    if result.self_ref_fixes > 0 {
        lines.push(format!("Tự xưng bản → bổn: {} chỗ", result.self_ref_fixes));
    }
    // Dòng kiểm tra VN không lưu vào state nên đáp thẳng; cảnh báo còn lại xem ở status.
    let (vn, stored): (Vec<&String>, Vec<&String>) =
        result.warnings.iter().partition(|w| w.starts_with(crate::vn_check::VN_WARNING_PREFIX));
    lines.extend(vn.into_iter().cloned());
    if !stored.is_empty() {
        lines.push(format!("Kèm {} cảnh báo — xem qt-ai status.", stored.len()));
    }
    Ok(Reply::ok(lines.join("\n")))
}

pub fn skip(root: &Path, id: &str, reason: &str) -> Result<Reply> {
    run_skip(root, id, reason)?;
    Ok(Reply::ok(format!("Đã skip chương {id}.")))
}

pub fn retry(root: &Path, id: &str) -> Result<Reply> {
    run_retry(root, id)?;
    Ok(Reply::ok(format!("Đã đưa chương {id} về hàng đợi — next sẽ phát lại nó.")))
}

pub fn status(root: &Path) -> Result<Reply> {
    Ok(Reply::ok(run_status(root)?))
}
