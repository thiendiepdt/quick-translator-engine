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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Lỗi thoáng qua (agy lỗi 2 lần, lượt không chốt được chương, không đọc được state.json) trong chế độ treo
    /// qua đêm: chờ tới `until` rồi thử lại, lần `attempt`/`max`. `reason` là lý do lẽ ra đã dừng.
    RetryWait { until: u64, attempt: u32, max: u32, reason: StopReason },
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
    /// Lịch chờ trước mỗi lần thử lại khi phiên gặp lỗi thoáng qua; rỗng = dừng ngay như trước. App đặt
    /// `TRANSIENT_RETRY_WAITS` khi bật tự dịch tiếp (treo qua đêm).
    pub retry_waits: Vec<Duration>,
}

/// Treo qua đêm: agy lỗi, lượt không chốt được chương hay state.json bị khoá thường là sự cố thoáng qua (mạng chập,
/// server Antigravity lỗi tạm, máy ngủ dậy, Google Drive / diệt virus khoá file). Chờ 5 → 15 → 30 → 60 → 60 → 60
/// phút (tổng ~3,8 giờ) rồi mới dừng hẳn; mỗi lần thử tốn tối đa một lượt quota.
pub const TRANSIENT_RETRY_WAITS: [Duration; 6] = [
    Duration::from_secs(5 * 60),
    Duration::from_secs(15 * 60),
    Duration::from_secs(30 * 60),
    Duration::from_secs(60 * 60),
    Duration::from_secs(60 * 60),
    Duration::from_secs(60 * 60),
];

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

/// Chủ khoá còn sống không. Chỉ hỏi "PID còn sống" là không đủ: app tắt ngang (tắt máy, crash, End task) để lại
/// khoá, Windows cấp lại PID đó cho tiến trình khác rất nhanh (nhất là sau khởi động lại) → truyện bị chặn "đang
/// có phiên khác chạy" tới khi tiến trình lạ kia thoát. Chủ khoá phải sinh ra trước lúc ghi khoá; tiến trình đang
/// giữ PID mà sinh sau là kẻ dùng lại PID → khoá mồ côi.
fn lock_owner_alive(lock: &LockFile) -> bool {
    #[cfg(windows)]
    {
        windows_process::born_ms(lock.pid).is_some_and(|born| born <= lock.started_at + LOCK_CLOCK_SLACK_MS)
    }
    #[cfg(not(windows))]
    {
        quiet_command("kill").args(["-0", &lock.pid.to_string()]).status().map(|s| s.success()).unwrap_or(false)
    }
}

/// Sai số giữa giờ tạo tiến trình (FILETIME) và giờ ghi khoá (SystemTime) — chủ khoá luôn sinh trước vài trăm ms.
#[cfg(windows)]
const LOCK_CLOCK_SLACK_MS: u64 = 2_000;

#[cfg(windows)]
mod windows_process {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    /// FILETIME (100ns từ 1601) của mốc Unix epoch.
    const UNIX_EPOCH_FILETIME: u64 = 116_444_736_000_000_000;

    /// Giờ tạo (Unix ms) của tiến trình đang sống mang `pid`; None = không có tiến trình đó hoặc không mở được.
    /// Không mở được (tiến trình hệ thống của user khác) cũng là None: app cùng user luôn mở được QUERY_LIMITED,
    /// nên đó không thể là chủ khoá.
    pub fn born_ms(pid: u32) -> Option<u64> {
        // SAFETY: handle mở rồi đóng trong hàm; các struct ra là FILETIME khởi tạo zero.
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return None;
            }
            let mut code = 0u32;
            let alive = GetExitCodeProcess(handle, &mut code) != 0 && code == STILL_ACTIVE as u32;
            let [mut created, mut exited, mut kernel, mut user]: [FILETIME; 4] = std::mem::zeroed();
            let timed = GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) != 0;
            CloseHandle(handle);
            if !alive || !timed {
                return None;
            }
            let ticks = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
            Some(ticks.saturating_sub(UNIX_EPOCH_FILETIME) / 10_000)
        }
    }
}

/// Giữ lock cho một folder truyện; lock mồ côi (PID chết) thì tự dọn. Trả `started_at` để thả đúng khoá của mình.
fn acquire_lock(root: &Path) -> Result<u64> {
    let path = lock_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(CoreError::io(parent))?;
    }
    if path.exists() {
        if let Some(lock) = read_lock(root) {
            // Chủ khoá còn sống (kể cả chính tiến trình này — phiên trước chưa dừng) → khoá.
            if lock_owner_alive(&lock) {
                return Err(CoreError::SessionLocked { pid: lock.pid });
            }
        }
    }
    let lock = LockFile { pid: std::process::id(), started_at: now_ms() };
    std::fs::write(&path, serde_json::to_string(&lock).unwrap()).map_err(CoreError::io(&path))?;
    Ok(lock.started_at)
}

fn read_lock(root: &Path) -> Option<LockFile> {
    std::fs::read_to_string(lock_path(root)).ok().and_then(|t| serde_json::from_str(&t).ok())
}

/// Chỉ xoá khi file vẫn là khoá của phiên này: phiên bị Kill tay (bảng Tiến trình) có thể kết thúc muộn, lúc
/// người dùng đã gỡ khoá và chạy phiên mới — không được xoá khoá của phiên mới.
fn release_lock(root: &Path, started_at: u64) {
    if read_lock(root).is_some_and(|lock| lock.pid == std::process::id() && lock.started_at == started_at) {
        let _ = std::fs::remove_file(lock_path(root));
    }
}

/// Khoá phiên của một truyện cho bảng Tiến trình.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockStatus {
    pub pid: u32,
    pub started_at: u64,
    /// Chủ khoá còn sống (PID sống và sinh trước lúc ghi khoá); false = khoá mồ côi, phiên mới tự gỡ.
    pub owner_alive: bool,
}

pub fn lock_status(root: &Path) -> Option<LockStatus> {
    let lock = read_lock(root)?;
    Some(LockStatus { pid: lock.pid, started_at: lock.started_at, owner_alive: lock_owner_alive(&lock) })
}

/// Gỡ khoá bằng tay (bảng Tiến trình) khi tự động không gỡ được. Caller phải chắc app không còn phiên của truyện.
pub fn remove_lock(root: &Path) -> Result<()> {
    let path = lock_path(root);
    match std::fs::remove_file(&path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(CoreError::io(&path)(error)),
        _ => Ok(()),
    }
}

fn kill_tree(child: &mut Child) {
    crate::child_guard::kill_tree(child);
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
    // Đóng app / app crash thì agy chết theo, không chạy mồ côi (xem child_guard).
    crate::child_guard::prepare(&mut command);
    let child = command.spawn().map_err(|_| CoreError::AgyMissing)?;
    crate::child_guard::bind(&child);
    Ok(child)
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

/// Đọc tiến độ, hỏng thì thử lại sau 1 → 3 → 6 giây: state.json có thể đang bị Google Drive / diệt virus khoá
/// trong chốc lát — một lần đọc hỏng không đáng dừng cả phiên.
fn read_progress_patient(root: &Path, cancel: &AtomicBool) -> Result<Progress> {
    let mut result = read_progress(root);
    for wait in [1, 3, 6] {
        if result.is_ok() || cancel.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_secs(wait));
        result = read_progress(root);
    }
    result
}

/// Chờ trước lần thử lại thứ `attempt` (đếm từ 1) theo `retry_waits`; hết lịch → None (dừng hẳn với `reason`).
/// Trả `Some(false)` khi người dùng bấm Dừng trong lúc chờ.
fn wait_for_retry(config: &SessionConfig, sink: &Sink, cancel: &AtomicBool, attempt: u32, reason: &StopReason) -> Option<bool> {
    let wait = *config.retry_waits.get(attempt as usize - 1)?;
    let max = config.retry_waits.len() as u32;
    let until = now_ms() + wait.as_millis() as u64;
    sink(SessionEvent::RetryWait { until, attempt, max, reason: reason.clone() });
    let line = format!(
        "[qt-ai] Lỗi tạm ({}) — chờ {} rồi thử lại (lần {attempt}/{max}).",
        reason_brief(reason),
        describe_wait(until - now_ms())
    );
    sink(SessionEvent::AgyLog { line, stream: LogStream::Stdout });
    while now_ms() < until {
        if cancel.load(Ordering::SeqCst) {
            return Some(false);
        }
        thread::sleep(config.poll_interval.min(Duration::from_secs(1)));
    }
    Some(true)
}

fn reason_brief(reason: &StopReason) -> String {
    match reason {
        StopReason::AgyFailed { code } => format!("agy thoát lỗi mã {code} hai lần liền"),
        StopReason::NoProgress => "lượt vừa rồi không chốt được chương nào".to_string(),
        StopReason::Internal { message } => format!("không đọc được tiến độ: {message}"),
        other => format!("{other:?}"),
    }
}

fn session_loop(config: SessionConfig, sink: Sink, cancel: Arc<AtomicBool>) -> StopReason {
    let child_slot = Mutex::new(None);
    let mut consecutive_failures = 0;
    let mut session_no = 0;
    // Số lần đã thử lại vì lỗi thoáng qua kể từ lượt chốt được chương gần nhất.
    let mut retries = 0u32;
    // Lỗi thoáng qua: còn lịch thử lại thì chờ rồi chạy lượt mới (không tính vào giới hạn số phiên), hết thì dừng.
    macro_rules! transient {
        ($reason:expr) => {{
            let reason = $reason;
            retries += 1;
            match wait_for_retry(&config, &sink, &cancel, retries, &reason) {
                None => return reason,
                Some(false) => return StopReason::UserCancelled,
                Some(true) => {
                    session_no -= 1;
                    consecutive_failures = 0;
                    continue;
                }
            }
        }};
    }
    loop {
        if session_no >= config.max_sessions {
            return StopReason::MaxSessions;
        }
        session_no += 1;
        let before = match read_progress_patient(&config.root, &cancel) {
            Ok(p) => p,
            Err(e) => transient!(StopReason::Internal { message: e.to_string() }),
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
        let after = match read_progress_patient(&config.root, &cancel) {
            Ok(p) => p,
            Err(e) => transient!(StopReason::Internal { message: e.to_string() }),
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
                transient!(StopReason::AgyFailed { code });
            }
            continue; // lần đầu lỗi (rate limit nhất thời?) → thử thêm một phiên
        }
        consecutive_failures = 0;
        if !progressed {
            transient!(StopReason::NoProgress);
        }
        retries = 0;
    }
}

/// Nhật ký phiên `work/session-log.jsonl`: mỗi lần dừng và mỗi lần chờ thử lại vì lỗi thoáng qua một dòng, kèm
/// đuôi log agy/API khi bất thường. Treo qua đêm thì toast lý do dừng đã biến mất từ lâu — file này là chỗ xem lại.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    /// ms epoch.
    pub at: u64,
    /// `stopped` hoặc `retry`.
    pub event: String,
    pub reason: StopReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tail: Vec<String>,
}

const JOURNAL_TAIL: usize = 40;
/// Quá cỡ này thì chỉ giữ `JOURNAL_KEEP` dòng cuối.
const JOURNAL_MAX_BYTES: u64 = 512 * 1024;
const JOURNAL_KEEP: usize = 200;

fn journal_path(root: &Path) -> PathBuf {
    root.join("work").join("session-log.jsonl")
}

/// Ghi hỏng thì thôi — nhật ký không phải lý do làm hỏng phiên.
fn journal_append(root: &Path, entry: &JournalEntry) {
    use std::io::Write;
    let path = journal_path(root);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > JOURNAL_MAX_BYTES) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            let lines: Vec<&str> = text.lines().collect();
            let kept = lines[lines.len().saturating_sub(JOURNAL_KEEP)..].join("
");
            let _ = std::fs::write(&path, kept + "
");
        }
    }
    let Ok(line) = serde_json::to_string(entry) else { return };
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
}

/// Lần dừng gần nhất ghi trong nhật ký (để hiện lý do sau khi mở lại app).
pub fn last_stop(root: &Path) -> Option<JournalEntry> {
    let text = std::fs::read_to_string(journal_path(root)).ok()?;
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<JournalEntry>(line).ok())
        .find(|entry| entry.event == "stopped")
}

fn abnormal(reason: &StopReason) -> bool {
    !matches!(reason, StopReason::Finished | StopReason::UserCancelled)
}

/// Chạy `run` trên thread riêng dưới lock `work/.session.lock`; panic → Internal; luôn phát Stopped.
/// Dùng chung cho vòng agy và vòng API. `run` nhận sink đã bọc nhật ký phiên.
pub fn spawn_runner(
    root: PathBuf,
    sink: Sink,
    run: impl FnOnce(Sink, Arc<AtomicBool>) -> StopReason + Send + 'static,
) -> Result<SessionHandle> {
    let lock_started = acquire_lock(&root)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let thread_cancel = cancel.clone();
    let thread_running = running.clone();
    let tail: Arc<Mutex<std::collections::VecDeque<String>>> = Arc::default();
    let journaled: Sink = {
        let (tail, root, inner) = (tail.clone(), root.clone(), sink.clone());
        Arc::new(move |event: SessionEvent| {
            match &event {
                SessionEvent::AgyLog { line, .. } => {
                    let mut tail = tail.lock().unwrap();
                    tail.push_back(line.clone());
                    if tail.len() > JOURNAL_TAIL {
                        tail.pop_front();
                    }
                }
                SessionEvent::RetryWait { attempt, reason, .. } => {
                    let tail = tail.lock().unwrap().iter().cloned().collect();
                    let entry = JournalEntry {
                        at: now_ms(),
                        event: "retry".to_string(),
                        reason: reason.clone(),
                        attempt: Some(*attempt),
                        tail,
                    };
                    journal_append(&root, &entry);
                }
                _ => {}
            }
            inner(event)
        })
    };
    let thread = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(journaled, thread_cancel)));
        let reason = result.unwrap_or_else(|_| StopReason::Internal { message: "runner panic".to_string() });
        let tail = if abnormal(&reason) { tail.lock().unwrap().iter().cloned().collect() } else { vec![] };
        journal_append(&root, &JournalEntry { at: now_ms(), event: "stopped".to_string(), reason: reason.clone(), attempt: None, tail });
        release_lock(&root, lock_started);
        sink(SessionEvent::Stopped(reason.clone()));
        thread_running.store(false, Ordering::SeqCst);
        reason
    });
    Ok(SessionHandle { cancel, running, thread: Some(thread) })
}

/// Bắt đầu vòng phiên agy trên thread riêng. Giữ lock `work/.session.lock` tới khi dừng.
pub fn start_session(config: SessionConfig, sink: Sink) -> Result<SessionHandle> {
    let root = config.root.clone();
    spawn_runner(root, sink, move |sink, cancel| session_loop(config, sink, cancel))
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

#[cfg(test)]
mod lock_tests {
    use super::*;

    #[test]
    fn tha_khoa_chi_xoa_khoa_cua_minh_va_go_tay_duoc() {
        let dir = tempfile::tempdir().unwrap();
        let started = acquire_lock(dir.path()).unwrap();
        let status = lock_status(dir.path()).unwrap();
        assert_eq!((status.pid, status.started_at, status.owner_alive), (std::process::id(), started, true));
        // Phiên cũ bị Kill tay, người dùng gỡ khoá rồi chạy phiên mới; phiên cũ kết thúc muộn không xoá khoá mới.
        remove_lock(dir.path()).unwrap();
        assert!(lock_status(dir.path()).is_none());
        let newer = LockFile { pid: std::process::id(), started_at: started + 1 };
        std::fs::write(lock_path(dir.path()), serde_json::to_string(&newer).unwrap()).unwrap();
        release_lock(dir.path(), started);
        assert!(lock_status(dir.path()).is_some());
        release_lock(dir.path(), started + 1);
        assert!(lock_status(dir.path()).is_none());
        remove_lock(dir.path()).unwrap();
    }
}
