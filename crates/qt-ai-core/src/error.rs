use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Thiếu story.json/state.json hoặc chương không tồn tại.
    #[error("{0}")]
    StoryNotFound(String),
    /// Vi phạm state machine (còn chương translating, hết hàng đợi, retry chương done…).
    #[error("{0}")]
    InvalidState(String),
    /// story.json không parse được / sai schema.
    #[error("{0}")]
    InvalidStory(String),
    #[error("Không tìm thấy agy — cài Antigravity CLI (irm https://antigravity.google/cli/install.ps1 | iex) rồi đăng nhập Google.")]
    AgyMissing,
    #[error("agy thoát mã {code}: {stderr_tail}")]
    AgyFailed { code: i32, stderr_tail: String },
    #[error("Truyện đang có phiên khác chạy (PID {pid}).")]
    SessionLocked { pid: u32 },
    #[error("Lỗi IO {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Lỗi nội bộ: {0}")]
    Internal(String),
}

impl CoreError {
    /// Mã ổn định để UI quyết định hành động (Tauri serialize `{kind, message}`).
    pub fn kind(&self) -> &'static str {
        match self {
            CoreError::StoryNotFound(_) => "story_not_found",
            CoreError::InvalidState(_) => "invalid_state",
            CoreError::InvalidStory(_) => "invalid_story",
            CoreError::AgyMissing => "agy_missing",
            CoreError::AgyFailed { .. } => "agy_failed",
            CoreError::SessionLocked { .. } => "session_locked",
            CoreError::Io { .. } => "io",
            CoreError::Internal(_) => "internal",
        }
    }

    pub fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> CoreError {
        let path = path.into();
        move |source| CoreError::Io { path, source }
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;
