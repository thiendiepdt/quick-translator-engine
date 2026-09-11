//! HttpModel gọi thật qua reqwest tới server TCP giả: kiểm đường dẫn, header, body và đọc SSE.
use qt_ai_core::api::{ApiConfig, ApiError, ApiProvider, HttpModel, TextModel};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    assert_eq!(model.complete_json("S", "U", &AtomicBool::new(false)).unwrap(), "{\"entries\":[]}");
    let (head, body) = captured(&slot);
    assert!(head.contains("accept: application/json"));
    assert_eq!(body["response_format"]["type"], "json_object");
    assert!(body.get("reasoning_effort").is_none() && body.get("stream").is_none());

    let (base, slot) = serve_once("200 OK", "application/json", "{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"{}\"}]}}]}");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::Gemini, "AIza", "gemini-3.7-flash", &base, true, ""));
    assert_eq!(model.complete_json("S", "U", &AtomicBool::new(false)).unwrap(), "{}");
    let (head, body) = captured(&slot);
    assert!(head.starts_with("post /v1beta/models/gemini-3.7-flash:generatecontent http"), "{head}");
    assert_eq!(body["generationConfig"]["responseMimeType"], "application/json");
}

/// Nhận lần lượt N request (mỗi request một kết nối), trả theo thứ tự; ghi lại body từng request.
fn serve_sequence(responses: Vec<(&'static str, &'static str, &'static str)>) -> (String, Arc<Mutex<Vec<serde_json::Value>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let slot = bodies.clone();
    std::thread::spawn(move || {
        for (status, content_type, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 4096];
            let (head_end, content_length) = loop {
                let read = stream.read(&mut chunk).unwrap();
                assert!(read > 0, "client đóng sớm");
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
            let request_body = String::from_utf8_lossy(&buffer[head_end..head_end + content_length]).to_string();
            slot.lock().unwrap().push(serde_json::from_str(&request_body).unwrap());
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });
    (base, bodies)
}

#[test]
fn complete_json_hub_tu_choi_json_mode_thi_fallback_text_thuong_va_boc_json() {
    // Hub kiểu gemini-proxy: 400 khi có response_format → gọi lại stream thường, model bọc ```json.
    let (base, bodies) = serve_sequence(vec![
        ("400 Bad Request", "application/json", "{\"error\":{\"message\":\"response_format is not supported\"}}"),
        (
            "200 OK",
            "text/event-stream",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Đây là kết quả:\\n```json\\n{\\\"entries\\\":[{\\\"source\\\":\\\"赵静文\\\"}]}\\n```\"}}]}\n\ndata: [DONE]\n\n",
        ),
    ]);
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "gemini-3.8-flash", &base, true, "high"));
    let out = model.complete_json("Trích glossary", "U", &AtomicBool::new(false)).unwrap();
    assert_eq!(out, "{\"entries\":[{\"source\":\"赵静文\"}]}");
    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[0]["response_format"]["type"], "json_object");
    assert!(bodies[1].get("response_format").is_none());
    assert_eq!(bodies[1]["stream"], true);
    assert!(bodies[1]["messages"][0]["content"].as_str().unwrap().contains("Chỉ trả về đúng một JSON object"));
}

#[test]
fn complete_json_boc_rao_markdown_ngay_o_json_mode_va_content_dang_mang() {
    let (base, _) = serve_once(
        "200 OK",
        "application/json",
        "{\"choices\":[{\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"```json\\n{\\\"a\\\":1}\\n```\"}]}}]}",
    );
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "m", &base, true, ""));
    assert_eq!(model.complete_json("S", "U", &AtomicBool::new(false)).unwrap(), "{\"a\":1}");

    let (base, _) = serve_once(
        "200 OK",
        "application/json",
        "{\"candidates\":[{\"content\":{\"parts\":[{\"thought\":true,\"text\":\"nghĩ đã\"},{\"text\":\"{\\\"b\\\":2}\"}]}}]}",
    );
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::Gemini, "AIza", "gemini-3.7-flash", &base, true, ""));
    assert_eq!(model.complete_json("S", "U", &AtomicBool::new(false)).unwrap(), "{\"b\":2}");
}

#[test]
fn complete_json_loi_thoang_qua_hay_bi_chan_thi_khong_fallback() {
    let (base, bodies) = serve_sequence(vec![("429 Too Many Requests", "application/json", "{\"error\":{\"message\":\"slow down\"}}")]);
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "m", &base, true, ""));
    let error = model.complete_json("S", "U", &AtomicBool::new(false)).unwrap_err();
    assert!(error.is_transient(), "{error:?}");
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(bodies.lock().unwrap().len(), 1); // không gọi lần hai
}

#[test]
fn extract_json_object_cac_truong_hop() {
    use qt_ai_core::api::extract_json_object;
    assert_eq!(extract_json_object("```json\n{\"a\":1}\n```").as_deref(), Some("{\"a\":1}"));
    assert_eq!(extract_json_object("Kết quả: {\"a\":{\"b\":[1,2]}} xong").as_deref(), Some("{\"a\":{\"b\":[1,2]}}"));
    assert_eq!(extract_json_object("không có gì"), None);
    assert_eq!(extract_json_object("{hỏng"), None);
    assert_eq!(extract_json_object("[1,2]"), None);
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

/// Server trả header 200 rồi im lặng (model đang "nghĩ") — giữ kết nối tới khi client đóng.
fn serve_stalled(content_type: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut chunk = [0u8; 4096];
        let mut seen = Vec::new();
        while !seen.windows(4).any(|w| w == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).unwrap();
            seen.extend_from_slice(&chunk[..read]);
        }
        stream
            .write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nTransfer-Encoding: chunked\r\n\r\n").as_bytes())
            .unwrap();
        stream.flush().unwrap();
        // Không gửi gì nữa; đọc tới khi client đóng kết nối.
        while stream.read(&mut chunk).map(|n| n > 0).unwrap_or(false) {}
    });
    base
}

fn cancel_after(delay: Duration) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    cancel
}

#[test]
fn huy_giua_luc_hub_im_lang_thi_generate_tra_cancelled_ngay() {
    let base = serve_stalled("text/event-stream");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::OpenAi, "k", "m", &format!("{base}/v1/"), false, ""));
    let cancel = cancel_after(Duration::from_millis(300));
    let started = Instant::now();
    let result = model.generate("S", "U", &cancel, &mut |_| {});
    assert_eq!(result, Err(ApiError::Cancelled));
    assert!(started.elapsed() < Duration::from_secs(3), "huỷ phải có tác dụng trong ~1 nhịp poll, không đợi hub");
}

#[test]
fn huy_giua_luc_cho_json_thi_complete_json_tra_cancelled_ngay() {
    let base = serve_stalled("application/json");
    let model = HttpModel::new(ApiConfig::resolve(ApiProvider::Gemini, "k", "m", &base, false, ""));
    let cancel = cancel_after(Duration::from_millis(300));
    let started = Instant::now();
    assert_eq!(model.complete_json("S", "U", &cancel), Err(ApiError::Cancelled));
    assert!(started.elapsed() < Duration::from_secs(3));
}
