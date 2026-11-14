//! Auth đã nhập: của riêng một extension, hoặc auth chung của một kho (spec kho 2026-11-14 mục 4). Trường text ở
//! `auth.json`, trường secret ở kho khoá; tên khoá `<tiền tố>/auth/<kiểu>/<trường>` với tiền tố `qt-ext/<id>`
//! hoặc `qt-ext/repo/<id kho>`. Có gì đổi thì sinh mã phiên bản mới — session data ghi theo mã cũ thành trống.

use super::{as_text, random_hex, remove_if_exists, write_atomic, SecretBackend};
use crate::manifest::{AuthSpec, FieldKind};
use crate::ExtError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// Auth người dùng đã nhập cho một kiểu (kể cả trường secret). Đưa cho extension qua `ctx.auth`.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthValues {
    pub scheme: String,
    pub values: Map<String, Value>,
}

impl AuthValues {
    /// Giá trị các trường secret — để che trong log.
    pub fn secret_values(&self, spec: Option<&AuthSpec>) -> Vec<String> {
        let Some(scheme) = spec.iter().flat_map(|a| &a.schemes).find(|s| s.id == self.scheme) else {
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
pub struct AuthStore {
    prefix: String,
    path: PathBuf,
    secrets: SecretBackend,
    /// Auth chung của kho: spec của kho, thay cho `auth` trong manifest. `None` = auth riêng của extension.
    shared: Option<(String, AuthSpec)>,
}

impl AuthStore {
    /// Auth riêng của extension `id` (file ở `data_dir/auth.json`).
    pub fn extension(data_dir: &Path, id: &str, secrets: SecretBackend) -> AuthStore {
        AuthStore { prefix: format!("qt-ext/{id}"), path: data_dir.join("auth.json"), secrets, shared: None }
    }

    /// Auth chung của kho `repo_id`, theo `spec` kho khai.
    pub fn repository(config_root: &Path, repo_id: &str, spec: AuthSpec, secrets: SecretBackend) -> AuthStore {
        AuthStore {
            prefix: format!("qt-ext/repo/{repo_id}"),
            path: config_root.join("extension-data").join(format!("repo-{repo_id}")).join("auth.json"),
            secrets,
            shared: Some((repo_id.to_string(), spec)),
        }
    }

    /// Kho máy này dùng (Credential Manager trên Windows, file ở máy khác).
    pub fn repository_on_platform(config_root: &Path, repo_id: &str, spec: AuthSpec) -> AuthStore {
        let file = config_root.join("extension-data").join(format!("repo-{repo_id}")).join("secrets.json");
        AuthStore::repository(config_root, repo_id, spec, SecretBackend::platform(file))
    }

    /// Id kho khi đây là auth chung của kho.
    pub fn repository_id(&self) -> Option<&str> {
        self.shared.as_ref().map(|(id, _)| id.as_str())
    }

    /// Spec áp dụng: của kho nếu là auth chung, không thì `fallback` (auth trong manifest).
    pub fn spec<'a>(&'a self, fallback: Option<&'a AuthSpec>) -> Option<&'a AuthSpec> {
        self.shared.as_ref().map(|(_, spec)| spec).or(fallback)
    }

    pub fn secret_kind(&self) -> &'static str {
        self.secrets.kind()
    }

    fn name(&self, scheme: &str, field: &str) -> String {
        format!("{}/auth/{scheme}/{field}", self.prefix)
    }

    fn load(&self) -> AuthFile {
        std::fs::read_to_string(&self.path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    /// Mã phiên bản auth hiện tại ("" = chưa có auth).
    pub fn version(&self) -> String {
        self.load().version
    }

    /// Auth đầy đủ của kiểu đã chọn; `None` khi chưa nhập gì hoặc kiểu không còn trong spec.
    pub fn values(&self, fallback: Option<&AuthSpec>) -> Result<Option<AuthValues>, ExtError> {
        let file = self.load();
        let Some(scheme) = self.spec(fallback).iter().flat_map(|a| &a.schemes).find(|s| s.id == file.scheme) else {
            return Ok(None);
        };
        let mut values = Map::new();
        for field in &scheme.fields {
            let value = match field.kind {
                FieldKind::Text => file.values.get(&field.key).map(as_text).filter(|v| !v.is_empty()),
                FieldKind::Secret => self.secrets.get(&self.name(&scheme.id, &field.key))?,
            };
            if let Some(value) = value {
                values.insert(field.key.clone(), Value::String(value));
            }
        }
        Ok((!values.is_empty()).then(|| AuthValues { scheme: scheme.id.clone(), values }))
    }

    pub fn state(&self, fallback: Option<&AuthSpec>) -> Result<AuthState, ExtError> {
        let Some(auth) = self.values(fallback)? else {
            return Ok(AuthState::default());
        };
        let scheme = self.spec(fallback).iter().flat_map(|a| &a.schemes).find(|s| s.id == auth.scheme).expect("values() chỉ trả kiểu có trong spec");
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
    /// thì sinh mã phiên bản mới.
    pub fn save(&self, fallback: Option<&AuthSpec>, scheme_id: &str, values: &Map<String, Value>) -> Result<(), ExtError> {
        let spec = self.spec(fallback).ok_or_else(|| ExtError::Store("extension không khai auth".into()))?;
        let scheme = spec.schemes.iter().find(|s| s.id == scheme_id).ok_or_else(|| ExtError::Store(format!("kiểu auth lạ: {scheme_id}")))?;
        let old = self.values(fallback)?;
        let mut file = self.load();
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
                    let name = self.name(&scheme.id, &field.key);
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
                self.secrets.delete(&self.name(&other.id, &field.key))?;
            }
        }
        let now = (!saved.is_empty()).then(|| AuthValues { scheme: scheme.id.clone(), values: saved });
        if now != old || file.version.is_empty() {
            file.version = random_hex();
        }
        file.scheme = scheme.id.clone();
        file.values = text;
        let json = serde_json::to_string_pretty(&file).map_err(|e| ExtError::Store(e.to_string()))?;
        write_atomic(&self.path, json.as_bytes())
    }

    /// Xoá mọi khoá và `auth.json` (mã phiên bản về rỗng → session data theo auth cũ thành trống).
    pub fn clear(&self, fallback: Option<&AuthSpec>) -> Result<(), ExtError> {
        for scheme in self.spec(fallback).iter().flat_map(|a| &a.schemes) {
            for field in scheme.fields.iter().filter(|f| f.kind == FieldKind::Secret) {
                self.secrets.delete(&self.name(&scheme.id, &field.key))?;
            }
        }
        remove_if_exists(&self.path)
    }
}
