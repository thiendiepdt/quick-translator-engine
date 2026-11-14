//! Dữ liệu của một extension phía app (spec mục 7), nằm trong thư mục cấu hình, không nằm trong thư mục truyện:
//!
//! ```text
//! <config>/extensions/<id>/settings.json      cài đặt chung, trừ trường secret
//! <config>/extension-data/<id>/auth.json      kiểu auth đã chọn, trường text, mã phiên bản auth
//! <config>/extension-data/<id>/session.json   session data, phiên login, hũ cookie
//! <config>/extension-data/<id>/cache.sqlite   cache
//! kho khoá: qt-ext/<id>/auth/<kiểu>/<trường>, qt-ext/<id>/setting/<khoá>
//! ```
//!
//! Đổi auth thì sinh mã phiên bản mới; session data ghi mã lúc tạo, lệch thì coi như trống. Không lưu hash của
//! khoá ra đĩa và vẫn đúng khi một lượt tải cũ còn đang ghi session.

pub mod auth;
pub mod cache;
pub mod secrets;
pub mod session_data;

use crate::manifest::{check_setting, AuthSpec, Manifest, Scope, SettingKind};
use crate::ExtError;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub use auth::{AuthState, AuthStore, AuthValues};
pub use cache::Cache;
pub use secrets::SecretBackend;
pub use session_data::{LoginRecord, SessionData, SessionStatus};

pub(crate) fn io_error(path: &Path) -> impl Fn(std::io::Error) -> ExtError + '_ {
    move |e| ExtError::Store(format!("{}: {e}", path.display()))
}

/// Ghi file tạm cạnh đích rồi đổi tên.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ExtError> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_error(dir))?;
    }
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = path.with_file_name(format!("{name}.{}-{}.tmp", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&tmp, bytes).map_err(io_error(&tmp))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io_error(path)(e)
    })
}

pub(crate) fn remove_if_exists(path: &Path) -> Result<(), ExtError> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io_error(path)(e)),
        _ => Ok(()),
    }
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

pub(crate) fn random_hex() -> String {
    let mut bytes = [0u8; 8];
    if getrandom::fill(&mut bytes).is_err() {
        return format!("{:016x}", now_ms());
    }
    hex::encode(bytes)
}

pub(crate) fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct ExtStore {
    id: String,
    settings_file: PathBuf,
    data_dir: PathBuf,
    secrets: SecretBackend,
    /// Auth riêng, hoặc auth chung của kho (`with_shared_auth`).
    auth: AuthStore,
}

impl ExtStore {
    /// `config_root`: thư mục cấu hình của app (chứa `extensions/`).
    pub fn new(config_root: &Path, id: &str) -> ExtStore {
        let backend = SecretBackend::platform(config_root.join("extension-data").join(id).join("secrets.json"));
        ExtStore::with_backend(config_root, id, backend)
    }

    pub fn with_backend(config_root: &Path, id: &str, secrets: SecretBackend) -> ExtStore {
        let data_dir = config_root.join("extension-data").join(id);
        ExtStore {
            id: id.to_string(),
            settings_file: config_root.join("extensions").join(id).join("settings.json"),
            auth: AuthStore::extension(&data_dir, id, secrets.clone()),
            data_dir,
            secrets,
        }
    }

    /// Dùng auth chung của kho thay cho auth riêng (extension cài từ kho, được kho cho dùng chung).
    pub fn with_shared_auth(mut self, auth: AuthStore) -> ExtStore {
        self.auth = auth;
        self
    }

    /// Auth chung của kho đang dùng, nếu có.
    pub fn shared_auth(&self) -> Option<&AuthStore> {
        self.auth.repository_id().map(|_| &self.auth)
    }

    /// Spec auth áp dụng: của kho nếu dùng chung, không thì của manifest.
    pub fn auth_spec<'a>(&'a self, manifest: &'a Manifest) -> Option<&'a AuthSpec> {
        self.auth.spec(manifest.auth.as_ref())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn session_path(&self) -> PathBuf {
        self.data_dir.join("session.json")
    }

    pub fn cache_path(&self) -> PathBuf {
        self.data_dir.join("cache.sqlite")
    }

    /// Hồ sơ trình duyệt của webview (cookie giữ giữa các lần mở).
    pub fn webview_dir(&self) -> PathBuf {
        self.data_dir.join("webview")
    }

    pub fn secret_kind(&self) -> &'static str {
        self.secrets.kind()
    }

    fn setting_name(&self, key: &str) -> String {
        format!("qt-ext/{}/setting/{key}", self.id)
    }

    /// Mã phiên bản auth hiện tại ("" = chưa có auth).
    pub fn auth_version(&self) -> String {
        self.auth.version()
    }

    /// Auth đầy đủ của kiểu đã chọn; `None` khi chưa nhập gì hoặc kiểu không còn trong spec.
    pub fn auth(&self, manifest: &Manifest) -> Result<Option<AuthValues>, ExtError> {
        self.auth.values(manifest.auth.as_ref())
    }

    pub fn auth_state(&self, manifest: &Manifest) -> Result<AuthState, ExtError> {
        self.auth.state(manifest.auth.as_ref())
    }

    /// Lưu auth của một kiểu. Trường secret để trống thì giữ giá trị cũ; khoá của các kiểu khác bị xoá. Có gì đổi
    /// thì sinh mã phiên bản mới (session data cũ thành trống).
    pub fn save_auth(&self, manifest: &Manifest, scheme_id: &str, values: &Map<String, Value>) -> Result<(), ExtError> {
        self.auth.save(manifest.auth.as_ref(), scheme_id, values)
    }

    /// Xoá auth riêng (mọi khoá, `auth.json`) và session data. Đang dùng auth chung của kho thì chỉ xoá session
    /// data của extension này; auth của kho sửa ở phần kho.
    pub fn clear_auth(&self, manifest: &Manifest) -> Result<(), ExtError> {
        if self.shared_auth().is_none() {
            self.auth.clear(manifest.auth.as_ref())?;
        }
        self.logout()
    }

    fn load_settings_file(&self) -> Map<String, Value> {
        std::fs::read_to_string(&self.settings_file).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    /// Cài đặt chung đã lưu (kể cả secret); trường chưa lưu không có mặt (dùng `default` của manifest).
    pub fn settings(&self, manifest: &Manifest) -> Result<Map<String, Value>, ExtError> {
        let file = self.load_settings_file();
        let mut out = Map::new();
        for field in manifest.settings.iter().filter(|f| f.scope == Scope::Global) {
            let value = match field.kind {
                SettingKind::Secret => self.secrets.get(&self.setting_name(&field.key))?.map(Value::String),
                _ => file.get(&field.key).and_then(|v| check_setting(field, v).ok()),
            };
            if let Some(value) = value {
                out.insert(field.key.clone(), value);
            }
        }
        Ok(out)
    }

    /// Cho giao diện: giá trị trường không bí mật, và khoá các trường secret đã nhập.
    pub fn settings_state(&self, manifest: &Manifest) -> Result<(Map<String, Value>, Vec<String>), ExtError> {
        let mut values = self.settings(manifest)?;
        let mut filled = Vec::new();
        for field in manifest.settings.iter().filter(|f| f.kind == SettingKind::Secret) {
            if values.remove(&field.key).is_some() {
                filled.push(field.key.clone());
            }
        }
        Ok((values, filled))
    }

    /// Lưu cài đặt chung. Khoá không có trong `values` thì giữ nguyên; `null` = về mặc định. Trường secret: chuỗi
    /// rỗng = giữ, `null` = xoá. Giá trị sai kiểu thì không lưu gì.
    pub fn save_settings(&self, manifest: &Manifest, values: &Map<String, Value>) -> Result<(), ExtError> {
        let current = self.load_settings_file();
        let mut file = Map::new();
        let mut secret_ops: Vec<(String, Option<String>)> = Vec::new();
        for field in manifest.settings.iter().filter(|f| f.scope == Scope::Global) {
            let given = values.get(&field.key);
            if field.kind == SettingKind::Secret {
                match given {
                    Some(Value::Null) => secret_ops.push((self.setting_name(&field.key), None)),
                    Some(value) => {
                        let value = check_setting(field, value).map_err(ExtError::Store)?;
                        let text = as_text(&value);
                        if !text.is_empty() {
                            secret_ops.push((self.setting_name(&field.key), Some(text)));
                        }
                    }
                    None => {}
                }
                continue;
            }
            match given {
                None => {
                    if let Some(value) = current.get(&field.key).and_then(|v| check_setting(field, v).ok()) {
                        file.insert(field.key.clone(), value);
                    }
                }
                Some(Value::Null) => {}
                Some(value) => {
                    file.insert(field.key.clone(), check_setting(field, value).map_err(ExtError::Store)?);
                }
            }
        }
        for (name, value) in secret_ops {
            match value {
                Some(value) => self.secrets.set(&name, &value)?,
                None => self.secrets.delete(&name)?,
            }
        }
        let json = serde_json::to_string_pretty(&file).map_err(|e| ExtError::Store(e.to_string()))?;
        write_atomic(&self.settings_file, json.as_bytes())
    }

    /// Xoá session data (phiên login, cookie, giá trị `session.set`) và hồ sơ webview.
    pub fn logout(&self) -> Result<(), ExtError> {
        remove_if_exists(&self.session_path())?;
        let dir = self.webview_dir();
        match std::fs::remove_dir_all(&dir) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(ExtError::Store(format!("không xoá được hồ sơ webview ({e}) — đóng cửa sổ của extension rồi thử lại")))
            }
            _ => Ok(()),
        }
    }

    /// Gỡ extension: xoá khoá (auth và cài đặt secret), `settings.json`, toàn bộ thư mục dữ liệu (auth.json,
    /// session, cache, hồ sơ webview).
    pub fn remove_all(&self, manifest: &Manifest) -> Result<(), ExtError> {
        // Auth riêng thì xoá; auth chung của kho để nguyên cho các extension khác của kho.
        AuthStore::extension(&self.data_dir, &self.id, self.secrets.clone()).clear(manifest.auth.as_ref())?;
        self.logout()?;
        for field in manifest.settings.iter().filter(|f| f.kind == SettingKind::Secret) {
            self.secrets.delete(&self.setting_name(&field.key))?;
        }
        remove_if_exists(&self.settings_file)?;
        match std::fs::remove_dir_all(&self.data_dir) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(ExtError::Store(format!("không xoá được {}: {e}", self.data_dir.display()))),
            _ => Ok(()),
        }
    }

    pub fn clear_cache(&self) -> Result<(), ExtError> {
        let path = self.cache_path();
        for suffix in ["", "-wal", "-shm"] {
            remove_if_exists(&PathBuf::from(format!("{}{suffix}", path.display())))?;
        }
        Ok(())
    }

    /// Trạng thái phiên đăng nhập và session data, theo auth hiện tại.
    pub fn session_status(&self) -> SessionStatus {
        SessionData::open(Some(self.session_path()), &self.auth_version()).status()
    }

    /// Dung lượng file cache trên đĩa.
    pub fn cache_bytes(&self) -> u64 {
        let path = self.cache_path();
        ["", "-wal"].iter().filter_map(|suffix| std::fs::metadata(format!("{}{suffix}", path.display())).ok()).map(|m| m.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_manifest;
    use serde_json::json;

    const MANIFEST: &str = r#"{ "id": "io.github.dev.auth", "name": "A", "version": "1.0.0", "apiVersion": 1,
        "description": "d", "developer": { "name": "dev" }, "match": ["^https://a\\.test/"], "hosts": ["a.test"],
        "auth": { "schemes": [
            { "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key", "type": "secret" }] },
            { "id": "password", "label": "Tài khoản", "fields": [
                { "key": "username", "label": "Tên", "type": "text" },
                { "key": "password", "label": "Mật khẩu", "type": "secret" } ] } ] },
        "settings": [
            { "key": "server", "label": "Máy chủ", "type": "select", "choices": ["cn", "global"], "default": "cn" },
            { "key": "pageSize", "label": "Cỡ trang", "type": "number" },
            { "key": "token", "label": "Token", "type": "secret" },
            { "key": "proxy", "label": "Proxy", "type": "text", "scope": "story" } ] }"#;

    fn setup() -> (tempfile::TempDir, ExtStore, Manifest) {
        let dir = tempfile::tempdir().unwrap();
        let store = ExtStore::with_backend(dir.path(), "io.github.dev.auth", SecretBackend::memory());
        (dir, store, parse_manifest(MANIFEST, false).unwrap())
    }

    fn map(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn luu_auth_giu_secret_khi_de_trong_va_doi_ma_phien_ban() {
        let (dir, store, m) = setup();
        assert_eq!(store.auth(&m).unwrap(), None);
        assert_eq!(store.auth_version(), "");

        store.save_auth(&m, "password", &map(json!({ "username": " an ", "password": "mk-1" }))).unwrap();
        let auth = store.auth(&m).unwrap().unwrap();
        assert_eq!(auth.values, map(json!({ "username": "an", "password": "mk-1" })));
        assert_eq!(auth.secret_values(m.auth.as_ref()), vec!["mk-1".to_string()]);
        let v1 = store.auth_version();
        assert!(!v1.is_empty());
        let on_disk = std::fs::read_to_string(dir.path().join("extension-data/io.github.dev.auth/auth.json")).unwrap();
        assert!(on_disk.contains("an") && !on_disk.contains("mk-1"), "mật khẩu không ghi ra file: {on_disk}");

        store.save_auth(&m, "password", &map(json!({ "username": "an", "password": "" }))).unwrap();
        assert_eq!(store.auth(&m).unwrap().unwrap().values["password"], json!("mk-1"), "ô secret trống = giữ");
        assert_eq!(store.auth_version(), v1, "lưu y nguyên thì không đổi phiên bản");

        store.save_auth(&m, "password", &map(json!({ "username": "an", "password": "mk-2" }))).unwrap();
        let v2 = store.auth_version();
        assert_ne!(v2, v1);
        let state = store.auth_state(&m).unwrap();
        assert_eq!(state.scheme.as_deref(), Some("password"));
        assert_eq!(state.text, map(json!({ "username": "an" })));
        assert_eq!(state.filled, vec!["username".to_string(), "password".to_string()]);

        store.save_auth(&m, "apiKey", &map(json!({ "apiKey": "k-123" }))).unwrap();
        assert_eq!(store.auth(&m).unwrap().unwrap().scheme, "apiKey");
        store.save_auth(&m, "password", &map(json!({ "username": "an" }))).unwrap();
        assert_eq!(store.auth(&m).unwrap().unwrap().values.get("password"), None, "đổi kiểu thì khoá của kiểu cũ bị xoá");
        assert!(store.save_auth(&m, "oauth", &Map::new()).is_err());
    }

    #[test]
    fn xoa_auth_xoa_ca_session() {
        let (_dir, store, m) = setup();
        store.save_auth(&m, "apiKey", &map(json!({ "apiKey": "k-123" }))).unwrap();
        write_atomic(&store.session_path(), b"{}").unwrap();
        store.clear_auth(&m).unwrap();
        assert_eq!(store.auth(&m).unwrap(), None);
        assert_eq!(store.auth_version(), "");
        assert!(!store.session_path().exists());
    }

    #[test]
    fn cai_dat_chung_tach_secret_va_kiem_kieu() {
        let (dir, store, m) = setup();
        store.save_settings(&m, &map(json!({ "server": "global", "pageSize": "50", "token": "tk-9", "proxy": "bỏ qua" }))).unwrap();
        assert_eq!(store.settings(&m).unwrap(), map(json!({ "server": "global", "pageSize": 50, "token": "tk-9" })), "proxy là scope story");
        let file = std::fs::read_to_string(dir.path().join("extensions/io.github.dev.auth/settings.json")).unwrap();
        assert!(!file.contains("tk-9"));
        assert_eq!(store.settings_state(&m).unwrap(), (map(json!({ "server": "global", "pageSize": 50 })), vec!["token".to_string()]));

        store.save_settings(&m, &map(json!({ "token": "", "pageSize": null }))).unwrap();
        assert_eq!(store.settings(&m).unwrap(), map(json!({ "server": "global", "token": "tk-9" })), "thiếu khoá = giữ, null = mặc định, secret rỗng = giữ");
        store.save_settings(&m, &map(json!({ "token": null }))).unwrap();
        assert_eq!(store.settings(&m).unwrap(), map(json!({ "server": "global" })));

        let err = store.save_settings(&m, &map(json!({ "server": "eu", "pageSize": 3 }))).unwrap_err();
        assert!(err.to_string().contains("Máy chủ"));
        assert_eq!(store.settings(&m).unwrap(), map(json!({ "server": "global" })), "sai kiểu thì không lưu gì");
    }

    #[test]
    fn auth_chung_cua_kho() {
        let (dir, own, m) = setup();
        let backend = SecretBackend::memory();
        let spec = crate::parse_auth_value(&json!({ "required": true, "schemes": [{ "id": "apiKey", "label": "Key", "fields": [{ "key": "apiKey", "label": "K", "type": "secret" }] }] })).unwrap();
        let repo = AuthStore::repository(dir.path(), "org.kho", spec, backend.clone());
        let a = ExtStore::with_backend(dir.path(), "org.kho.a", backend.clone()).with_shared_auth(repo.clone());
        let b = ExtStore::with_backend(dir.path(), "org.kho.b", backend.clone()).with_shared_auth(repo.clone());
        assert_eq!(a.shared_auth().and_then(|s| s.repository_id()), Some("org.kho"));
        assert!(own.shared_auth().is_none());

        repo.save(None, "apiKey", &map(json!({ "apiKey": "u:s" }))).unwrap();
        assert_eq!(a.auth(&m).unwrap().unwrap().values["apiKey"], json!("u:s"), "manifest của a có kiểu khác — dùng spec của kho");
        assert_eq!(b.auth(&m).unwrap().unwrap().values["apiKey"], json!("u:s"), "hai extension đọc chung một key");
        assert_eq!(own.auth(&m).unwrap(), None, "extension không được kho cho dùng chung thì không thấy key kho");
        assert!(dir.path().join("extension-data/repo-org.kho/auth.json").exists());

        let v1 = a.auth_version();
        write_atomic(&a.session_path(), b"{}").unwrap();
        repo.save(None, "apiKey", &map(json!({ "apiKey": "u:s2" }))).unwrap();
        assert_ne!(a.auth_version(), v1, "đổi key kho thì đổi phiên bản → session data cũ thành trống");
        assert_eq!(a.auth_version(), b.auth_version());

        a.clear_auth(&m).unwrap();
        assert!(!a.session_path().exists());
        assert!(b.auth(&m).unwrap().is_some(), "xoá auth ở một extension không xoá key chung của kho");
        a.remove_all(&m).unwrap();
        assert!(b.auth(&m).unwrap().is_some(), "gỡ một extension không xoá key chung");
    }

    #[test]
    fn go_xoa_het_du_lieu() {
        let (dir, store, m) = setup();
        store.save_auth(&m, "apiKey", &map(json!({ "apiKey": "k-1" }))).unwrap();
        store.save_settings(&m, &map(json!({ "server": "global", "token": "tk" }))).unwrap();
        write_atomic(&store.cache_path(), b"x").unwrap();
        store.remove_all(&m).unwrap();
        assert!(!dir.path().join("extension-data/io.github.dev.auth").exists());
        assert!(!dir.path().join("extensions/io.github.dev.auth/settings.json").exists());
        let SecretBackend::Memory(map) = &store.secrets else { unreachable!() };
        assert!(map.lock().unwrap().is_empty(), "khoá auth và cài đặt secret đều xoá");
    }

    #[test]
    fn xoa_cache_va_dung_luong() {
        let (_dir, store, _m) = setup();
        assert_eq!(store.cache_bytes(), 0);
        write_atomic(&store.cache_path(), &[0u8; 100]).unwrap();
        write_atomic(&store.webview_dir().join("EBWebView/Cookies"), b"x").unwrap();
        assert_eq!(store.cache_bytes(), 100);
        store.clear_cache().unwrap();
        assert_eq!(store.cache_bytes(), 0);
        store.logout().unwrap();
        assert!(!store.webview_dir().exists(), "đăng xuất xoá hồ sơ webview");
    }
}
