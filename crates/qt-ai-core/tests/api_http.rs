//! HttpModel gọi thật qua reqwest tới server TCP giả: kiểm đường dẫn, header, body và đọc SSE.
use qt_ai_core::api::{ApiConfig, ApiError, ApiProvider, HttpModel, TextModel};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

struct Captured {
    head: String,
    body: String,
}

/// Nhận đúng một request, trả `status` + `body`, ghi lại request để assert.
fn serve_once(status: &'static str, content_type: &'static str, body: &'static str) -> (String, Arc<Mutex<Option<Captured>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let captured = Arc::new(Mutex::new(None));
    let slot = captured.clone();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 4096];
        let (head_end, content_length) = loop {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                panic!("client đóng sớm");
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(index) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buffer[..index]).to_string();
                let length = head
                    .lines()
                    .find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap()))
                    .unwrap_or(0);
                break (index + 4, length);
            }
        };
        while buffer.len() < head_end + content_length {
            let read = stream.read(&mut chunk).unwrap();
            buffer.extend_from_slice(&chunk[..read]);
        }
        let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
        let request_body = String::from_utf8_lossy(&buffer[head_end..head_end + content_length]).to_string();
        *slot.lock().unwrap() = Some(Captured { head, body: request_body });
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });
    (base, captured)
}

fn captured(slot: &Arc<Mutex<Option<Captured>>>) -> (String, serde_json::Value) {
    let guard = slot.lock().unwrap();
    let request = guard.as_ref().expect("server đã nhận request");
    (request.head.to_ascii_lowercase(), serde_json::from_str(&request.body).unwrap())
}

#[test]
fn openai_stream_qua_hub_http_gui_dung_header_body_va_doc_sse() {
    let (base, slot) = serve_once(
        "200 OK",
        "text/event-stream",
        "data: {\"choices\":[{\"delta\":{\"reasoning\":\"nghĩ\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"Xin \"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"chào\"}}]}\n\ndata: [DONE]\n\n",
    );
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk-hub", "gemini-3.7-flash", &format!("{base}/v1/"), true, "xhigh"));
    let mut progress = Vec::new();
    let out = model.generate("SYS", "USER", &AtomicBool::new(false), &mut |n| progress.push(n)).unwrap();
    assert_eq!(out, "Xin chào");
    assert_eq!(progress, vec![4, 8]);
    let (head, body) = captured(&slot);
    assert!(head.starts_with("post /v1/chat/completions http/1.1"), "{head}");
    assert!(head.contains("authorization: bearer sk-hub"));
    assert!(head.contains("content-type: application/json"));
    assert_eq!(body["model"], "gemini-3.7-flash");
    assert_eq!(body["reasoning_effort"], "xhigh");
    assert_eq!(body["messages"][1]["content"], "USER");
}

#[test]
fn gemini_stream_gui_key_qua_header_va_bao_blocked() {
    let (base, slot) = serve_once("200 OK", "text/event-stream", "data: {\"promptFeedback\":{\"blockReason\":\"SAFETY\"}}\n\n");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::Gemini, "AIza-key", "gemini-3.7-flash", &base, false, ""));
    let result = model.generate("SYS", "USER", &AtomicBool::new(false), &mut |_| {});
    assert_eq!(result, Err(ApiError::Blocked("SAFETY".into())));
    let (head, body) = captured(&slot);
    assert!(head.starts_with("post /v1beta/models/gemini-3.7-flash:streamgeneratecontent?alt=sse"), "{head}");
    assert!(head.contains("x-goog-api-key: aiza-key"));
    assert_eq!(body["generationConfig"]["thinkingConfig"]["thinkingLevel"], "minimal");
}

#[test]
fn http_loi_tra_status_va_message_cua_provider() {
    let (base, _) = serve_once("401 Unauthorized", "application/json", "{\"error\":{\"message\":\"Invalid API key\"}}");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "bad", "", &base, true, ""));
    let result = model.generate("S", "U", &AtomicBool::new(false), &mut |_| {});
    assert_eq!(result, Err(ApiError::Http { provider: "OpenAI", status: 401, message: "Invalid API key".into() }));
    assert!(!result.unwrap_err().is_transient());
}

#[test]
fn complete_json_openai_va_gemini() {
    let (base, slot) = serve_once("200 OK", "application/json", "{\"choices\":[{\"message\":{\"content\":\"{\\\"entries\\\":[]}\"}}]}");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "gpt-5.6-sol", &base, true, "high"));
    assert_eq!(model.complete_json("S", "U").unwrap(), "{\"entries\":[]}");
    let (head, body) = captured(&slot);
    assert!(head.contains("accept: application/json"));
    assert_eq!(body["response_format"]["type"], "json_object");
    assert!(body.get("reasoning_effort").is_none() && body.get("stream").is_none());

    let (base, slot) = serve_once("200 OK", "application/json", "{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"{}\"}]}}]}");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::Gemini, "AIza", "gemini-3.7-flash", &base, true, ""));
    assert_eq!(model.complete_json("S", "U").unwrap(), "{}");
    let (head, body) = captured(&slot);
    assert!(head.starts_with("post /v1beta/models/gemini-3.7-flash:generatecontent http"), "{head}");
    assert_eq!(body["generationConfig"]["responseMimeType"], "application/json");
}

#[test]
fn khong_ket_noi_duoc_la_network_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "", &base, true, ""));
    let result = model.generate("S", "U", &AtomicBool::new(false), &mut |_| {});
    assert!(matches!(result, Err(ApiError::Network { provider: "OpenAI", .. })), "{result:?}");
    assert!(result.unwrap_err().is_transient());
}
