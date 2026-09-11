//! OpenAI-compatible chat/completions: OpenAI chính chủ hoặc hub bất kỳ qua base URL. Port từ qt-web
//! (provider "openai"): max_completion_tokens, reasoning_effort, đọc reasoning_content/reasoning.

use crate::api::sse::read_sse;
use crate::api::{ApiConfig, ApiError, Effort, Generated, Usage, MAX_OUTPUT_TOKENS};
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

/// `reasoning_effort` gửi đi: rỗng = người dùng không muốn gửi (hub không nhận) → không gửi ở mọi lượt;
/// "none" = tắt hẳn → giữ; còn lại: dịch dùng mức đã chọn, soát/glossary ép "low" (mức thấp mọi hub đều
/// nhận; "minimal" chỉ GPT-5 hiểu).
fn reasoning_effort(config: &ApiConfig, effort: Effort) -> Option<&str> {
    let chosen = config.reasoning_effort.as_str();
    if chosen.is_empty() {
        return None;
    }
    Some(match effort {
        Effort::Full => chosen,
        Effort::Low | Effort::Minimal if chosen == "none" => "none",
        Effort::Low | Effort::Minimal => "low",
    })
}

pub fn stream_body(config: &ApiConfig, system: &str, user: &str, effort: Effort) -> Value {
    let mut body = json!({
        "model": config.model,
        "messages": messages(system, user),
        "max_completion_tokens": MAX_OUTPUT_TOKENS,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(level) = reasoning_effort(config, effort) {
        body["reasoning_effort"] = json!(level);
    }
    body
}

/// GPT-5/o-series chỉ nhận temperature mặc định — không gửi.
pub fn json_body(config: &ApiConfig, system: &str, user: &str, effort: Effort) -> Value {
    let mut body = json!({
        "model": config.model,
        "response_format": { "type": "json_object" },
        "messages": messages(system, user),
    });
    if let Some(level) = reasoning_effort(config, effort) {
        body["reasoning_effort"] = json!(level);
    }
    body
}

/// `usage` của response hoặc chunk cuối (khi bật stream_options.include_usage).
pub fn usage_of(payload: &Value) -> Option<Usage> {
    let usage = payload.get("usage").filter(|value| value.is_object())?;
    let count = |pointer: &str| usage.pointer(pointer).and_then(Value::as_u64).unwrap_or(0);
    Some(Usage {
        input: count("/prompt_tokens"),
        cached: count("/prompt_tokens_details/cached_tokens"),
        output: count("/completion_tokens"),
        thoughts: count("/completion_tokens_details/reasoning_tokens"),
    })
}

pub fn parse_stream<R: BufRead>(
    reader: R,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(usize),
) -> Result<Generated, ApiError> {
    let mut output = String::new();
    let mut usage: Option<Usage> = None;
    read_sse(reader, cancel, |payload| {
        if let Some(message) = payload.pointer("/error/message").and_then(Value::as_str) {
            return Err(ApiError::Stream(message.to_string()));
        }
        if let Some(found) = usage_of(&payload) {
            usage = Some(found);
        }
        let Some(delta) = payload.pointer("/choices/0/delta") else { return Ok(()) };
        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            if !text.is_empty() {
                output.push_str(text);
                on_progress(output.chars().count());
            }
        }
        Ok(())
    })?;
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
        let body = stream_body(&config, "S", "U", Effort::Full);
        assert_eq!(body["model"], "gemini-3.7-flash");
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["max_completion_tokens"], 65536);
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        assert!(body.get("max_tokens").is_none() && body.get("temperature").is_none());
        assert_eq!(body["messages"][0], json!({ "role": "system", "content": "S" }));

        let no_effort = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "", "", true, "");
        assert!(stream_body(&no_effort, "S", "U", Effort::Full).get("reasoning_effort").is_none());
        let json = json_body(&no_effort, "S", "U", Effort::Low);
        assert_eq!(json["response_format"]["type"], "json_object");
        assert!(json.get("temperature").is_none() && json.get("reasoning_effort").is_none());
    }

    #[test]
    fn effort_thap_ep_low_giu_none_va_khong_gui_khi_rong() {
        let high = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "m", "", true, "high");
        assert_eq!(stream_body(&high, "S", "U", Effort::Minimal)["reasoning_effort"], "low");
        assert_eq!(json_body(&high, "S", "U", Effort::Low)["reasoning_effort"], "low");
        let none = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "m", "", true, "none");
        assert_eq!(stream_body(&none, "S", "U", Effort::Low)["reasoning_effort"], "none");
        let empty = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "m", "", true, "");
        assert!(json_body(&empty, "S", "U", Effort::Minimal).get("reasoning_effort").is_none());
    }

    #[test]
    fn usage_of_doc_usage_ke_ca_reasoning_va_cache() {
        let payload = json!({ "usage": { "prompt_tokens": 100, "completion_tokens": 40, "prompt_tokens_details": { "cached_tokens": 60 }, "completion_tokens_details": { "reasoning_tokens": 25 } } });
        assert_eq!(usage_of(&payload), Some(Usage { input: 100, cached: 60, output: 40, thoughts: 25 }));
        assert_eq!(usage_of(&json!({ "usage": null })), None);
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
}
