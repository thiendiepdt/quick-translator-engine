//! Tải raw qua extension (spec 2026-11-07-tai-raw-extension-design): cấu hình tải của truyện trong
//! `download.json`, mẫu tên file, chuẩn hoá nội dung, khớp mục lục với raw đã có, danh sách extension và lượt
//! tải. File riêng (không nằm trong `story.json`) vì bản app cũ đọc `story.json` theo danh sách trường cố định
//! rồi ghi lại, sẽ xoá mất trường lạ — giống lý do `cast.json`, `titles.json` tách riêng.
//!
//! Thư mục truyện có thể là repo git, nên `download.json` không bao giờ chứa khoá, mật khẩu, cookie hay session.

pub mod content;
pub mod matching;
pub mod registry;
pub mod run;

pub mod template;

use crate::error::Result;
use crate::story::StoryConfig;
use crate::story_fs::{write_atomic, StoryPaths};
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
    fn link_tai_raw_mac_dinh_la_link_nguon() {
        let story = StoryConfig { source_url: " https://nguon ".into(), ..StoryConfig::empty() };
        assert_eq!(effective_raw_url(&DownloadConfig::default(), &story), "https://nguon");
        let blank = DownloadConfig { raw_url: "  ".into(), ..Default::default() };
        assert_eq!(effective_raw_url(&blank, &story), "https://nguon");
        let own = DownloadConfig { raw_url: "https://raw".into(), ..Default::default() };
        assert_eq!(effective_raw_url(&own, &story), "https://raw");
    }
}
