//! Đọc server-sent events từng dòng: mỗi `data: {json}` là một payload; `[DONE]` bỏ qua.

use crate::api::ApiError;
use serde_json::Value;
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn read_sse<R: BufRead>(
    mut reader: R,
    cancel: &AtomicBool,
    mut on_data: impl FnMut(Value) -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    let mut line = String::new();
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(ApiError::Cancelled);
        }
        line.clear();
        let read = reader.read_line(&mut line).map_err(|error| ApiError::Stream(error.to_string()))?;
        if read == 0 {
            return Ok(());
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let Some(data) = trimmed.strip_prefix("data:") else { continue };
        let data = data.trim_start();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let value: Value = serde_json::from_str(data).map_err(|error| ApiError::Stream(error.to_string()))?;
        on_data(value)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn doc_data_bo_qua_comment_va_done_dung_khi_cancel() {
        let body = ": ping\r\nevent: x\ndata: {\"a\":1}\n\ndata:{\"a\":2}\n\ndata: [DONE]\n\n";
        let mut seen = Vec::new();
        read_sse(Cursor::new(body), &AtomicBool::new(false), |value| {
            seen.push(value["a"].as_i64().unwrap());
            Ok(())
        })
        .unwrap();
        assert_eq!(seen, vec![1, 2]);

        let cancel = AtomicBool::new(true);
        let result = read_sse(Cursor::new(body), &cancel, |_| Ok(()));
        assert_eq!(result, Err(ApiError::Cancelled));

        let result = read_sse(Cursor::new("data: {hỏng\n"), &AtomicBool::new(false), |_| Ok(()));
        assert!(matches!(result, Err(ApiError::Stream(_))));
    }
}
