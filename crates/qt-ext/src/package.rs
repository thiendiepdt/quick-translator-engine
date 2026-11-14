//! Gói extension trong bộ nhớ: manifest + mọi file JS đọc sẵn lúc nạp. Lúc chạy, module JS chỉ lấy từ đây
//! (`engine::loader`), nên extension không có đường đọc ổ đĩa.

use crate::{parse_manifest, ExtError, HostPolicy, HttpClient, HttpRequest, Limits, Manifest};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
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

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn zip_error(e: zip::result::ZipError) -> ExtError {
    package_error(format!("không đọc được zip: {e}"))
}

fn extract_into(bytes: &[u8], dest: &Path) -> Result<(), ExtError> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(package_error("gói zip quá 5 MB"));
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(zip_error)?;
    let mut entries: Vec<(usize, PathBuf)> = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(zip_error)?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_string();
        if file.is_symlink() {
            return Err(package_error(format!("zip chứa symlink: {name}")));
        }
        let Some(path) = file.enclosed_name().filter(|p| p.is_relative() && !p.to_string_lossy().contains(':')) else {
            return Err(package_error(format!("đường dẫn không an toàn trong zip: {name}")));
        };
        entries.push((i, path));
    }
    if entries.len() > MAX_FILES {
        return Err(package_error(format!("gói có quá {MAX_FILES} file")));
    }
    // Zip cả thư mục (`ten-extension/manifest.json`): bỏ thư mục gốc đó.
    let prefix = if entries.iter().any(|(_, p)| p == Path::new("manifest.json")) {
        None
    } else {
        let roots: HashSet<PathBuf> = entries.iter().filter_map(|(_, p)| p.components().next()).map(|c| PathBuf::from(c.as_os_str())).collect();
        match roots.into_iter().collect::<Vec<_>>().as_slice() {
            [root] if entries.iter().any(|(_, p)| *p == root.join("manifest.json")) => Some(root.clone()),
            _ => None,
        }
    };
    std::fs::create_dir_all(dest).map_err(|e| package_error(format!("{}: {e}", dest.display())))?;
    let mut total = 0u64;
    for (i, path) in entries {
        let relative = match &prefix {
            Some(root) => match path.strip_prefix(root) {
                Ok(rest) => rest.to_path_buf(),
                Err(_) => continue,
            },
            None => path,
        };
        if relative.as_os_str().is_empty() || relative.components().any(|c| skipped(&c.as_os_str().to_string_lossy())) {
            continue;
        }
        let target = dest.join(&relative);
        if let Some(dir) = target.parent() {
            std::fs::create_dir_all(dir).map_err(|e| package_error(format!("{}: {e}", dir.display())))?;
        }
        let file = archive.by_index(i).map_err(zip_error)?;
        let mut out = std::fs::File::create(&target).map_err(|e| package_error(format!("{}: {e}", target.display())))?;
        let written = std::io::copy(&mut file.take(MAX_BYTES - total + 1), &mut out).map_err(|e| package_error(format!("{}: {e}", relative.display())))?;
        total += written;
        if total > MAX_BYTES {
            return Err(package_error("gói quá 5 MB sau giải nén"));
        }
    }
    Ok(())
}

/// Giải nén gói zip vào `dest` (thư mục mới). Chặn đường dẫn thoát thư mục, đường dẫn tuyệt đối, symlink; tối
/// đa 200 file, 5 MB trước và sau giải nén. Zip có một thư mục gốc chứa `manifest.json` thì bỏ thư mục đó. Lỗi
/// thì xoá `dest`.
pub fn extract_zip(bytes: &[u8], dest: &Path) -> Result<(), ExtError> {
    let result = extract_into(bytes, dest);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(dest);
    }
    result
}

/// Tải gói zip từ link cài: chỉ `https://` (kể cả các bước redirect), không tới mạng nội bộ, tối đa 5 MB.
pub fn fetch_zip(url: &str) -> Result<Vec<u8>, ExtError> {
    let parsed = url::Url::parse(url.trim()).map_err(|e| package_error(format!("link không hợp lệ: {e}")))?;
    let policy = HostPolicy::public_https();
    policy.check(&parsed).map_err(|_| package_error("link cài phải là https:// tới một trang công khai"))?;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| package_error(e.to_string()))?;
    runtime.block_on(async {
        let client = HttpClient::new(policy, Limits::default())?;
        let response = client.send(HttpRequest { method: "GET".into(), url: parsed.to_string(), timeout_ms: Some(60_000), ..Default::default() }).await?;
        if !(200..300).contains(&response.status) {
            return Err(package_error(format!("tải gói lỗi HTTP {}", response.status)));
        }
        if response.body.len() as u64 > MAX_BYTES {
            return Err(package_error("gói zip quá 5 MB"));
        }
        Ok(response.body)
    })
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

    /// Zip từ danh sách (tên, nội dung); tên bắt đầu bằng "@link:" là symlink.
    pub(crate) fn make_zip(entries: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, text) in entries {
            if let Some(link) = name.strip_prefix("@link:") {
                writer.add_symlink(link, *text, options).unwrap();
            } else {
                writer.start_file(*name, options).unwrap();
                writer.write_all(text.as_bytes()).unwrap();
            }
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn giai_nen_zip_thuong_va_bo_thu_muc_goc() {
        let dir = tempfile::tempdir().unwrap();
        let flat = make_zip(&[("manifest.json", MANIFEST), ("main.js", "export async function toc() {}"), ("lib/a.js", "export const a = 1;"), (".git/x", "x")]);
        extract_zip(&flat, &dir.path().join("a")).unwrap();
        let pkg = Package::load_dir(&dir.path().join("a"), false).unwrap();
        assert!(pkg.files.contains_key("lib/a.js"));
        assert!(!dir.path().join("a/.git").exists());

        let nested = make_zip(&[("ext-demo/manifest.json", MANIFEST), ("ext-demo/main.js", "export async function toc() {}")]);
        extract_zip(&nested, &dir.path().join("b")).unwrap();
        assert!(dir.path().join("b/manifest.json").exists(), "bỏ thư mục gốc ext-demo/");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn zip_nguy_hiem_bi_tu_choi() {
        let dir = tempfile::tempdir().unwrap();
        let cases: Vec<(Vec<u8>, &str)> = vec![
            (make_zip(&[("manifest.json", MANIFEST), ("../thoat.js", "x")]), "không an toàn"),
            (make_zip(&[("manifest.json", MANIFEST), ("/tuyet-doi.js", "x")]), "không an toàn"),
            (make_zip(&[("manifest.json", MANIFEST), ("@link:lib", "C:/Windows")]), "symlink"),
            (make_zip(&(0..=MAX_FILES).map(|i| (Box::leak(format!("f{i}.js").into_boxed_str()) as &str, "x")).collect::<Vec<_>>()), "200 file"),
        ];
        for (bytes, needle) in cases {
            let target = dir.path().join("t");
            let error = extract_zip(&bytes, &target).unwrap_err().to_string();
            assert!(error.contains(needle), "{error}");
            assert!(!target.exists(), "lỗi thì không để lại gì");
        }
        assert!(!dir.path().join("thoat.js").exists());
        // Nén tốt: zip nhỏ nhưng giải nén quá 5 MB.
        let big = "a".repeat(6 * 1024 * 1024);
        let bomb = make_zip(&[("manifest.json", MANIFEST), ("main.js", &big)]);
        assert!(bomb.len() < 1024 * 1024);
        assert!(extract_zip(&bomb, &dir.path().join("bomb")).unwrap_err().to_string().contains("5 MB"));
        assert!(extract_zip(&vec![0u8; 6 * 1024 * 1024], &dir.path().join("to")).unwrap_err().to_string().contains("5 MB"));
    }

    #[test]
    fn link_cai_phai_la_https_cong_khai() {
        assert!(fetch_zip("http://github.com/a.zip").unwrap_err().to_string().contains("https://"));
        assert!(fetch_zip("https://127.0.0.1/a.zip").unwrap_err().to_string().contains("https://"));
        assert!(fetch_zip("không phải link").is_err());
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
