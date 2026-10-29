//! Bảng Tiến trình: theo từng truyện hiện phiên dịch của app, tiến trình agy và khoá phiên `work/.session.lock`;
//! cho Kill / Gỡ khoá bằng tay khi Dừng hay tự gỡ không làm được (agy treo, khoá kẹt, agy mồ côi sau crash).
//! Chỉ đụng agy do app chạy (prompt có `APP_PROMPT_MARK`) — agy người dùng tự chạy trong IDE không hiện, không kill.

use crate::error::{blocking, CmdResult, CommandError};
use crate::session_registry::root_key;
use crate::AppState;
use qt_ai_core::agy_data::APP_PROMPT_MARK;
use qt_ai_core::session::{lock_status, remove_lock};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgyProcess {
    pub pid: u32,
    /// Unix ms.
    pub started_at: u64,
    /// RAM cả cây tiến trình (agy + con cháu), byte.
    pub memory: u64,
    /// CPU cả cây, % của cả máy; lần quét đầu luôn 0 (sysinfo cần hai lần đo).
    pub cpu: f32,
    /// `translate` dịch truyện, `setup` AI điền hồ sơ, `other` lệnh khác của app.
    pub kind: &'static str,
    /// Con trực tiếp của app này; false = mồ côi (app trước crash) hoặc của cửa sổ app khác.
    pub owned: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LockInfo {
    pub pid: u32,
    pub started_at: u64,
    /// `session` phiên đang chạy của app này; `app` app này giữ mà không còn phiên (phiên bị Kill chưa kết thúc);
    /// `other` tiến trình khác còn sống (cửa sổ app khác); `stale` mồ côi — bấm Dịch là tự gỡ.
    pub holder: &'static str,
    pub holder_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoryProcesses {
    pub root: String,
    pub name: String,
    /// App này đang có phiên dịch truyện này (registry).
    pub running: bool,
    pub agy: Vec<AgyProcess>,
    pub lock: Option<LockInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessOverview {
    pub app_pid: u32,
    pub stories: Vec<StoryProcesses>,
    /// agy của app mà không đọc ra truyện nào.
    pub stray: Vec<AgyProcess>,
}

/// Một System dùng lại giữa các lần quét: CPU% cần hai lần đo, dòng lệnh chỉ đọc một lần mỗi tiến trình.
fn system() -> &'static Mutex<System> {
    static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();
    SYSTEM.get_or_init(|| Mutex::new(System::new()))
}

fn refresh(system: &mut System) {
    let kind = ProcessRefreshKind::nothing().with_memory().with_cpu().with_cmd(UpdateKind::OnlyIfNotSet);
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
}

/// Folder truyện và loại lệnh từ prompt app đưa agy: "Mở file <root>\.agent\workflows\<file>.md (…) …".
/// Không có dấu prompt của app → None (agy người dùng tự chạy).
pub fn parse_agy_prompt(command_line: &str) -> Option<(Option<String>, &'static str)> {
    if !command_line.contains(APP_PROMPT_MARK) {
        return None;
    }
    let kind = if command_line.contains("translate.md") {
        "translate"
    } else if command_line.contains("setup-story.md") {
        "setup"
    } else {
        "other"
    };
    let root = command_line.find("Mở file ").and_then(|start| {
        let rest = &command_line[start + "Mở file ".len()..];
        let end = rest.find("\\.agent\\").or_else(|| rest.find("/.agent/"))?;
        Some(rest[..end].trim().to_string())
    });
    Some((root.filter(|root| !root.is_empty()), kind))
}

fn is_agy(name: &str) -> bool {
    let name = name.to_lowercase();
    name == "agy" || name == "agy.exe"
}

/// Pid của tiến trình và mọi con cháu (con trước cha không quan trọng — kill theo thứ tự lá trước).
fn tree(children: &HashMap<Pid, Vec<Pid>>, root: Pid) -> Vec<Pid> {
    let mut out = vec![root];
    let mut index = 0;
    while index < out.len() {
        if let Some(kids) = children.get(&out[index]) {
            out.extend(kids.iter().copied());
        }
        index += 1;
    }
    out
}

fn children_map(system: &System) -> HashMap<Pid, Vec<Pid>> {
    let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
    for (pid, process) in system.processes() {
        if let Some(parent) = process.parent() {
            children.entry(parent).or_default().push(*pid);
        }
    }
    children
}

/// Mọi agy của app trong snapshot, kèm root đọc từ prompt.
fn app_agy(system: &System) -> Vec<(Option<String>, AgyProcess)> {
    let app_pid = Pid::from_u32(std::process::id());
    let children = children_map(system);
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as f32;
    let mut out = vec![];
    for (pid, process) in system.processes() {
        if !is_agy(&process.name().to_string_lossy()) {
            continue;
        }
        let command_line = process.cmd().iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ");
        let Some((root, kind)) = parse_agy_prompt(&command_line) else { continue };
        let members = tree(&children, *pid);
        let (memory, cpu) = members
            .iter()
            .filter_map(|member| system.process(*member))
            .fold((0, 0.0), |(memory, cpu), p| (memory + p.memory(), cpu + p.cpu_usage()));
        out.push((
            root,
            AgyProcess {
                pid: pid.as_u32(),
                started_at: process.start_time() * 1000,
                memory,
                cpu: cpu / cores,
                kind,
                owned: process.parent() == Some(app_pid),
            },
        ));
    }
    out.sort_by_key(|(_, agy)| agy.started_at);
    out
}

/// Root cần soi khoá: thư viện, truyện gần đây, truyện đang chạy.
fn candidate_roots(state: &AppState) -> Vec<String> {
    let (library, recent) = {
        let config = state.config.lock().unwrap();
        (config.library_root.clone(), config.recent.clone())
    };
    let mut roots = recent;
    if let Some(library) = library {
        for entry in std::fs::read_dir(&library).into_iter().flatten().flatten() {
            if entry.path().join("work").join(".session.lock").is_file() {
                roots.push(entry.path().to_string_lossy().into_owned());
            }
        }
    }
    roots.extend(state.sessions.lock().unwrap().running_roots());
    roots
}

fn story_name(root: &str) -> String {
    Path::new(root).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| root.to_string())
}

fn story_entry<'a>(stories: &'a mut BTreeMap<String, StoryProcesses>, root: &str, running: bool) -> &'a mut StoryProcesses {
    stories.entry(root_key(root)).or_insert_with(|| StoryProcesses {
        root: root.to_string(),
        name: story_name(root),
        running,
        agy: vec![],
        lock: None,
    })
}

pub fn overview(state: &AppState) -> ProcessOverview {
    let app_pid = std::process::id();
    let mut system = system().lock().unwrap();
    refresh(&mut system);
    let running = state.sessions.lock().unwrap().running_roots();
    let is_running = |root: &str| running.iter().any(|r| root_key(r) == root_key(root));

    let mut stories: BTreeMap<String, StoryProcesses> = BTreeMap::new();
    let mut stray = vec![];
    for (root, agy) in app_agy(&system) {
        match root {
            Some(root) => story_entry(&mut stories, &root, is_running(&root)).agy.push(agy),
            None => stray.push(agy),
        }
    }
    for root in running.iter().cloned().chain(candidate_roots(state)) {
        let Some(lock) = lock_status(Path::new(&root)) else {
            if is_running(&root) {
                story_entry(&mut stories, &root, true);
            }
            continue;
        };
        let (holder, holder_name) = if !lock.owner_alive {
            ("stale", None)
        } else if lock.pid == app_pid {
            (if is_running(&root) { "session" } else { "app" }, None)
        } else {
            let name = system.process(Pid::from_u32(lock.pid)).map(|p| p.name().to_string_lossy().into_owned());
            ("other", name)
        };
        story_entry(&mut stories, &root, is_running(&root)).lock =
            Some(LockInfo { pid: lock.pid, started_at: lock.started_at, holder, holder_name });
    }
    let mut stories: Vec<_> = stories.into_values().collect();
    stories.sort_by(|a, b| b.running.cmp(&a.running).then_with(|| a.name.cmp(&b.name)));
    ProcessOverview { app_pid, stories, stray }
}

/// Giết cây tiến trình (lá trước). Trả số tiến trình đã giết.
fn kill_tree(system: &System, pid: Pid) -> usize {
    let members = tree(&children_map(system), pid);
    members.iter().rev().filter_map(|member| system.process(*member)).filter(|p| p.kill()).count()
}

#[tauri::command]
pub async fn process_overview<R: Runtime>(app: AppHandle<R>) -> CmdResult<ProcessOverview> {
    blocking(move || Ok(overview(&app.state::<AppState>()))).await
}

/// Kill truyện: huỷ phiên của app (không đợi — Dừng treo mới phải dùng tới đây) rồi giết mọi agy của app đang
/// dịch truyện đó, kể cả agy mồ côi. Thread phiên tự thả khoá khi thoát; kẹt thì người dùng Gỡ khoá.
#[tauri::command]
pub async fn story_kill<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<ProcessOverview> {
    blocking(move || {
        let state = app.state::<AppState>();
        if let Some(handle) = state.sessions.lock().unwrap().take(&root) {
            handle.cancel();
            // Không join: handle rơi ở đây thì thread chạy tách, tự kết thúc khi agy chết.
        }
        {
            let mut system = system().lock().unwrap();
            refresh(&mut system);
            let key = root_key(&root);
            for (agy_root, agy) in app_agy(&system) {
                if agy_root.is_some_and(|r| root_key(&r) == key) {
                    kill_tree(&system, Pid::from_u32(agy.pid));
                }
            }
        }
        Ok(overview(&state))
    })
    .await
}

/// Kill một agy của app theo PID (agy không đọc ra truyện). Từ chối PID không phải agy của app.
#[tauri::command]
pub async fn agy_kill<R: Runtime>(app: AppHandle<R>, pid: u32) -> CmdResult<ProcessOverview> {
    blocking(move || {
        {
            let mut system = system().lock().unwrap();
            refresh(&mut system);
            if !app_agy(&system).iter().any(|(_, agy)| agy.pid == pid) {
                return Err(CommandError::new("invalid_input", format!("PID {pid} không phải agy do app chạy.")));
            }
            kill_tree(&system, Pid::from_u32(pid));
        }
        Ok(overview(&app.state::<AppState>()))
    })
    .await
}

/// Gỡ khoá phiên bằng tay. Từ chối khi app này còn phiên của truyện — bấm Dừng hoặc Kill trước.
#[tauri::command]
pub async fn lock_remove<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<ProcessOverview> {
    blocking(move || {
        let state = app.state::<AppState>();
        if state.sessions.lock().unwrap().is_running(&root) {
            return Err(CommandError::new(
                "session_locked",
                "App đang dịch truyện này — bấm Dừng hoặc Kill trước khi gỡ khoá.",
            ));
        }
        remove_lock(&PathBuf::from(&root))?;
        Ok(overview(&state))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_root_va_loai_lenh_tu_prompt_cua_app() {
        let translate = format!(
            "C:\\agy\\agy.exe -p Mở file D:\\lib\\truyen-a\\.agent\\workflows\\translate.md{APP_PROMPT_MARK}: dịch … Thư mục truyện: D:\\lib\\truyen-a --print-timeout 3h"
        );
        assert_eq!(parse_agy_prompt(&translate), Some((Some("D:\\lib\\truyen-a".to_string()), "translate")));
        let setup = format!("agy -p Mở file /lib/b/.agent/workflows/setup-story.md{APP_PROMPT_MARK} với input");
        assert_eq!(parse_agy_prompt(&setup), Some((Some("/lib/b".to_string()), "setup")));
        // agy người dùng tự chạy trong IDE: không có dấu của app → bỏ qua.
        assert_eq!(parse_agy_prompt("agy -p dịch giúp chương này"), None);
        assert_eq!(parse_agy_prompt(&format!("agy -p x{APP_PROMPT_MARK}")), Some((None, "other")));
    }

    #[test]
    fn cay_tien_trinh_gom_ca_chau() {
        let pid = Pid::from_u32;
        let children = HashMap::from([(pid(1), vec![pid(2), pid(3)]), (pid(3), vec![pid(4)])]);
        assert_eq!(tree(&children, pid(1)), vec![pid(1), pid(2), pid(3), pid(4)]);
        assert_eq!(tree(&children, pid(9)), vec![pid(9)]);
    }
}
