//! `manifest.json` của extension: đọc rồi kiểm từng trường (spec mục 4). Sai thì từ chối cài, thông báo nêu
//! đúng trường để nhà phát triển sửa.

use crate::policy::{is_local_name, is_private_ip};
use crate::ExtError;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::net::IpAddr;

/// Phiên bản API cao nhất app này hiểu.
pub const API_VERSION: u32 = 1;
pub const MAX_CONCURRENCY: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Developer {
    pub name: String,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Webview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthField {
    pub key: String,
    pub label: String,
    pub kind: FieldKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthScheme {
    pub id: String,
    pub label: String,
    pub fields: Vec<AuthField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSpec {
    pub required: bool,
    /// Theo thứ tự ưu tiên của nhà phát triển (thường API key trước).
    pub schemes: Vec<AuthScheme>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    Text,
    Number,
    Boolean,
    Select,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Global,
    Story,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingField {
    pub key: String,
    pub label: String,
    pub kind: SettingKind,
    pub choices: Vec<String>,
    pub default: Option<Value>,
    pub scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub concurrency: usize,
    pub delay_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { concurrency: 1, delay_ms: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub api_version: u32,
    pub description: String,
    pub developer: Developer,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub matches: Vec<String>,
    pub hosts: Vec<String>,
    pub permissions: Vec<Permission>,
    pub auth: Option<AuthSpec>,
    pub settings: Vec<SettingField>,
    pub limits: Limits,
    match_regexes: Vec<Regex>,
}

impl Manifest {
    /// Link tải raw có thuộc extension này không (theo `match`).
    pub fn matches_url(&self, url: &str) -> bool {
        self.match_regexes.iter().any(|regex| regex.is_match(url.trim()))
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Khoá các trường `secret` (auth và cài đặt) — để che trong log.
    pub fn secret_keys(&self) -> Vec<String> {
        let auth = self.auth.iter().flat_map(|a| &a.schemes).flat_map(|s| &s.fields).filter(|f| f.kind == FieldKind::Secret).map(|f| f.key.clone());
        let settings = self.settings.iter().filter(|s| s.kind == SettingKind::Secret).map(|s| s.key.clone());
        auth.chain(settings).collect()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDeveloper {
    #[serde(default)]
    name: String,
    url: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
struct RawField {
    #[serde(default)]
    key: String,
    #[serde(default)]
    label: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    choices: Vec<String>,
    default: Option<Value>,
    scope: Option<String>,
}

#[derive(Deserialize)]
struct RawScheme {
    #[serde(default)]
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    fields: Vec<RawField>,
}

#[derive(Deserialize)]
struct RawAuth {
    #[serde(default)]
    required: bool,
    #[serde(default)]
    schemes: Vec<RawScheme>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLimits {
    concurrency: Option<i64>,
    delay_ms: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawManifest {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    api_version: Option<u32>,
    #[serde(default)]
    description: String,
    developer: Option<RawDeveloper>,
    license: Option<String>,
    homepage: Option<String>,
    #[serde(rename = "match", default)]
    matches: Vec<String>,
    #[serde(default)]
    hosts: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
    auth: Option<RawAuth>,
    #[serde(default)]
    settings: Vec<RawField>,
    limits: Option<RawLimits>,
}

fn bad(message: impl Into<String>) -> ExtError {
    ExtError::Manifest(message.into())
}

fn non_empty(value: &str, field: &str) -> Result<String, ExtError> {
    let value = value.trim();
    if value.is_empty() {
        Err(bad(format!("{field} không được để trống")))
    } else {
        Ok(value.to_string())
    }
}

fn valid_host(host: &str, allow_local: bool) -> bool {
    let host = host.trim();
    let name = host.strip_prefix("*.").unwrap_or(host);
    if name.is_empty() || name.contains("://") || name.contains(':') || name.contains('/') || name.contains('*') || name != name.to_ascii_lowercase() {
        return false;
    }
    if let Ok(ip) = name.parse::<IpAddr>() {
        return allow_local && is_private_ip(ip) && !host.starts_with("*.");
    }
    if is_local_name(name) {
        return allow_local && !host.starts_with("*.");
    }
    name.contains('.') && name.split('.').all(|label| !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

fn unique<'a>(keys: impl Iterator<Item = &'a str>, field: &str) -> Result<(), ExtError> {
    let mut seen = std::collections::HashSet::new();
    for key in keys {
        if !seen.insert(key) {
            return Err(bad(format!("{field}: khoá \"{key}\" bị trùng")));
        }
    }
    Ok(())
}

/// Đọc một khối `auth` (cùng cú pháp với manifest) — cho auth chung của kho.
pub fn parse_auth_value(value: &Value) -> Result<AuthSpec, ExtError> {
    let raw: RawAuth = serde_json::from_value(value.clone()).map_err(|e| bad(format!("auth: {e}")))?;
    parse_auth(raw)
}

fn parse_auth(raw: RawAuth) -> Result<AuthSpec, ExtError> {
    if raw.schemes.is_empty() {
        return Err(bad("auth.schemes phải có ít nhất một kiểu đăng nhập"));
    }
    let mut schemes = Vec::new();
    for (i, scheme) in raw.schemes.into_iter().enumerate() {
        let id = non_empty(&scheme.id, &format!("auth.schemes[{i}].id"))?;
        let label = non_empty(&scheme.label, &format!("auth.schemes[{i}].label"))?;
        if scheme.fields.is_empty() {
            return Err(bad(format!("auth.schemes[{i}].fields phải có ít nhất một trường")));
        }
        let mut fields = Vec::new();
        for (j, field) in scheme.fields.into_iter().enumerate() {
            let at = format!("auth.schemes[{i}].fields[{j}]");
            let kind = match field.kind.as_str() {
                "text" => FieldKind::Text,
                "secret" => FieldKind::Secret,
                other => return Err(bad(format!("{at}.type \"{other}\": chỉ có text hoặc secret"))),
            };
            fields.push(AuthField { key: non_empty(&field.key, &format!("{at}.key"))?, label: non_empty(&field.label, &format!("{at}.label"))?, kind });
        }
        unique(fields.iter().map(|f| f.key.as_str()), &format!("auth.schemes[{i}].fields"))?;
        schemes.push(AuthScheme { id, label, fields });
    }
    unique(schemes.iter().map(|s| s.id.as_str()), "auth.schemes")?;
    Ok(AuthSpec { required: raw.required, schemes })
}

fn parse_settings(raw: Vec<RawField>) -> Result<Vec<SettingField>, ExtError> {
    let mut settings = Vec::new();
    for (i, field) in raw.into_iter().enumerate() {
        let at = format!("settings[{i}]");
        let kind = match field.kind.as_str() {
            "text" => SettingKind::Text,
            "number" => SettingKind::Number,
            "boolean" => SettingKind::Boolean,
            "select" => SettingKind::Select,
            "secret" => SettingKind::Secret,
            other => return Err(bad(format!("{at}.type \"{other}\": chỉ có text, number, boolean, select, secret"))),
        };
        let scope = match field.scope.as_deref() {
            None | Some("global") => Scope::Global,
            Some("story") => Scope::Story,
            Some(other) => return Err(bad(format!("{at}.scope \"{other}\": chỉ có global hoặc story"))),
        };
        if kind == SettingKind::Secret && scope == Scope::Story {
            return Err(bad(format!("{at}: trường secret không được scope story (thư mục truyện có thể là repo git)")));
        }
        if kind == SettingKind::Select && field.choices.is_empty() {
            return Err(bad(format!("{at}: kiểu select phải có choices")));
        }
        let setting = SettingField {
            key: non_empty(&field.key, &format!("{at}.key"))?,
            label: non_empty(&field.label, &format!("{at}.label"))?,
            kind,
            choices: field.choices,
            default: field.default.filter(|v| !v.is_null()),
            scope,
        };
        if let Some(default) = &setting.default {
            check_setting(&setting, default).map_err(|e| bad(format!("{at}.default: {e}")))?;
        }
        settings.push(setting);
    }
    unique(settings.iter().map(|s| s.key.as_str()), "settings")?;
    Ok(settings)
}

/// Kiểm và chuẩn hoá giá trị của một trường cài đặt: text/secret nhận chuỗi, number nhận số hoặc chuỗi số,
/// boolean nhận true/false, select nhận một mục trong `choices`.
pub fn check_setting(field: &SettingField, value: &Value) -> Result<Value, String> {
    let label = &field.label;
    match field.kind {
        SettingKind::Text | SettingKind::Secret => match value {
            Value::String(s) => Ok(Value::String(s.clone())),
            Value::Number(n) => Ok(Value::String(n.to_string())),
            _ => Err(format!("{label}: phải là chữ")),
        },
        SettingKind::Number => match value {
            Value::Number(_) => Ok(value.clone()),
            Value::String(s) => {
                let s = s.trim();
                if let Ok(n) = s.parse::<i64>() {
                    return Ok(Value::from(n));
                }
                s.parse::<f64>().ok().and_then(serde_json::Number::from_f64).map(Value::Number).ok_or_else(|| format!("{label}: phải là số"))
            }
            _ => Err(format!("{label}: phải là số")),
        },
        SettingKind::Boolean => match value {
            Value::Bool(_) => Ok(value.clone()),
            Value::String(s) if s == "true" || s == "false" => Ok(Value::Bool(s == "true")),
            _ => Err(format!("{label}: phải là true hoặc false")),
        },
        SettingKind::Select => match value {
            Value::String(s) if field.choices.contains(s) => Ok(value.clone()),
            _ => Err(format!("{label}: phải là một trong {}", field.choices.join(", "))),
        },
    }
}

/// `allow_local`: chế độ developer — cho khai `localhost` / IP nội bộ để chạy thử với máy chủ giả.
pub fn parse_manifest(json: &str, allow_local: bool) -> Result<Manifest, ExtError> {
    let raw: RawManifest = serde_json::from_str(json).map_err(|e| bad(format!("không đọc được JSON: {e}")))?;

    let id = raw.id.trim().to_string();
    let id_pattern = Regex::new(r"^[a-z0-9]+(\.[a-z0-9-]+)+$").expect("regex id");
    if !id_pattern.is_match(&id) {
        return Err(bad(format!("id \"{id}\" phải dạng tên miền đảo ngược, chữ thường, ví dụ io.github.ten.nguon")));
    }
    let version = semver::Version::parse(raw.version.trim()).map_err(|_| bad(format!("version \"{}\" phải theo semver, ví dụ 1.2.0", raw.version)))?;
    let api_version = raw.api_version.ok_or_else(|| bad("apiVersion bị thiếu"))?;
    if api_version == 0 {
        return Err(bad("apiVersion phải từ 1"));
    }
    if api_version > API_VERSION {
        return Err(bad(format!("apiVersion {api_version}: extension cần app mới hơn")));
    }
    let developer = raw.developer.ok_or_else(|| bad("developer bị thiếu"))?;
    let developer = Developer { name: non_empty(&developer.name, "developer.name")?, url: developer.url.filter(|s| !s.trim().is_empty()), email: developer.email.filter(|s| !s.trim().is_empty()) };

    if raw.matches.is_empty() {
        return Err(bad("match phải có ít nhất một regex"));
    }
    let mut match_regexes = Vec::new();
    for (i, pattern) in raw.matches.iter().enumerate() {
        match_regexes.push(Regex::new(pattern).map_err(|e| bad(format!("match[{i}] không phải regex hợp lệ: {e}")))?);
    }
    if raw.hosts.is_empty() {
        return Err(bad("hosts phải có ít nhất một domain"));
    }
    for (i, host) in raw.hosts.iter().enumerate() {
        if !valid_host(host, allow_local) {
            return Err(bad(format!("hosts[{i}] \"{host}\": chỉ ghi tên miền (chữ thường, cho phép *. ở đầu), không scheme, cổng, đường dẫn, IP hay tên miền nội bộ")));
        }
    }
    let mut permissions = Vec::new();
    for permission in &raw.permissions {
        match permission.as_str() {
            "webview" => permissions.push(Permission::Webview),
            other => return Err(bad(format!("permissions \"{other}\": chỉ có webview"))),
        }
    }
    let limits = raw.limits.map(|l| Limits {
        concurrency: l.concurrency.unwrap_or(1).clamp(1, MAX_CONCURRENCY as i64) as usize,
        delay_ms: l.delay_ms.unwrap_or(0).max(0) as u64,
    });

    Ok(Manifest {
        id,
        name: non_empty(&raw.name, "name")?,
        version,
        api_version,
        description: non_empty(&raw.description, "description")?,
        developer,
        license: raw.license.filter(|s| !s.trim().is_empty()),
        homepage: raw.homepage.filter(|s| !s.trim().is_empty()),
        matches: raw.matches,
        hosts: raw.hosts.iter().map(|h| h.trim().to_string()).collect(),
        permissions,
        auth: raw.auth.map(parse_auth).transpose()?,
        settings: parse_settings(raw.settings)?,
        limits: limits.unwrap_or_default(),
        match_regexes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = r#"{
      "id": "io.github.dev.demo", "name": "Demo", "version": "1.2.0", "apiVersion": 1,
      "description": "Mẫu", "developer": { "name": "dev", "url": "https://x.dev" },
      "match": ["^https://demo\\.test/book/\\d+"], "hosts": ["demo.test", "*.cdn.test"],
      "permissions": ["webview"],
      "auth": { "required": false, "schemes": [ { "id": "apiKey", "label": "API key",
        "fields": [ { "key": "apiKey", "label": "API key", "type": "secret" } ] } ] },
      "settings": [ { "key": "proxy", "label": "Proxy", "type": "text", "scope": "story" },
                    { "key": "token", "label": "Token", "type": "secret" } ],
      "limits": { "concurrency": 20, "delayMs": 300 }
    }"#;

    #[test]
    fn doc_manifest_day_du_va_chan_tran_concurrency() {
        let m = parse_manifest(OK, false).unwrap();
        assert_eq!(m.id, "io.github.dev.demo");
        assert_eq!(m.version.to_string(), "1.2.0");
        assert_eq!(m.limits, Limits { concurrency: 8, delay_ms: 300 });
        assert!(m.matches_url("https://demo.test/book/12"));
        assert!(!m.matches_url("https://other.test/book/12"));
        assert_eq!(m.permissions, vec![Permission::Webview]);
        assert_eq!(m.settings[0].scope, Scope::Story);
        assert_eq!(m.secret_keys(), vec!["apiKey".to_string(), "token".to_string()]);
        assert_eq!(m.developer.url.as_deref(), Some("https://x.dev"));
    }

    #[test]
    fn mac_dinh_limits_va_truong_tuy_chon() {
        let mut v: Value = serde_json::from_str(OK).unwrap();
        let o = v.as_object_mut().unwrap();
        for key in ["limits", "auth", "settings", "permissions"] {
            o.remove(key);
        }
        let m = parse_manifest(&v.to_string(), false).unwrap();
        assert_eq!(m.limits, Limits { concurrency: 1, delay_ms: 0 });
        assert!(m.auth.is_none() && m.settings.is_empty() && m.permissions.is_empty());
    }

    #[test]
    fn tu_choi_manifest_sai() {
        let bad = |patch: &str, needle: &str| {
            let mut v: Value = serde_json::from_str(OK).unwrap();
            let p: Value = serde_json::from_str(patch).unwrap();
            for (k, val) in p.as_object().unwrap() {
                v[k] = val.clone();
            }
            let err = parse_manifest(&v.to_string(), false).unwrap_err().to_string();
            assert!(err.contains(needle), "{patch} → {err}");
        };
        bad(r#"{"id": "Demo"}"#, "id");
        bad(r#"{"id": "demo"}"#, "id");
        bad(r#"{"version": "1.2"}"#, "version");
        bad(r#"{"apiVersion": 2}"#, "app mới hơn");
        bad(r#"{"match": ["("]}"#, "match[0]");
        bad(r#"{"match": []}"#, "match");
        bad(r#"{"hosts": ["http://demo.test"]}"#, "hosts[0]");
        bad(r#"{"hosts": ["demo.test:8080"]}"#, "hosts[0]");
        bad(r#"{"hosts": ["127.0.0.1"]}"#, "hosts[0]");
        bad(r#"{"hosts": ["localhost"]}"#, "hosts[0]");
        bad(r#"{"hosts": ["Demo.test"]}"#, "hosts[0]");
        bad(r#"{"hosts": ["*"]}"#, "hosts[0]");
        bad(r#"{"permissions": ["fs"]}"#, "permissions");
        bad(r#"{"settings": [{"key":"k","label":"K","type":"secret","scope":"story"}]}"#, "secret");
        bad(r#"{"settings": [{"key":"k","label":"K","type":"select"}]}"#, "choices");
        bad(r#"{"settings": [{"key":"k","label":"K","type":"text"},{"key":"k","label":"K2","type":"text"}]}"#, "trùng");
        bad(r#"{"auth": {"schemes": []}}"#, "auth.schemes");
        bad(r#"{"auth": {"schemes": [{"id":"a","label":"A","fields":[{"key":"x","label":"X","type":"number"}]}]}}"#, "text hoặc secret");
        bad(r#"{"developer": {"name": ""}}"#, "developer.name");
        bad(r#"{"description": " "}"#, "description");
        bad(r#"{"settings": [{"key":"k","label":"K","type":"number","default":"x"}]}"#, "settings[0].default");
        bad(r#"{"settings": [{"key":"k","label":"K","type":"select","choices":["a"],"default":"b"}]}"#, "settings[0].default");
    }

    #[test]
    fn kiem_gia_tri_cai_dat() {
        let field = |kind: SettingKind| SettingField { key: "k".into(), label: "K".into(), kind, choices: vec!["cn".into(), "global".into()], default: None, scope: Scope::Global };
        let check = |kind, value: Value| check_setting(&field(kind), &value);
        assert_eq!(check(SettingKind::Text, serde_json::json!("a")), Ok(serde_json::json!("a")));
        assert_eq!(check(SettingKind::Secret, serde_json::json!(12)), Ok(serde_json::json!("12")));
        assert!(check(SettingKind::Text, serde_json::json!(true)).is_err());
        assert_eq!(check(SettingKind::Number, serde_json::json!(" 3 ")), Ok(serde_json::json!(3)));
        assert_eq!(check(SettingKind::Number, serde_json::json!("1.5")), Ok(serde_json::json!(1.5)));
        assert!(check(SettingKind::Number, serde_json::json!("ba")).is_err());
        assert_eq!(check(SettingKind::Boolean, serde_json::json!("true")), Ok(serde_json::json!(true)));
        assert!(check(SettingKind::Boolean, serde_json::json!(1)).is_err());
        assert_eq!(check(SettingKind::Select, serde_json::json!("global")), Ok(serde_json::json!("global")));
        assert!(check(SettingKind::Select, serde_json::json!("eu")).unwrap_err().contains("cn, global"));
    }

    #[test]
    fn localhost_chi_duoc_khi_allow_local() {
        let mut v: Value = serde_json::from_str(OK).unwrap();
        v["hosts"] = serde_json::json!(["localhost", "127.0.0.1"]);
        assert!(parse_manifest(&v.to_string(), true).is_ok());
        v["hosts"] = serde_json::json!(["8.8.8.8"]);
        assert!(parse_manifest(&v.to_string(), true).is_err(), "IP công khai không bao giờ được khai");
    }
}
