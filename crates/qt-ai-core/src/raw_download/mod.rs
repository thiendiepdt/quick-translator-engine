//! Tải raw qua extension (spec 2026-11-07-tai-raw-extension-design): cấu hình tải của truyện trong
//! `download.json`, mẫu tên file, chuẩn hoá nội dung, khớp mục lục với raw đã có, danh sách extension và lượt
//! tải. File riêng (không nằm trong `story.json`) vì bản app cũ đọc `story.json` theo danh sách trường cố định
//! rồi ghi lại, sẽ xoá mất trường lạ — giống lý do `cast.json`, `titles.json` tách riêng.
//!
//! Thư mục truyện có thể là repo git, nên `download.json` không bao giờ chứa khoá, mật khẩu, cookie hay session.

pub mod content;
pub mod matching;
pub mod registry;
pub mod repository;
pub mod run;

pub mod template;

use crate::error::{CoreError, Result};
use crate::story::StoryConfig;
use crate::story_fs::{list_raw_chapter_ids, read_raw_chapter, write_atomic, StoryPaths};
use registry::{pick, Installed};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

pub const DOWNLOAD_JSON: &str = "download.json";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DownloadConfig {
    /// Rỗng = dùng `sourceUrl` của `story.json`.
    pub raw_url: String,
    /// Extension đã chọn; rỗng = tự chọn theo `match`.
    pub extension: String,
    /// Rỗng = mẫu chung (`template::DEFAULT_TEMPLATE`).
    pub file_template: String,
    /// Cài đặt `scope: "story"` theo id extension. Không chứa trường `secret`.
    pub settings: IndexMap<String, Map<String, Value>>,
    /// Sổ chương: mã chương trong mục lục → id file raw.
    pub chapters: IndexMap<String, String>,
}

/// Lưu cài đặt `scope: "story"` của một extension cho truyện. `values` là đủ bộ trường của truyện: thiếu hoặc
/// `null` = dùng mặc định. Sai kiểu thì báo đúng trường, không đổi gì.
pub fn set_story_settings(config: &mut DownloadConfig, manifest: &qt_ext::Manifest, values: &Map<String, Value>) -> std::result::Result<(), String> {
    let mut own = Map::new();
    for field in manifest.settings.iter().filter(|f| f.scope == qt_ext::Scope::Story) {
        if let Some(value) = values.get(&field.key).filter(|v| !v.is_null()) {
            own.insert(field.key.clone(), qt_ext::check_setting(field, value)?);
        }
    }
    if own.is_empty() {
        config.settings.shift_remove(&manifest.id);
    } else {
        config.settings.insert(manifest.id.clone(), own);
    }
    Ok(())
}

pub fn download_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join(DOWNLOAD_JSON)
}

/// Thiếu hoặc hỏng → mặc định. Hỏng thì chép sang `download.json.bad` để lần ghi sau không xoá mất sổ chương.
pub fn load_download(paths: &StoryPaths) -> DownloadConfig {
    let path = download_path(paths);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return DownloadConfig::default();
    };
    match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(config) => config,
        Err(_) => {
            let _ = std::fs::copy(&path, paths.root.join(format!("{DOWNLOAD_JSON}.bad")));
            DownloadConfig::default()
        }
    }
}

pub fn save_download(paths: &StoryPaths, config: &DownloadConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config).map_err(|e| crate::error::CoreError::Internal(e.to_string()))?;
    write_atomic(&download_path(paths), &format!("{json}\n"))
}

pub fn effective_raw_url(config: &DownloadConfig, story: &StoryConfig) -> String {
    let raw = config.raw_url.trim();
    if raw.is_empty() {
        story.source_url.trim().to_string()
    } else {
        raw.to_string()
    }
}

/// Nhãn nguồn của truyện ở màn chọn truyện: tên extension sẽ tải raw, hoặc tên miền khi chưa có extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLabel {
    pub label: String,
    /// Có extension khớp link tải raw (tải raw được).
    pub extension: bool,
    pub url: String,
}

/// Nguồn con trong link SangTacViet (`/truyen/qidian/1/123/` → `qidian`).
fn stv_source(url: &url::Url) -> Option<String> {
    let host = url.host_str()?;
    if !(host.contains("sangtacviet") || host == "14.225.254.182" || host == "103.82.20.93") {
        return None;
    }
    let mut segments = url.path_segments()?;
    (segments.next()? == "truyen").then_some(())?;
    segments.next().filter(|s| !s.is_empty()).map(str::to_string)
}

/// `"sourceUrl": "…"` trong đoạn đầu story.json (app ghi nó là khoá thứ hai).
fn source_url_in_head(head: &[u8]) -> Option<String> {
    const KEY: &[u8] = b"\"sourceUrl\"";
    let at = head.windows(KEY.len()).position(|w| w == KEY)?;
    if at > 0 && head[at - 1] == b'\\' {
        return None;
    }
    let rest = head[at + KEY.len()..].trim_ascii_start().strip_prefix(b":")?.trim_ascii_start();
    let value = serde_json::Deserializer::from_slice(rest).into_iter::<Value>().next()?.ok()?;
    Some(value.as_str().unwrap_or("").to_string())
}

/// Chỉ lấy `sourceUrl` của story.json cho danh sách truyện: file có khi vài MB vì glossary (thư viện 126 truyện ~30 MB),
/// đọc hết để parse thì mất 120–500 ms. Đọc 8 KB đầu là đủ; không thấy khoá thì mới parse cả file.
fn story_source_url(paths: &StoryPaths) -> Option<String> {
    use std::io::Read;
    let mut head = Vec::with_capacity(8192);
    std::fs::File::open(&paths.story_json).ok()?.take(8192).read_to_end(&mut head).ok()?;
    if let Some(found) = source_url_in_head(&head) {
        return Some(found);
    }
    #[derive(Deserialize)]
    struct OnlySource {
        #[serde(rename = "sourceUrl", default)]
        source_url: Option<Value>,
    }
    let text = std::fs::read_to_string(&paths.story_json).ok()?;
    let parsed: OnlySource = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    Some(parsed.source_url.as_ref().and_then(Value::as_str).unwrap_or("").to_string())
}

/// Tên extension bỏ chữ Hán cho chip gọn ("Fanqie 番茄" → "Fanqie"); tên toàn chữ Hán thì giữ nguyên.
fn short_name(name: &str) -> String {
    let cjk = |c: char| matches!(c as u32, 0x2E80..=0x9FFF | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFFEF | 0x20000..=0x3FFFF);
    let kept: String = name.chars().filter(|&c| !cjk(c)).collect();
    let kept = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    if kept.chars().any(char::is_alphanumeric) {
        kept
    } else {
        name.trim().to_string()
    }
}

/// Nhãn nguồn của truyện từ link tải raw (mặc định là link nguồn); `None` khi truyện không có link.
pub fn source_label(paths: &StoryPaths, installed: &[Installed]) -> Option<SourceLabel> {
    let story = StoryConfig { source_url: story_source_url(paths)?, ..StoryConfig::empty() };
    let config = load_download(paths);
    let raw = effective_raw_url(&config, &story);
    let parsed = url::Url::parse(&raw).ok();
    if let Some(ext) = pick(installed, &raw, &config.extension) {
        let name = short_name(&ext.package.manifest.name);
        let label = match parsed.as_ref().and_then(stv_source) {
            Some(sub) => format!("{name} · {sub}"),
            None => name,
        };
        return Some(SourceLabel { label, extension: true, url: raw });
    }
    let host = parsed?.host_str()?.trim_start_matches("www.").to_string();
    Some(SourceLabel { label: host, extension: false, url: raw })
}

/// Một file chương trong `raw/` (tab "Raw trong máy" của trang Tải raw).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawFile {
    /// Tên file bỏ `.txt`.
    pub id: String,
    /// Dòng đầu không trống (tiêu đề chương).
    pub title: String,
    /// Số ký tự không tính khoảng trắng; `None` khi không đọc được file (không phải UTF-8…).
    pub chars: Option<usize>,
}

fn raw_file(paths: &StoryPaths, id: &str) -> RawFile {
    match read_raw_chapter(paths, id) {
        Ok(text) => {
            let title = text.lines().map(|l| l.trim().trim_start_matches('\u{feff}')).find(|l| !l.is_empty()).unwrap_or("").to_string();
            RawFile { id: id.to_string(), title, chars: Some(text.chars().filter(|&c| !c.is_whitespace() && c != '\u{feff}').count()) }
        }
        Err(_) => RawFile { id: id.to_string(), title: String::new(), chars: None },
    }
}

/// File chương trong `raw/`, xếp như hàng đợi dịch. Đọc song song theo khối: truyện 2.510 chương lần đầu (đĩa chưa
/// cache) đọc tuần tự mất ~1,1 s.
pub fn list_raw_files(paths: &StoryPaths) -> Result<Vec<RawFile>> {
    let ids = list_raw_chapter_ids(paths)?;
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
    let size = ids.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = ids.chunks(size).map(|part| scope.spawn(move || part.iter().map(|id| raw_file(paths, id)).collect::<Vec<_>>())).collect();
        let mut all = Vec::with_capacity(ids.len());
        for worker in workers {
            all.extend(worker.join().map_err(|_| CoreError::Internal("luồng đọc raw bị lỗi".into()))?);
        }
        Ok(all)
    })
}

/// Kết quả dọn raw cũ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearOutcome {
    /// Số file đã chuyển đi.
    pub moved: usize,
    /// Thư mục giữ bản cũ (rỗng khi `raw/` không có file nào).
    pub backup: String,
}

/// Dọn raw cũ để tải lại từ đầu: chuyển mọi file `.txt` trong `raw/` sang `work/raw-backup/xoa-<thời điểm>/` (không
/// xoá — nhầm thì chép ngược lại), xoá sổ chương trong `download.json`. Bản dịch trong `out/` giữ nguyên.
pub fn clear_raw(paths: &StoryPaths) -> Result<ClearOutcome> {
    let ids = list_raw_chapter_ids(paths)?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let backup = paths.work_dir.join("raw-backup").join(format!("xoa-{stamp}"));
    if !ids.is_empty() {
        std::fs::create_dir_all(&backup).map_err(CoreError::io(&backup))?;
    }
    for id in &ids {
        let name = format!("{id}.txt");
        let from = paths.raw_dir.join(&name);
        let to = backup.join(&name);
        if std::fs::rename(&from, &to).is_err() {
            std::fs::copy(&from, &to).map_err(CoreError::io(&to))?;
            std::fs::remove_file(&from).map_err(CoreError::io(&from))?;
        }
    }
    let mut config = load_download(paths);
    if !config.chapters.is_empty() {
        config.chapters.clear();
        save_download(paths, &config)?;
    }
    Ok(ClearOutcome { moved: ids.len(), backup: if ids.is_empty() { String::new() } else { backup.display().to_string() } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_fs::story_paths;

    #[test]
    fn ghi_doc_lai_va_file_hong() {
        let dir = tempfile::tempdir().unwrap();
        let paths = story_paths(dir.path());
        assert_eq!(load_download(&paths), DownloadConfig::default());
        let mut config = DownloadConfig { raw_url: "https://a/b".into(), file_template: "chuong-{chuong:4}".into(), ..Default::default() };
        config.chapters.insert("7445".into(), "0001_7445".into());
        save_download(&paths, &config).unwrap();
        assert_eq!(load_download(&paths), config);
        assert!(std::fs::read_to_string(download_path(&paths)).unwrap().ends_with("}\n"));
        std::fs::write(download_path(&paths), "{ hỏng").unwrap();
        assert_eq!(load_download(&paths), DownloadConfig::default());
        assert!(dir.path().join("download.json.bad").exists());
    }

    #[test]
    fn don_raw_cu_chuyen_sang_backup_va_xoa_so() {
        let dir = tempfile::tempdir().unwrap();
        let paths = story_paths(dir.path());
        assert_eq!(clear_raw(&paths).unwrap(), ClearOutcome { moved: 0, backup: String::new() }, "chưa có raw/");
        std::fs::create_dir_all(&paths.raw_dir).unwrap();
        for id in ["0001_a", "0002_b"] {
            std::fs::write(paths.raw_dir.join(format!("{id}.txt")), format!("{id}

nội dung")).unwrap();
        }
        std::fs::write(paths.raw_dir.join("ghi-chu.md"), "không phải chương").unwrap();
        let mut config = DownloadConfig { raw_url: "https://a/b".into(), ..Default::default() };
        config.chapters.insert("a".into(), "0001_a".into());
        save_download(&paths, &config).unwrap();

        let outcome = clear_raw(&paths).unwrap();
        assert_eq!(outcome.moved, 2);
        let backup = std::path::Path::new(&outcome.backup);
        assert!(backup.join("0001_a.txt").exists() && backup.join("0002_b.txt").exists());
        assert!(!paths.raw_dir.join("0001_a.txt").exists());
        assert!(paths.raw_dir.join("ghi-chu.md").exists(), "chỉ đụng file chương .txt");
        let after = load_download(&paths);
        assert!(after.chapters.is_empty());
        assert_eq!(after.raw_url, "https://a/b", "giữ link và cài đặt khác");
    }

    #[test]
    fn nhan_nguon_ten_extension_hoac_ten_mien() {
        let dir = tempfile::tempdir().unwrap();
        let story_dir = dir.path().join("truyen");
        let paths = story_paths(&story_dir);
        let ext_dir = dir.path().join("extensions");
        let dev = dir.path().join("dev");
        std::fs::create_dir_all(&dev).unwrap();
        let manifest = serde_json::json!({
            "id": "org.test.stv", "name": "SangTacViet", "version": "1.0.0", "apiVersion": 1, "description": "d",
            "developer": { "name": "dev" }, "match": [r"^https://sangtacviet\.vip/truyen/"], "hosts": ["sangtacviet.vip"],
            "permissions": []
        });
        std::fs::write(dev.join("manifest.json"), manifest.to_string()).unwrap();
        std::fs::write(dev.join("main.js"), "export async function toc() { return []; }").unwrap();
        registry::Registry::new(&ext_dir).add_dev(&dev).unwrap();
        let (installed, errors) = registry::list_installed(&ext_dir);
        assert!(errors.is_empty(), "{errors:?}");

        let write_story = |url: &str| {
            std::fs::create_dir_all(&story_dir).unwrap();
            let story = StoryConfig { source_url: url.into(), ..StoryConfig::empty() };
            crate::story_fs::save_story_config(&paths, &story).unwrap();
        };
        write_story("https://sangtacviet.vip/truyen/qidian/1/1036370336/");
        let label = source_label(&paths, &installed).unwrap();
        assert_eq!((label.label.as_str(), label.extension), ("SangTacViet · qidian", true));

        write_story("https://www.69shuba.com/book/123.htm");
        let label = source_label(&paths, &installed).unwrap();
        assert_eq!((label.label.as_str(), label.extension), ("69shuba.com", false), "không có extension → tên miền");

        write_story("");
        assert_eq!(source_label(&paths, &installed), None, "chưa có link");

        // story.json không do app ghi: sourceUrl nằm sau glossary dài → parse cả file.
        let glossary: serde_json::Map<String, Value> = (0..2000).map(|i| (format!("từ{i}"), Value::from("nghĩa dài dài"))).collect();
        let late = serde_json::json!({ "name": "x", "glossary": glossary, "sourceUrl": "https://www.qimao.com/shuku/1/" });
        std::fs::write(&paths.story_json, late.to_string()).unwrap();
        assert!(std::fs::metadata(&paths.story_json).unwrap().len() > 8192);
        assert_eq!(source_label(&paths, &installed).unwrap().label, "qimao.com");
    }

    #[test]
    fn ten_extension_bo_chu_han() {
        assert_eq!(short_name("Fanqie 番茄"), "Fanqie");
        assert_eq!(short_name("Qimao 七猫"), "Qimao");
        assert_eq!(short_name("七猫 Qimao（官方）"), "Qimao");
        assert_eq!(short_name("SangTacViet"), "SangTacViet");
        assert_eq!(short_name("番茄小说"), "番茄小说", "toàn chữ Hán thì giữ");
    }

    #[test]
    fn doc_source_url_o_dau_file() {
        assert_eq!(source_url_in_head(br#"{ "name": "a", "sourceUrl" : "https://x/1" , "#).as_deref(), Some("https://x/1"));
        assert_eq!(source_url_in_head(br#"{"name":"a","sourceUrl":null}"#).as_deref(), Some(""));
        assert_eq!(source_url_in_head(br#"{"name":"a","summary":"long"#), None, "chưa tới khoá");
        assert_eq!(source_url_in_head(br#"{"name":"a","sourceUrl":"https://cut"#), None, "giá trị bị cắt ngang");
    }

    #[test]
    fn liet_ke_file_raw_tieu_de_so_chu() {
        let dir = tempfile::tempdir().unwrap();
        let paths = story_paths(dir.path());
        assert!(list_raw_files(&paths).unwrap().is_empty(), "chưa có raw/");
        std::fs::create_dir_all(&paths.raw_dir).unwrap();
        std::fs::write(paths.raw_dir.join("0010_b.txt"), "\u{feff}\n  第10章 乙  \n\n正文 内容").unwrap();
        std::fs::write(paths.raw_dir.join("0002_a.txt"), "第2章 甲\n文").unwrap();
        std::fs::write(paths.raw_dir.join("0003_c.txt"), [0xffu8, 0xfe, 0x00]).unwrap();
        std::fs::write(paths.raw_dir.join("ghi-chu.md"), "x").unwrap();
        let files = list_raw_files(&paths).unwrap();
        assert_eq!(
            files,
            vec![
                RawFile { id: "0002_a".into(), title: "第2章 甲".into(), chars: Some(5) },
                RawFile { id: "0003_c".into(), title: String::new(), chars: None },
                RawFile { id: "0010_b".into(), title: "第10章 乙".into(), chars: Some(9) },
            ]
        );
    }

    #[test]
    fn link_tai_raw_mac_dinh_la_link_nguon() {
        let story = StoryConfig { source_url: " https://nguon ".into(), ..StoryConfig::empty() };
        assert_eq!(effective_raw_url(&DownloadConfig::default(), &story), "https://nguon");
        let blank = DownloadConfig { raw_url: "  ".into(), ..Default::default() };
        assert_eq!(effective_raw_url(&blank, &story), "https://nguon");
        let own = DownloadConfig { raw_url: "https://raw".into(), ..Default::default() };
        assert_eq!(effective_raw_url(&own, &story), "https://raw");
    }
}
