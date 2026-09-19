//! OpenAI-compatible chat/completions: OpenAI chính chủ hoặc hub bất kỳ qua base URL. Port từ qt-web
//! (provider "openai"): max_completion_tokens, reasoning_effort, đọc reasoning_content/reasoning.

use crate::api::sse::read_sse;
use crate::api::{ApiConfig, ApiError, ApiStep, Generated, Usage, MAX_OUTPUT_TOKENS};
use serde_json::{json, Value};
use std::io::BufRead;
use std::sync::atomic::AtomicBool;

pub fn url(config: &ApiConfig) -> String {
    format!("{}/chat/completions", config.base_url)
}

pub fn headers(config: &ApiConfig, stream: bool) -> Vec<(&'static str, String)> {
    vec![
        ("authorization", format!("Bearer {}", config.api_key)),
        ("accept", if stream { "text/event-stream" } else { "application/json" }.to_string()),
    ]
}

fn messages(system: &str, user: &str) -> Value {
    json!([{ "role": "system", "content": system }, { "role": "user", "content": user }])
}

pub fn stream_body(config: &ApiConfig, step: ApiStep, system: &str, user: &str) -> Value {
    let mut body = json!({
        "model": config.model,
        "messages": messages(system, user),
        "max_completion_tokens": MAX_OUTPUT_TOKENS,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    let effort = config.effort(step);
    if !effort.is_empty() {
        body["reasoning_effort"] = json!(effort);
    }
    body
}

/// GPT-5/o-series chỉ nhận temperature mặc định — không gửi. `reasoning_effort` chỉ khi bước có mức nghĩ.
pub fn json_body(config: &ApiConfig, step: ApiStep, system: &str, user: &str) -> Value {
    let mut body = json!({
        "model": config.model,
        "response_format": { "type": "json_object" },
        "messages": messages(system, user),
    });
    let effort = config.effort(step);
    if !effort.is_empty() {
        body["reasoning_effort"] = json!(effort);
    }
    body
}

/// `usage` của response hoặc chunk cuối (khi bật stream_options.include_usage). `completion_tokens` của
/// OpenAI đã gồm `reasoning_tokens` → tách ra để `output` chỉ là chữ trả về, không đếm đôi khi cộng tổng.
pub fn usage_of(payload: &Value) -> Option<Usage> {
    let usage = payload.get("usage").filter(|value| value.is_object())?;
    let count = |pointer: &str| usage.pointer(pointer).and_then(Value::as_u64).unwrap_or(0);
    let thoughts = count("/completion_tokens_details/reasoning_tokens");
    Some(Usage {
        input: count("/prompt_tokens"),
        cached: count("/prompt_tokens_details/cached_tokens"),
        output: count("/completion_tokens").saturating_sub(thoughts),
        thoughts,
    })
}

pub fn parse_stream<R: BufRead>(
    reader: R,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(usize),
) -> Result<Generated, ApiError> {
    let mut output = String::new();
    let mut refusal = String::new();
    let mut filtered = false;
    let mut usage: Option<Usage> = None;
    read_sse(reader, cancel, |payload| {
        if let Some(message) = payload.pointer("/error/message").and_then(Value::as_str) {
            return Err(ApiError::Stream(message.to_string()));
        }
        if let Some(found) = usage_of(&payload) {
            usage = Some(found);
        }
        // Bộ lọc nội dung phía hub/OpenAI: `finish_reason: content_filter` (Azure/OpenAI) hoặc
        // `delta.refusal` (OpenAI) — là model từ chối, không phải sai định dạng, không thử lại.
        if payload.pointer("/choices/0/finish_reason").and_then(Value::as_str) == Some("content_filter") {
            filtered = true;
        }
        let Some(delta) = payload.pointer("/choices/0/delta") else { return Ok(()) };
        if let Some(text) = delta.get("refusal").and_then(Value::as_str) {
            refusal.push_str(text);
        }
        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            if !text.is_empty() {
                output.push_str(text);
                on_progress(output.chars().count());
            }
        }
        Ok(())
    })?;
    if filtered {
        return Err(ApiError::Blocked("content_filter".to_string()));
    }
    if !refusal.is_empty() && output.is_empty() {
        return Err(ApiError::Blocked(refusal));
    }
    if output.is_empty() {
        return Err(ApiError::Empty("OpenAI"));
    }
    Ok(Generated { text: output, usage })
}

/// `message.content` là chuỗi; vài hub trả mảng part `[{type:"text", text}]` — nối text lại.
pub fn parse_json_response(payload: &Value) -> Option<String> {
    match payload.pointer("/choices/0/message/content")? {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => {
            let text: String = parts.iter().filter_map(|part| part.get("text").and_then(Value::as_str)).collect();
            (!text.is_empty()).then_some(text)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiProvider;
    use std::io::Cursor;

    #[test]
    fn body_stream_co_reasoning_effort_max_completion_tokens_khong_temperature() {
        let config = ApiConfig::resolve(ApiProvider::OpenAi, "sk-hub", "gemini-3.7-flash", "http://192.0.2.10/v1", true, "high");
        assert_eq!(url(&config), "http://192.0.2.10/v1/chat/completions");
        assert_eq!(headers(&config, true)[0], ("authorization", "Bearer sk-hub".to_string()));
        let body = stream_body(&config, ApiStep::Translate, "S", "U");
        assert_eq!(body["model"], "gemini-3.7-flash");
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["max_completion_tokens"], 65536);
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        assert!(body.get("max_tokens").is_none() && body.get("temperature").is_none());
        assert_eq!(body["messages"][0], json!({ "role": "system", "content": "S" }));

        let no_effort = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "", "", true, "");
        assert!(stream_body(&no_effort, ApiStep::Translate, "S", "U").get("reasoning_effort").is_none());
        let json = json_body(&no_effort, ApiStep::Glossary, "S", "U");
        assert_eq!(json["response_format"]["type"], "json_object");
        assert!(json.get("temperature").is_none());
    }

    #[test]
    fn usage_of_tach_reasoning_khoi_completion_va_doc_cache() {
        // completion_tokens = 40 gồm 25 reasoning → ra 15, nghĩ 25; tổng 100 + 15 + 25 = 140, không đếm đôi.
        let payload = json!({ "usage": { "prompt_tokens": 100, "completion_tokens": 40, "prompt_tokens_details": { "cached_tokens": 60 }, "completion_tokens_details": { "reasoning_tokens": 25 } } });
        let usage = usage_of(&payload).unwrap();
        assert_eq!(usage, Usage { input: 100, cached: 60, output: 15, thoughts: 25 });
        assert_eq!(usage.total(), 140);
        assert_eq!(usage_of(&json!({ "usage": null })), None);
        // Hub báo reasoning lớn hơn completion (lệch chuẩn) → không tràn số.
        let odd = json!({ "usage": { "completion_tokens": 5, "completion_tokens_details": { "reasoning_tokens": 9 } } });
        assert_eq!(usage_of(&odd).unwrap().output, 0);
    }

    #[test]
    fn parse_stream_gom_content_bo_reasoning() {
        let sse = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({ "choices": [{ "delta": { "reasoning": "nghĩ" } }] }),
            json!({ "choices": [{ "delta": { "content": "Bản dịch" } }] }),
        );
        let out = parse_stream(Cursor::new(sse), &AtomicBool::new(false), &mut |_| {}).unwrap();
        assert_eq!(out.text, "Bản dịch");
        assert_eq!(out.usage, None);
        let with_usage = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({ "choices": [{ "delta": { "content": "x" } }] }),
            json!({ "choices": [], "usage": { "prompt_tokens": 7, "completion_tokens": 1 } }),
        );
        let out = parse_stream(Cursor::new(with_usage), &AtomicBool::new(false), &mut |_| {}).unwrap();
        assert_eq!(out.usage, Some(Usage { input: 7, cached: 0, output: 1, thoughts: 0 }));
        let err = format!("data: {}\n\n", json!({ "error": { "message": "quota" } }));
        assert_eq!(
            parse_stream(Cursor::new(err), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Stream("quota".into()))
        );
        let payload = json!({ "choices": [{ "message": { "content": "{}" } }] });
        assert_eq!(parse_json_response(&payload).as_deref(), Some("{}"));
    }

    #[test]
    fn parse_stream_content_filter_hoac_refusal_la_blocked() {
        // Azure/OpenAI cắt giữa chừng: đã có content nhưng finish_reason content_filter → vẫn Blocked.
        let filtered = format!(
            "data: {}

data: {}

data: [DONE]

",
            json!({ "choices": [{ "delta": { "content": "[[1]] Nửa" } }] }),
            json!({ "choices": [{ "delta": {}, "finish_reason": "content_filter" }] }),
        );
        assert_eq!(
            parse_stream(Cursor::new(filtered), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Blocked("content_filter".into()))
        );
        let refusal = format!(
            "data: {}

data: {}

data: [DONE]

",
            json!({ "choices": [{ "delta": { "refusal": "I can't " } }] }),
            json!({ "choices": [{ "delta": { "refusal": "help with that." }, "finish_reason": "stop" }] }),
        );
        assert_eq!(
            parse_stream(Cursor::new(refusal), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Blocked("I can't help with that.".into()))
        );
        // finish_reason stop bình thường thì không đổi gì.
        let ok = format!(
            "data: {}

data: [DONE]

",
            json!({ "choices": [{ "delta": { "content": "x" }, "finish_reason": "stop" }] }),
        );
        assert_eq!(parse_stream(Cursor::new(ok), &AtomicBool::new(false), &mut |_| {}).unwrap().text, "x");
    }
}
