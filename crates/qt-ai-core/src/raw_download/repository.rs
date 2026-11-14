//! Kho extension (spec 2026-11-14-kho-extension-design). Một kho là một `index.json` công khai liệt kê nhiều
//! extension (link zip + sha256), có thể kèm auth dùng chung. App lưu danh sách kho ở
//! `<thư mục extension>/repositories.json` và bản `index.json` tải gần nhất ở `.repos/<id kho>.json` (đọc được khi
//! offline, và để biết extension nào được dùng auth chung).

use super::registry::{Inspection, Registry, Source};
use qt_ext::{fetch_zip, parse_auth_value, sha256_hex, AuthSpec, AuthStore, ExtError, ExtStore, SecretBackend};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub const FORMAT_VERSION: u64 = 1;
pub const MAX_INDEX: usize = 2 * 1024 * 1024;
const RECORDS: &str = "repositories.json";
const CACHE_DIR: &str = ".repos";

static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9]+(\.[a-z0-9-]+)+$").expect("regex id"));
static SHA256: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("regex sha256"));

/// Tải một file từ kho (mặc định: https công khai, tối đa 5 MB). Test thay bằng bộ nhớ.
pub type Fetcher = Arc<dyn Fn(&str) -> Result<Vec<u8>, ExtError> + Send + Sync>;

fn repo_error(message: impl Into<String>) -> ExtError {
    ExtError::Package(message.into())
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Một extension trong kho. `url`, `icon` đã ghép thành link tuyệt đối.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub developer: Option<Person>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub hosts: Vec<String>,
    pub permissions: Vec<String>,
    pub auth_required: bool,
    /// Extension dùng auth chung của kho.
    pub shared_auth: bool,
    pub url: String,
    pub sha256: String,
    pub size: Option<u64>,
    pub icon: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoIndex {
    pub id: String,
    pub name: String,
    pub description: String,
    pub maintainer: Person,
    pub homepage: Option<String>,
    pub auth: Option<AuthSpec>,
    pub extensions: Vec<RepoEntry>,
}

fn text_of(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value.and_then(Value::as_array).map(|list| list.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
}

fn person(value: Option<&Value>) -> Option<Person> {
    let object = value?.as_object()?;
    let name = text_of(object.get("name"))?;
    Some(Person { name, url: text_of(object.get("url")), email: text_of(object.get("email")) })
}

/// Ghép link tương đối với link `index.json`; kết quả phải là https.
fn resolve(base: &Url, link: &str) -> Result<String, String> {
    let joined = base.join(link.trim()).map_err(|e| format!("link {link} không hợp lệ: {e}"))?;
    if joined.scheme() != "https" {
        return Err(format!("link {joined} phải là https://"));
    }
    Ok(joined.to_string())
}

fn parse_entry(value: &Value, base: &Url, has_auth: bool) -> Result<RepoEntry, String> {
    let object = value.as_object().ok_or("không phải object")?;
    let id = text_of(object.get("id")).ok_or("thiếu id")?;
    if !ID.is_match(&id) {
        return Err(format!("id {id} không phải tên miền đảo ngược"));
    }
    let version = text_of(object.get("version")).ok_or("thiếu version")?;
    semver::Version::parse(&version).map_err(|_| format!("version {version} không phải semver"))?;
    if let Some(api) = object.get("apiVersion").and_then(Value::as_u64) {
        if api > u64::from(qt_ext::manifest::API_VERSION) {
            return Err(format!("cần app mới hơn (apiVersion {api})"));
        }
    }
    let url = resolve(base, &text_of(object.get("url")).ok_or("thiếu url")?)?;
    let sha256 = text_of(object.get("sha256")).map(|s| s.to_ascii_lowercase()).ok_or("thiếu sha256")?;
    if !SHA256.is_match(&sha256) {
        return Err("sha256 phải là 64 ký tự hex".into());
    }
    let shared = object.get("sharedAuth").and_then(Value::as_bool).unwrap_or(false);
    if shared && !has_auth {
        return Err("sharedAuth nhưng kho không khai auth".into());
    }
    let icon = match text_of(object.get("icon")) {
        Some(link) => Some(resolve(base, &link)?),
        None => None,
    };
    Ok(RepoEntry {
        name: text_of(object.get("name")).unwrap_or_else(|| id.clone()),
        id,
        version,
        description: text_of(object.get("description")).unwrap_or_default(),
        developer: person(object.get("developer")),
        license: text_of(object.get("license")),
        homepage: text_of(object.get("homepage")),
        hosts: strings(object.get("hosts")),
        permissions: strings(object.get("permissions")),
        auth_required: object.get("authRequired").and_then(Value::as_bool).unwrap_or(false),
        shared_auth: shared,
        url,
        sha256,
        size: object.get("size").and_then(Value::as_u64),
        icon,
        updated_at: text_of(object.get("updatedAt")),
    })
}

/// Đọc và kiểm `index.json`. Lỗi cả file → `Err`; mục lỗi bị bỏ, trả kèm cảnh báo.
pub fn parse_index(text: &str, index_url: &Url) -> Result<(RepoIndex, Vec<String>), String> {
    if text.len() > MAX_INDEX {
        return Err("index.json quá 2 MB".into());
    }
    let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("index.json không phải JSON: {e}"))?;
    let object = value.as_object().ok_or("index.json phải là object")?;
    match object.get("formatVersion").and_then(Value::as_u64) {
        None => return Err("index.json thiếu formatVersion".into()),
        Some(v) if v > FORMAT_VERSION => return Err(format!("kho cần app mới hơn (formatVersion {v})")),
        Some(_) => {}
    }
    let id = text_of(object.get("id")).ok_or("index.json thiếu id")?;
    if !ID.is_match(&id) {
        return Err(format!("id kho {id} không phải tên miền đảo ngược"));
    }
    let name = text_of(object.get("name")).ok_or("index.json thiếu name")?;
    let maintainer = person(object.get("maintainer")).ok_or("index.json thiếu maintainer.name")?;
    let auth = match object.get("auth").filter(|v| !v.is_null()) {
        Some(value) => Some(parse_auth_value(value).map_err(|e| format!("auth của kho: {e}"))?),
        None => None,
    };
    let mut warnings = Vec::new();
    let mut extensions: Vec<RepoEntry> = Vec::new();
    for (i, entry) in object.get("extensions").and_then(Value::as_array).into_iter().flatten().enumerate() {
        let label = entry.get("id").and_then(Value::as_str).unwrap_or("?");
        match parse_entry(entry, index_url, auth.is_some()) {
            Ok(parsed) if extensions.iter().any(|e| e.id == parsed.id) => warnings.push(format!("extensions[{i}] ({label}): id bị trùng, bỏ qua")),
            Ok(parsed) => extensions.push(parsed),
            Err(error) => warnings.push(format!("extensions[{i}] ({label}): {error}, bỏ qua")),
        }
    }
    let index = RepoIndex {
        id,
        name,
        description: text_of(object.get("description")).unwrap_or_default(),
        maintainer,
        homepage: text_of(object.get("homepage")),
        auth,
        extensions,
    };
    Ok((index, warnings))
}

/// Link người dùng dán → link `index.json` (thư mục thì thêm `index.json`).
pub fn index_url(input: &str) -> Result<Url, ExtError> {
    let mut text = input.trim().to_string();
    if !text.to_ascii_lowercase().ends_with(".json") {
        if !text.ends_with('/') {
            text.push('/');
        }
        text.push_str("index.json");
    }
    let url = Url::parse(&text).map_err(|e| repo_error(format!("link kho không hợp lệ: {e}")))?;
    if url.scheme() != "https" {
        return Err(repo_error("link kho phải là https://"));
    }
    Ok(url)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoRecord {
    pub id: String,
    pub url: String,
    pub added_at: i64,
    pub refreshed_at: i64,
}

/// Kho cho giao diện.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoView {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub maintainer: Person,
    pub homepage: Option<String>,
    pub has_auth: bool,
    pub auth_required: bool,
    pub extensions: Vec<RepoEntry>,
    pub warnings: Vec<String>,
    pub added_at: i64,
    pub refreshed_at: i64,
    /// Bản đã lưu không đọc được (giao diện vẫn hiện dòng kho để gỡ / làm mới).
    pub error: Option<String>,
}

pub struct Repositories {
    dir: PathBuf,
    fetch: Fetcher,
    /// Kho khoá cho auth chung (test dùng bộ nhớ); `None` = theo máy.
    secrets: Option<SecretBackend>,
}

impl Repositories {
    pub fn new(extensions_dir: &Path) -> Repositories {
        Repositories { dir: extensions_dir.to_path_buf(), fetch: Arc::new(fetch_zip), secrets: None }
    }

    pub fn with_fetcher(mut self, fetch: Fetcher) -> Repositories {
        self.fetch = fetch;
        self
    }

    pub fn with_secrets(mut self, secrets: SecretBackend) -> Repositories {
        self.secrets = Some(secrets);
        self
    }

    fn records_path(&self) -> PathBuf {
        self.dir.join(RECORDS)
    }

    fn cache_path(&self, id: &str) -> PathBuf {
        self.dir.join(CACHE_DIR).join(format!("{id}.json"))
    }

    pub fn records(&self) -> Vec<RepoRecord> {
        std::fs::read_to_string(self.records_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    fn save_records(&self, records: &[RepoRecord]) -> Result<(), ExtError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| repo_error(format!("{}: {e}", self.dir.display())))?;
        let json = serde_json::to_string_pretty(records).map_err(|e| repo_error(e.to_string()))?;
        crate::story_fs::write_atomic(&self.records_path(), &format!("{json}\n")).map_err(|e| repo_error(e.to_string()))
    }

    fn save_cache(&self, id: &str, text: &str) -> Result<(), ExtError> {
        let path = self.cache_path(id);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| repo_error(format!("{}: {e}", dir.display())))?;
        }
        crate::story_fs::write_atomic(&path, text).map_err(|e| repo_error(e.to_string()))
    }

    /// `index.json` đã lưu của kho, đọc lại theo link của kho.
    pub fn index(&self, record: &RepoRecord) -> Result<(RepoIndex, Vec<String>), String> {
        let text = std::fs::read_to_string(self.cache_path(&record.id)).map_err(|_| "chưa tải được index.json của kho".to_string())?;
        let url = Url::parse(&record.url).map_err(|e| e.to_string())?;
        parse_index(&text, &url)
    }

    fn view(&self, record: &RepoRecord) -> RepoView {
        let base = RepoView {
            id: record.id.clone(),
            url: record.url.clone(),
            name: record.id.clone(),
            description: String::new(),
            maintainer: Person::default(),
            homepage: None,
            has_auth: false,
            auth_required: false,
            extensions: Vec::new(),
            warnings: Vec::new(),
            added_at: record.added_at,
            refreshed_at: record.refreshed_at,
            error: None,
        };
        match self.index(record) {
            Err(error) => RepoView { error: Some(error), ..base },
            Ok((index, warnings)) => RepoView {
                name: index.name,
                description: index.description,
                maintainer: index.maintainer,
                homepage: index.homepage,
                has_auth: index.auth.is_some(),
                auth_required: index.auth.as_ref().is_some_and(|a| a.required),
                extensions: index.extensions,
                warnings,
                ..base
            },
        }
    }

    pub fn list(&self) -> Vec<RepoView> {
        self.records().iter().map(|r| self.view(r)).collect()
    }

    fn download_index(&self, url: &Url) -> Result<(String, RepoIndex), ExtError> {
        let bytes = (self.fetch)(url.as_str())?;
        if bytes.len() > MAX_INDEX {
            return Err(repo_error("index.json quá 2 MB"));
        }
        let text = String::from_utf8(bytes).map_err(|_| repo_error("index.json không phải UTF-8"))?;
        let (index, _) = parse_index(&text, url).map_err(repo_error)?;
        Ok((text, index))
    }

    /// Thêm kho từ link (link `index.json` hoặc thư mục chứa nó).
    pub fn add(&self, input: &str) -> Result<RepoView, ExtError> {
        let url = index_url(input)?;
        let (text, index) = self.download_index(&url)?;
        let mut records = self.records();
        if let Some(existing) = records.iter().find(|r| r.id == index.id) {
            return Err(repo_error(format!("đã có kho {} (id {}, link {})", index.name, index.id, existing.url)));
        }
        self.save_cache(&index.id, &text)?;
        let stamp = now();
        let record = RepoRecord { id: index.id.clone(), url: url.to_string(), added_at: stamp, refreshed_at: stamp };
        records.push(record.clone());
        self.save_records(&records)?;
        Ok(self.view(&record))
    }

    /// Tải lại `index.json`; lỗi thì giữ bản cũ.
    pub fn refresh(&self, id: &str) -> Result<RepoView, ExtError> {
        let mut records = self.records();
        let record = records.iter_mut().find(|r| r.id == id).ok_or_else(|| repo_error(format!("chưa thêm kho {id}")))?;
        let url = Url::parse(&record.url).map_err(|e| repo_error(e.to_string()))?;
        let (text, index) = self.download_index(&url)?;
        if index.id != id {
            return Err(repo_error(format!("link kho giờ trả kho khác (id {})", index.id)));
        }
        self.save_cache(id, &text)?;
        record.refreshed_at = now();
        let record = record.clone();
        self.save_records(&records)?;
        Ok(self.view(&record))
    }

    /// Làm mới mọi kho; trả lỗi của từng kho ("<id>: <lỗi>").
    pub fn refresh_all(&self) -> Vec<String> {
        self.records().iter().filter_map(|r| self.refresh(&r.id).err().map(|e| format!("{}: {e}", r.id))).collect()
    }

    fn auth_store(&self, config_root: &Path, repo_id: &str, spec: AuthSpec) -> AuthStore {
        match &self.secrets {
            Some(secrets) => AuthStore::repository(config_root, repo_id, spec, secrets.clone()),
            None => AuthStore::repository_on_platform(config_root, repo_id, spec),
        }
    }

    /// Auth chung của kho (theo `index.json` đã lưu); `None` khi kho không khai auth.
    pub fn repo_auth(&self, config_root: &Path, repo_id: &str) -> Option<AuthStore> {
        let record = self.records().into_iter().find(|r| r.id == repo_id)?;
        let (index, _) = self.index(&record).ok()?;
        Some(self.auth_store(config_root, repo_id, index.auth?))
    }

    /// Gỡ kho: xoá khỏi danh sách, xoá `index.json` đã lưu và auth chung. Extension đã cài từ kho vẫn giữ (về auth
    /// riêng, hết cập nhật theo kho).
    pub fn remove(&self, id: &str, config_root: &Path) -> Result<(), ExtError> {
        let mut records = self.records();
        if !records.iter().any(|r| r.id == id) {
            return Err(repo_error(format!("chưa thêm kho {id}")));
        }
        if let Some(auth) = self.repo_auth(config_root, id) {
            auth.clear(None)?;
        }
        let _ = std::fs::remove_dir_all(config_root.join("extension-data").join(format!("repo-{id}")));
        let _ = std::fs::remove_file(self.cache_path(id));
        records.retain(|r| r.id != id);
        self.save_records(&records)
    }

    pub fn entry(&self, repo_id: &str, ext_id: &str) -> Result<RepoEntry, ExtError> {
        let record = self.records().into_iter().find(|r| r.id == repo_id).ok_or_else(|| repo_error(format!("chưa thêm kho {repo_id}")))?;
        let (index, _) = self.index(&record).map_err(repo_error)?;
        index.extensions.into_iter().find(|e| e.id == ext_id).ok_or_else(|| repo_error(format!("kho {repo_id} không có extension {ext_id}")))
    }

    /// Tải gói của một extension trong kho, kiểm sha256 và manifest khớp kho, giải nén chờ người dùng đồng ý quyền.
    pub fn inspect_entry(&self, registry: &Registry, repo_id: &str, ext_id: &str) -> Result<(Inspection, Source), ExtError> {
        let entry = self.entry(repo_id, ext_id)?;
        let bytes = (self.fetch)(&entry.url)?;
        if sha256_hex(&bytes) != entry.sha256 {
            return Err(repo_error(format!("gói {} {} trên kho không khớp sha256 trong index.json — kho chưa cập nhật xong hoặc gói bị sửa", entry.id, entry.version)));
        }
        let inspection = registry.inspect(&bytes)?;
        let manifest = &inspection.package.manifest;
        if manifest.id != entry.id || manifest.version.to_string() != entry.version {
            return Err(repo_error(format!("gói chứa {} {}, kho ghi {} {}", manifest.id, manifest.version, entry.id, entry.version)));
        }
        Ok((inspection, Source::Repository { repository: repo_id.to_string(), url: entry.url }))
    }

    /// Kho dữ liệu của extension `id`: dùng auth chung của kho khi extension được cài từ chính kho đó và kho đánh
    /// dấu `sharedAuth`; còn lại auth riêng.
    pub fn ext_store(&self, config_root: &Path, id: &str) -> ExtStore {
        let own = match &self.secrets {
            Some(secrets) => ExtStore::with_backend(config_root, id, secrets.clone()),
            None => ExtStore::new(config_root, id),
        };
        let installed = Registry::new(&self.dir).entries().into_iter().find(|e| e.id == id);
        let Some(Source::Repository { repository, .. }) = installed.map(|e| e.source) else {
            return own;
        };
        let Some(record) = self.records().into_iter().find(|r| r.id == repository) else {
            return own;
        };
        let Ok((index, _)) = self.index(&record) else {
            return own;
        };
        match (index.auth, index.extensions.iter().any(|e| e.id == id && e.shared_auth)) {
            (Some(spec), true) => own.with_shared_auth(self.auth_store(config_root, &repository, spec)),
            _ => own,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::Mutex;

    const BASE: &str = "https://kho.test/";

    fn zip(id: &str, version: &str, hosts: &[&str]) -> Vec<u8> {
        let manifest = serde_json::json!({
            "id": id, "name": id, "version": version, "apiVersion": 1, "description": "d", "developer": { "name": "dev" },
            "match": [r"^https://a\.test/"], "hosts": hosts,
            "auth": { "required": true, "schemes": [{ "id": "apiKey", "label": "Key", "fields": [{ "key": "apiKey", "label": "K", "type": "secret" }] }] }
        });
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(manifest.to_string().as_bytes()).unwrap();
        writer.start_file("main.js", options).unwrap();
        writer.write_all(b"export async function toc() { return []; }").unwrap();
        writer.finish().unwrap().into_inner()
    }

    struct Kho {
        files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl Kho {
        fn new() -> Kho {
            Kho { files: Arc::default() }
        }

        fn put(&self, path: &str, bytes: Vec<u8>) {
            self.files.lock().unwrap().insert(format!("{BASE}{path}"), bytes);
        }

        /// Đăng index với các mục (id, version, hosts, sharedAuth); đưa zip lên theo đường dẫn chuẩn.
        fn publish(&self, id: &str, entries: &[(&str, &str, &[&str], bool)], extra: Vec<Value>) {
            let mut list: Vec<Value> = entries
                .iter()
                .map(|(ext, version, hosts, shared)| {
                    let bytes = zip(ext, version, hosts);
                    let path = format!("extensions/{ext}/{version}.zip");
                    let entry = serde_json::json!({ "id": ext, "name": ext, "version": version, "url": path, "sha256": sha256_hex(&bytes), "sharedAuth": shared, "hosts": hosts, "authRequired": true });
                    self.put(&path, bytes);
                    entry
                })
                .collect();
            list.extend(extra);
            let index = serde_json::json!({
                "formatVersion": 1, "id": id, "name": "Kho thử", "maintainer": { "name": "dev" },
                "auth": { "required": true, "schemes": [{ "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "K", "type": "secret" }] }] },
                "extensions": list
            });
            self.put("index.json", index.to_string().into_bytes());
        }

        fn repos(&self, dir: &Path, secrets: &SecretBackend) -> Repositories {
            let files = self.files.clone();
            Repositories::new(dir)
                .with_fetcher(Arc::new(move |url| files.lock().unwrap().get(url).cloned().ok_or_else(|| ExtError::Network(format!("404 {url}")))))
                .with_secrets(secrets.clone())
        }
    }

    #[test]
    fn doc_index_bo_muc_loi_va_tu_choi_file_hong() {
        let base = Url::parse("https://kho.test/a/index.json").unwrap();
        let sha = "a".repeat(64);
        let text = serde_json::json!({
            "formatVersion": 1, "id": "org.kho", "name": "Kho", "maintainer": { "name": "dev" },
            "extensions": [
                { "id": "org.kho.a", "version": "1.0.0", "url": "x/a.zip", "sha256": sha, "icon": "x/a.png" },
                { "id": "org.kho.a", "version": "1.0.1", "url": "x/a2.zip", "sha256": sha },
                { "id": "Sai", "version": "1.0.0", "url": "x.zip", "sha256": sha },
                { "id": "org.kho.b", "version": "1.0", "url": "x.zip", "sha256": sha },
                { "id": "org.kho.c", "version": "1.0.0", "url": "http://kho.test/c.zip", "sha256": sha },
                { "id": "org.kho.d", "version": "1.0.0", "url": "d.zip", "sha256": "abc" },
                { "id": "org.kho.e", "version": "1.0.0", "url": "e.zip", "sha256": sha, "sharedAuth": true },
                { "id": "org.kho.f", "version": "1.0.0", "url": "f.zip", "sha256": sha, "apiVersion": 9 }
            ]
        })
        .to_string();
        let (index, warnings) = parse_index(&text, &base).unwrap();
        assert_eq!(index.extensions.len(), 1);
        assert_eq!(index.extensions[0].url, "https://kho.test/a/x/a.zip", "link tương đối ghép theo link index.json");
        assert_eq!(index.extensions[0].icon.as_deref(), Some("https://kho.test/a/x/a.png"));
        assert_eq!(warnings.len(), 7, "{warnings:?}");
        for needle in ["trùng", "tên miền", "semver", "https://", "64 ký tự", "sharedAuth", "app mới hơn"] {
            assert!(warnings.iter().any(|w| w.contains(needle)), "{needle}: {warnings:?}");
        }
        assert!(parse_index(r#"{"formatVersion":2,"id":"org.kho","name":"K","maintainer":{"name":"d"}}"#, &base).unwrap_err().contains("app mới hơn"));
        assert!(parse_index(r#"{"id":"org.kho","name":"K","maintainer":{"name":"d"}}"#, &base).unwrap_err().contains("formatVersion"));
        assert!(parse_index("[]", &base).is_err());
        assert_eq!(index_url("https://kho.example.org").unwrap().as_str(), "https://kho.example.org/index.json");
        assert_eq!(index_url("https://x.test/kho/main.json").unwrap().as_str(), "https://x.test/kho/main.json");
        assert!(index_url("http://x.test/").is_err());
    }

    #[test]
    fn them_kho_cai_tu_kho_kiem_sha_va_cap_nhat() {
        let dir = tempfile::tempdir().unwrap();
        let secrets = SecretBackend::memory();
        let kho = Kho::new();
        kho.publish("org.kho", &[("org.kho.a", "1.0.0", &["a.test"], true), ("org.kho.b", "1.0.0", &["b.test"], false)], vec![]);
        let repos = kho.repos(dir.path(), &secrets);
        let registry = Registry::new(dir.path());

        let view = repos.add(BASE).unwrap();
        assert_eq!((view.id.as_str(), view.extensions.len(), view.has_auth), ("org.kho", 2, true));
        assert!(repos.add(&format!("{BASE}index.json")).unwrap_err().to_string().contains("đã có kho"));

        let (inspection, source) = repos.inspect_entry(&registry, "org.kho", "org.kho.a").unwrap();
        assert!(inspection.needs_consent);
        assert_eq!(source, Source::Repository { repository: "org.kho".into(), url: format!("{BASE}extensions/org.kho.a/1.0.0.zip") });
        registry.install(&inspection.sha256, source).unwrap();
        let (b, source) = repos.inspect_entry(&registry, "org.kho", "org.kho.b").unwrap();
        registry.install(&b.sha256, source).unwrap();

        // Gói bị thay mà index chưa đổi → từ chối.
        kho.put("extensions/org.kho.a/1.0.0.zip", zip("org.kho.a", "1.0.0", &["evil.test"]));
        assert!(repos.inspect_entry(&registry, "org.kho", "org.kho.a").unwrap_err().to_string().contains("sha256"));
        // Index ghi version khác gói → từ chối.
        let bytes = zip("org.kho.a", "2.0.0", &["a.test"]);
        let entry = serde_json::json!({ "id": "org.kho.c", "version": "1.0.0", "url": "c.zip", "sha256": sha256_hex(&bytes) });
        kho.put("c.zip", bytes);
        kho.publish("org.kho", &[("org.kho.a", "1.1.0", &["a.test", "cdn.a.test"], true)], vec![entry]);
        repos.refresh("org.kho").unwrap();
        assert!(repos.inspect_entry(&registry, "org.kho", "org.kho.c").unwrap_err().to_string().contains("kho ghi org.kho.c"));
        let (update, _) = repos.inspect_entry(&registry, "org.kho", "org.kho.a").unwrap();
        assert!(update.newer && update.needs_consent && update.new_hosts == ["cdn.a.test"], "cập nhật thêm host cần đồng ý");

        // Kho đổi id ở cùng link → làm mới từ chối, giữ bản cũ.
        kho.publish("org.khac", &[], vec![]);
        assert!(repos.refresh("org.kho").unwrap_err().to_string().contains("kho khác"));
        assert_eq!(repos.list()[0].extensions.len(), 2, "giữ index cũ");
        assert!(repos.refresh_all().len() == 1);
    }

    #[test]
    fn auth_chung_chi_cho_extension_cai_tu_kho() {
        let dir = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        let secrets = SecretBackend::memory();
        let kho = Kho::new();
        kho.publish("org.kho", &[("org.kho.a", "1.0.0", &["a.test"], true), ("org.kho.b", "1.0.0", &["b.test"], false)], vec![]);
        let repos = kho.repos(dir.path(), &secrets);
        let registry = Registry::new(dir.path());
        repos.add(BASE).unwrap();
        for id in ["org.kho.a", "org.kho.b"] {
            let (i, s) = repos.inspect_entry(&registry, "org.kho", id).unwrap();
            registry.install(&i.sha256, s).unwrap();
        }
        let auth = repos.repo_auth(config.path(), "org.kho").unwrap();
        auth.save(None, "apiKey", serde_json::json!({ "apiKey": "u:s" }).as_object().unwrap()).unwrap();

        let a = repos.ext_store(config.path(), "org.kho.a");
        let manifest_a = registry.list_enabled().0.into_iter().find(|e| e.package.manifest.id == "org.kho.a").unwrap().package.manifest;
        assert_eq!(a.auth(&manifest_a).unwrap().unwrap().values["apiKey"], serde_json::json!("u:s"));
        let b = repos.ext_store(config.path(), "org.kho.b");
        assert!(b.shared_auth().is_none(), "kho không đánh dấu sharedAuth cho b");

        // Cài cùng id từ file thì không còn nguồn kho → không đọc key kho.
        let file = registry.inspect(&zip("org.kho.a", "1.0.1", &["a.test"])).unwrap();
        registry.install(&file.sha256, Source::File { path: "x.zip".into() }).unwrap();
        assert!(repos.ext_store(config.path(), "org.kho.a").shared_auth().is_none());

        repos.remove("org.kho", config.path()).unwrap();
        assert!(repos.list().is_empty());
        assert!(repos.repo_auth(config.path(), "org.kho").is_none());
        assert!(!config.path().join("extension-data/repo-org.kho").exists());
    }
}
