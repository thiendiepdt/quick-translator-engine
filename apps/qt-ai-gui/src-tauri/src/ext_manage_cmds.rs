//! Cài và quản lý extension (spec mục 11): danh sách, cài từ file zip hoặc link (kiểm gói → hộp xin quyền → cài),
//! bật/tắt, kiểm tra cập nhật (cài từ link), gỡ, nạp thư mục dev, chạy thử `toc` / `chapter`.
//! Gói vừa kiểm nằm ở `.staging/<sha256>/`; nguồn cài nhớ theo sha256 tới khi người dùng bấm Cài.

use crate::error::{blocking, CmdResult, CommandError};
use crate::ext_cmds::{find, store_error, store_for};
use crate::raw_cmds::{extensions_dir, DeveloperView};
use crate::AppState;
use base64::Engine as _;
use qt_ai_core::raw_download::registry::{Inspection, Registry, Source};
use qt_ai_core::raw_download::repository::Repositories;
use qt_ai_core::raw_download::run::{test_chapter, test_toc, RunOptions, TestReport};
use qt_ext::{fetch_zip, Package, Permission, TocItem};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{AppHandle, Manager, Runtime};

/// sha256 gói đã kiểm → nguồn cài.
static PENDING: LazyLock<Mutex<HashMap<String, Source>>> = LazyLock::new(Default::default);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtListItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub developer: Option<DeveloperView>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub hosts: Vec<String>,
    pub permissions: Vec<String>,
    pub auth_required: bool,
    /// Có auth hoặc cài đặt để mở hộp "Auth và cài đặt".
    pub configurable: bool,
    pub enabled: bool,
    pub dev: bool,
    pub source: Source,
    pub installed_at: i64,
    pub updated_at: i64,
    /// `data:image/png;base64,…`
    pub icon: Option<String>,
    /// Gói nạp lỗi (manifest hỏng, thư mục dev mất…).
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionView {
    pub sha256: String,
    pub id: String,
    pub name: String,
    pub version: String,
    /// Bản đang cài (cập nhật).
    pub current_version: Option<String>,
    pub description: String,
    pub developer: DeveloperView,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub hosts: Vec<String>,
    pub new_hosts: Vec<String>,
    pub permissions: Vec<String>,
    pub new_permissions: Vec<String>,
    pub auth_required: bool,
    pub newer: bool,
    pub needs_consent: bool,
    pub icon: Option<String>,
    /// Cài từ kho mà kho cho dùng API key chung: tên kho.
    pub shared_auth_repo: Option<String>,
}

fn permissions_of(package: &Package) -> Vec<String> {
    package.manifest.permissions.iter().map(|p| match p {
        Permission::Webview => "webview".to_string(),
    }).collect()
}

fn icon_of(package: &Package) -> Option<String> {
    package.icon.as_ref().map(|bytes| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn developer_of(package: &Package) -> DeveloperView {
    let d = &package.manifest.developer;
    DeveloperView { name: d.name.clone(), url: d.url.clone(), email: d.email.clone() }
}

fn registry<R: Runtime>(app: &AppHandle<R>) -> Registry {
    Registry::new(&extensions_dir(&app.state::<AppState>().config_path))
}

fn list_view(registry: &Registry) -> Vec<ExtListItem> {
    registry
        .list_all()
        .into_iter()
        .map(|(entry, loaded)| match loaded {
            Ok(ext) => {
                let m = &ext.package.manifest;
                ExtListItem {
                    id: entry.id.clone(),
                    name: m.name.clone(),
                    version: m.version.to_string(),
                    description: m.description.clone(),
                    developer: Some(developer_of(&ext.package)),
                    license: m.license.clone(),
                    homepage: m.homepage.clone(),
                    hosts: m.hosts.clone(),
                    permissions: permissions_of(&ext.package),
                    auth_required: m.auth.as_ref().is_some_and(|a| a.required),
                    configurable: m.auth.is_some() || !m.settings.is_empty(),
                    enabled: entry.enabled,
                    dev: ext.dev,
                    source: entry.source,
                    installed_at: entry.installed_at,
                    updated_at: entry.updated_at,
                    icon: icon_of(&ext.package),
                    error: None,
                }
            }
            Err(error) => ExtListItem {
                name: entry.id.clone(),
                id: entry.id,
                version: entry.version,
                description: String::new(),
                developer: None,
                license: None,
                homepage: None,
                hosts: entry.hosts,
                permissions: entry.permissions,
                auth_required: false,
                configurable: false,
                enabled: entry.enabled,
                dev: matches!(entry.source, Source::Dev { .. }),
                source: entry.source,
                installed_at: entry.installed_at,
                updated_at: entry.updated_at,
                icon: None,
                error: Some(error),
            },
        })
        .collect()
}

fn inspection_view(i: &Inspection) -> InspectionView {
    let m = &i.package.manifest;
    InspectionView {
        sha256: i.sha256.clone(),
        id: m.id.clone(),
        name: m.name.clone(),
        version: m.version.to_string(),
        current_version: i.current.as_ref().map(|c| c.version.clone()).filter(|v| !v.is_empty()),
        description: m.description.clone(),
        developer: developer_of(&i.package),
        license: m.license.clone(),
        homepage: m.homepage.clone(),
        hosts: m.hosts.clone(),
        new_hosts: i.new_hosts.clone(),
        permissions: permissions_of(&i.package),
        new_permissions: i.new_permissions.clone(),
        auth_required: m.auth.as_ref().is_some_and(|a| a.required),
        newer: i.newer,
        needs_consent: i.needs_consent,
        icon: icon_of(&i.package),
        shared_auth_repo: None,
    }
}

/// Nhớ nguồn cài của gói vừa kiểm tới khi người dùng bấm Cài.
pub(crate) fn remember(sha256: &str, source: Source) {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).insert(sha256.to_string(), source);
}

/// Kiểm gói của một extension trong kho (chưa cài).
pub(crate) fn inspect_from_repo(registry: &Registry, repos: &Repositories, repo_id: &str, ext_id: &str) -> CmdResult<InspectionView> {
    let (inspection, source) = repos.inspect_entry(registry, repo_id, ext_id).map_err(store_error)?;
    remember(&inspection.sha256, source);
    let mut view = inspection_view(&inspection);
    let entry = repos.entry(repo_id, ext_id).map_err(store_error)?;
    if entry.shared_auth {
        view.shared_auth_repo = Some(repos.list().into_iter().find(|r| r.id == repo_id).map(|r| r.name).unwrap_or_else(|| repo_id.to_string()));
    }
    Ok(view)
}

fn inspect(registry: &Registry, bytes: &[u8], source: Source) -> CmdResult<InspectionView> {
    let inspection = registry.inspect(bytes).map_err(store_error)?;
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).insert(inspection.sha256.clone(), source);
    Ok(inspection_view(&inspection))
}

#[tauri::command]
pub async fn ext_list<R: Runtime>(app: AppHandle<R>) -> CmdResult<Vec<ExtListItem>> {
    blocking(move || Ok(list_view(&registry(&app)))).await
}

/// Kiểm gói zip trên máy (chưa cài): manifest, domain, quyền, so với bản đang cài.
#[tauri::command]
pub async fn ext_inspect_file<R: Runtime>(app: AppHandle<R>, path: String) -> CmdResult<InspectionView> {
    blocking(move || {
        let file = Path::new(&path);
        let size = std::fs::metadata(file).map_err(|e| CommandError::new("not_found", format!("{path}: {e}")))?.len();
        if size > qt_ext::package::MAX_BYTES {
            return Err(CommandError::new("invalid_state", "Gói zip quá 5 MB."));
        }
        let bytes = std::fs::read(file).map_err(|e| CommandError::new("invalid_state", format!("{path}: {e}")))?;
        inspect(&registry(&app), &bytes, Source::File { path: path.clone() })
    })
    .await
}

/// Tải gói từ link https rồi kiểm (chưa cài).
#[tauri::command]
pub async fn ext_inspect_url<R: Runtime>(app: AppHandle<R>, url: String) -> CmdResult<InspectionView> {
    blocking(move || {
        let bytes = fetch_zip(&url).map_err(store_error)?;
        inspect(&registry(&app), &bytes, Source::Link { url: url.trim().to_string() })
    })
    .await
}

/// Cài gói đã kiểm (người dùng đã đồng ý quyền).
#[tauri::command]
pub async fn ext_install<R: Runtime>(app: AppHandle<R>, sha256: String) -> CmdResult<Vec<ExtListItem>> {
    blocking(move || {
        let source = PENDING.lock().unwrap_or_else(|e| e.into_inner()).remove(&sha256).ok_or_else(|| CommandError::new("invalid_state", "Gói đã hết hạn chờ — chọn lại file hoặc link."))?;
        let registry = registry(&app);
        registry.install(&sha256, source).map_err(store_error)?;
        Ok(list_view(&registry))
    })
    .await
}

/// Gỡ: xoá gói, cài đặt, auth, session data, cache, hồ sơ webview. `download.json` của truyện giữ nguyên.
#[tauri::command]
pub async fn ext_uninstall<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<Vec<ExtListItem>> {
    blocking(move || {
        let registry = registry(&app);
        let store = store_for(&app.state::<AppState>().config_path, &id);
        registry.uninstall(&id, &store).map_err(store_error)?;
        Ok(list_view(&registry))
    })
    .await
}

#[tauri::command]
pub async fn ext_set_enabled<R: Runtime>(app: AppHandle<R>, id: String, enabled: bool) -> CmdResult<Vec<ExtListItem>> {
    blocking(move || {
        let registry = registry(&app);
        registry.set_enabled(&id, enabled).map_err(store_error)?;
        Ok(list_view(&registry))
    })
    .await
}

/// Tải lại gói từ link đã cài. `None` = đang là bản mới nhất. Có bản mới thì trả kết quả kiểm để giao diện cài
/// luôn (không thêm quyền) hoặc hỏi lại quyền.
#[tauri::command]
pub async fn ext_check_update<R: Runtime>(app: AppHandle<R>, id: String) -> CmdResult<Option<InspectionView>> {
    blocking(move || {
        let registry = registry(&app);
        let entry = registry.entries().into_iter().find(|e| e.id == id).ok_or_else(|| CommandError::new("not_found", format!("Chưa cài extension {id}.")))?;
        if let Source::Repository { repository, .. } = &entry.source {
            let repos = Repositories::new(&extensions_dir(&app.state::<AppState>().config_path));
            repos.refresh(repository).map_err(store_error)?;
            let latest = repos.entry(repository, &id).map_err(store_error)?;
            if !qt_ai_core::raw_download::repository::is_newer(&latest.version, &entry.version) {
                return Ok(None);
            }
            return inspect_from_repo(&registry, &repos, repository, &id).map(Some);
        }
        let Source::Link { url } = &entry.source else {
            return Err(CommandError::new("invalid_state", "Chỉ kiểm tra cập nhật được với extension cài từ link hoặc từ kho."));
        };
        let bytes = fetch_zip(url).map_err(store_error)?;
        let inspection = registry.inspect(&bytes).map_err(store_error)?;
        if inspection.package.manifest.id != id {
            return Err(CommandError::new("invalid_state", format!("Link giờ trả extension {}, không phải {id}.", inspection.package.manifest.id)));
        }
        if !inspection.newer {
            return Ok(None);
        }
        PENDING.lock().unwrap_or_else(|e| e.into_inner()).insert(inspection.sha256.clone(), entry.source.clone());
        Ok(Some(inspection_view(&inspection)))
    })
    .await
}

/// Chế độ developer: chạy extension từ thư mục đang viết.
#[tauri::command]
pub async fn ext_add_dev<R: Runtime>(app: AppHandle<R>, dir: String) -> CmdResult<Vec<ExtListItem>> {
    blocking(move || {
        let registry = registry(&app);
        registry.add_dev(Path::new(&dir)).map_err(store_error)?;
        Ok(list_view(&registry))
    })
    .await
}

fn test_options<R: Runtime>(app: &AppHandle<R>, id: &str) -> CmdResult<RunOptions> {
    let config_path = &app.state::<AppState>().config_path;
    let installed = find(config_path, id)?;
    let mut opts = RunOptions::new(installed);
    opts.session.store = Some(store_for(config_path, id));
    opts.session.webview = Some(crate::ext_webview::host(app));
    Ok(opts)
}

/// Chạy thử `toc` với một link; không ghi gì vào truyện.
#[tauri::command]
pub async fn ext_test_toc<R: Runtime>(app: AppHandle<R>, id: String, url: String) -> CmdResult<TestReport> {
    blocking(move || Ok(test_toc(&test_options(&app, &id)?, &url, Arc::new(AtomicBool::new(false)))?)).await
}

/// Chạy thử `chapter` cho một mục của mục lục vừa chạy thử.
#[tauri::command]
pub async fn ext_test_chapter<R: Runtime>(app: AppHandle<R>, id: String, url: String, item: TocItem, index: usize) -> CmdResult<TestReport> {
    blocking(move || Ok(test_chapter(&test_options(&app, &id)?, &url, item, index, Arc::new(AtomicBool::new(false)))?)).await
}
