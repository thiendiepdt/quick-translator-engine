//! OpenAI-compatible chat/completions: OpenAI chính chủ hoặc hub bất kỳ qua base URL. Port từ qt-web
//! (provider "openai"): max_completion_tokens, reasoning_effort, đọc reasoning_content/reasoning.

use crate::api::sse::read_sse;
use crate::api::{ApiConfig, ApiError, MAX_OUTPUT_TOKENS};
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

pub fn stream_body(config: &ApiConfig, system: &str, user: &str) -> Value {
    let mut body = json!({
        "model": config.model,
        "messages": messages(system, user),
        "max_completion_tokens": MAX_OUTPUT_TOKENS,
        "stream": true,
    });
    if !config.reasoning_effort.is_empty() {
        body["reasoning_effort"] = json!(config.reasoning_effort);
    }
    body
}

/// GPT-5/o-series chỉ nhận temperature mặc định — không gửi.
pub fn json_body(config: &ApiConfig, system: &str, user: &str) -> Value {
    json!({
        "model": config.model,
        "response_format": { "type": "json_object" },
        "messages": messages(system, user),
    })
}

pub fn parse_stream<R: BufRead>(
    reader: R,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(usize),
) -> Result<String, ApiError> {
    let mut output = String::new();
    read_sse(reader, cancel, |payload| {
        if let Some(message) = payload.pointer("/error/message").and_then(Value::as_str) {
            return Err(ApiError::Stream(message.to_string()));
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
    Ok(output)
}

pub fn parse_json_response(payload: &Value) -> Option<String> {
    payload.pointer("/choices/0/message/content")?.as_str().map(String::from)
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
        let body = stream_body(&config, "S", "U");
        assert_eq!(body["model"], "gemini-3.7-flash");
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["max_completion_tokens"], 65536);
        assert_eq!(body["stream"], true);
        assert!(body.get("max_tokens").is_none() && body.get("temperature").is_none());
        assert_eq!(body["messages"][0], json!({ "role": "system", "content": "S" }));

        let no_effort = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "", "", true, "");
        assert!(stream_body(&no_effort, "S", "U").get("reasoning_effort").is_none());
        let json = json_body(&no_effort, "S", "U");
        assert_eq!(json["response_format"]["type"], "json_object");
        assert!(json.get("temperature").is_none());
    }

    #[test]
    fn parse_stream_gom_content_bo_reasoning() {
        let sse = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({ "choices": [{ "delta": { "reasoning": "nghĩ" } }] }),
            json!({ "choices": [{ "delta": { "content": "Bản dịch" } }] }),
        );
        let out = parse_stream(Cursor::new(sse), &AtomicBool::new(false), &mut |_| {}).unwrap();
        assert_eq!(out, "Bản dịch");
        let err = format!("data: {}\n\n", json!({ "error": { "message": "quota" } }));
        assert_eq!(
            parse_stream(Cursor::new(err), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Stream("quota".into()))
        );
        let payload = json!({ "choices": [{ "message": { "content": "{}" } }] });
        assert_eq!(parse_json_response(&payload).as_deref(), Some("{}"));
    }
}
