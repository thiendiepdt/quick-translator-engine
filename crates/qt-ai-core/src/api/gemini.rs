//! Gemini API chính chủ: streamGenerateContent (SSE) + generateContent (JSON). Port 1-1 từ qt-web.

use crate::api::sse::read_sse;
use crate::api::{ApiConfig, ApiError, MAX_OUTPUT_TOKENS};
use serde_json::{json, Value};
use std::io::BufRead;
use std::sync::atomic::AtomicBool;

const SAFETY_CATEGORIES: [&str; 5] = [
    "HARM_CATEGORY_HARASSMENT",
    "HARM_CATEGORY_HATE_SPEECH",
    "HARM_CATEGORY_SEXUALLY_EXPLICIT",
    "HARM_CATEGORY_DANGEROUS_CONTENT",
    "HARM_CATEGORY_CIVIC_INTEGRITY",
];

pub fn normalized_model(model: &str) -> &str {
    model.strip_prefix("models/").unwrap_or(model)
}

fn model_major(model: &str) -> Option<u32> {
    let rest = normalized_model(model).strip_prefix("gemini-")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Port `buildGeminiTextGenerationConfig`: 2.5 dùng thinkingBudget, 3.x dùng thinkingLevel high/minimal.
pub fn generation_config(model: &str, thinking: bool) -> Value {
    let major = model_major(model);
    let mut config = serde_json::Map::new();
    config.insert("maxOutputTokens".into(), json!(MAX_OUTPUT_TOKENS));
    if major.is_none_or(|major| major < 3) {
        config.insert("temperature".into(), json!(0.3));
    }
    let mut thinking_config = serde_json::Map::new();
    if major == Some(2) && normalized_model(model).contains("2.5") {
        thinking_config.insert("thinkingBudget".into(), json!(if thinking { -1 } else { 0 }));
    } else if major.is_some_and(|major| major >= 3) {
        thinking_config.insert("thinkingLevel".into(), json!(if thinking { "high" } else { "minimal" }));
    }
    if major.is_some_and(|major| major >= 3) || !thinking_config.is_empty() {
        thinking_config.insert("includeThoughts".into(), json!(true));
    }
    if !thinking_config.is_empty() {
        config.insert("thinkingConfig".into(), Value::Object(thinking_config));
    }
    Value::Object(config)
}

fn safety_settings() -> Value {
    Value::Array(SAFETY_CATEGORIES.iter().map(|category| json!({ "category": category, "threshold": "OFF" })).collect())
}

pub fn stream_url(config: &ApiConfig) -> String {
    format!("{}/v1beta/models/{}:streamGenerateContent?alt=sse", config.base_url, normalized_model(&config.model))
}

pub fn json_url(config: &ApiConfig) -> String {
    format!("{}/v1beta/models/{}:generateContent", config.base_url, normalized_model(&config.model))
}

pub fn headers(config: &ApiConfig) -> Vec<(&'static str, String)> {
    vec![("x-goog-api-key", config.api_key.clone())]
}

pub fn stream_body(config: &ApiConfig, system: &str, user: &str) -> Value {
    json!({
        "systemInstruction": { "parts": [{ "text": system }] },
        "contents": [{ "role": "user", "parts": [{ "text": user }] }],
        "safetySettings": safety_settings(),
        "generationConfig": generation_config(&config.model, config.thinking),
    })
}

pub fn json_body(_config: &ApiConfig, system: &str, user: &str) -> Value {
    json!({
        "systemInstruction": { "parts": [{ "text": system }] },
        "contents": [{ "role": "user", "parts": [{ "text": user }] }],
        "generationConfig": { "temperature": 0.0, "responseMimeType": "application/json" },
    })
}

fn blocked_reason(payload: &Value) -> Option<String> {
    if let Some(reason) = payload.pointer("/promptFeedback/blockReason").and_then(Value::as_str) {
        return Some(reason.to_string());
    }
    let reason = payload.pointer("/candidates/0/finishReason")?.as_str()?;
    (reason != "STOP" && reason != "MAX_TOKENS").then(|| reason.to_string())
}

pub fn parse_stream<R: BufRead>(
    reader: R,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(usize),
) -> Result<String, ApiError> {
    let mut output = String::new();
    let mut blocked: Option<String> = None;
    read_sse(reader, cancel, |payload| {
        if let Some(message) = payload.pointer("/error/message").and_then(Value::as_str) {
            return Err(ApiError::Stream(message.to_string()));
        }
        if let Some(reason) = blocked_reason(&payload) {
            blocked = Some(reason);
        }
        let Some(parts) = payload.pointer("/candidates/0/content/parts").and_then(Value::as_array) else {
            return Ok(());
        };
        for part in parts {
            let Some(text) = part.get("text").and_then(Value::as_str) else { continue };
            if text.is_empty() || part.get("thought").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            output.push_str(text);
            on_progress(output.chars().count());
        }
        Ok(())
    })?;
    if !output.is_empty() {
        return Ok(output);
    }
    match blocked {
        Some(reason) => Err(ApiError::Blocked(reason)),
        None => Err(ApiError::Empty("Gemini")),
    }
}

pub fn parse_json_response(payload: &Value) -> Option<String> {
    payload
        .pointer("/candidates/0/content/parts")?
        .as_array()?
        .iter()
        .find_map(|part| part.get("text").and_then(Value::as_str).map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiProvider;
    use std::io::Cursor;

    fn config() -> ApiConfig {
        ApiConfig::resolve(ApiProvider::Gemini, "AIza", "gemini-3.7-flash", "", true, "")
    }

    #[test]
    fn generation_config_theo_the_he_model() {
        assert_eq!(
            generation_config("gemini-3.7-flash", true),
            json!({ "maxOutputTokens": 65536, "thinkingConfig": { "thinkingLevel": "high", "includeThoughts": true } })
        );
        assert_eq!(
            generation_config("models/gemini-3.1-flash-lite", false)["thinkingConfig"]["thinkingLevel"],
            json!("minimal")
        );
        assert_eq!(
            generation_config("gemini-2.5-flash", false),
            json!({ "maxOutputTokens": 65536, "temperature": 0.3, "thinkingConfig": { "thinkingBudget": 0, "includeThoughts": true } })
        );
        assert_eq!(generation_config("gemini-2.0-flash", true), json!({ "maxOutputTokens": 65536, "temperature": 0.3 }));
    }

    #[test]
    fn url_header_body_stream() {
        let config = config();
        assert_eq!(
            stream_url(&config),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.7-flash:streamGenerateContent?alt=sse"
        );
        assert!(json_url(&config).ends_with(":generateContent"));
        assert_eq!(headers(&config), vec![("x-goog-api-key", "AIza".to_string())]);
        let body = stream_body(&config, "SYS", "USER");
        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "SYS");
        assert_eq!(body["contents"][0]["parts"][0]["text"], "USER");
        assert_eq!(body["safetySettings"].as_array().unwrap().len(), 5);
        assert_eq!(body["safetySettings"][0]["threshold"], "OFF");
        let json = json_body(&config, "S", "U");
        assert_eq!(json["generationConfig"]["responseMimeType"], "application/json");
    }

    #[test]
    fn parse_stream_gom_text_bo_thought_va_bao_blocked() {
        let sse = format!(
            "data: {}\n\ndata: {}\n\n",
            json!({ "candidates": [{ "content": { "parts": [{ "text": "nghĩ", "thought": true }, { "text": "Xin " }] } }] }),
            json!({ "candidates": [{ "content": { "parts": [{ "text": "chào" }] }, "finishReason": "STOP" }] }),
        );
        let mut progress = Vec::new();
        let out = parse_stream(Cursor::new(sse), &AtomicBool::new(false), &mut |n| progress.push(n)).unwrap();
        assert_eq!(out, "Xin chào");
        assert_eq!(progress, vec![4, 8]);

        let blocked = format!("data: {}\n\n", json!({ "promptFeedback": { "blockReason": "PROHIBITED_CONTENT" } }));
        assert_eq!(
            parse_stream(Cursor::new(blocked), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Blocked("PROHIBITED_CONTENT".into()))
        );
        let safety = format!("data: {}\n\n", json!({ "candidates": [{ "finishReason": "SAFETY" }] }));
        assert_eq!(
            parse_stream(Cursor::new(safety), &AtomicBool::new(false), &mut |_| {}),
            Err(ApiError::Blocked("SAFETY".into()))
        );
        assert_eq!(parse_stream(Cursor::new(""), &AtomicBool::new(false), &mut |_| {}), Err(ApiError::Empty("Gemini")));
        let payload = json!({ "candidates": [{ "content": { "parts": [{ "text": "{\"entries\":[]}" }] } }] });
        assert_eq!(parse_json_response(&payload).as_deref(), Some("{\"entries\":[]}"));
    }
}
