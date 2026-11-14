//! Auth, cài đặt, phiên đăng nhập và cache của extension (spec mục 7, 11). Giá trị secret không bao giờ gửi lên
//! giao diện: form chỉ biết trường nào đã nhập; ô secret để trống khi lưu = giữ giá trị cũ. Dữ liệu nằm ở
//! `<app_config_dir>/extensions/<id>/settings.json`, `<app_config_dir>/extension-data/<id>/` và Credential Manager.

use crate::error::{blocking, CmdResult, CommandError};
use crate::raw_cmds::extensions_dir;
use crate::AppState;
use qt_ai_core::raw_download::registry::{Installed, Registry};
use qt_ai_core::raw_download::run::{login, RunOptions};
use qt_ai_core::raw_download::repository::Repositories;
use qt_ai_core::raw_download::DownloadConfig;
use qt_ext::{check_setting, AuthSpec, AuthState, ExtError, ExtStore, FieldKind, Manifest, Scope, SessionStatus, SettingField, SettingKind};
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};

/// Thư mục cấu hình của app (cha của `config.json`).
pub fn config_root(config_path: &Path) -> PathBuf {
    config_path.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Dữ liệu của extension; cài từ kho và được kho cho dùng chung thì auth là auth chung của kho.
pub fn store_for(config_path: &Path, id: &str) -> ExtStore {
    Repositories::new(&extensions_dir(config_path)).ext_store(&config_root(config_path), id)
}

pub fn store_error(error: ExtError) -> CommandError {
    CommandError::new("invalid_state", error.to_string())
}

/// Extension đã cài theo id, kể cả đang tắt (trang quản lý mở Auth / Chạy thử được).
pub(crate) fn find(config_path: &Path, id: &str) -> CmdResult<Installed> {
    let (_, loaded) = Registry::new(&extensions_dir(config_path))
        .list_all()
        .into_iter()
        .find(|(entry, _)| entry.id == id)
        .ok_or_else(|| CommandError::new("not_found", format!("Chưa cài extension {id}.")))?;
    loaded.map_err(|error| CommandError::new("invalid_state", format!("Extension {id} nạp lỗi: {error}")))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldView {
    pub key: String,
    pub label: String,
    /// "text" | "secret"
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemeView {
    pub id: String,
    pub label: String,
    pub fields: Vec<FieldView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthView {
    pub required: bool,
    /// Theo thứ tự ưu tiên của nhà phát triển.
    pub schemes: Vec<SchemeView>,
    /// Kiểu đã lưu (`None` = chưa nhập).
    pub scheme: Option<String>,
    /// Giá trị các trường text đã lưu.
    pub values: Map<String, Value>,
    /// Khoá các trường đã nhập (text và secret).
    pub filled: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingView {
    pub key: String,
    pub label: String,
    /// "text" | "number" | "boolean" | "select" | "secret"
    pub kind: &'static str,
    pub choices: Vec<String>,
    pub default: Value,
    /// "global" | "story"
    pub scope: &'static str,
    /// Giá trị đã lưu; `null` = chưa lưu (dùng mặc định) hoặc trường secret.
    pub value: Value,
    /// Trường secret đã nhập.
    pub filled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtConfigView {
    pub id: String,
    pub name: String,
    pub auth: Option<AuthView>,
    /// Đang dùng auth chung của kho: auth chỉ xem ở đây, sửa ở phần kho.
    pub shared_auth: Option<SharedAuthView>,
    /// Cài đặt chung (trường `scope: "story"` sửa ở trang Tải raw).
    pub settings: Vec<SettingView>,
    pub session: SessionStatus,
    /// "keyring" = Credential Manager; khác thì khoá đang nằm trong file.
    pub secret_store: &'static str,
    pub cache_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedAuthView {
    pub repository: String,
    pub name: String,
}

/// Form auth vẽ từ spec (của manifest hoặc của kho) và trạng thái đã nhập.
pub fn auth_view(spec: &AuthSpec, state: AuthState) -> AuthView {
    AuthView {
        required: spec.required,
        schemes: spec
            .schemes
            .iter()
            .map(|s| SchemeView {
                id: s.id.clone(),
                label: s.label.clone(),
                fields: s.fields.iter().map(|f| FieldView { key: f.key.clone(), label: f.label.clone(), kind: if f.kind == FieldKind::Secret { "secret" } else { "text" } }).collect(),
            })
            .collect(),
        scheme: state.scheme,
        values: state.text,
        filled: state.filled,
    }
}

fn setting_kind(kind: SettingKind) -> &'static str {
    match kind {
        SettingKind::Text => "text",
        SettingKind::Number => "number",
        SettingKind::Boolean => "boolean",
        SettingKind::Select => "select",
        SettingKind::Secret => "secret",
    }
}

fn setting_view(field: &SettingField, value: Value, filled: bool) -> SettingView {
    SettingView {
        key: field.key.clone(),
        label: field.label.clone(),
        kind: setting_kind(field.kind),
        choices: field.choices.clone(),
        default: field.default.clone().unwrap_or(Value::Null),
        scope: if field.scope == Scope::Story { "story" } else { "global" },
        value,
        filled,
    }
}

pub fn config_view(installed: &Installed, store: &ExtStore) -> CmdResult<ExtConfigView> {
    let m: &Manifest = &installed.package.manifest;
    let auth = match store.auth_spec(m) {
        None => None,
        Some(spec) => Some(auth_view(spec, store.auth_state(m).map_err(store_error)?)),
    };
    let shared_auth = store.shared_auth().and_then(|a| a.repository_id()).map(|repository| {
        let name = config_root_repos(store).into_iter().find(|r| r.id == repository).map(|r| r.name).unwrap_or_else(|| repository.to_string());
        SharedAuthView { repository: repository.to_string(), name }
    });
    let (values, filled) = store.settings_state(m).map_err(store_error)?;
    let settings = m
        .settings
        .iter()
        .filter(|f| f.scope == Scope::Global)
        .map(|f| setting_view(f, values.get(&f.key).cloned().unwrap_or(Value::Null), filled.contains(&f.key)))
        .collect();
    Ok(ExtConfigView {
        id: m.id.clone(),
        name: m.name.clone(),
        auth,
        shared_auth,
        settings,
        session: store.session_status(),
        secret_store: store.secret_kind(),
        cache_bytes: store.cache_bytes(),
    })
}

/// Các trường `scope: "story"` kèm giá trị đã lưu trong `download.json` của truyện.
pub fn story_settings_view(manifest: &Manifest, download: &DownloadConfig) -> Vec<SettingView> {
    let own = download.settings.get(&manifest.id);
    manifest
        .settings
        .iter()
        .filter(|f| f.scope == Scope::Story)
        .map(|f| {
            let value = own.and_then(|o| o.get(&f.key)).and_then(|v| check_setting(f, v).ok()).unwrap_or(Value::Null);
            setting_view(f, value, false)
        })
        .collect()
}

/// Tên các kho (để ghi "Dùng API key chung của kho <tên>"). Thư mục extension suy từ thư mục dữ liệu của store.
fn config_root_repos(store: &ExtStore) -> Vec<qt_ai_core::raw_download::repository::RepoView> {
    let root = store.data_dir().parent().and_then(Path::parent).map(Path::to_path_buf).unwrap_or_default();
    Repositories::new(&root.join("extensions")).list()
}

fn with_ext<R: Runtime, T>(app: &AppHandle<R>, id: &str, f: impl FnOnce(&Installed, &ExtStore) -> CmdResult<T>) -> CmdResult<T> {
    let state = app.state::<AppState>();
    let installed = find(&state.config_path, id)?;
    let store = store_for(&state.config_path, id);
    f(&installed, &store)
}

#[tauri::command]
pub async fn ext_config<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<ExtConfigView> {
    blocking(move || with_ext(&app, &id, config_view)).await
}

#[tauri::command]
pub async fn ext_save_auth<R: Runtime>(app: AppHandle<R>, id: String, scheme: String, values: Map<String, Value>) -> CmdResult<ExtConfigView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            if store.shared_auth().is_some() {
                return Err(CommandError::new("invalid_state", "Extension này dùng API key chung của kho — sửa ở Cài đặt → Extension tải raw → Kho extension."));
            }
            store.save_auth(&installed.package.manifest, &scheme, &values).map_err(store_error)?;
            config_view(installed, store)
        })
    })
    .await
}

#[tauri::command]
pub async fn ext_clear_auth<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<ExtConfigView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            store.clear_auth(&installed.package.manifest).map_err(store_error)?;
            config_view(installed, store)
        })
    })
    .await
}

#[tauri::command]
pub async fn ext_save_settings<R: Runtime>(app: AppHandle<R>, id: String, values: Map<String, Value>) -> CmdResult<ExtConfigView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            store.save_settings(&installed.package.manifest, &values).map_err(store_error)?;
            config_view(installed, store)
        })
    })
    .await
}

/// Xoá phiên đăng nhập, cookie và session data; auth đã nhập giữ nguyên.
#[tauri::command]
pub async fn ext_logout<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<ExtConfigView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            store.logout().map_err(store_error)?;
            config_view(installed, store)
        })
    })
    .await
}

#[tauri::command]
pub async fn ext_clear_cache<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<ExtConfigView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            store.clear_cache().map_err(store_error)?;
            config_view(installed, store)
        })
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtLoginView {
    pub config: ExtConfigView,
    /// Extension có hàm `login` không (không có thì auth dùng thẳng khi tải).
    pub has_login: bool,
}

/// Gọi `login` của extension ngay. `root`: truyện đang mở (ctx có link và cài đặt của truyện).
#[tauri::command]
pub async fn ext_login<R: Runtime>(app: AppHandle<R>, root: Option<String>, id: String) -> CmdResult<ExtLoginView> {
    blocking(move || {
        with_ext(&app, &id, |installed, store| {
            let mut opts = RunOptions::new(installed.clone());
            opts.session.store = Some(store.clone());
            opts.session.webview = Some(crate::ext_webview::host(&app));
            if let Some(root) = &root {
                crate::raw_cmds::wire_events(&mut opts, &app, root);
            }
            let record = login(root.as_deref().map(Path::new), &opts, Arc::new(AtomicBool::new(false)))?;
            Ok(ExtLoginView { config: config_view(installed, store)?, has_login: record.is_some() })
        })
    })
    .await
}
