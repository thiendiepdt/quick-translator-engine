//! Bảng phiên dịch theo folder truyện: nhiều truyện chạy song song, mỗi truyện tối đa một phiên,
//! tổng số phiên đang chạy bị chặn bởi `max_parallel` (config). Thuần Rust, không dính Tauri để test được.

use crate::error::{CmdResult, CommandError};
use qt_ai_core::session::{SessionHandle, StopReason};
use std::collections::HashMap;

/// Phần registry cần từ một phiên; `SessionHandle` thật và handle giả trong test đều cài được.
pub trait SessionLike: Send {
    fn is_running(&self) -> bool;
    fn cancel(&self);
    fn join(self: Box<Self>) -> StopReason;
}

impl SessionLike for SessionHandle {
    fn is_running(&self) -> bool {
        SessionHandle::is_running(self)
    }
    fn cancel(&self) {
        SessionHandle::cancel(self)
    }
    fn join(self: Box<Self>) -> StopReason {
        SessionHandle::join(*self)
    }
}

/// Khoá theo đường dẫn không phân biệt hoa thường và dấu `/` `\` cuối (Windows).
pub fn root_key(root: &str) -> String {
    root.trim_end_matches(['\\', '/']).replace('/', "\\").to_lowercase()
}

#[derive(Default)]
pub struct SessionRegistry {
    sessions: HashMap<String, (String, Box<dyn SessionLike>)>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bỏ các phiên đã tự kết thúc (thread runner xong) khỏi bảng.
    fn prune(&mut self) {
        self.sessions.retain(|_, (_, handle)| handle.is_running());
    }

    /// Root (đúng chữ lúc start) của mọi phiên đang chạy, sắp theo tên để UI ổn định.
    pub fn running_roots(&mut self) -> Vec<String> {
        self.prune();
        let mut roots: Vec<String> = self.sessions.values().map(|(root, _)| root.clone()).collect();
        roots.sort();
        roots
    }

    pub fn is_running(&mut self, root: &str) -> bool {
        self.prune();
        self.sessions.contains_key(&root_key(root))
    }

    /// Kiểm trước khi start: truyện này chưa chạy và còn chỗ dưới `max_parallel`.
    pub fn check_can_start(&mut self, root: &str, max_parallel: usize) -> CmdResult<()> {
        self.prune();
        if self.sessions.contains_key(&root_key(root)) {
            return Err(CommandError::new("session_locked", "Truyện này đang có phiên dịch chạy — bấm Dừng trước."));
        }
        let running = self.sessions.len();
        if running >= max_parallel.max(1) {
            return Err(CommandError::new(
                "session_limit",
                format!(
                    "Đang chạy {running}/{max_parallel} truyện — dừng một truyện hoặc nâng \"Số truyện dịch song song\" trong Cài đặt."
                ),
            ));
        }
        Ok(())
    }

    pub fn insert(&mut self, root: &str, handle: Box<dyn SessionLike>) {
        self.sessions.insert(root_key(root), (root.to_string(), handle));
    }

    /// Gỡ phiên khỏi bảng; caller cancel + join ngoài lock để không chặn lệnh khác.
    pub fn take(&mut self, root: &str) -> Option<Box<dyn SessionLike>> {
        self.sessions.remove(&root_key(root)).map(|(_, handle)| handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct Fake {
        running: Arc<AtomicBool>,
    }

    impl SessionLike for Fake {
        fn is_running(&self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
        fn cancel(&self) {
            self.running.store(false, Ordering::SeqCst);
        }
        fn join(self: Box<Self>) -> StopReason {
            StopReason::UserCancelled
        }
    }

    fn fake() -> (Arc<AtomicBool>, Box<dyn SessionLike>) {
        let flag = Arc::new(AtomicBool::new(true));
        (flag.clone(), Box::new(Fake { running: flag }))
    }

    #[test]
    fn chay_song_song_toi_max_va_tu_choi_cung_root() {
        let mut reg = SessionRegistry::new();
        reg.check_can_start("D:\\lib\\a", 2).unwrap();
        reg.insert("D:\\lib\\a", fake().1);
        assert_eq!(reg.check_can_start("d:/lib/a/", 2).unwrap_err().kind, "session_locked");
        reg.check_can_start("D:\\lib\\b", 2).unwrap();
        reg.insert("D:\\lib\\b", fake().1);
        let err = reg.check_can_start("D:\\lib\\c", 2).unwrap_err();
        assert_eq!(err.kind, "session_limit");
        assert!(err.message.contains("2/2"));
        assert_eq!(reg.running_roots(), vec!["D:\\lib\\a", "D:\\lib\\b"]);
        assert!(reg.is_running("D:\\LIB\\A") && !reg.is_running("D:\\lib\\c"));
        // max_parallel 0 vẫn cho chạy 1 (không tự khoá app).
        let mut solo = SessionRegistry::new();
        solo.check_can_start("x", 0).unwrap();
    }

    #[test]
    fn phien_tu_ket_thuc_thi_tu_don_va_take_gỡ_dung_root() {
        let mut reg = SessionRegistry::new();
        let (flag_a, a) = fake();
        reg.insert("D:\\a", a);
        reg.insert("D:\\b", fake().1);
        flag_a.store(false, Ordering::SeqCst);
        assert_eq!(reg.running_roots(), vec!["D:\\b"]);
        assert!(reg.take("D:\\a").is_none());
        let handle = reg.take("d:/b").unwrap();
        handle.cancel();
        assert!(!handle.is_running());
        assert!(matches!(handle.join(), StopReason::UserCancelled));
        assert!(reg.running_roots().is_empty());
    }
}
