use crate::error::{CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::agy::find_agy;
use qt_ai_core::session::{run_once, start_session, SessionConfig, SessionEvent, Sink};
use qt_ai_core::story::StoryConfig;
use qt_ai_core::story_fs::{load_story_config, save_story_config, story_paths};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

pub const SESSION_EVENT: &str = "session-event";

#[derive(Debug, Clone, Serialize)]
pub struct SessionStatus {
    pub running: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFillResult {
    pub before: StoryConfig,
    pub after: StoryConfig,
    pub exit_code: i32,
    pub log: Vec<String>,
}

pub fn session_config(root: &Path, agy: PathBuf, model: Option<String>, max_sessions: u32) -> SessionConfig {
    SessionConfig { root: root.to_path_buf(), agy, model, max_sessions, poll_interval: Duration::from_secs(2) }
}

fn status(state: &State<'_, AppState>) -> SessionStatus {
    SessionStatus { running: state.session.lock().unwrap().as_ref().is_some_and(|h| h.is_running()) }
}

fn resolve_agy(state: &State<'_, AppState>) -> CmdResult<PathBuf> {
    let configured = state.config.lock().unwrap().agy_path.clone();
    Ok(find_agy(configured.as_deref().map(Path::new))?)
}

#[tauri::command]
pub fn session_state(state: State<'_, AppState>) -> CmdResult<SessionStatus> {
    Ok(status(&state))
}

/// Bắt đầu vòng phiên; event phát lên UI qua `session-event`. Đang chạy rồi thì từ chối.
#[tauri::command]
pub fn session_start(
    app: AppHandle,
    state: State<'_, AppState>,
    root: String,
    model: Option<String>,
) -> CmdResult<SessionStatus> {
    if status(&state).running {
        return Err(CommandError::new("session_locked", "Đang có phiên dịch chạy — bấm Dừng trước."));
    }
    let agy = resolve_agy(&state)?;
    let max_sessions = state.config.lock().unwrap().max_sessions;
    let config = session_config(Path::new(&root), agy, model, max_sessions);
    let sink: Sink = Arc::new(move |event: SessionEvent| {
        let _ = app.emit(SESSION_EVENT, &event);
    });
    let handle = start_session(config, sink)?;
    *state.session.lock().unwrap() = Some(handle);
    Ok(status(&state))
}

/// Dừng: cancel (core giết process tree agy) rồi đợi thread runner kết thúc.
#[tauri::command]
pub fn session_stop(state: State<'_, AppState>) -> CmdResult<SessionStatus> {
    let handle = state.session.lock().unwrap().take();
    if let Some(handle) = handle {
        handle.cancel();
        let _ = handle.join();
    }
    Ok(status(&state))
}

pub fn build_setup_prompt(root: &Path, name: &str, source_url: &str) -> String {
    let workflow = root.join(".agent").join("workflows").join("setup-story.md");
    format!(
        "Mở file {} (đường dẫn tuyệt đối, tồn tại sẵn, KHÔNG cần tìm kiếm) rồi làm đúng theo nó với input: tên truyện tiếng Việt = \"{}\", link truyện tiếng Trung = \"{}\". KHÔNG hỏi lại người dùng; ghi kết quả thẳng vào story.json trong thư mục truyện {} (bỏ bước trình duyệt — giao diện sẽ hiện diff cho người dùng duyệt).",
        workflow.display(),
        name,
        source_url,
        root.display()
    )
}

/// Chạy agy một lượt để điền story.json, rồi KHÔI PHỤC bản trước: UI hiện diff, người dùng Áp dụng
/// bằng `save_story(after)`. Tên + link được ghi tạm vào story.json để agent thấy.
#[tauri::command]
pub fn ai_fill_story(
    state: State<'_, AppState>,
    root: String,
    name: String,
    source_url: String,
) -> CmdResult<AiFillResult> {
    if status(&state).running {
        return Err(CommandError::new(
            "session_locked",
            "Đang có phiên dịch chạy — bấm Dừng trước khi AI điền hồ sơ.",
        ));
    }
    let root_path = Path::new(&root);
    let paths = story_paths(root_path);
    let before = load_story_config(&paths)?;
    let mut seeded = before.clone();
    seeded.name = name.clone();
    seeded.source_url = source_url.clone();
    save_story_config(&paths, &seeded)?;

    let agy = resolve_agy(&state)?;
    let model = state.config.lock().unwrap().model.clone();
    let config = session_config(root_path, agy, model, 1);
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_sink = log.clone();
    let outcome = run_once(&config, &build_setup_prompt(root_path, &name, &source_url), &move |event| {
        if let SessionEvent::AgyLog { line, .. } = event {
            log_sink.lock().unwrap().push(line);
        }
    });
    // Đọc kết quả agent viết, rồi trả story.json về bản trước dù agy lỗi hay không.
    let after = load_story_config(&paths).unwrap_or_else(|_| seeded.clone());
    save_story_config(&paths, &before)?;
    let exit_code = outcome?;
    let log = log.lock().unwrap().clone();
    Ok(AiFillResult { before, after, exit_code, log })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_prompt_dua_ten_link_va_duong_dan_tuyet_doi() {
        let prompt = build_setup_prompt(Path::new("D:\\truyen"), "Kỳ Chiêu Nguyệt", "https://x/y");
        assert!(prompt.contains("setup-story.md") && prompt.contains("KHÔNG cần tìm kiếm"));
        assert!(prompt.contains("Kỳ Chiêu Nguyệt") && prompt.contains("https://x/y"));
        assert!(prompt.contains("KHÔNG hỏi lại") && prompt.contains("story.json"));
    }

    #[test]
    fn session_config_lay_max_sessions_tu_app_config_va_poll_2s() {
        let config = session_config(Path::new("D:\\truyen"), PathBuf::from("agy.exe"), Some("m".into()), 7);
        assert_eq!(config.max_sessions, 7);
        assert_eq!(config.poll_interval, Duration::from_secs(2));
        assert_eq!(config.model.as_deref(), Some("m"));
    }
}
