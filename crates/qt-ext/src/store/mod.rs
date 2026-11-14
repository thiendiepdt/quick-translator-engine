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

pub mod cache;
pub mod secrets;
pub mod session_data;

use crate::manifest::{check_setting, FieldKind, Manifest, Scope, SettingKind};
use crate::ExtError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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

fn remove_if_exists(path: &Path) -> Result<(), ExtError> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io_error(path)(e)),
        _ => Ok(()),
    }
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn random_hex() -> String {
    let mut bytes = [0u8; 8];
    if getrandom::fill(&mut bytes).is_err() {
        return format!("{:016x}", now_ms());
    }
    hex::encode(bytes)
}

fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Auth người dùng đã nhập cho một kiểu trong manifest (kể cả trường secret). Đưa cho extension qua `ctx.auth`.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthValues {
    pub scheme: String,
    pub values: Map<String, Value>,
}

impl AuthValues {
    /// Giá trị các trường secret — để che trong log.
    pub fn secret_values(&self, manifest: &Manifest) -> Vec<String> {
        let Some(scheme) = manifest.auth.iter().flat_map(|a| &a.schemes).find(|s| s.id == self.scheme) else {
            return Vec::new();
        };
        scheme.fields.iter().filter(|f| f.kind == FieldKind::Secret).filter_map(|f| self.values.get(&f.key)).map(as_text).filter(|v| !v.is_empty()).collect()
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({ "scheme": self.scheme, "values": self.values })
    }
}

/// Auth cho giao diện: không có giá trị secret, chỉ biết trường nào đã nhập.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    pub scheme: Option<String>,
    /// Giá trị các trường text.
    pub text: Map<String, Value>,
    /// Khoá các trường đã nhập (text và secret).
    pub filled: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct AuthFile {
    scheme: String,
    values: Map<String, Value>,
    version: String,
}

#[derive(Debug, Clone)]
pub struct ExtStore {
    id: String,
    settings_file: PathBuf,
    data_dir: PathBuf,
    secrets: SecretBackend,
}

impl ExtStore {
    /// `config_root`: thư mục cấu hình của app (chứa `extensions/`).
    pub fn new(config_root: &Path, id: &str) -> ExtStore {
        let backend = SecretBackend::platform(config_root.join("extension-data").join(id).join("secrets.json"));
        ExtStore::with_backend(config_root, id, backend)
    }

    pub fn with_backend(config_root: &Path, id: &str, secrets: SecretBackend) -> ExtStore {
        ExtStore {
            id: id.to_string(),
            settings_file: config_root.join("extensions").join(id).join("settings.json"),
            data_dir: config_root.join("extension-data").join(id),
            secrets,
        }
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

    fn auth_path(&self) -> PathBuf {
        self.data_dir.join("auth.json")
    }

    fn auth_name(&self, scheme: &str, field: &str) -> String {
        format!("qt-ext/{}/auth/{scheme}/{field}", self.id)
    }

    fn setting_name(&self, key: &str) -> String {
        format!("qt-ext/{}/setting/{key}", self.id)
    }

    fn load_auth_file(&self) -> AuthFile {
        std::fs::read_to_string(self.auth_path()).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    /// Mã phiên bản auth hiện tại ("" = chưa có auth).
    pub fn auth_version(&self) -> String {
        self.load_auth_file().version
    }

    /// Auth đầy đủ của kiểu đã chọn; `None` khi chưa nhập gì hoặc kiểu không còn trong manifest.
    pub fn auth(&self, manifest: &Manifest) -> Result<Option<AuthValues>, ExtError> {
        let file = self.load_auth_file();
        let Some(scheme) = manifest.auth.iter().flat_map(|a| &a.schemes).find(|s| s.id == file.scheme) else {
            return Ok(None);
        };
        let mut values = Map::new();
        for field in &scheme.fields {
            let value = match field.kind {
                FieldKind::Text => file.values.get(&field.key).map(as_text).filter(|v| !v.is_empty()),
                FieldKind::Secret => self.secrets.get(&self.auth_name(&scheme.id, &field.key))?,
            };
            if let Some(value) = value {
                values.insert(field.key.clone(), Value::String(value));
            }
        }
        Ok((!values.is_empty()).then(|| AuthValues { scheme: scheme.id.clone(), values }))
    }

    pub fn auth_state(&self, manifest: &Manifest) -> Result<AuthState, ExtError> {
        let Some(auth) = self.auth(manifest)? else {
            return Ok(AuthState::default());
        };
        let scheme = manifest.auth.iter().flat_map(|a| &a.schemes).find(|s| s.id == auth.scheme).expect("auth() chỉ trả kiểu có trong manifest");
        let mut text = Map::new();
        let mut filled = Vec::new();
        for field in &scheme.fields {
            if let Some(value) = auth.values.get(&field.key) {
                filled.push(field.key.clone());
                if field.kind == FieldKind::Text {
                    text.insert(field.key.clone(), value.clone());
                }
            }
        }
        Ok(AuthState { scheme: Some(auth.scheme), text, filled })
    }

    /// Lưu auth của một kiểu. Trường secret để trống thì giữ giá trị cũ; khoá của các kiểu khác bị xoá. Có gì đổi
    /// thì sinh mã phiên bản mới (session data cũ thành trống).
    pub fn save_auth(&self, manifest: &Manifest, scheme_id: &str, values: &Map<String, Value>) -> Result<(), ExtError> {
        let spec = manifest.auth.as_ref().ok_or_else(|| ExtError::Store("extension không khai auth".into()))?;
        let scheme = spec.schemes.iter().find(|s| s.id == scheme_id).ok_or_else(|| ExtError::Store(format!("kiểu auth lạ: {scheme_id}")))?;
        let old = self.auth(manifest)?;
        let mut file = self.load_auth_file();
        let mut text = Map::new();
        let mut saved = Map::new();
        for field in &scheme.fields {
            let given = values.get(&field.key).map(as_text).unwrap_or_default();
            match field.kind {
                FieldKind::Text => {
                    let given = given.trim();
                    if !given.is_empty() {
                        text.insert(field.key.clone(), Value::String(given.to_string()));
                        saved.insert(field.key.clone(), Value::String(given.to_string()));
                    }
                }
                FieldKind::Secret => {
                    let name = self.auth_name(&scheme.id, &field.key);
                    if given.is_empty() {
                        if let Some(value) = self.secrets.get(&name)? {
                            saved.insert(field.key.clone(), Value::String(value));
                        }
                    } else {
                        self.secrets.set(&name, &given)?;
                        saved.insert(field.key.clone(), Value::String(given));
                    }
                }
            }
        }
        for other in spec.schemes.iter().filter(|s| s.id != scheme.id) {
            for field in other.fields.iter().filter(|f| f.kind == FieldKind::Secret) {
                self.secrets.delete(&self.auth_name(&other.id, &field.key))?;
            }
        }
        let now = (!saved.is_empty()).then(|| AuthValues { scheme: scheme.id.clone(), values: saved });
        if now != old || file.version.is_empty() {
            file.version = random_hex();
        }
        file.scheme = scheme.id.clone();
        file.values = text;
        let json = serde_json::to_string_pretty(&file).map_err(|e| ExtError::Store(e.to_string()))?;
        write_atomic(&self.auth_path(), json.as_bytes())
    }

    /// Xoá mọi khoá auth, `auth.json` và session data.
    pub fn clear_auth(&self, manifest: &Manifest) -> Result<(), ExtError> {
        for scheme in manifest.auth.iter().flat_map(|a| &a.schemes) {
            for field in scheme.fields.iter().filter(|f| f.kind == FieldKind::Secret) {
                self.secrets.delete(&self.auth_name(&scheme.id, &field.key))?;
            }
        }
        remove_if_exists(&self.auth_path())?;
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
        self.clear_auth(manifest)?;
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
        assert_eq!(auth.secret_values(&m), vec!["mk-1".to_string()]);
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
