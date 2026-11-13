//! Lỗi của một lượt chạy extension. Thông báo đã che khoá (xem `engine::redact`) trước khi tới đây.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExtError {
    #[error("manifest: {0}")]
    Manifest(String),
    #[error("gói extension: {0}")]
    Package(String),
    #[error("không được gọi tới {0}: domain không có trong hosts của extension")]
    HostDenied(String),
    #[error("lỗi mạng: {0}")]
    Network(String),
    #[error("phản hồi quá lớn (trên {0} byte)")]
    TooLarge(usize),
    #[error("{0} chạy quá thời gian cho phép")]
    Timeout(&'static str),
    #[error("extension chạy liên tục quá lâu, đã bị ngắt")]
    Interrupted,
    #[error("extension dùng quá bộ nhớ cho phép")]
    OutOfMemory,
    #[error("đã huỷ")]
    Cancelled,
    #[error("lỗi script: {0}")]
    Script(String),
    #[error("phiên đăng nhập hết hạn: {0}")]
    AuthExpired(String),
    #[error("chương bị khoá: {0}")]
    Locked(String),
    #[error("cần mở app để thao tác: {0}")]
    NeedsInteraction(String),
    #[error("{message}")]
    Extension { message: String, retry: bool },
    #[error("extension trả kết quả sai: {0}")]
    BadResult(String),
}
