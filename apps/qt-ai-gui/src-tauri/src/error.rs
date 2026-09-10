use qt_ai_core::CoreError;
use serde::Serialize;

/// Lỗi trả về UI: `kind` để quyết định hành động (vd. agy_missing → màn cài agy), `message` để hiển thị.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub kind: String,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: &str, message: impl Into<String>) -> Self {
        CommandError { kind: kind.to_string(), message: message.into() }
    }
}

impl From<CoreError> for CommandError {
    fn from(error: CoreError) -> Self {
        CommandError { kind: error.kind().to_string(), message: error.to_string() }
    }
}

pub type CmdResult<T> = Result<T, CommandError>;

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::CoreError;

    #[test]
    fn map_kind_va_message_tu_core_error() {
        let error: CommandError = CoreError::AgyMissing.into();
        assert_eq!(error.kind, "agy_missing");
        assert!(error.message.contains("agy"));
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["kind"], "agy_missing");
    }
}

/// Thân lệnh tốn thời gian (HTTP, spawn tiến trình, quét raw/…) chạy trên pool blocking của tokio.
/// KHÔNG dùng `#[tauri::command(async)]` cho hàm sync: Tauri gọi thân hàm ngay trong task async, chiếm
/// worker của runtime; đủ lệnh chặn cùng lúc (12 worker) là mọi lệnh async khác treo vô hạn.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> CmdResult<T> + Send + 'static) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|error| CommandError::new("internal", format!("Thread lệnh kết thúc bất thường: {error}")))?
}
