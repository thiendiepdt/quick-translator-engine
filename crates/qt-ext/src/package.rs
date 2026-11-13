//! Gói extension trong bộ nhớ: manifest + mọi file JS đọc sẵn lúc nạp. Lúc chạy, module JS chỉ lấy từ đây
//! (`engine::loader`), nên extension không có đường đọc ổ đĩa.

use crate::{parse_manifest, ExtError, Manifest};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub const MAX_FILES: usize = 200;
pub const MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const ENTRY: &str = "main.js";

#[derive(Debug, Clone)]
pub struct Package {
    pub manifest: Manifest,
    /// Đường dẫn tương đối dùng `/` → mã nguồn.
    pub files: Arc<HashMap<String, String>>,
    pub icon: Option<Vec<u8>>,
}

fn package_error(message: impl Into<String>) -> ExtError {
    ExtError::Package(message.into())
}

fn skipped(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules"
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, std::path::PathBuf)>, total: &mut u64) -> Result<(), ExtError> {
    let entries = std::fs::read_dir(dir).map_err(|e| package_error(format!("không đọc được {}: {e}", dir.display())))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if skipped(&name) {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| package_error(e.to_string()))?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            walk(root, &path, out, total)?;
            continue;
        }
        *total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        let relative = path.strip_prefix(root).expect("trong root").to_string_lossy().replace('\\', "/");
        out.push((relative, path));
        if out.len() > MAX_FILES {
            return Err(package_error(format!("gói có quá {MAX_FILES} file")));
        }
        if *total > MAX_BYTES {
            return Err(package_error("gói quá 5 MB"));
        }
    }
    Ok(())
}

impl Package {
    /// Nạp từ thư mục (chế độ developer, và gói đã giải nén lúc cài).
    pub fn load_dir(dir: &Path, allow_local: bool) -> Result<Package, ExtError> {
        let mut found = Vec::new();
        let mut total = 0;
        walk(dir, dir, &mut found, &mut total)?;
        let manifest_text = std::fs::read_to_string(dir.join("manifest.json")).map_err(|_| package_error("thiếu manifest.json"))?;
        let manifest = parse_manifest(&manifest_text, allow_local)?;
        let mut files = HashMap::new();
        let mut icon = None;
        for (relative, path) in found {
            if relative.ends_with(".js") || relative.ends_with(".mjs") {
                let source = std::fs::read_to_string(&path).map_err(|e| package_error(format!("{relative}: {e}")))?;
                files.insert(relative, source);
            } else if relative == "icon.png" {
                icon = std::fs::read(&path).ok();
            }
        }
        if !files.contains_key(ENTRY) {
            return Err(package_error("thiếu main.js"));
        }
        Ok(Package { manifest, files: Arc::new(files), icon })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const MANIFEST: &str = r#"{ "id": "io.github.dev.demo", "name": "Demo", "version": "1.0.0", "apiVersion": 1,
        "description": "Mẫu", "developer": { "name": "dev" }, "match": ["^http"], "hosts": ["demo.test"] }"#;

    fn write(dir: &Path, path: &str, text: &str) {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }

    #[test]
    fn nap_thu_muc_bo_qua_an_va_node_modules() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "manifest.json", MANIFEST);
        write(dir.path(), "main.js", "export async function toc() {}");
        write(dir.path(), "lib/a.js", "export const a = 1;");
        write(dir.path(), "node_modules/x.js", "x");
        write(dir.path(), ".git/y.js", "y");
        write(dir.path(), "README.md", "doc");
        let pkg = Package::load_dir(dir.path(), false).unwrap();
        let mut names: Vec<&String> = pkg.files.keys().collect();
        names.sort();
        assert_eq!(names, ["lib/a.js", "main.js"]);
        assert_eq!(pkg.manifest.id, "io.github.dev.demo");
    }

    #[test]
    fn thieu_main_hoac_qua_nhieu_file() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "manifest.json", MANIFEST);
        write(dir.path(), "lib/a.js", "");
        assert!(Package::load_dir(dir.path(), false).unwrap_err().to_string().contains("main.js"));
        write(dir.path(), "main.js", "");
        for i in 0..MAX_FILES {
            write(dir.path(), &format!("many/{i}.js"), "");
        }
        assert!(Package::load_dir(dir.path(), false).unwrap_err().to_string().contains("200"));
    }

    #[test]
    fn thieu_manifest() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "main.js", "");
        assert!(Package::load_dir(dir.path(), false).unwrap_err().to_string().contains("manifest.json"));
    }
}
