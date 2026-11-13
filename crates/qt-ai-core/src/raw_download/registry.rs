//! Extension đã cài: mỗi extension một thư mục `<thư mục extension>/<id>/pkg/` (gói đã giải nén). Cài từ file
//! hay link (mốc 5) cũng ghi vào đúng chỗ này. Chọn extension cho truyện theo `match` của manifest.

use qt_ext::Package;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Installed {
    pub package: Package,
    pub dir: PathBuf,
    /// Nạp ở chế độ developer (được gọi `localhost`).
    pub dev: bool,
}

/// Trả extension nạp được và lỗi của từng thư mục hỏng ("<tên thư mục>: <lỗi>").
pub fn list_installed(extensions_dir: &Path) -> (Vec<Installed>, Vec<String>) {
    let mut installed = Vec::new();
    let mut errors = Vec::new();
    let Ok(entries) = std::fs::read_dir(extensions_dir) else {
        return (installed, errors);
    };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.join("pkg").is_dir()).collect();
    dirs.sort();
    for dir in dirs {
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        match Package::load_dir(&dir.join("pkg"), false) {
            Ok(package) => installed.push(Installed { package, dir, dev: false }),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    (installed, errors)
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

    fn install(root: &Path, folder: &str, id: &str, pattern: &str) {
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
        install(dir.path(), "a", "io.github.dev.a", r"^https://a\.test/");
        install(dir.path(), "b", "io.github.dev.b", r"^https://(a|b)\.test/");
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
}
