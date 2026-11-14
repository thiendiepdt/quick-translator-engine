//! Kho extension (spec 2026-11-14-kho-extension-design): thêm kho bằng link `index.json`, làm mới, gỡ, API key
//! chung của kho, kiểm gói một extension trong kho để cài (rồi `ext_install` như cài từ file).

use crate::error::{blocking, CmdResult, CommandError};
use crate::ext_cmds::{auth_view, config_root, store_error, AuthView};
use crate::ext_manage_cmds::{inspect_from_repo, InspectionView};
use crate::raw_cmds::extensions_dir;
use crate::AppState;
use qt_ai_core::raw_download::registry::Registry;
use qt_ai_core::raw_download::repository::{RepoView, Repositories};
use qt_ext::AuthStore;
use serde::Serialize;
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, Runtime};

fn repos<R: Runtime>(app: &AppHandle<R>) -> Repositories {
    Repositories::new(&extensions_dir(&app.state::<AppState>().config_path))
}

/// Kho cho giao diện, kèm việc đã nhập API key chung chưa (không gửi giá trị key).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoItem {
    #[serde(flatten)]
    pub repo: RepoView,
    pub auth_filled: bool,
}

fn items<R: Runtime>(app: &AppHandle<R>, repos: &Repositories) -> Vec<RepoItem> {
    let root = config_root(&app.state::<AppState>().config_path);
    repos
        .list()
        .into_iter()
        .map(|repo| {
            let auth_filled = repo.has_auth && repos.repo_auth(&root, &repo.id).is_some_and(|a| matches!(a.values(None), Ok(Some(_))));
            RepoItem { repo, auth_filled }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoRefreshView {
    pub repositories: Vec<RepoItem>,
    /// Kho làm mới lỗi ("<id>: <lỗi>"); bản cũ vẫn giữ.
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoAuthView {
    pub id: String,
    pub name: String,
    pub auth: AuthView,
    pub secret_store: &'static str,
}

#[tauri::command]
pub async fn repo_list<R: Runtime>(app: AppHandle<R>) -> CmdResult<Vec<RepoItem>> {
    blocking(move || Ok(items(&app, &repos(&app)))).await
}

/// Thêm kho từ link `index.json` (hoặc thư mục chứa nó). Chỉ đọc danh sách, chưa cài gì.
#[tauri::command]
pub async fn repo_add<R: Runtime>(app: AppHandle<R>, url: String) -> CmdResult<Vec<RepoItem>> {
    blocking(move || {
        let repos = repos(&app);
        repos.add(&url).map_err(store_error)?;
        Ok(items(&app, &repos))
    })
    .await
}

/// Làm mới một kho (`id`) hoặc mọi kho.
#[tauri::command]
pub async fn repo_refresh<R: Runtime>(app: AppHandle<R>, id: Option<String>) -> CmdResult<RepoRefreshView> {
    blocking(move || {
        let repos = repos(&app);
        let errors = match id {
            Some(id) => repos.refresh(&id).err().map(|e| vec![format!("{id}: {e}")]).unwrap_or_default(),
            None => repos.refresh_all(),
        };
        Ok(RepoRefreshView { repositories: items(&app, &repos), errors })
    })
    .await
}

/// Gỡ kho: xoá khỏi danh sách và xoá API key chung; extension đã cài từ kho giữ nguyên.
#[tauri::command]
pub async fn repo_remove<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<Vec<RepoItem>> {
    blocking(move || {
        let repos = repos(&app);
        repos.remove(&id, &config_root(&app.state::<AppState>().config_path)).map_err(store_error)?;
        Ok(items(&app, &repos))
    })
    .await
}

fn with_auth<R: Runtime, T>(app: &AppHandle<R>, id: &str, f: impl FnOnce(&AuthStore) -> CmdResult<T>) -> CmdResult<T> {
    let repos = repos(app);
    let store = repos
        .repo_auth(&config_root(&app.state::<AppState>().config_path), id)
        .ok_or_else(|| CommandError::new("invalid_state", format!("Kho {id} không có API key chung.")))?;
    f(&store)
}

fn repo_auth_view<R: Runtime>(app: &AppHandle<R>, id: &str, store: &AuthStore) -> CmdResult<RepoAuthView> {
    let spec = store.spec(None).ok_or_else(|| CommandError::new("invalid_state", "Kho không khai auth."))?;
    let name = repos(app).list().into_iter().find(|r| r.id == id).map(|r| r.name).unwrap_or_else(|| id.to_string());
    Ok(RepoAuthView { id: id.to_string(), name, auth: auth_view(spec, store.state(None).map_err(store_error)?), secret_store: store.secret_kind() })
}

#[tauri::command]
pub async fn repo_auth<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<RepoAuthView> {
    blocking(move || with_auth(&app, &id, |store| repo_auth_view(&app, &id, store))).await
}

/// Lưu API key chung của kho; trường secret để trống = giữ. Đổi key thì phiên cũ của mọi extension dùng chung mất.
#[tauri::command]
pub async fn repo_save_auth<R: Runtime>(app: AppHandle<R>, id: String, scheme: String, values: Map<String, Value>) -> CmdResult<RepoAuthView> {
    blocking(move || {
        with_auth(&app, &id, |store| {
            store.save(None, &scheme, &values).map_err(store_error)?;
            repo_auth_view(&app, &id, store)
        })
    })
    .await
}

#[tauri::command]
pub async fn repo_clear_auth<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<RepoAuthView> {
    blocking(move || {
        with_auth(&app, &id, |store| {
            store.clear(None).map_err(store_error)?;
            repo_auth_view(&app, &id, store)
        })
    })
    .await
}

/// Tải gói một extension trong kho, kiểm sha256 và manifest, chờ người dùng đồng ý quyền (rồi `ext_install`).
#[tauri::command]
pub async fn repo_inspect<R: Runtime>(app: AppHandle<R>, repo_id: String, ext_id: String) -> CmdResult<InspectionView> {
    blocking(move || {
        let registry = Registry::new(&extensions_dir(&app.state::<AppState>().config_path));
        inspect_from_repo(&registry, &repos(&app), &repo_id, &ext_id)
    })
    .await
}
