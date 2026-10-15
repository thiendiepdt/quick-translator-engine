//! Runner phiên agy — port auto-translate.ps1: mỗi phiên `agy -p` là một context sạch; poll state.json
//! để báo tiến độ; cầu dao hết queue / không tiến độ / agy lỗi 2 lần / max phiên; cancel giết process tree.

use crate::agy_data::{clean, quota_reset_since, Scope, APP_PROMPT_MARK};
use crate::commands::status::count_chapters_live;
use crate::error::{CoreError, Result};
use crate::story_fs::{load_state, now_ms, story_paths, ChapterStatus};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use crate::process::quiet_command;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub done: usize,
    pub queued: usize,
    pub translating: usize,
    pub error: usize,
    pub skipped: usize,
    pub warnings_count: usize,
    pub current: Option<String>,
}

impl Progress {
    /// Chương đã xử lý xong, không quay lại nữa — thước đo "có tiến độ".
    fn settled(&self) -> usize {
        self.done + self.error + self.skipped
    }
}

pub fn read_progress(root: &Path) -> Result<Progress> {
    let paths = story_paths(root);
    let state = load_state(&paths)?;
    let counts = count_chapters_live(&state, &crate::vn_check::open_by_chapter(&crate::vn_check::load_file(&paths)));
    let current = state
        .chapters
        .iter()
        .find(|(_, c)| c.status == ChapterStatus::Translating)
        .map(|(id, _)| id.clone());
    Ok(Progress {
        done: counts.done,
        queued: counts.queued,
        translating: counts.translating,
        error: counts.error,
        skipped: counts.skipped,
        warnings_count: counts.with_warnings,
        current,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StopReason {
    Finished,
    NoProgress,
    AgyFailed { code: i32 },
    /// Động cơ API: lỗi gọi model lặp lại (mạng/key/quota) — dừng để khỏi skip hàng loạt.
    ApiFailed { message: String },
    UserCancelled,
    MaxSessions,
    /// agy hết quota (máy chủ trả RESOURCE_EXHAUSTED) và phiên không được phép tự chờ. `reset_at`: ms epoch.
    QuotaExhausted { reset_at: u64 },
    Internal { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    Started { session_no: u32 },
    Progress(Progress),
    AgyLog { line: String, stream: LogStream },
    /// Hết quota agy: phiên đang chờ tới `until` (ms epoch) rồi tự chạy tiếp. Lượt kế phát `Started` như thường.
    QuotaWait { until: u64 },
    Stopped(StopReason),
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub root: PathBuf,
    pub agy: PathBuf,
    pub model: Option<String>,
    pub max_sessions: u32,
    pub poll_interval: Duration,
    /// Folder chứa binary `qt-ai` — chèn đầu PATH của agy để AGENTS.md chỉ cần ghi `qt-ai` trần,
    /// folder truyện mang sang máy khác vẫn chạy.
    pub extra_path: Option<PathBuf>,
    /// Thư mục dữ liệu của agy (`agy_data::default_data_dir`) — có thì hội thoại agy của truyện này bị xoá
    /// ngay sau mỗi lượt chạy; None = không đụng tới.
    pub agy_data_dir: Option<PathBuf>,
    /// agy hết quota thì chờ tới giờ reset rồi tự chạy tiếp thay vì dừng phiên. Cần `agy_data_dir` để đọc giờ
    /// reset từ hội thoại agy vừa ghi.
    pub resume_on_quota: bool,
}

/// PATH hiện tại với `dir` chèn lên đầu.
pub fn path_with(dir: &Path) -> std::ffi::OsString {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(dir.to_path_buf()).chain(std::env::split_paths(&current));
    std::env::join_paths(paths).unwrap_or(current)
}

pub type Sink = Arc<dyn Fn(SessionEvent) + Send + Sync>;

/// Prompt cho agy — đường dẫn tuyệt đối để agent mở thẳng file, không đi search filesystem.
pub fn build_translate_prompt(root: &Path) -> String {
    let workflow = root.join(".agent").join("workflows").join("translate.md");
    format!(
        "Mở file {}{APP_PROMPT_MARK}: dịch tới khi chạm giới hạn chương/phiên hoặc hết hàng đợi thì dừng. Thư mục truyện: {}",
        workflow.display(),
        root.display()
    )
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LockFile {
    pid: u32,
    started_at: u64,
}

fn lock_path(root: &Path) -> PathBuf {
    root.join("work").join(".session.lock")
}

fn pid_alive(pid: u32) -> bool {
    if cfg!(windows) {
        quiet_command("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\"")))
            .unwrap_or(false)
    } else {
        quiet_command("kill").args(["-0", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false)
    }
}

/// Giữ lock cho một folder truyện; lock mồ côi (PID chết) thì tự dọn.
fn acquire_lock(root: &Path) -> Result<()> {
    let path = lock_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(CoreError::io(parent))?;
    }
    if path.exists() {
        let existing: Option<LockFile> =
            std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok());
        if let Some(lock) = existing {
            // PID còn sống (kể cả chính tiến trình này — phiên trước chưa dừng) → khoá.
            if pid_alive(lock.pid) {
                return Err(CoreError::SessionLocked { pid: lock.pid });
            }
        }
    }
    let lock = LockFile { pid: std::process::id(), started_at: now_ms() };
    std::fs::write(&path, serde_json::to_string(&lock).unwrap()).map_err(CoreError::io(&path))
}

fn release_lock(root: &Path) {
    let _ = std::fs::remove_file(lock_path(root));
}

fn kill_tree(child: &mut Child) {
    if cfg!(windows) {
        let _ = quiet_command("taskkill").args(["/PID", &child.id().to_string(), "/T", "/F"]).output();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// agy 1.2 mặc định `--print-timeout 5m`: một lượt `agy -p` dịch nhiều chương (chaptersPerSession) dễ quá
/// 5 phút → agy cắt giữa chừng, in "print timeout ... returning partial output", chương đang dịch bỏ dở.
/// Nới lên 3h; giới hạn thật vẫn là max_sessions và nút Dừng (kill_tree).
pub const AGY_PRINT_TIMEOUT: &str = "3h";

fn spawn_agy(config: &SessionConfig, prompt: &str) -> Result<Child> {
    let mut command = quiet_command(&config.agy);
    command.arg("-p").arg(prompt).arg("--dangerously-skip-permissions").arg("--print-timeout").arg(AGY_PRINT_TIMEOUT);
    if let Some(model) = &config.model {
        command.arg("--model").arg(model);
    }
    if let Some(dir) = &config.extra_path {
        command.env("PATH", path_with(dir));
    }
    command.current_dir(&config.root).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    command.spawn().map_err(|_| CoreError::AgyMissing)
}

fn pump<R: std::io::Read + Send + 'static>(reader: R, stream: LogStream, sink: Sink) -> JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(std::result::Result::ok) {
            sink(SessionEvent::AgyLog { line, stream });
        }
    })
}

/// Chạy một lượt agy tới khi xong (hoặc cancel), stream log, poll tiến độ.
/// Trả `Some(exit code)` khi agy tự thoát, `None` khi bị cancel.
fn run_child(
    config: &SessionConfig,
    prompt: &str,
    sink: &Sink,
    cancel: Option<&AtomicBool>,
    child_slot: Option<&Mutex<Option<u32>>>,
) -> Result<Option<i32>> {
    let mut child = spawn_agy(config, prompt)?;
    if let Some(slot) = child_slot {
        *slot.lock().unwrap() = Some(child.id());
    }
    let out = pump(child.stdout.take().unwrap(), LogStream::Stdout, sink.clone());
    let err = pump(child.stderr.take().unwrap(), LogStream::Stderr, sink.clone());
    let mut last = read_progress(&config.root).ok();
    let code = loop {
        if let Some(status) = child.try_wait().map_err(|e| CoreError::Internal(e.to_string()))? {
            break Some(status.code().unwrap_or(-1));
        }
        if cancel.is_some_and(|flag| flag.load(Ordering::SeqCst)) {
            kill_tree(&mut child);
            break None;
        }
        thread::sleep(config.poll_interval);
        if let Ok(now) = read_progress(&config.root) {
            if last.as_ref() != Some(&now) {
                sink(SessionEvent::Progress(now.clone()));
                last = Some(now);
            }
        }
    };
    let _ = out.join();
    let _ = err.join();
    if let Some(slot) = child_slot {
        *slot.lock().unwrap() = None;
    }
    if let Ok(now) = read_progress(&config.root) {
        if last.as_ref() != Some(&now) {
            sink(SessionEvent::Progress(now));
        }
    }
    Ok(code)
}

/// Mỗi lượt `agy -p` để lại một hội thoại vài MB trong thư mục dữ liệu của agy và agy không tự xoá — dọn hội
/// thoại của truyện này ngay khi agy thoát. Dọn không được thì thôi, không phải lý do dừng dịch.
fn sweep_conversations(config: &SessionConfig, sink: &Sink) {
    let Some(data_dir) = &config.agy_data_dir else { return };
    let report = clean(data_dir, Scope::Story(&config.root));
    if report.removed > 0 {
        let line = format!(
            "[qt-ai] Đã dọn {} hội thoại agy của truyện này ({:.1} MB).",
            report.removed,
            report.freed_bytes as f64 / 1_048_576.0
        );
        sink(SessionEvent::AgyLog { line, stream: LogStream::Stdout });
    }
}

/// Một lượt agy đồng bộ với prompt tuỳ ý (dùng cho "AI điền hồ sơ"). Trả exit code của agy.
/// `run_child` cần `Sink` (Arc + Send) vì thread đọc stdout/stderr; gom event qua channel rồi
/// phát lại cho `sink` của caller sau khi lượt chạy xong.
pub fn run_once(config: &SessionConfig, prompt: &str, sink: &dyn Fn(SessionEvent)) -> Result<i32> {
    let (tx, rx) = std::sync::mpsc::channel::<SessionEvent>();
    let forward: Sink = Arc::new(move |event| {
        let _ = tx.send(event);
    });
    let code = run_child(config, prompt, &forward, None, None);
    sweep_conversations(config, &forward);
    let code = code?;
    drop(forward); // đóng channel để vòng for dưới kết thúc
    for event in rx {
        sink(event);
    }
    Ok(code.unwrap_or(-1))
}

pub struct SessionHandle {
    cancel: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<StopReason>>,
}

impl SessionHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn join(mut self) -> StopReason {
        self.thread
            .take()
            .map(|t| {
                t.join().unwrap_or_else(|_| StopReason::Internal { message: "thread runner panic".to_string() })
            })
            .unwrap_or(StopReason::Internal { message: "đã join".to_string() })
    }
}

/// Số lần `poll_interval` chờ thêm sau giờ quota reset (2s × 30 = 1 phút): đồng hồ máy lệch hay máy chủ mở
/// quota trễ vài giây thì lượt chạy lại không đâm ngay vào lỗi cũ.
const QUOTA_MARGIN_POLLS: u32 = 30;

fn describe_wait(ms: u64) -> String {
    let minutes = ms.div_ceil(60_000);
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{} phút", minutes.max(1)),
        (hours, 0) => format!("{hours} giờ"),
        (hours, minutes) => format!("{hours} giờ {minutes} phút"),
    }
}

/// Chờ tới giờ quota reset (cộng biên an toàn) rồi cho vòng phiên chạy tiếp. false = người dùng bấm Dừng.
fn wait_for_quota(config: &SessionConfig, sink: &Sink, cancel: &AtomicBool, reset_at: u64) -> bool {
    let margin = (config.poll_interval * QUOTA_MARGIN_POLLS).as_millis() as u64;
    let until = reset_at.max(now_ms()) + margin;
    sink(SessionEvent::QuotaWait { until });
    let line = format!("[qt-ai] Hết quota agy — chờ khoảng {} rồi tự dịch tiếp.", describe_wait(until - now_ms()));
    sink(SessionEvent::AgyLog { line, stream: LogStream::Stdout });
    while now_ms() < until {
        if cancel.load(Ordering::SeqCst) {
            return false;
        }
        thread::sleep(config.poll_interval.min(Duration::from_secs(1)));
    }
    true
}

fn session_loop(config: SessionConfig, sink: Sink, cancel: Arc<AtomicBool>) -> StopReason {
    let child_slot = Mutex::new(None);
    let mut consecutive_failures = 0;
    let mut session_no = 0;
    loop {
        if session_no >= config.max_sessions {
            return StopReason::MaxSessions;
        }
        session_no += 1;
        let before = match read_progress(&config.root) {
            Ok(p) => p,
            Err(e) => return StopReason::Internal { message: e.to_string() },
        };
        if before.queued == 0 && before.translating == 0 {
            return StopReason::Finished;
        }
        if cancel.load(Ordering::SeqCst) {
            return StopReason::UserCancelled;
        }
        sink(SessionEvent::Started { session_no });
        let prompt = build_translate_prompt(&config.root);
        let round_started = SystemTime::now();
        let outcome = run_child(&config, &prompt, &sink, Some(&cancel), Some(&child_slot));
        // Giờ quota reset nằm trong hội thoại agy vừa ghi — đọc trước khi dọn hội thoại đó.
        let quota_reset =
            config.agy_data_dir.as_deref().and_then(|dir| quota_reset_since(dir, &config.root, round_started));
        sweep_conversations(&config, &sink);
        let code = match outcome {
            Ok(Some(code)) => code,
            Ok(None) => return StopReason::UserCancelled,
            Err(e) => return StopReason::Internal { message: e.to_string() },
        };
        let after = match read_progress(&config.root) {
            Ok(p) => p,
            Err(e) => return StopReason::Internal { message: e.to_string() },
        };
        let progressed = after.settled() > before.settled();
        // Lượt hỏng (agy lỗi hoặc không chốt thêm chương nào) vì hết quota: không phải lỗi cần dừng để người
        // dùng xem. Lượt đó không tính vào giới hạn số phiên, cũng không tính là một lần agy lỗi.
        if let Some(reset_at) = quota_reset.filter(|_| code != 0 || !progressed) {
            if !config.resume_on_quota {
                return StopReason::QuotaExhausted { reset_at };
            }
            if !wait_for_quota(&config, &sink, &cancel, reset_at) {
                return StopReason::UserCancelled;
            }
            session_no -= 1;
            consecutive_failures = 0;
            continue;
        }
        if code != 0 {
            consecutive_failures += 1;
            if consecutive_failures >= 2 {
                return StopReason::AgyFailed { code };
            }
            continue; // lần đầu lỗi (rate limit nhất thời?) → thử thêm một phiên
        }
        consecutive_failures = 0;
        if !progressed {
            return StopReason::NoProgress;
        }
    }
}

/// Chạy `run` trên thread riêng dưới lock `work/.session.lock`; panic → Internal; luôn phát Stopped.
/// Dùng chung cho vòng agy và vòng API.
pub fn spawn_runner(
    root: PathBuf,
    sink: Sink,
    run: impl FnOnce(Arc<AtomicBool>) -> StopReason + Send + 'static,
) -> Result<SessionHandle> {
    acquire_lock(&root)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let thread_cancel = cancel.clone();
    let thread_running = running.clone();
    let thread = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(thread_cancel)));
        let reason = result.unwrap_or_else(|_| StopReason::Internal { message: "runner panic".to_string() });
        release_lock(&root);
        sink(SessionEvent::Stopped(reason.clone()));
        thread_running.store(false, Ordering::SeqCst);
        reason
    });
    Ok(SessionHandle { cancel, running, thread: Some(thread) })
}

/// Bắt đầu vòng phiên agy trên thread riêng. Giữ lock `work/.session.lock` tới khi dừng.
pub fn start_session(config: SessionConfig, sink: Sink) -> Result<SessionHandle> {
    let root = config.root.clone();
    let loop_sink = sink.clone();
    spawn_runner(root, sink, move |cancel| session_loop(config, loop_sink, cancel))
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn path_with_chen_folder_len_dau_va_giu_phan_cu() {
        let dir = std::env::temp_dir();
        let joined = path_with(&dir);
        let mut parts = std::env::split_paths(&joined);
        assert_eq!(parts.next().as_deref(), Some(dir.as_path()));
        let old: Vec<_> = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        assert_eq!(parts.collect::<Vec<_>>(), old);
    }
}
