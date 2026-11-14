//! Extension đã cài (spec mục 7, 11). Sổ `<thư mục extension>/installed.json` ghi nguồn cài, sha256, quyền đã cấp
//! và bật/tắt; gói giải nén ở `<id>/pkg/`. Cài qua `.staging/<sha256>/`: kiểm manifest, người dùng đồng ý quyền,
//! rồi mới đổi chỗ vào `pkg/`. Thư mục dev chạy thẳng từ chỗ đang viết, đọc lại mỗi lượt. Thư mục `<id>/pkg/` đặt
//! tay mà sổ chưa biết (mốc 2) coi như đã cài, nguồn "đặt tay".

use qt_ext::{extract_zip, sha256_hex, ExtError, ExtStore, Package, Permission};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const INDEX: &str = "installed.json";
const STAGING: &str = ".staging";
const STAGING_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone)]
pub struct Installed {
    pub package: Package,
    pub dir: PathBuf,
    /// Nạp ở chế độ developer (được gọi `localhost`).
    pub dev: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Source {
    File { path: String },
    Link { url: String },
    Dev { dir: String },
    /// Cài từ kho extension (`repository` = id kho, `url` = link zip).
    Repository { repository: String, url: String },
    /// Thư mục đặt tay vào `<id>/pkg/`.
    Manual,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledEntry {
    pub id: String,
    pub version: String,
    pub source: Source,
    #[serde(default)]
    pub sha256: String,
    /// Domain và quyền người dùng đã đồng ý lúc cài.
    #[serde(default)]
    pub hosts: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub installed_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

/// Gói vừa kiểm, chờ người dùng đồng ý quyền.
#[derive(Debug, Clone)]
pub struct Inspection {
    pub package: Package,
    pub sha256: String,
    /// Bản đang cài cùng id (cập nhật).
    pub current: Option<InstalledEntry>,
    /// So với quyền đã cấp của bản đang cài; cài mới thì là mọi domain / quyền.
    pub new_hosts: Vec<String>,
    pub new_permissions: Vec<String>,
    /// Bản trong gói mới hơn bản đang cài (cài mới cũng tính là mới hơn).
    pub newer: bool,
    /// Phải hỏi người dùng: cài mới, hoặc cập nhật thêm domain / quyền.
    pub needs_consent: bool,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn store_error(message: impl Into<String>) -> ExtError {
    ExtError::Package(message.into())
}

fn permission_names(package: &Package) -> Vec<String> {
    package.manifest.permissions.iter().map(|p| match p {
        Permission::Webview => "webview".to_string(),
    }).collect()
}

fn remove_dir(path: &Path) -> Result<(), ExtError> {
    match std::fs::remove_dir_all(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(store_error(format!("không xoá được {}: {e}", path.display()))),
        _ => Ok(()),
    }
}

pub struct Registry {
    dir: PathBuf,
}

impl Registry {
    pub fn new(extensions_dir: &Path) -> Registry {
        Registry { dir: extensions_dir.to_path_buf() }
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join(INDEX)
    }

    fn staging(&self, sha256: &str) -> PathBuf {
        self.dir.join(STAGING).join(sha256)
    }

    fn saved(&self) -> Vec<InstalledEntry> {
        std::fs::read_to_string(self.index_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    fn save(&self, entries: &[InstalledEntry]) -> Result<(), ExtError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| store_error(format!("{}: {e}", self.dir.display())))?;
        let json = serde_json::to_string_pretty(entries).map_err(|e| store_error(e.to_string()))?;
        crate::story_fs::write_atomic(&self.index_path(), &format!("{json}\n")).map_err(|e| store_error(e.to_string()))
    }

    /// Mọi extension trong sổ, cộng thư mục `<id>/pkg/` đặt tay mà sổ chưa có.
    pub fn entries(&self) -> Vec<InstalledEntry> {
        let mut entries = self.saved();
        let Ok(dirs) = std::fs::read_dir(&self.dir) else {
            return entries;
        };
        let mut manual: Vec<String> = dirs
            .flatten()
            .filter(|d| d.path().join("pkg").is_dir())
            .map(|d| d.file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.') && !entries.iter().any(|e| &e.id == name))
            .collect();
        manual.sort();
        for id in manual {
            entries.push(InstalledEntry { id, version: String::new(), source: Source::Manual, sha256: String::new(), hosts: vec![], permissions: vec![], enabled: true, installed_at: 0, updated_at: 0 });
        }
        entries
    }

    fn load(&self, entry: &InstalledEntry) -> Result<Installed, String> {
        let (package, dir, dev) = match &entry.source {
            Source::Dev { dir } => (Package::load_dir(Path::new(dir), true), PathBuf::from(dir), true),
            _ => {
                let dir = self.dir.join(&entry.id);
                (Package::load_dir(&dir.join("pkg"), false), dir, false)
            }
        };
        let package = package.map_err(|e| e.to_string())?;
        if package.manifest.id != entry.id {
            return Err(format!("id trong manifest là {}, khác thư mục đã cài", package.manifest.id));
        }
        Ok(Installed { package, dir, dev })
    }

    /// Cho trang quản lý: cả extension đang tắt và extension nạp lỗi.
    pub fn list_all(&self) -> Vec<(InstalledEntry, Result<Installed, String>)> {
        self.entries().into_iter().map(|entry| {
            let loaded = self.load(&entry);
            (entry, loaded)
        }).collect()
    }

    /// Extension đang bật nạp được, và lỗi từng cái ("<id>: <lỗi>").
    pub fn list_enabled(&self) -> (Vec<Installed>, Vec<String>) {
        let mut installed = Vec::new();
        let mut errors = Vec::new();
        for (entry, loaded) in self.list_all() {
            if !entry.enabled {
                continue;
            }
            match loaded {
                Ok(ext) => installed.push(ext),
                Err(error) => errors.push(format!("{}: {error}", entry.id)),
            }
        }
        (installed, errors)
    }

    fn clean_staging(&self) {
        let Ok(dirs) = std::fs::read_dir(self.dir.join(STAGING)) else {
            return;
        };
        for dir in dirs.flatten() {
            let old = dir.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age > STAGING_TTL);
            if old {
                let _ = std::fs::remove_dir_all(dir.path());
            }
        }
    }

    /// Giải nén gói vào `.staging/<sha256>/`, kiểm manifest, so quyền với bản đang cài.
    pub fn inspect(&self, bytes: &[u8]) -> Result<Inspection, ExtError> {
        self.clean_staging();
        let sha256 = sha256_hex(bytes);
        let staging = self.staging(&sha256);
        remove_dir(&staging)?;
        extract_zip(bytes, &staging)?;
        let package = match Package::load_dir(&staging, false) {
            Ok(package) => package,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging);
                return Err(error);
            }
        };
        let current = self.entries().into_iter().find(|e| e.id == package.manifest.id);
        if let Some(Source::Dev { dir }) = current.as_ref().map(|c| &c.source) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(store_error(format!("đang nạp thư mục dev {dir} cùng id — gỡ bản dev trước")));
        }
        let permissions = permission_names(&package);
        let granted_hosts = current.as_ref().map(|c| c.hosts.clone()).unwrap_or_default();
        let granted_permissions = current.as_ref().map(|c| c.permissions.clone()).unwrap_or_default();
        let new_hosts: Vec<String> = package.manifest.hosts.iter().filter(|h| !granted_hosts.contains(h)).cloned().collect();
        let new_permissions: Vec<String> = permissions.into_iter().filter(|p| !granted_permissions.contains(p)).collect();
        let newer = match &current {
            None => true,
            Some(c) => semver::Version::parse(&c.version).map_or(true, |v| package.manifest.version > v),
        };
        let needs_consent = current.is_none() || !new_hosts.is_empty() || !new_permissions.is_empty();
        Ok(Inspection { package, sha256, current, new_hosts, new_permissions, newer, needs_consent })
    }

    /// Cài gói đã `inspect` (người dùng đã đồng ý): đổi chỗ `.staging/<sha256>/` vào `<id>/pkg/`.
    pub fn install(&self, sha256: &str, source: Source) -> Result<InstalledEntry, ExtError> {
        let staging = self.staging(sha256);
        if !staging.is_dir() {
            return Err(store_error("gói đã hết hạn chờ — chọn lại file hoặc link"));
        }
        let package = Package::load_dir(&staging, false)?;
        let id = package.manifest.id.clone();
        let mut entries = self.saved();
        let current = self.entries().into_iter().find(|e| e.id == id);
        if matches!(current.as_ref().map(|c| &c.source), Some(Source::Dev { .. })) {
            return Err(store_error("đang nạp thư mục dev cùng id — gỡ bản dev trước"));
        }
        let home = self.dir.join(&id);
        std::fs::create_dir_all(&home).map_err(|e| store_error(format!("{}: {e}", home.display())))?;
        let target = home.join("pkg");
        let old = home.join(format!("pkg.old-{}", now()));
        if target.exists() {
            std::fs::rename(&target, &old).map_err(|e| store_error(format!("không thay được bản cũ: {e}")))?;
        }
        if let Err(e) = std::fs::rename(&staging, &target) {
            let _ = std::fs::rename(&old, &target);
            return Err(store_error(format!("không cài được: {e}")));
        }
        let _ = std::fs::remove_dir_all(&old);
        let stamp = now();
        let entry = InstalledEntry {
            id: id.clone(),
            version: package.manifest.version.to_string(),
            source,
            sha256: sha256.to_string(),
            hosts: package.manifest.hosts.clone(),
            permissions: permission_names(&package),
            enabled: current.as_ref().is_none_or(|c| c.enabled),
            installed_at: current.as_ref().map(|c| c.installed_at).filter(|t| *t > 0).unwrap_or(stamp),
            updated_at: stamp,
        };
        entries.retain(|e| e.id != id);
        entries.push(entry.clone());
        self.save(&entries)?;
        Ok(entry)
    }

    /// Gỡ: xoá gói (thư mục dev thì không đụng), cài đặt, auth, session, cache, hồ sơ webview.
    pub fn uninstall(&self, id: &str, store: &ExtStore) -> Result<(), ExtError> {
        let entry = self.entries().into_iter().find(|e| e.id == id).ok_or_else(|| store_error(format!("chưa cài {id}")))?;
        match self.load(&entry) {
            Ok(installed) => store.remove_all(&installed.package.manifest)?,
            // Gói hỏng không đọc được manifest: vẫn xoá thư mục dữ liệu.
            Err(_) => remove_dir(store.data_dir())?,
        }
        remove_dir(&self.dir.join(id))?;
        let mut entries = self.saved();
        entries.retain(|e| e.id != id);
        self.save(&entries)
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), ExtError> {
        let mut entries = self.entries();
        let entry = entries.iter_mut().find(|e| e.id == id).ok_or_else(|| store_error(format!("chưa cài {id}")))?;
        entry.enabled = enabled;
        self.save(&entries)
    }

    /// Chế độ developer: chạy extension từ thư mục đang viết (được khai `localhost`).
    pub fn add_dev(&self, dir: &Path) -> Result<InstalledEntry, ExtError> {
        let package = Package::load_dir(dir, true)?;
        let id = package.manifest.id.clone();
        let mut entries = self.entries();
        if entries.iter().any(|e| e.id == id && !matches!(e.source, Source::Dev { .. })) {
            return Err(store_error(format!("đã cài extension {id} — gỡ bản đã cài trước khi nạp thư mục")));
        }
        let stamp = now();
        let entry = InstalledEntry {
            id: id.clone(),
            version: package.manifest.version.to_string(),
            source: Source::Dev { dir: dir.display().to_string() },
            sha256: String::new(),
            hosts: package.manifest.hosts.clone(),
            permissions: permission_names(&package),
            enabled: true,
            installed_at: stamp,
            updated_at: stamp,
        };
        entries.retain(|e| e.id != id);
        entries.push(entry.clone());
        self.save(&entries)?;
        Ok(entry)
    }
}

/// Extension đang bật, và lỗi của từng extension nạp hỏng ("<id>: <lỗi>").
pub fn list_installed(extensions_dir: &Path) -> (Vec<Installed>, Vec<String>) {
    Registry::new(extensions_dir).list_enabled()
}

pub fn candidates<'a>(installed: &'a [Installed], url: &str) -> Vec<&'a Installed> {
    if url.trim().is_empty() {
        return vec![];
    }
    installed.iter().filter(|e| e.package.manifest.matches_url(url)).collect()
}

/// Extension đã chọn mà còn khớp link thì dùng; không thì chỉ khi có đúng một ứng viên.
pub fn pick<'a>(installed: &'a [Installed], url: &str, chosen: &str) -> Option<&'a Installed> {
    let list = candidates(installed, url);
    if let Some(found) = list.iter().find(|e| e.package.manifest.id == chosen.trim()) {
        return Some(found);
    }
    match list.as_slice() {
        [only] => Some(only),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn manifest(id: &str, version: &str, hosts: &[&str], webview: bool) -> String {
        serde_json::json!({
            "id": id, "name": id, "version": version, "apiVersion": 1, "description": "d",
            "developer": { "name": "dev" }, "match": [r"^https://a\.test/"], "hosts": hosts,
            "permissions": if webview { vec!["webview"] } else { vec![] }
        })
        .to_string()
    }

    fn zip(manifest: &str) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(manifest.as_bytes()).unwrap();
        writer.start_file("main.js", options).unwrap();
        writer.write_all(b"export async function toc() { return []; }").unwrap();
        writer.finish().unwrap().into_inner()
    }

    fn install_manual(root: &Path, folder: &str, id: &str, pattern: &str) {
        let pkg = root.join(folder).join("pkg");
        std::fs::create_dir_all(&pkg).unwrap();
        let manifest = serde_json::json!({
            "id": id, "name": id, "version": "1.0.0", "apiVersion": 1, "description": "d",
            "developer": { "name": "dev" }, "match": [pattern], "hosts": ["demo.test"]
        });
        std::fs::write(pkg.join("manifest.json"), manifest.to_string()).unwrap();
        std::fs::write(pkg.join("main.js"), "export async function toc() { return []; }").unwrap();
    }

    #[test]
    fn doc_thu_muc_va_chon_theo_link() {
        let dir = tempfile::tempdir().unwrap();
        install_manual(dir.path(), "io.github.dev.a", "io.github.dev.a", r"^https://a\.test/");
        install_manual(dir.path(), "io.github.dev.b", "io.github.dev.b", r"^https://(a|b)\.test/");
        std::fs::create_dir_all(dir.path().join("hong/pkg")).unwrap();
        std::fs::write(dir.path().join("hong/pkg/manifest.json"), "{").unwrap();
        std::fs::create_dir_all(dir.path().join("khong-pkg")).unwrap();
        let (list, errors) = list_installed(dir.path());
        assert_eq!(list.len(), 2);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("hong: "));

        assert_eq!(candidates(&list, "https://a.test/book/1").len(), 2);
        assert_eq!(pick(&list, "https://a.test/book/1", "").map(|e| e.package.manifest.id.as_str()), None, "hai ứng viên chưa chọn");
        assert_eq!(pick(&list, "https://a.test/book/1", "io.github.dev.b").map(|e| e.package.manifest.id.as_str()), Some("io.github.dev.b"));
        assert_eq!(pick(&list, "https://b.test/book/1", "io.github.dev.a").map(|e| e.package.manifest.id.as_str()), Some("io.github.dev.b"), "đã chọn mà không còn khớp thì lấy ứng viên duy nhất");
        assert!(pick(&list, "https://c.test/", "").is_none());
        assert!(candidates(&list, " ").is_empty());
        assert!(list_installed(&dir.path().join("không có")).0.is_empty());
    }

    #[test]
    fn cai_cap_nhat_hoi_lai_quyen_tat_va_go() {
        let dir = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        let registry = Registry::new(dir.path());
        let id = "io.github.dev.ext";

        let first = registry.inspect(&zip(&manifest(id, "1.0.0", &["a.test"], false))).unwrap();
        assert!(first.needs_consent && first.newer && first.current.is_none());
        assert_eq!(first.new_hosts, ["a.test"]);
        let entry = registry.install(&first.sha256, Source::Link { url: "https://x/e.zip".into() }).unwrap();
        assert_eq!((entry.version.as_str(), entry.enabled), ("1.0.0", true));
        assert!(!dir.path().join(".staging").join(&first.sha256).exists());
        assert_eq!(list_installed(dir.path()).0.len(), 1);

        let same = registry.inspect(&zip(&manifest(id, "1.0.0", &["a.test"], false))).unwrap();
        assert!(!same.newer && !same.needs_consent);
        let patch = registry.inspect(&zip(&manifest(id, "1.0.1", &["a.test"], false))).unwrap();
        assert!(patch.newer && !patch.needs_consent, "cập nhật không thêm gì thì cài luôn");
        let wider = registry.inspect(&zip(&manifest(id, "1.1.0", &["a.test", "cdn.a.test"], true))).unwrap();
        assert!(wider.needs_consent);
        assert_eq!((wider.new_hosts.clone(), wider.new_permissions.clone()), (vec!["cdn.a.test".to_string()], vec!["webview".to_string()]));
        let updated = registry.install(&wider.sha256, Source::Link { url: "https://x/e.zip".into() }).unwrap();
        assert_eq!(updated.installed_at, entry.installed_at);
        assert_eq!(list_installed(dir.path()).0[0].package.manifest.version.to_string(), "1.1.0");
        assert!(registry.install("khong-co", Source::Manual).is_err());

        registry.set_enabled(id, false).unwrap();
        assert!(list_installed(dir.path()).0.is_empty(), "tắt thì không còn là ứng viên");
        assert_eq!(registry.list_all().len(), 1);

        let store = ExtStore::with_backend(config.path(), id, qt_ext::SecretBackend::memory());
        std::fs::create_dir_all(store.data_dir()).unwrap();
        registry.uninstall(id, &store).unwrap();
        assert!(!dir.path().join(id).exists());
        assert!(!store.data_dir().exists());
        assert!(registry.entries().is_empty());
    }

    #[test]
    fn nap_thu_muc_dev() {
        let dir = tempfile::tempdir().unwrap();
        let dev = tempfile::tempdir().unwrap();
        let registry = Registry::new(dir.path());
        std::fs::write(dev.path().join("manifest.json"), manifest("io.github.dev.local", "0.1.0", &["127.0.0.1"], false)).unwrap();
        std::fs::write(dev.path().join("main.js"), "export async function toc() { return []; }").unwrap();
        registry.add_dev(dev.path()).unwrap();
        let (list, errors) = list_installed(dir.path());
        assert!(errors.is_empty(), "{errors:?}");
        assert!(list[0].dev, "thư mục dev được khai 127.0.0.1");
        assert!(registry.inspect(&zip(&manifest("io.github.dev.local", "1.0.0", &["a.test"], false))).unwrap_err().to_string().contains("dev"));

        let store = ExtStore::with_backend(dir.path(), "io.github.dev.local", qt_ext::SecretBackend::memory());
        registry.uninstall("io.github.dev.local", &store).unwrap();
        assert!(dev.path().join("main.js").exists(), "gỡ bản dev không xoá thư mục đang viết");

        let installed = registry.inspect(&zip(&manifest("io.github.dev.x", "1.0.0", &["a.test"], false))).unwrap();
        registry.install(&installed.sha256, Source::Manual).unwrap();
        std::fs::write(dev.path().join("manifest.json"), manifest("io.github.dev.x", "0.1.0", &["127.0.0.1"], false)).unwrap();
        assert!(registry.add_dev(dev.path()).unwrap_err().to_string().contains("gỡ bản đã cài"));
    }
}
