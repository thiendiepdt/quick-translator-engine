//! Mục server MCP `qt-ai` trong `<truyện>/.agents/mcp_config.json` — agy nạp file này của workspace lúc mở phiên.
//!
//! Lý do có MCP: agy (Windows 10) chạy mỗi lệnh terminal trên một desktop ẩn mới, và DWM giữ lại 2 đối tượng
//! Composition cho mỗi desktop đã huỷ — dịch lâu (~3,6 lệnh qt-ai/chương) là cả máy lag khi kéo cửa sổ.
//! Server MCP là một tiến trình sống suốt phiên agy; gọi tool qua stdin/stdout, không tạo desktop nào.
//!
//! App ghi mục này trước mỗi phiên khi bật MCP, gỡ khi tắt. Template AGENTS.md/translate.md chọn bản gọi tool
//! hay bản chạy lệnh theo đúng việc file có mục `qt-ai` hay không (`has_server`), nên hai thứ luôn khớp nhau.
//! File của người dùng (server khác, khoá khác) giữ nguyên; file hỏng thì không ghi đè.

use crate::error::{CoreError, Result};
use crate::story_fs::write_text;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const SERVER_NAME: &str = "qt-ai";

pub fn config_path(root: &Path) -> PathBuf {
    root.join(".agents").join("mcp_config.json")
}

/// Nội dung file: Ok(None) khi chưa có file; Err khi file có mà không phải object JSON với `mcpServers` là object.
fn load(root: &Path) -> Result<Option<Map<String, Value>>> {
    let path = config_path(root);
    let Ok(text) = fs::read_to_string(&path) else { return Ok(None) };
    let broken = || CoreError::InvalidState(format!("{} không đúng dạng JSON của agy — không ghi đè.", path.display()));
    let Value::Object(config) = serde_json::from_str::<Value>(&text).map_err(|_| broken())? else { return Err(broken()) };
    if config.get("mcpServers").is_some_and(|servers| !servers.is_object()) {
        return Err(broken());
    }
    Ok(Some(config))
}

fn save(root: &Path, config: &Map<String, Value>) -> Result<()> {
    let path = config_path(root);
    let dir = root.join(".agents");
    fs::create_dir_all(&dir).map_err(CoreError::io(&dir))?;
    let text = serde_json::to_string_pretty(config).map_err(|e| CoreError::Internal(e.to_string()))?;
    write_text(&path, &format!("{text}\n"))
}

/// File có mục server `qt-ai` (file hỏng coi như không có).
pub fn has_server(root: &Path) -> bool {
    matches!(load(root), Ok(Some(config)) if config.get("mcpServers").and_then(|s| s.get(SERVER_NAME)).is_some())
}

/// Ghi mục `qt-ai` chạy `<exe> mcp <root>`. Trả true nếu có ghi (mục đã đúng thì không ghi lại).
pub fn install(root: &Path, exe: &Path) -> Result<bool> {
    let mut config = load(root)?.unwrap_or_default();
    let wanted = json!({"command": exe.display().to_string(), "args": ["mcp", root.display().to_string()]});
    let servers = config.entry("mcpServers").or_insert_with(|| Value::Object(Map::new()));
    let servers = servers.as_object_mut().expect("load đã kiểm mcpServers là object");
    if servers.get(SERVER_NAME) == Some(&wanted) {
        return Ok(false);
    }
    servers.insert(SERVER_NAME.to_string(), wanted);
    save(root, &config)?;
    Ok(true)
}

/// Trước mỗi phiên agy: `exe` = binary qt-ai khi bật MCP (None = tắt, hoặc chưa build binary) → ghi hoặc gỡ mục
/// `qt-ai`, rồi làm mới AGENTS.md / translate.md cho khớp (template tự chọn bản theo file này). Phải làm lúc bắt
/// đầu phiên chứ không chỉ lúc mở truyện: bấm dịch nhiều truyện từ thư viện chỉ mở truyện đầu. File của người
/// dùng hỏng thì không ghi đè, phiên chạy lệnh terminal như cũ. Trả true nếu phiên này đi qua MCP.
pub fn prepare(root: &Path, exe: Option<&Path>, qt_ai_command: &str) -> Result<bool> {
    match exe {
        Some(exe) => {
            let _ = install(root, exe);
        }
        None => {
            uninstall(root)?;
        }
    }
    crate::templates::copy_templates(root, qt_ai_command)?;
    Ok(has_server(root))
}

/// Gỡ mục `qt-ai`. File chỉ còn `{"mcpServers": {}}` thì xoá file, rồi xoá `.agents` nếu rỗng.
/// Trả true nếu có gỡ. File hỏng hay chưa có → không làm gì.
pub fn uninstall(root: &Path) -> Result<bool> {
    let Ok(Some(mut config)) = load(root) else { return Ok(false) };
    let Some(servers) = config.get_mut("mcpServers").and_then(Value::as_object_mut) else { return Ok(false) };
    if servers.shift_remove(SERVER_NAME).is_none() {
        return Ok(false);
    }
    let empty = servers.is_empty() && config.len() == 1;
    if empty {
        let path = config_path(root);
        fs::remove_file(&path).map_err(CoreError::io(&path))?;
        let _ = fs::remove_dir(root.join(".agents")); // còn thứ khác trong đó thì remove_dir tự từ chối
    } else {
        save(root, &config)?;
    }
    Ok(true)
}
