//! Runner phiên agy — port auto-translate.ps1: mỗi phiên `agy -p` là một context sạch; poll state.json
//! để báo tiến độ; cầu dao hết queue / không tiến độ / agy lỗi 2 lần / max phiên; cancel giết process tree.

use crate::commands::status::count_chapters;
use crate::error::{CoreError, Result};
use crate::story_fs::{load_state, now_ms, story_paths, ChapterStatus};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    let state = load_state(&story_paths(root))?;
    let counts = count_chapters(&state);
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
    Internal { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    Started { session_no: u32 },
    Progress(Progress),
    AgyLog { line: String, stream: LogStream },
    Stopped(StopReason),
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub root: PathBuf,
    pub agy: PathBuf,
    pub model: Option<String>,
    pub max_sessions: u32,
    pub poll_interval: Duration,
}

pub type Sink = Arc<dyn Fn(SessionEvent) + Send + Sync>;

/// Prompt cho agy — đường dẫn tuyệt đối để agent mở thẳng file, không đi search filesystem.
pub fn build_translate_prompt(root: &Path) -> String {
    let workflow = root.join(".agent").join("workflows").join("translate.md");
    format!(
        "Mở file {} (đường dẫn tuyệt đối, tồn tại sẵn, KHÔNG cần tìm kiếm) rồi làm đúng theo nó: dịch tới khi chạm giới hạn chương/phiên hoặc hết hàng đợi thì dừng. Thư mục truyện: {}",
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
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\"")))
            .unwrap_or(false)
    } else {
        Command::new("kill").args(["-0", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false)
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
        let _ = Command::new("taskkill").args(["/PID", &child.id().to_string(), "/T", "/F"]).output();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn spawn_agy(config: &SessionConfig, prompt: &str) -> Result<Child> {
    let mut command = Command::new(&config.agy);
    command.arg("-p").arg(prompt).arg("--dangerously-skip-permissions");
    if let Some(model) = &config.model {
        command.arg("--model").arg(model);
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

/// Một lượt agy đồng bộ với prompt tuỳ ý (dùng cho "AI điền hồ sơ"). Trả exit code của agy.
/// `run_child` cần `Sink` (Arc + Send) vì thread đọc stdout/stderr; gom event qua channel rồi
/// phát lại cho `sink` của caller sau khi lượt chạy xong.
pub fn run_once(config: &SessionConfig, prompt: &str, sink: &dyn Fn(SessionEvent)) -> Result<i32> {
    let (tx, rx) = std::sync::mpsc::channel::<SessionEvent>();
    let forward: Sink = Arc::new(move |event| {
        let _ = tx.send(event);
    });
    let code = run_child(config, prompt, &forward, None, None)?;
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

fn session_loop(config: SessionConfig, sink: Sink, cancel: Arc<AtomicBool>) -> StopReason {
    let child_slot = Mutex::new(None);
    let mut consecutive_failures = 0;
    for session_no in 1..=config.max_sessions {
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
        let code = match run_child(&config, &prompt, &sink, Some(&cancel), Some(&child_slot)) {
            Ok(Some(code)) => code,
            Ok(None) => return StopReason::UserCancelled,
            Err(e) => return StopReason::Internal { message: e.to_string() },
        };
        if code != 0 {
            consecutive_failures += 1;
            if consecutive_failures >= 2 {
                return StopReason::AgyFailed { code };
            }
            continue; // lần đầu lỗi (rate limit nhất thời?) → thử thêm một phiên
        }
        consecutive_failures = 0;
        let after = match read_progress(&config.root) {
            Ok(p) => p,
            Err(e) => return StopReason::Internal { message: e.to_string() },
        };
        if after.settled() <= before.settled() {
            return StopReason::NoProgress;
        }
    }
    StopReason::MaxSessions
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
