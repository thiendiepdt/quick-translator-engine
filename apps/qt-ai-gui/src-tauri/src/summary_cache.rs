//! Cache tóm tắt truyện (tên, done/total) cho màn chọn truyện và dialog chuyển truyện. Tính một tóm tắt
//! phải parse state.json (hàng chục KB mỗi truyện); thư viện vài trăm truyện là hàng MB JSON mỗi lần quay
//! về danh sách. Khoá theo root_key, hợp lệ khi mtime của state.json và story.json không đổi.
use crate::session_registry::root_key;
use crate::story_cmds::RecentSummary;
use qt_ai_core::commands::status::count_chapters;
use qt_ai_core::story_fs::{load_state, load_story_config, story_paths};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

type Stamp = (Option<SystemTime>, Option<SystemTime>);

struct Cached {
    stamp: Stamp,
    summary: RecentSummary,
}

#[derive(Default)]
pub struct SummaryCache {
    entries: Mutex<HashMap<String, Cached>>,
    hits: Mutex<usize>,
}

/// Đọc thẳng từ đĩa: folder hỏng/chưa init → chỉ có root (UI hiện mờ).
pub fn summarize_uncached(root: &Path) -> RecentSummary {
    let paths = story_paths(root);
    match (load_state(&paths), load_story_config(&paths)) {
        (Ok(state), Ok(story)) => {
            let counts = count_chapters(&state);
            RecentSummary {
                root: root.display().to_string(),
                name: Some(story.name).filter(|n| !n.trim().is_empty()),
                done: Some(counts.done),
                total: Some(counts.total),
            }
        }
        _ => RecentSummary { root: root.display().to_string(), name: None, done: None, total: None },
    }
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

impl SummaryCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tóm tắt của `root`; `root` trong kết quả giữ nguyên chuỗi caller đưa (recent giữ nguyên dạng đã lưu).
    pub fn summarize(&self, root: &Path) -> RecentSummary {
        let paths = story_paths(root);
        let stamp: Stamp = (mtime(&paths.state_json), mtime(&paths.story_json));
        let key = root_key(&root.display().to_string());
        let root_text = root.display().to_string();
        if let Some(cached) = self.entries.lock().unwrap().get(&key) {
            if cached.stamp == stamp {
                *self.hits.lock().unwrap() += 1;
                return RecentSummary { root: root_text, ..cached.summary.clone() };
            }
        }
        let summary = summarize_uncached(root);
        self.entries.lock().unwrap().insert(key, Cached { stamp, summary: summary.clone() });
        summary
    }

    /// Số lần trả từ cache (test).
    pub fn hits(&self) -> usize {
        *self.hits.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::commands::init::run_init;
    use qt_ai_core::story_fs::{save_story_config, story_paths};
    use std::time::Duration;

    #[test]
    fn cache_theo_mtime_va_giu_root_cua_caller() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Truyen");
        std::fs::create_dir_all(root.join("raw")).unwrap();
        std::fs::write(root.join("raw").join("1.txt"), "a").unwrap();
        run_init(&root, "qt-ai").unwrap();
        let paths = story_paths(&root);
        let mut story = load_story_config(&paths).unwrap();
        story.name = "Tên A".into();
        save_story_config(&paths, &story).unwrap();

        let cache = SummaryCache::new();
        let first = cache.summarize(&root);
        assert_eq!(first.name.as_deref(), Some("Tên A"));
        assert_eq!(first.total, Some(1));
        assert_eq!(cache.hits(), 0);

        // Cùng truyện, khác hoa thường + dấu cuối → trúng cache, root trả về là chuỗi caller đưa.
        let alias = root.display().to_string().to_lowercase() + "\\";
        let second = cache.summarize(Path::new(&alias));
        assert_eq!(cache.hits(), 1);
        assert_eq!(second.root, alias);
        assert_eq!(second.name.as_deref(), Some("Tên A"));

        // Đổi story.json (mtime đẩy tới trước để không đụng độ phân giải đồng hồ) → tính lại.
        story.name = "Tên B".into();
        save_story_config(&paths, &story).unwrap();
        let file = std::fs::File::options().write(true).open(&paths.story_json).unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(5)).unwrap();
        let third = cache.summarize(&root);
        assert_eq!(third.name.as_deref(), Some("Tên B"));
        assert_eq!(cache.hits(), 1);

        // Folder chưa init: không có mtime, vẫn trả tóm tắt rỗng và không panic.
        let empty = cache.summarize(&dir.path().join("khong-co"));
        assert!(empty.total.is_none());
    }
}
