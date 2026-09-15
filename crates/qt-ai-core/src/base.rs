//! Bản mặc định người dùng sửa được ở mức app: base prompt theo genre, bộ rule theo bối cảnh, kho
//! glossary chung theo bối cảnh — file rời trong `<app_config_dir>/base/`. Không có file = bản cứng.
//! Core đọc (không cache) để app, phiên API và `qt-ai next` trong phiên agy cùng thấy một bản.

use crate::check::default_rules_as_check_rules;
use crate::error::{CoreError, Result};
use crate::prompt::base_prompt;
use crate::story::{CheckRule, GenreSetting, Glossary, StoryGenre, StringMap, GLOSSARY_CATEGORIES};
use crate::story_fs::write_atomic;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const APP_IDENTIFIER: &str = "com.vn-converter.qt-ai-gui";
/// GUI đặt biến này lúc khởi động (tiến trình agy con thừa hưởng) để mọi bên cùng một thư mục.
pub const BASE_DIR_ENV: &str = "QT_AI_BASE_DIR";

#[derive(Debug, Clone, PartialEq)]
pub enum BaseSource {
    Builtin,
    File(PathBuf),
}

/// `<config_dir>/<APP_IDENTIFIER>/base` — cùng cách Tauri tính `app_config_dir`.
pub fn platform_base_dir() -> Option<PathBuf> {
    let config_dir = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    };
    config_dir.map(|dir| dir.join(APP_IDENTIFIER).join("base"))
}

#[derive(Debug, Clone)]
pub struct BaseStore {
    dir: Option<PathBuf>,
}

impl BaseStore {
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        BaseStore { dir: Some(dir.into()) }
    }

    /// Không thư mục: luôn bản cứng (test / môi trường không xác định được config dir).
    pub fn none() -> Self {
        BaseStore { dir: None }
    }

    pub fn from_env() -> Self {
        let dir = std::env::var_os(BASE_DIR_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(platform_base_dir);
        BaseStore { dir }
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    pub fn prompt_path(&self, genre: &StoryGenre) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| {
            dir.join("prompts").join(format!("{}-{}.md", genre.setting.as_str(), genre.names.as_str()))
        })
    }

    pub fn rules_path(&self, setting: GenreSetting) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| dir.join("rules").join(format!("{}.json", setting.as_str())))
    }

    pub fn glossary_path(&self, setting: GenreSetting) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| dir.join("glossary").join(format!("{}.json", setting.as_str())))
    }

    /// Nội dung file nếu có và không rỗng sau trim; file hỏng/không đọc được coi như không có.
    fn read_prompt_file(&self, genre: &StoryGenre) -> Option<(PathBuf, String)> {
        let path = self.prompt_path(genre)?;
        let text = fs::read_to_string(&path).ok()?;
        (!text.trim().is_empty()).then_some((path, text))
    }

    fn read_rules_file(&self, setting: GenreSetting) -> Option<(PathBuf, Vec<CheckRule>)> {
        let path = self.rules_path(setting)?;
        let text = fs::read_to_string(&path).ok()?;
        match serde_json::from_str::<Vec<CheckRule>>(&text) {
            Ok(rules) => Some((path, rules)),
            Err(error) => {
                eprintln!("[qt-ai] bỏ qua {} hỏng: {error}", path.display());
                None
            }
        }
    }

    fn read_glossary_file(&self, setting: GenreSetting) -> Option<(PathBuf, Glossary)> {
        let path = self.glossary_path(setting)?;
        let text = fs::read_to_string(&path).ok()?;
        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("[qt-ai] bỏ qua {} hỏng: {error}", path.display());
                return None;
            }
        };
        Some((path, glossary_from_value(&value)))
    }

    pub fn prompt(&self, genre: &StoryGenre) -> String {
        self.read_prompt_file(genre).map(|(_, text)| text).unwrap_or_else(|| base_prompt(genre).to_string())
    }

    pub fn rules(&self, setting: GenreSetting) -> Vec<CheckRule> {
        self.read_rules_file(setting)
            .map(|(_, rules)| rules)
            .unwrap_or_else(|| default_rules_as_check_rules(setting))
    }

    pub fn glossary(&self, setting: GenreSetting) -> Glossary {
        self.read_glossary_file(setting)
            .map(|(_, glossary)| glossary)
            .unwrap_or_else(|| glossary_from_value(&Value::Null))
    }

    pub fn prompt_source(&self, genre: &StoryGenre) -> BaseSource {
        self.read_prompt_file(genre).map(|(path, _)| BaseSource::File(path)).unwrap_or(BaseSource::Builtin)
    }

    pub fn rules_source(&self, setting: GenreSetting) -> BaseSource {
        self.read_rules_file(setting).map(|(path, _)| BaseSource::File(path)).unwrap_or(BaseSource::Builtin)
    }

    pub fn glossary_source(&self, setting: GenreSetting) -> BaseSource {
        self.read_glossary_file(setting).map(|(path, _)| BaseSource::File(path)).unwrap_or(BaseSource::Builtin)
    }

    fn require(&self, path: Option<PathBuf>) -> Result<PathBuf> {
        path.ok_or_else(|| CoreError::Internal("Không xác định được thư mục bản mặc định của app".to_string()))
    }

    fn write(&self, path: Option<PathBuf>, content: &str) -> Result<()> {
        let path = self.require(path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(CoreError::io(parent))?;
        }
        write_atomic(&path, content)
    }

    fn remove(&self, path: Option<PathBuf>) -> Result<()> {
        let path = self.require(path)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CoreError::io(&path)(error)),
        }
    }

    /// Text trống = về mặc định (xoá file).
    pub fn save_prompt(&self, genre: &StoryGenre, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return self.reset_prompt(genre);
        }
        self.write(self.prompt_path(genre), text)
    }

    pub fn save_rules(&self, setting: GenreSetting, rules: &[CheckRule]) -> Result<()> {
        let json = serde_json::to_string_pretty(rules).expect("CheckRule luôn serialize được");
        self.write(self.rules_path(setting), &format!("{json}\n"))
    }

    pub fn save_glossary(&self, setting: GenreSetting, glossary: &Glossary) -> Result<()> {
        let json = serde_json::to_string_pretty(glossary).expect("Glossary luôn serialize được");
        self.write(self.glossary_path(setting), &format!("{json}\n"))
    }

    pub fn reset_prompt(&self, genre: &StoryGenre) -> Result<()> {
        self.remove(self.prompt_path(genre))
    }

    pub fn reset_rules(&self, setting: GenreSetting) -> Result<()> {
        self.remove(self.rules_path(setting))
    }

    pub fn reset_glossary(&self, setting: GenreSetting) -> Result<()> {
        self.remove(self.glossary_path(setting))
    }
}

/// Đủ 8 nhóm theo thứ tự chuẩn; entry key trống hoặc giá trị không phải chuỗi bị bỏ (như glossary truyện).
fn glossary_from_value(value: &Value) -> Glossary {
    let object = value.as_object();
    GLOSSARY_CATEGORIES
        .iter()
        .map(|key| {
            let entries: StringMap = object
                .and_then(|o| o.get(*key))
                .and_then(Value::as_object)
                .map(|group| {
                    group
                        .iter()
                        .filter_map(|(source, target)| {
                            let target = target.as_str()?;
                            (!source.trim().is_empty()).then(|| (source.clone(), target.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            (key.to_string(), entries)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::GenreNames;

    fn store() -> (tempfile::TempDir, BaseStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = BaseStore::at(dir.path().join("base"));
        (dir, store)
    }

    #[test]
    fn khong_co_file_thi_prompt_rule_cung_glossary_rong() {
        let (_dir, store) = store();
        let genre = StoryGenre::default();
        assert_eq!(store.prompt(&genre), base_prompt(&genre));
        assert_eq!(store.rules(GenreSetting::Ancient), default_rules_as_check_rules(GenreSetting::Ancient));
        assert!(store.glossary(GenreSetting::Ancient).values().all(|group| group.is_empty()));
        assert!(matches!(store.prompt_source(&genre), BaseSource::Builtin));
        assert!(matches!(store.rules_source(GenreSetting::Modern), BaseSource::Builtin));
        assert!(matches!(store.glossary_source(GenreSetting::Mixed), BaseSource::Builtin));
        // Store không thư mục: luôn builtin, save là lỗi rõ ràng.
        let none = BaseStore::none();
        assert_eq!(none.prompt(&genre), base_prompt(&genre));
        assert!(none.save_prompt(&genre, "x").is_err());
    }

    #[test]
    fn save_ghi_dung_duong_dan_va_doc_lai_reset_xoa() {
        let (dir, store) = store();
        let genre = StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign, ..StoryGenre::default() };
        store.save_prompt(&genre, "# Prompt của tôi\n").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("base/prompts/modern-foreign.md")).unwrap(),
            "# Prompt của tôi\n"
        );
        assert_eq!(store.prompt(&genre), "# Prompt của tôi\n");
        assert!(matches!(store.prompt_source(&genre), BaseSource::File(_)));
        // Genre khác vẫn builtin.
        assert!(matches!(store.prompt_source(&StoryGenre::default()), BaseSource::Builtin));

        let rules = vec![CheckRule { pattern: "abc".into(), flags: Some("i".into()), message: "abc → xyz".into() }];
        store.save_rules(GenreSetting::Modern, &rules).unwrap();
        assert!(dir.path().join("base/rules/modern.json").is_file());
        assert_eq!(store.rules(GenreSetting::Modern), rules);
        assert_eq!(store.rules(GenreSetting::Ancient), default_rules_as_check_rules(GenreSetting::Ancient));

        let mut glossary = Glossary::new();
        glossary.insert("names".into(), StringMap::from([("赵".to_string(), "Triệu".to_string())]));
        store.save_glossary(GenreSetting::Ancient, &glossary).unwrap();
        let read = store.glossary(GenreSetting::Ancient);
        assert_eq!(read["names"]["赵"], "Triệu");
        assert_eq!(read.len(), GLOSSARY_CATEGORIES.len(), "đủ 8 nhóm dù file thiếu");

        store.reset_prompt(&genre).unwrap();
        store.reset_rules(GenreSetting::Modern).unwrap();
        store.reset_glossary(GenreSetting::Ancient).unwrap();
        assert!(!dir.path().join("base/prompts/modern-foreign.md").exists());
        assert!(matches!(store.rules_source(GenreSetting::Modern), BaseSource::Builtin));
        assert!(matches!(store.glossary_source(GenreSetting::Ancient), BaseSource::Builtin));
        // Reset lần hai không lỗi.
        store.reset_prompt(&genre).unwrap();
    }

    #[test]
    fn file_rong_hoac_hong_coi_nhu_khong_co_va_prompt_trong_la_xoa() {
        let (dir, store) = store();
        let genre = StoryGenre::default();
        fs::create_dir_all(dir.path().join("base/prompts")).unwrap();
        fs::create_dir_all(dir.path().join("base/rules")).unwrap();
        fs::write(dir.path().join("base/prompts/ancient-han.md"), "   \n").unwrap();
        fs::write(dir.path().join("base/rules/ancient.json"), "{ hỏng").unwrap();
        assert_eq!(store.prompt(&genre), base_prompt(&genre));
        assert!(matches!(store.prompt_source(&genre), BaseSource::Builtin));
        assert_eq!(store.rules(GenreSetting::Ancient), default_rules_as_check_rules(GenreSetting::Ancient));
        assert!(matches!(store.rules_source(GenreSetting::Ancient), BaseSource::Builtin));
        store.save_prompt(&genre, "x").unwrap();
        store.save_prompt(&genre, "  ").unwrap();
        assert!(!dir.path().join("base/prompts/ancient-han.md").exists());
    }

    #[test]
    fn glossary_file_bo_key_trong_va_gia_tri_khong_phai_chuoi() {
        let (dir, store) = store();
        fs::create_dir_all(dir.path().join("base/glossary")).unwrap();
        fs::write(
            dir.path().join("base/glossary/mixed.json"),
            r#"{"names":{"赵":"Triệu"," ":"bỏ","k":5},"laLam":{"a":"b"}}"#,
        )
        .unwrap();
        let read = store.glossary(GenreSetting::Mixed);
        assert_eq!(read["names"].len(), 1);
        assert!(!read.contains_key("laLam"));
    }

    #[test]
    fn from_env_uu_tien_bien_moi_truong_roi_toi_thu_muc_he() {
        // Env là toàn cục — gộp mọi trường hợp vào một test để không đua với test khác.
        std::env::set_var(BASE_DIR_ENV, "D:/x/base");
        assert_eq!(BaseStore::from_env().dir(), Some(Path::new("D:/x/base")));
        std::env::remove_var(BASE_DIR_ENV);
        let platform = platform_base_dir();
        if let Some(dir) = &platform {
            assert!(dir.ends_with(Path::new(APP_IDENTIFIER).join("base")), "{}", dir.display());
        }
        assert_eq!(BaseStore::from_env().dir().map(Path::to_path_buf), platform);
    }
}
