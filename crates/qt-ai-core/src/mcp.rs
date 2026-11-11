//! Server MCP stdio cho agy: `qt-ai mcp <truyện>` (agy tự chạy theo `<truyện>/.agents/mcp_config.json`, xem
//! `crate::mcp_config`). Mỗi dòng vào là một message JSON-RPC 2.0, mỗi response là một dòng ra. Tool `qt_*`
//! gọi đúng code của lệnh CLI cùng tên (`crate::agent_reply`), chữ đáp y như CLI; mã thoát khác 0 thành
//! `isError: true`. Chỉ ghi JSON-RPC vào `output` — cảnh báo của core đi stderr.

use crate::agent_reply::{self, Reply};
use crate::commands::next::Via;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

/// Phiên bản giao thức biết trả lời, mới nhất đầu. Client xin bản khác thì đáp bản mới nhất (theo spec MCP).
const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

fn tools() -> Value {
    let id = json!({"type": "string", "description": "Id chương, chép nguyên văn chuỗi id qt_next trả về (thường là tên file raw, có thể dài kèm tên chương)."});
    let none = json!({"type": "object", "properties": {}});
    json!([
        {"name": "qt_status",
         "description": "Bảng tiến độ của truyện; dòng cuối ghi số chương tối đa mỗi phiên. Gọi đầu phiên và khi hết hàng đợi.",
         "inputSchema": none},
        {"name": "qt_next",
         "description": "Phát chương kế tiếp: trả id chương và đường dẫn work/<id>.prompt.md cần đọc để dịch. Báo lỗi khi còn chương dịch dở chưa chốt hoặc hết hàng đợi.",
         "inputSchema": none},
        {"name": "qt_check",
         "description": "Kiểm tra bản dịch work/<id>.draft.md. PASS thì gọi qt_accept; FAIL thì sửa theo work/<id>.review.md rồi gọi lại.",
         "inputSchema": {"type": "object", "properties": {"id": id}, "required": ["id"]}},
        {"name": "qt_accept",
         "description": "Chốt chương đã PASS: ghi out/, gộp glossary. Không tự đặt force — force là quyết định của người dùng.",
         "inputSchema": {"type": "object", "properties": {
             "id": id,
             "force": {"type": "boolean", "description": "Chốt dù check chưa PASS — chỉ khi người dùng bảo."}
         }, "required": ["id"]}},
        {"name": "qt_skip",
         "description": "Bỏ qua chương, kèm lý do (ví dụ bộ lọc chặn cả 3 lượt).",
         "inputSchema": {"type": "object", "properties": {
             "id": id,
             "reason": {"type": "string", "description": "Lý do ngắn."}
         }, "required": ["id", "reason"]}},
        {"name": "qt_retry",
         "description": "Đưa chương về hàng đợi để dịch lại (bản dịch cũ giữ .bak).",
         "inputSchema": {"type": "object", "properties": {"id": id}, "required": ["id"]}},
    ])
}

/// Đọc tới EOF (agy đóng stdin khi thoát), đáp từng request. Notification (không có id) không đáp.
pub fn serve<R: BufRead, W: Write>(input: R, mut output: W, root: &Path) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&message, root),
            Err(_) => Some(error(Value::Null, -32700, "Parse error")),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn handle(message: &Value, root: &Path) -> Option<Value> {
    let id = message.get("id")?.clone();
    let method = message["method"].as_str().unwrap_or_default();
    let params = &message["params"];
    let outcome = match method {
        "initialize" => Ok(initialize(params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => call_tool(params, root),
        _ => Err((-32601, format!("Method not found: {method}"))),
    };
    Some(match outcome {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err((code, message)) => error(id, code, &message),
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params["protocolVersion"].as_str().unwrap_or_default();
    let version = PROTOCOL_VERSIONS.iter().find(|v| **v == asked).unwrap_or(&PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {"name": "qt-ai", "version": env!("CARGO_PKG_VERSION")},
    })
}

/// Tham số chữ (model đôi khi gửi id dạng số → đổi ra chữ); thiếu/rỗng → lời nhắc để model gọi lại cho đúng.
fn text_arg(args: &Value, key: &str) -> Result<String, String> {
    match &args[key] {
        Value::String(text) if !text.trim().is_empty() => Ok(text.trim().to_string()),
        Value::Number(number) => Ok(number.to_string()),
        _ => Err(format!("Thiếu tham số \"{key}\".")),
    }
}

/// None = không có tool tên này.
fn run_tool(name: &str, args: &Value, root: &Path) -> Option<Result<Reply, String>> {
    let run = |result: crate::Result<Reply>| result.map_err(|e| e.to_string());
    let id = || text_arg(args, "id");
    Some(match name {
        "qt_status" => run(agent_reply::status(root)),
        "qt_next" => run(agent_reply::next(root, Via::Mcp)),
        "qt_check" => id().and_then(|id| run(agent_reply::check(root, &id))),
        "qt_accept" => id().and_then(|id| run(agent_reply::accept(root, &id, args["force"].as_bool().unwrap_or(false)))),
        "qt_skip" => id().and_then(|id| text_arg(args, "reason").and_then(|reason| run(agent_reply::skip(root, &id, &reason)))),
        "qt_retry" => id().and_then(|id| run(agent_reply::retry(root, &id))),
        _ => return None,
    })
}

fn call_tool(params: &Value, root: &Path) -> Result<Value, (i64, String)> {
    let name = params["name"].as_str().unwrap_or_default();
    // Server sống suốt phiên agy: một chương lỗi lạ không được giết cả phiên (CLI cũ chết riêng từng lệnh).
    let outcome = catch_unwind(AssertUnwindSafe(|| run_tool(name, &params["arguments"], root)))
        .unwrap_or_else(|_| Some(Err("Lỗi nội bộ qt-ai (panic) — xem stderr.".into())))
        .ok_or_else(|| (-32602, format!("Unknown tool: {name}")))?;
    let (text, is_error) = match outcome {
        Ok(reply) => (reply.text, reply.code != 0),
        Err(message) => (message, true),
    };
    Ok(json!({"content": [{"type": "text", "text": text}], "isError": is_error}))
}
