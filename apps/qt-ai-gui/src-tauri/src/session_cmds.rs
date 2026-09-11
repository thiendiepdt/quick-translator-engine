use crate::app_config::{ApiSettings, Engine};
use crate::error::{blocking, CmdResult, CommandError};
use crate::sidecar::qt_ai_dir;
use crate::AppState;
use qt_ai_core::agy::find_agy;
use qt_ai_core::api::{ApiConfig, HttpModel, TextModel};
use qt_ai_core::api_fill::{fill_story, sample_chapters, SAMPLE_CHAPTERS};
use qt_ai_core::api_session::{start_api_session, ApiSessionConfig};
use qt_ai_core::session::{run_once, start_session, SessionConfig, SessionEvent, SessionHandle, Sink};
use qt_ai_core::story::StoryConfig;
use qt_ai_core::story_fs::{load_story_config, save_story_config, story_paths};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

pub const SESSION_EVENT: &str = "session-event";

/// Root (đúng chữ lúc start) của mọi truyện đang có phiên chạy.
#[derive(Debug, Clone, Serialize)]
pub struct SessionStatus {
    pub running: Vec<String>,
}

/// Event phiên phát lên UI kèm root để store tách tiến độ/log theo truyện:
/// `{root, type: "progress", ...}`.
#[derive(Debug, Clone, Serialize)]
struct RootedEvent {
    root: String,
    #[serde(flatten)]
    event: SessionEvent,
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
    SessionConfig {
        root: root.to_path_buf(),
        agy,
        model,
        max_sessions,
        poll_interval: Duration::from_secs(2),
        extra_path: qt_ai_dir(),
    }
}

/// Động cơ API: thiếu key là lỗi cấu hình rõ ràng trước khi đụng tới folder truyện.
pub fn resolve_api(api: &ApiSettings) -> CmdResult<ApiConfig> {
    let config = api.resolve();
    if config.api_key.is_empty() {
        return Err(CommandError::new(
            "api_key_missing",
            format!("Chưa nhập API key {} — vào Cài đặt → Động cơ dịch.", config.provider.label()),
        ));
    }
    Ok(config)
}

pub fn api_session_config(root: &Path) -> ApiSessionConfig {
    ApiSessionConfig { root: root.to_path_buf(), retry_delay: Duration::from_secs(5) }
}

fn status(state: &State<'_, AppState>) -> SessionStatus {
    SessionStatus { running: state.sessions.lock().unwrap().running_roots() }
}

fn session_running(state: &State<'_, AppState>, root: &str) -> bool {
    state.sessions.lock().unwrap().is_running(root)
}

fn resolve_agy(state: &State<'_, AppState>) -> CmdResult<PathBuf> {
    let configured = state.config.lock().unwrap().agy_path.clone();
    Ok(find_agy(configured.as_deref().map(Path::new))?)
}

#[tauri::command]
pub fn session_state(state: State<'_, AppState>) -> CmdResult<SessionStatus> {
    Ok(status(&state))
}

/// Bắt đầu vòng phiên cho một truyện theo động cơ trong config (agy hoặc API key); event phát lên UI
/// qua `session-event` kèm root. Truyện đang chạy hoặc đã đủ `max_parallel` truyện thì từ chối.
/// `model` chỉ áp dụng cho agy.
#[tauri::command]
pub async fn session_start<R: Runtime>(app: AppHandle<R>, root: String, model: Option<String>) -> CmdResult<SessionStatus> {
    blocking(move || {
        let state = app.state::<AppState>();
        let (engine, api, max_sessions, max_parallel) = {
            let config = state.config.lock().unwrap();
            (config.engine, config.api.clone(), config.max_sessions, config.max_parallel as usize)
        };
        state.sessions.lock().unwrap().check_can_start(&root, max_parallel)?;
        let event_root = root.clone();
        let emitter = app.clone();
        let sink: Sink = Arc::new(move |event: SessionEvent| {
            let _ = emitter.emit(SESSION_EVENT, &RootedEvent { root: event_root.clone(), event });
        });
        let handle: SessionHandle = match engine {
            Engine::Agy => {
                let agy = resolve_agy(&state)?;
                start_session(session_config(Path::new(&root), agy, model, max_sessions), sink)?
            }
            Engine::Api => {
                let model = Arc::new(HttpModel::new(resolve_api(&api)?));
                start_api_session(api_session_config(Path::new(&root)), model, sink)?
            }
        };
        state.sessions.lock().unwrap().insert(&root, Box::new(handle));
        Ok(status(&state))
    })
    .await
}

/// Dừng phiên của một truyện: cancel (core giết process tree agy) rồi đợi thread runner kết thúc,
/// ngoài lock để truyện khác không bị chặn.
#[tauri::command]
pub async fn session_stop<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<SessionStatus> {
    blocking(move || {
        let state = app.state::<AppState>();
        let handle = state.sessions.lock().unwrap().take(&root);
        if let Some(handle) = handle {
            handle.cancel();
            let _ = handle.join();
        }
        Ok(status(&state))
    })
    .await
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

/// AI điền hồ sơ theo động cơ đang chọn. agy: chạy một lượt để agent điền story.json rồi KHÔI PHỤC
/// bản trước; API key: model đọc 3 chương đầu, không đụng đĩa. Cả hai chỉ trả before/after — UI hiện
/// diff, người dùng Áp dụng bằng `save_story(after)`.
#[tauri::command]
pub async fn ai_fill_story<R: Runtime>(app: AppHandle<R>, root: String, name: String, source_url: String) -> CmdResult<AiFillResult> {
    blocking(move || {
        let state = app.state::<AppState>();
        if session_running(&state, &root) {
            return Err(CommandError::new(
                "session_locked",
                "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi AI điền hồ sơ.",
            ));
        }
        let (engine, api) = {
            let config = state.config.lock().unwrap();
            (config.engine, config.api.clone())
        };
        match engine {
            Engine::Api => {
                fill_story_via_api(Path::new(&root), &HttpModel::new(resolve_api(&api)?), &name, &source_url)
            }
            Engine::Agy => fill_story_via_agy(&state, Path::new(&root), &name, &source_url),
        }
    })
    .await
}

/// Đường API: đọc `SAMPLE_CHAPTERS` chương đầu, một lượt `complete_json`, merge vào hồ sơ hiện tại.
pub fn fill_story_via_api(root: &Path, model: &dyn TextModel, name: &str, source_url: &str) -> CmdResult<AiFillResult> {
    let paths = story_paths(root);
    let before = load_story_config(&paths)?;
    let samples = sample_chapters(&paths)?;
    let mut log = vec![format!(
        "Đọc {} chương đầu trong raw/ ({}), gửi {}…",
        samples.len(),
        samples.iter().map(|s| s.id.as_str()).collect::<Vec<_>>().join(", "),
        model.label()
    )];
    if samples.is_empty() {
        log.push(format!("raw/ chưa có chương nào — model chỉ có tên + link (tối đa {SAMPLE_CHAPTERS} chương được đọc)."));
    }
    let after = fill_story(model, &before, name, source_url, &samples)
        .map_err(|error| CommandError::new("api_failed", error.to_string()))?;
    log.push("Model đã trả hồ sơ. Không tra web — kiểm tra lại tên nhân vật và tóm tắt trước khi Áp dụng.".to_string());
    Ok(AiFillResult { before, after, exit_code: 0, log })
}

/// Đường agy: tên + link ghi tạm vào story.json để agent thấy, chạy xong trả file về bản trước.
fn fill_story_via_agy(state: &State<'_, AppState>, root_path: &Path, name: &str, source_url: &str) -> CmdResult<AiFillResult> {
    let paths = story_paths(root_path);
    let before = load_story_config(&paths)?;
    let mut seeded = before.clone();
    seeded.name = name.to_string();
    seeded.source_url = source_url.to_string();
    save_story_config(&paths, &seeded)?;

    let agy = resolve_agy(state)?;
    let model = state.config.lock().unwrap().model.clone();
    let config = session_config(root_path, agy, model, 1);
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_sink = log.clone();
    let outcome = run_once(&config, &build_setup_prompt(root_path, name, source_url), &move |event| {
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
    use std::sync::atomic::AtomicBool;

    #[test]
    fn setup_prompt_dua_ten_link_va_duong_dan_tuyet_doi() {
        let prompt = build_setup_prompt(Path::new("D:\\truyen"), "Kỳ Chiêu Nguyệt", "https://x/y");
        assert!(prompt.contains("setup-story.md") && prompt.contains("KHÔNG cần tìm kiếm"));
        assert!(prompt.contains("Kỳ Chiêu Nguyệt") && prompt.contains("https://x/y"));
        assert!(prompt.contains("KHÔNG hỏi lại") && prompt.contains("story.json"));
    }

    struct FakeModel(String);

    impl TextModel for FakeModel {
        fn label(&self) -> String {
            "Fake".into()
        }
        fn generate(
            &self,
            _: &str,
            _: &str,
            _: qt_ai_core::api::Effort,
            _: &std::sync::atomic::AtomicBool,
            _: &mut dyn FnMut(usize),
        ) -> Result<qt_ai_core::api::Generated, qt_ai_core::api::ApiError> {
            unreachable!()
        }
        fn complete_json(
            &self,
            _: &str,
            user: &str,
            _: qt_ai_core::api::Effort,
            _: &AtomicBool,
        ) -> Result<qt_ai_core::api::Generated, qt_ai_core::api::ApiError> {
            assert!(user.contains("第一章"), "prompt phải kèm chương raw");
            Ok(qt_ai_core::api::Generated::text(self.0.clone()))
        }
    }

    #[test]
    fn fill_via_api_khong_ghi_dia_tra_before_after_va_log() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("raw")).unwrap();
        std::fs::write(dir.path().join("raw").join("0001.txt"), "第一章 赵静文").unwrap();
        qt_ai_core::commands::init::run_init(dir.path(), "qt-ai").unwrap();
        let model = FakeModel(r#"{"protagonist":"Triệu Tĩnh Văn","genre":{"setting":"modern","names":"han"}}"#.into());
        let result = fill_story_via_api(dir.path(), &model, "Tên", "https://src").unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.before.protagonist, "");
        assert_eq!(result.after.protagonist, "Triệu Tĩnh Văn");
        assert_eq!(result.after.name, "Tên");
        assert!(result.log[0].contains("0001") && result.log[0].contains("Fake"));
        // story.json trên đĩa vẫn là bản trước — người dùng phải bấm Áp dụng.
        let on_disk = load_story_config(&story_paths(dir.path())).unwrap();
        assert_eq!(on_disk, result.before);
        let bad = FakeModel("không phải json".into());
        let error = fill_story_via_api(dir.path(), &bad, "Tên", "").unwrap_err();
        assert_eq!(error.kind, "api_failed");
    }

    #[test]
    fn resolve_api_thieu_key_bao_api_key_missing_co_ten_provider() {
        let mut api = ApiSettings::default();
        let error = resolve_api(&api).unwrap_err();
        assert_eq!(error.kind, "api_key_missing");
        assert!(error.message.contains("Gemini"));
        api.gemini.api_key = "AIza".into();
        let config = resolve_api(&api).unwrap();
        assert_eq!(config.model, "gemini-3.7-flash");
        assert_eq!(api_session_config(Path::new("D:\\truyen")).retry_delay, Duration::from_secs(5));
    }

    #[test]
    fn session_config_lay_max_sessions_tu_app_config_va_poll_2s() {
        let config = session_config(Path::new("D:\\truyen"), PathBuf::from("agy.exe"), Some("m".into()), 7);
        assert_eq!(config.max_sessions, 7);
        assert_eq!(config.poll_interval, Duration::from_secs(2));
        assert_eq!(config.model.as_deref(), Some("m"));
    }
}
