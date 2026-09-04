use crate::app_config::AppConfig;
use crate::error::CmdResult;
use crate::AppState;
use qt_ai_core::agy::{agy_models, agy_version, find_agy};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgyStatus {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub models: Vec<String>,
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
pub fn agy_status(state: State<'_, AppState>, configured: Option<String>) -> CmdResult<AgyStatus> {
    let from_config = state.config.lock().unwrap().agy_path.clone();
    let chosen = configured.or(from_config);
    Ok(probe_agy(chosen.as_deref().map(Path::new)))
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
