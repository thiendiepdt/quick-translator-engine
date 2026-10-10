use crate::app_config::AppConfig;
use crate::error::{blocking, CmdResult};
use crate::AppState;
use qt_ai_core::agy::{agy_models, agy_version, find_agy, AgyModel};
use qt_ai_core::agy_data::{self, CleanReport, DataStats, Scope};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgyStatus {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub models: Vec<AgyModel>,
    pub message: Option<String>,
}

pub fn probe_agy(configured: Option<&Path>) -> AgyStatus {
    let path: PathBuf = match find_agy(configured) {
        Ok(path) => path,
        Err(error) => {
            return AgyStatus { found: false, path: None, version: None, models: vec![], message: Some(error.to_string()) }
        }
    };
    let version = agy_version(&path).ok();
    let models = agy_models(&path).unwrap_or_default();
    AgyStatus { found: true, path: Some(path.display().to_string()), version, models, message: None }
}

/// Không truyền `configured` thì dùng agy_path trong config app.
#[tauri::command]
pub async fn agy_status<R: Runtime>(app: AppHandle<R>, configured: Option<String>) -> CmdResult<AgyStatus> {
    blocking(move || {
        let state = app.state::<AppState>();
        let from_config = state.config.lock().unwrap().agy_path.clone();
        let chosen = configured.or(from_config);
        Ok(probe_agy(chosen.as_deref().map(Path::new)))
    })
    .await
}

/// Hội thoại agy đang nằm trên đĩa. `data_dir` None = máy chưa từng chạy agy CLI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgyDataStats {
    pub data_dir: Option<String>,
    #[serde(flatten)]
    pub stats: DataStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgyDataCleaned {
    #[serde(flatten)]
    pub report: CleanReport,
    /// Số liệu sau khi dọn.
    pub stats: AgyDataStats,
}

fn data_stats() -> AgyDataStats {
    match agy_data::default_data_dir() {
        Some(dir) => AgyDataStats { stats: agy_data::stats(&dir), data_dir: Some(dir.display().to_string()) },
        None => AgyDataStats { data_dir: None, stats: DataStats::default() },
    }
}

/// Đếm hội thoại agy do app sinh ra (và phần còn lại) — mở từng file hội thoại nên vài trăm cái mất vài giây.
#[tauri::command]
pub async fn agy_data_stats() -> CmdResult<AgyDataStats> {
    blocking(|| Ok(data_stats())).await
}

/// Xoá hội thoại agy do app sinh ra, trừ của truyện đang có phiên chạy và hội thoại vừa ghi (xem
/// `agy_data::Scope::All`). Phiên dịch tự dọn truyện của nó sau mỗi lượt; lệnh này dọn phần tồn từ trước.
#[tauri::command]
pub async fn agy_data_clean<R: Runtime>(app: AppHandle<R>) -> CmdResult<AgyDataCleaned> {
    blocking(move || {
        let state = app.state::<AppState>();
        let busy_roots: Vec<PathBuf> = state.sessions.lock().unwrap().running_roots().into_iter().map(PathBuf::from).collect();
        let report = match agy_data::default_data_dir() {
            Some(dir) => agy_data::clean(&dir, Scope::All { busy_roots: &busy_roots }),
            None => CleanReport::default(),
        };
        Ok(AgyDataCleaned { report, stats: data_stats() })
    })
    .await
}

#[tauri::command]
pub fn app_config_get(state: State<'_, AppState>) -> CmdResult<AppConfig> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn app_config_set(state: State<'_, AppState>, config: AppConfig) -> CmdResult<AppConfig> {
    config.save(&state.config_path)?;
    *state.config.lock().unwrap() = config.clone();
    Ok(config)
}
