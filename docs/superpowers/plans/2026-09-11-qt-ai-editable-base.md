# Base prompt / rule / glossary sửa được — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Người dùng sửa base prompt (từng genre), bộ rule (từng bối cảnh) và kho glossary chung (từng bối cảnh) ở mức app; app, phiên API và `qt-ai next` trong phiên agy đều dùng bản sửa.

**Architecture:** Core Rust có module `base` (`BaseStore`) đọc/ghi file rời trong `<app_config_dir>/base/`; thư mục lấy từ env `QT_AI_BASE_DIR` (GUI đặt lúc khởi động, tiến trình agy con thừa hưởng) hoặc tự tính theo hệ. Ba chỗ ghép (prompt, glossary nền, rule) hỏi `BaseStore` thay vì bản cứng. GUI thêm command `base_get/base_save/base_reset`, card "Bản mặc định" ở Cài đặt mở ba dialog dùng lại editor sẵn có.

**Tech Stack:** Rust (qt-ai-core, Tauri 2), React + zustand + zod + react-hook-form, vitest.

Spec: `docs/superpowers/specs/2026-09-11-qt-ai-editable-base-design.md`.

## Global Constraints

- **Không chạy `cargo fmt`**. Không commit `apps/qt-ai-gui/package.json` (user đang đổi version). Trước mỗi commit: `git checkout -- apps/qt-ai-gui/src-tauri/Cargo.toml` nếu chỉ đổi line ending.
- Test crate Tauri không chạy được trên máy dev Win10 (`STATUS_ENTRYPOINT_NOT_FOUND`): kiểm bằng `cargo clippy --all-targets` trong `apps/qt-ai-gui/src-tauri`.
- File Rust/TSX trong repo lẫn CRLF; Bash tool bóc backslash trong heredoc → sửa file bằng script Python viết qua Write tool (normalize `\r\n`), hoặc Edit tool.
- Identifier app: `com.vn-converter.qt-ai-gui` (hằng `APP_IDENTIFIER` trong core; test GUI đối chiếu `tauri.conf.json`).
- Đường dẫn file base: `base/prompts/<setting>-<names>.md`, `base/rules/<setting>.json`, `base/glossary/<setting>.json`.
- Không có file / file rỗng / file hỏng = dùng bản cứng (rules, prompt) hoặc rỗng (glossary). Rule bắt buộc "còn Hán tự" không nằm trong file, luôn chạy.
- Ưu tiên: prompt riêng truyện > file base > cứng; rule riêng truyện > file rule > cứng; glossary: file base nền, truyện đè key trùng, rồi lọc theo chương.
- Lệnh kiểm tra: core `cargo test -p qt-ai-core`; GUI `cd apps/qt-ai-gui && npm run check` (typecheck + lint + vitest + build).
- Commit kết thúc bằng `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.

---

## File structure

| File | Trách nhiệm |
|---|---|
| `crates/qt-ai-core/src/base.rs` (mới) | `BaseStore`: tính thư mục, đọc/ghi/xoá prompt/rules/glossary, `BaseSource` |
| `crates/qt-ai-core/src/lib.rs` | `pub mod base;` |
| `crates/qt-ai-core/src/prompt.rs` | `build_system_prompt` lấy base qua `BaseStore::from_env()` |
| `crates/qt-ai-core/src/check.rs` | `check_violations` lấy rule mặc định qua `BaseStore::from_env()` |
| `crates/qt-ai-core/src/commands/next.rs`, `src/api_session.rs` | truyền glossary nền `BaseStore::from_env().glossary(setting)` |
| `apps/qt-ai-gui/src-tauri/src/base_cmds.rs` (mới) | command `base_get`, `base_save`, `base_reset` |
| `apps/qt-ai-gui/src-tauri/src/lib.rs` | `AppState.base_dir`, đặt env `QT_AI_BASE_DIR`, đăng ký command |
| `apps/qt-ai-gui/src-tauri/src/story_cmds.rs` | `StoryDefaults` thêm `prompt_source`, `rules_source`, đọc qua `BaseStore` |
| `apps/qt-ai-gui/src/lib/schema.ts`, `api.ts`, `types.ts` | schema/API base, `storyDefaultsSchema` thêm source |
| `apps/qt-ai-gui/src/store/story.ts` | `baseVersion`, `bumpBaseVersion` |
| `apps/qt-ai-gui/src/hooks/use-story-defaults.ts` | nạp lại theo `baseVersion` |
| `apps/qt-ai-gui/src/components/rule-table.tsx` (mới) | bảng rule thuần props, dùng cho truyện lẫn base |
| `apps/qt-ai-gui/src/components/check-rules-editor.tsx` | dùng `RuleTable` |
| `apps/qt-ai-gui/src/components/base-prompt-dialog.tsx` (mới) | dialog sửa base prompt theo genre |
| `apps/qt-ai-gui/src/components/base-rules-dialog.tsx` (mới) | dialog sửa rule theo bối cảnh |
| `apps/qt-ai-gui/src/components/base-glossary-dialog.tsx` (mới) | dialog sửa glossary chung theo bối cảnh |
| `apps/qt-ai-gui/src/components/pages/settings-page.tsx` | card "Bản mặc định" |
| `apps/qt-ai-gui/src/components/prompt-editor.tsx`, `check-rules-editor.tsx`, `pages/story-page.tsx` | nhãn "mặc định của app (đã sửa)", dòng kho chung |
| `apps/qt-ai-gui/README.md`, spec | tài liệu |

---

### Task 1: Core `BaseStore` — thư mục, đọc, ghi, xoá

**Files:**
- Create: `crates/qt-ai-core/src/base.rs`
- Modify: `crates/qt-ai-core/src/lib.rs` (thêm `pub mod base;`)

**Interfaces:**
- Consumes: `crate::story::{Glossary, CheckRule, GenreSetting, StoryGenre, GLOSSARY_CATEGORIES}`, `crate::prompt::base_prompt`, `crate::check::default_rules_as_check_rules`, `crate::story_fs::write_atomic`.
- Produces:
  ```rust
  pub const APP_IDENTIFIER: &str = "com.vn-converter.qt-ai-gui";
  pub const BASE_DIR_ENV: &str = "QT_AI_BASE_DIR";
  pub enum BaseSource { Builtin, File(PathBuf) }
  pub struct BaseStore { dir: Option<PathBuf> }
  impl BaseStore {
      pub fn at(dir: impl Into<PathBuf>) -> Self;
      pub fn none() -> Self;                    // luôn builtin (test)
      pub fn from_env() -> Self;                // QT_AI_BASE_DIR hoặc platform_base_dir()
      pub fn dir(&self) -> Option<&Path>;
      pub fn prompt_path(&self, genre: &StoryGenre) -> Option<PathBuf>;
      pub fn rules_path(&self, setting: GenreSetting) -> Option<PathBuf>;
      pub fn glossary_path(&self, setting: GenreSetting) -> Option<PathBuf>;
      pub fn prompt(&self, genre: &StoryGenre) -> String;
      pub fn rules(&self, setting: GenreSetting) -> Vec<CheckRule>;
      pub fn glossary(&self, setting: GenreSetting) -> Glossary;
      pub fn prompt_source(&self, genre: &StoryGenre) -> BaseSource;
      pub fn rules_source(&self, setting: GenreSetting) -> BaseSource;
      pub fn glossary_source(&self, setting: GenreSetting) -> BaseSource;
      pub fn save_prompt(&self, genre: &StoryGenre, text: &str) -> Result<()>;   // trống → xoá
      pub fn save_rules(&self, setting: GenreSetting, rules: &[CheckRule]) -> Result<()>;
      pub fn save_glossary(&self, setting: GenreSetting, glossary: &Glossary) -> Result<()>;
      pub fn reset_prompt(&self, genre: &StoryGenre) -> Result<()>;
      pub fn reset_rules(&self, setting: GenreSetting) -> Result<()>;
      pub fn reset_glossary(&self, setting: GenreSetting) -> Result<()>;
  }
  pub fn platform_base_dir() -> Option<PathBuf>;  // <config_dir>/<APP_IDENTIFIER>/base
  ```

- [ ] **Step 1: Viết test đỏ** — tạo `crates/qt-ai-core/src/base.rs` với phần test trước (module thân rỗng để biên dịch thất bại vì thiếu hàm):

```rust
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
        let genre = StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign };
        store.save_prompt(&genre, "# Prompt của tôi\n").unwrap();
        assert_eq!(fs::read_to_string(dir.path().join("base/prompts/modern-foreign.md")).unwrap(), "# Prompt của tôi\n");
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
```

- [ ] **Step 2: Đăng ký module và chạy test để thấy đỏ**

Thêm `pub mod base;` vào `crates/qt-ai-core/src/lib.rs` ngay sau `pub mod check;`.

Run: `cargo test -p qt-ai-core base::`
Expected: lỗi biên dịch `cannot find type BaseStore`.

- [ ] **Step 3: Viết phần thân module** (chèn giữa `use` và `#[cfg(test)]`):

```rust
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
        self.read_rules_file(setting).map(|(_, rules)| rules).unwrap_or_else(|| default_rules_as_check_rules(setting))
    }

    pub fn glossary(&self, setting: GenreSetting) -> Glossary {
        self.read_glossary_file(setting).map(|(_, glossary)| glossary).unwrap_or_else(|| glossary_from_value(&Value::Null))
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
        path.ok_or_else(|| CoreError::Invalid("Không xác định được thư mục bản mặc định của app".to_string()))
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
```

Kiểm tra `CoreError` có biến thể lỗi chuỗi: mở `crates/qt-ai-core/src/error.rs`; nếu không có `Invalid(String)` thì dùng biến thể sẵn có tương đương (grep `pub enum CoreError`) và sửa `require` cho khớp. `CoreError::io(path)` trả closure `impl Fn(std::io::Error) -> CoreError` (đã dùng ở `story_fs.rs`).

- [ ] **Step 4: Chạy test xanh**

Run: `cargo test -p qt-ai-core base::`
Expected: `test result: ok. 5 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/base.rs crates/qt-ai-core/src/lib.rs
git commit -m "feat(qt-ai-core): BaseStore — base prompt/rule/glossary sửa được, file rời trong thư mục app

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 2: Core dùng `BaseStore` khi ghép prompt, glossary nền và rule

**Files:**
- Modify: `crates/qt-ai-core/src/prompt.rs` (`build_system_prompt`, dòng `let base = …`)
- Modify: `crates/qt-ai-core/src/check.rs` (`check_violations`, nhánh `configured.is_empty()`)
- Modify: `crates/qt-ai-core/src/commands/next.rs:35`, `crates/qt-ai-core/src/api_session.rs:237`
- Test: `crates/qt-ai-core/tests/base_wiring.rs` (mới)

**Interfaces:**
- Consumes: `qt_ai_core::base::{BaseStore, BASE_DIR_ENV}`, `prompt::build_system_prompt(workspace, story, source)`, `check::check_violations(text, configured, setting)`.
- Produces: không đổi chữ ký; hành vi đọc env `QT_AI_BASE_DIR`.

- [ ] **Step 1: Viết test tích hợp đỏ** `crates/qt-ai-core/tests/base_wiring.rs`:

```rust
//! BaseStore nối vào chỗ ghép prompt / rule qua env QT_AI_BASE_DIR. Env là toàn cục nên gộp một test.
use qt_ai_core::base::{BaseStore, BASE_DIR_ENV};
use qt_ai_core::check::check_violations;
use qt_ai_core::prompt::{build_system_prompt, TranslationGlossary};
use qt_ai_core::story::{CheckRule, GenreSetting, StoryConfig, StringMap};

#[test]
fn base_file_thay_ban_cung_nhung_truyen_van_thang() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base");
    std::env::set_var(BASE_DIR_ENV, &base);
    let store = BaseStore::at(&base);
    let mut story = StoryConfig::empty();

    // Prompt: chưa có file → bản cứng; có file → file; truyện có prompt riêng → riêng.
    let builtin = build_system_prompt(&TranslationGlossary::new(), Some(&story), None);
    assert!(builtin.contains("| 我          | **ta**"));
    store.save_prompt(&story.genre, "# BASE CUA TOI\n").unwrap();
    let from_file = build_system_prompt(&TranslationGlossary::new(), Some(&story), None);
    assert!(from_file.starts_with("# BASE CUA TOI"));
    assert!(from_file.contains("Dịch raw text tiếng Trung"), "suffix vẫn nối");
    story.custom_prompt = "# RIENG".to_string();
    assert!(build_system_prompt(&TranslationGlossary::new(), Some(&story), None).starts_with("# RIENG"));
    story.custom_prompt.clear();

    // Glossary nền: caller truyền store.glossary(setting); truyện đè key trùng; lọc theo chương vẫn chạy.
    let mut base_glossary = TranslationGlossary::new();
    base_glossary.insert("names".into(), StringMap::from([("赵".to_string(), "Triệu".to_string()), ("钱".to_string(), "Tiền".to_string())]));
    store.save_glossary(GenreSetting::Ancient, &base_glossary).unwrap();
    story.glossary.get_mut("names").unwrap().insert("赵".to_string(), "Triệu (truyện)".to_string());
    let prompt = build_system_prompt(&store.glossary(story.genre.setting), Some(&story), Some("赵 đi chợ"));
    assert!(prompt.contains("\"赵\": \"Triệu (truyện)\""));
    assert!(!prompt.contains("钱"), "钱 không có trong chương → bị lọc");

    // Rule: file thay bộ cứng; rule Hán tự vẫn chạy; truyện có rule riêng thì riêng thắng.
    let ancient_builtin = check_violations("Hừm, vợ hắn", &[], GenreSetting::Ancient);
    assert!(ancient_builtin.iter().any(|v| v.message.contains("thê tử/phu quân")));
    store.save_rules(GenreSetting::Ancient, &[CheckRule { pattern: "Hừm".into(), flags: None, message: "Hừm → Ân (base)".into() }]).unwrap();
    let ancient_file = check_violations("Hừm, vợ hắn 赵", &[], GenreSetting::Ancient);
    assert!(ancient_file.iter().any(|v| v.message == "Hừm → Ân (base)"));
    assert!(!ancient_file.iter().any(|v| v.message.contains("thê tử/phu quân")));
    assert!(ancient_file.iter().any(|v| v.message.contains("CJK còn sót")));
    let own = vec![CheckRule { pattern: "vợ".into(), flags: None, message: "riêng".into() }];
    let with_own = check_violations("Hừm, vợ hắn", &own, GenreSetting::Ancient);
    assert!(with_own.iter().any(|v| v.message == "riêng"));
    assert!(!with_own.iter().any(|v| v.message == "Hừm → Ân (base)"));

    std::env::remove_var(BASE_DIR_ENV);
}
```

Kiểm tra `Violation` có field `message` (grep `pub struct Violation` trong `check.rs`); nếu tên khác thì đổi cho khớp.

- [ ] **Step 2: Chạy để thấy đỏ**

Run: `cargo test -p qt-ai-core --test base_wiring`
Expected: FAIL ở `from_file.starts_with("# BASE CUA TOI")` (prompt vẫn bản cứng).

- [ ] **Step 3: Nối `BaseStore` vào core**

`prompt.rs`, trong `build_system_prompt`, thay:

```rust
    let default_genre = StoryGenre::default();
    let base = story
        .map(|s| s.custom_prompt.trim())
        .filter(|custom| !custom.is_empty())
        .unwrap_or_else(|| base_prompt(story.map(|s| &s.genre).unwrap_or(&default_genre)));
    format!("{base}{story_context}{glossary_section}{style_section}{}", prompt_suffix())
```

bằng:

```rust
    let default_genre = StoryGenre::default();
    let base = match story.map(|s| s.custom_prompt.trim()).filter(|custom| !custom.is_empty()) {
        Some(custom) => custom.to_string(),
        None => crate::base::BaseStore::from_env().prompt(story.map(|s| &s.genre).unwrap_or(&default_genre)),
    };
    format!("{base}{story_context}{glossary_section}{style_section}{}", prompt_suffix())
```

`check.rs`, trong `check_violations`, thay nhánh rỗng:

```rust
    let mut rules: Vec<CompiledRule> = if configured.is_empty() {
        crate::base::BaseStore::from_env()
            .rules(setting)
            .iter()
            .filter_map(|rule| compile(&rule.pattern, rule.flags.as_deref().unwrap_or(""), &rule.message))
            .collect()
    } else {
```

`commands/next.rs` dòng 35 và `api_session.rs` dòng 237: thay `&TranslationGlossary::new()` bằng
`&crate::base::BaseStore::from_env().glossary(story.genre.setting)` (ở `api_session.rs` biến là `story`, cùng tên).

- [ ] **Step 4: Chạy test**

Run: `cargo test -p qt-ai-core`
Expected: tất cả xanh, kể cả `base_wiring` và golden test cũ (không env → bản cứng, không đổi).

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/prompt.rs crates/qt-ai-core/src/check.rs crates/qt-ai-core/src/commands/next.rs crates/qt-ai-core/src/api_session.rs crates/qt-ai-core/tests/base_wiring.rs
git commit -m "feat(qt-ai-core): ghép prompt/rule/glossary nền qua BaseStore (env QT_AI_BASE_DIR)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 3: Tauri — `base_dir` trong AppState, env cho agy, command `base_get/base_save/base_reset`, `story_defaults` kèm source

**Files:**
- Create: `apps/qt-ai-gui/src-tauri/src/base_cmds.rs`
- Modify: `apps/qt-ai-gui/src-tauri/src/lib.rs` (AppState, setup, handler)
- Modify: `apps/qt-ai-gui/src-tauri/src/story_cmds.rs` (`StoryDefaults`, `defaults`)
- Modify: `docs/superpowers/specs/2026-09-11-qt-ai-editable-base-design.md` (mục 3: env đặt ở setup thay vì `SessionConfig.base_dir`)

**Interfaces:**
- Consumes: `qt_ai_core::base::{BaseStore, BaseSource, BASE_DIR_ENV}`.
- Produces (JSON camelCase):
  - `base_get(kind: "prompt"|"rules"|"glossary", setting, names?)` → `BaseView { kind, setting, names?, source: "builtin"|"file", text?: string, rules?: CheckRule[], glossary?: Glossary }`
  - `base_save(kind, setting, names?, text?, rules?, glossary?)` → `BaseView` (sau khi ghi)
  - `base_reset(kind, setting, names?)` → `BaseView`
  - `StoryDefaults` thêm `prompt_source: "builtin"|"file"`, `rules_source`.
  - `AppState.base_dir: PathBuf`; `defaults(genre, store: &BaseStore)`.

- [ ] **Step 1: Viết `base_cmds.rs` với test trước**

```rust
//! Bản mặc định của app (base prompt / rule / glossary) — bọc `qt_ai_core::base::BaseStore`.
use crate::error::{blocking, CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::base::{BaseSource, BaseStore};
use qt_ai_core::story::{CheckRule, GenreNames, GenreSetting, Glossary, StoryGenre};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseKind {
    Prompt,
    Rules,
    Glossary,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseView {
    pub kind: BaseKind,
    pub setting: GenreSetting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<GenreNames>,
    /// "builtin" | "file"
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<CheckRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary: Option<Glossary>,
}

fn source_label(source: BaseSource) -> String {
    match source {
        BaseSource::Builtin => "builtin".to_string(),
        BaseSource::File(_) => "file".to_string(),
    }
}

fn genre_of(kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<StoryGenre> {
    match (kind, names) {
        (BaseKind::Prompt, Some(names)) => Ok(StoryGenre { setting, names }),
        (BaseKind::Prompt, None) => Err(CommandError::new("invalid", "Base prompt cần cả bối cảnh và kiểu tên riêng")),
        (_, _) => Ok(StoryGenre { setting, names: names.unwrap_or_default() }),
    }
}

pub fn view(store: &BaseStore, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    let mut out = BaseView { kind, setting, names: None, source: String::new(), text: None, rules: None, glossary: None };
    match kind {
        BaseKind::Prompt => {
            out.names = Some(genre.names);
            out.source = source_label(store.prompt_source(&genre));
            out.text = Some(store.prompt(&genre));
        }
        BaseKind::Rules => {
            out.source = source_label(store.rules_source(setting));
            out.rules = Some(store.rules(setting));
        }
        BaseKind::Glossary => {
            out.source = source_label(store.glossary_source(setting));
            out.glossary = Some(store.glossary(setting));
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasePayload {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<CheckRule>>,
    #[serde(default)]
    pub glossary: Option<Glossary>,
}

pub fn save(store: &BaseStore, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>, payload: BasePayload) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    match kind {
        BaseKind::Prompt => store.save_prompt(&genre, payload.text.as_deref().unwrap_or(""))?,
        BaseKind::Rules => store.save_rules(setting, &payload.rules.unwrap_or_default())?,
        BaseKind::Glossary => store.save_glossary(setting, &payload.glossary.unwrap_or_default())?,
    }
    view(store, kind, setting, names)
}

pub fn reset(store: &BaseStore, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    match kind {
        BaseKind::Prompt => store.reset_prompt(&genre)?,
        BaseKind::Rules => store.reset_rules(setting)?,
        BaseKind::Glossary => store.reset_glossary(setting)?,
    }
    view(store, kind, setting, names)
}

#[tauri::command]
pub async fn base_get<R: Runtime>(app: AppHandle<R>, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    blocking(move || view(&app.state::<AppState>().base_store(), kind, setting, names)).await
}

#[tauri::command]
pub async fn base_save<R: Runtime>(
    app: AppHandle<R>,
    kind: BaseKind,
    setting: GenreSetting,
    names: Option<GenreNames>,
    payload: BasePayload,
) -> CmdResult<BaseView> {
    blocking(move || save(&app.state::<AppState>().base_store(), kind, setting, names, payload)).await
}

#[tauri::command]
pub async fn base_reset<R: Runtime>(app: AppHandle<R>, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    blocking(move || reset(&app.state::<AppState>().base_store(), kind, setting, names)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_save_reset_theo_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = BaseStore::at(dir.path());
        let v = view(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han)).unwrap();
        assert_eq!(v.source, "builtin");
        assert!(v.text.as_deref().unwrap().len() > 5000);
        assert!(view(&store, BaseKind::Prompt, GenreSetting::Modern, None).is_err());

        let saved = save(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han), BasePayload { text: Some("# x".into()), rules: None, glossary: None }).unwrap();
        assert_eq!(saved.source, "file");
        assert_eq!(saved.text.as_deref(), Some("# x"));
        assert_eq!(reset(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han)).unwrap().source, "builtin");

        let rules = vec![CheckRule { pattern: "a".into(), flags: None, message: "b".into() }];
        let saved = save(&store, BaseKind::Rules, GenreSetting::Ancient, None, BasePayload { text: None, rules: Some(rules.clone()), glossary: None }).unwrap();
        assert_eq!(saved.rules, Some(rules));
        assert_eq!(saved.source, "file");

        let json = serde_json::to_value(&saved).unwrap();
        assert_eq!(json["kind"], "rules");
        assert!(json.get("names").is_none());
        assert!(json.get("text").is_none());

        let g = view(&store, BaseKind::Glossary, GenreSetting::Mixed, None).unwrap();
        assert_eq!(g.glossary.as_ref().unwrap().len(), 8);
    }
}
```

`GenreNames` cần `Default` (kiểm `derive(Default)` trong `story.rs`; `#[default] Han` — nếu thiếu thì thêm). `GenreSetting`/`GenreNames` phải `Deserialize` với `rename_all = "lowercase"` (đã có, dùng trong story.json).

- [ ] **Step 2: `lib.rs`** — thêm `mod base_cmds;`, field và helper trong `AppState`, đặt env ở setup, đăng ký 3 command:

```rust
pub struct AppState {
    pub config_path: PathBuf,
    /// `<app_config_dir>/base` — bản mặc định sửa được (qt_ai_core::base). Cũng đặt vào env
    /// QT_AI_BASE_DIR lúc khởi động để phiên API trong app và tiến trình agy con cùng thấy.
    pub base_dir: PathBuf,
    pub config: Mutex<AppConfig>,
    pub sessions: Mutex<SessionRegistry>,
}

impl AppState {
    pub fn base_store(&self) -> qt_ai_core::base::BaseStore {
        qt_ai_core::base::BaseStore::at(&self.base_dir)
    }
}
```

Trong `setup`:

```rust
            let config_path = resolve_config_path(app.path().app_config_dir().ok());
            migrate_legacy_config(&config_path);
            let base_dir = config_path.parent().map(|dir| dir.join("base")).unwrap_or_else(|| PathBuf::from("base"));
            std::env::set_var(qt_ai_core::base::BASE_DIR_ENV, &base_dir);
            let config = AppConfig::load(&config_path);
            app.manage(AppState {
                config_path,
                base_dir,
                config: Mutex::new(config),
                sessions: Mutex::new(SessionRegistry::new()),
            });
```

Handler: thêm `base_cmds::base_get, base_cmds::base_save, base_cmds::base_reset,` vào `generate_handler!`. Danh sách `heavy` trong test `lenh_nang_phai_la_async_fn_boc_blocking` thêm `"base_get", "base_save", "base_reset"` và `include_str!("base_cmds.rs")` vào `sources`.

- [ ] **Step 3: `story_cmds.rs`** — `StoryDefaults` và `defaults`:

```rust
pub struct StoryDefaults {
    pub base_prompt: String,
    /// "builtin" | "file" — file = người dùng đã sửa base ở Cài đặt.
    pub prompt_source: String,
    pub prompt_suffix: String,
    pub check_rules: Vec<CheckRule>,
    pub rules_source: String,
}

pub fn defaults(genre: &StoryGenre, store: &BaseStore) -> StoryDefaults {
    let label = |source: BaseSource| match source {
        BaseSource::Builtin => "builtin".to_string(),
        BaseSource::File(_) => "file".to_string(),
    };
    StoryDefaults {
        base_prompt: store.prompt(genre),
        prompt_source: label(store.prompt_source(genre)),
        prompt_suffix: prompt_suffix().to_string(),
        check_rules: store.rules(genre.setting),
        rules_source: label(store.rules_source(genre.setting)),
    }
}

#[tauri::command]
pub fn story_defaults(state: State<'_, AppState>, genre: StoryGenre) -> CmdResult<StoryDefaults> {
    Ok(defaults(&genre, &state.base_store()))
}
```

Import `qt_ai_core::base::{BaseSource, BaseStore}`; bỏ import `base_prompt` và `default_rules_as_check_rules` nếu không còn dùng. Test `defaults_theo_genre` gọi `defaults(&…, &BaseStore::none())` và thêm `assert_eq!(d.prompt_source, "builtin")`.

- [ ] **Step 4: Sửa spec** mục 3, câu "Khi spawn agy (`session_config`) đặt env…" thành: "Lúc khởi động app đặt env `QT_AI_BASE_DIR` = `<app_config_dir>/base` (tiến trình agy con thừa hưởng env; phiên API và `story_defaults` trong app dùng `AppState.base_dir`)."

- [ ] **Step 5: Biên dịch kiểm tra**

Run: `cd apps/qt-ai-gui/src-tauri && cargo clippy --all-targets`
Expected: không lỗi; chỉ còn cảnh báo cũ (`div_ceil`, `items after a test module`).

- [ ] **Step 6: Commit**

```bash
git add apps/qt-ai-gui/src-tauri/src/base_cmds.rs apps/qt-ai-gui/src-tauri/src/lib.rs apps/qt-ai-gui/src-tauri/src/story_cmds.rs docs/superpowers/specs/2026-09-11-qt-ai-editable-base-design.md
git commit -m "feat(qt-ai-gui): command base_get/base_save/base_reset, story_defaults kèm nguồn, env QT_AI_BASE_DIR

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 4: GUI — schema, API, store `baseVersion`, hook nạp lại

**Files:**
- Modify: `apps/qt-ai-gui/src/lib/schema.ts` (`storyDefaultsSchema`, thêm `baseViewSchema`)
- Modify: `apps/qt-ai-gui/src/lib/types.ts` (`BaseView`, `BaseKind`)
- Modify: `apps/qt-ai-gui/src/lib/api.ts` (`baseGet`, `baseSave`, `baseReset`)
- Modify: `apps/qt-ai-gui/src/store/story.ts` (`baseVersion`, `bumpBaseVersion`)
- Modify: `apps/qt-ai-gui/src/hooks/use-story-defaults.ts`
- Test: `apps/qt-ai-gui/src/hooks/use-story-defaults.test.ts` (mới), `apps/qt-ai-gui/src/store/story.test.ts`

**Interfaces:**
- Produces:
  ```ts
  export const baseViewSchema = z.object({ kind: z.enum(["prompt","rules","glossary"]), setting: z.enum(GENRE_SETTINGS), names: z.enum(GENRE_NAMES).optional(), source: z.enum(["builtin","file"]), text: z.string().optional(), rules: z.array(checkRuleSchema).optional(), glossary: glossaryRecordSchema.optional() });
  export type BaseView = z.infer<typeof baseViewSchema>; export type BaseKind = BaseView["kind"];
  baseGet(kind, setting, names?) / baseSave(kind, setting, names | undefined, payload: { text?; rules?; glossary? }) / baseReset(kind, setting, names?) → Promise<BaseView>
  useStoryStore: baseVersion: number; bumpBaseVersion(): void
  storyDefaultsSchema += promptSource, rulesSource: z.enum(["builtin","file"])
  ```

- [ ] **Step 1: Test đỏ** — `src/store/story.test.ts` thêm:

```ts
describe("baseVersion", () => {
  it("bumpBaseVersion tăng để hook defaults nạp lại", () => {
    const before = useStoryStore.getState().baseVersion;
    useStoryStore.getState().bumpBaseVersion();
    expect(useStoryStore.getState().baseVersion).toBe(before + 1);
  });
});
```

và `src/hooks/use-story-defaults.test.ts`:

```ts
import { renderHook, waitFor } from "@testing-library/react";
import { act } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useStoryDefaults } from "@/hooks/use-story-defaults";
import { storyDefaults } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ storyDefaults: vi.fn() }));

const value = (tag: string) => ({ basePrompt: tag, promptSource: "file" as const, promptSuffix: "s", checkRules: [], rulesSource: "builtin" as const });

describe("useStoryDefaults", () => {
  beforeEach(() => {
    vi.mocked(storyDefaults).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp theo genre, dùng lại kết quả cho cùng genre, nạp lại khi baseVersion tăng", async () => {
    vi.mocked(storyDefaults).mockResolvedValueOnce(value("v1")).mockResolvedValueOnce(value("v2"));
    const genre = { setting: "ancient" as const, names: "han" as const };
    const { result, rerender } = renderHook(() => useStoryDefaults(genre));
    await waitFor(() => expect(result.current?.basePrompt).toBe("v1"));
    rerender();
    expect(storyDefaults).toHaveBeenCalledTimes(1);
    act(() => useStoryStore.getState().bumpBaseVersion());
    await waitFor(() => expect(result.current?.basePrompt).toBe("v2"));
    expect(storyDefaults).toHaveBeenCalledTimes(2);
  });
});
```

Run: `cd apps/qt-ai-gui && npx vitest run src/hooks/use-story-defaults.test.ts src/store/story.test.ts`
Expected: FAIL (`bumpBaseVersion is not a function`, typecheck lỗi `baseVersion`).

- [ ] **Step 2: schema/types/api**

`schema.ts`: tách record glossary thành hằng dùng chung (đặt trên `storyConfigSchema`):

```ts
export const glossaryRecordSchema = z.object({
  names: stringRecord,
  places: stringRecord,
  items: stringRecord,
  creatures: stringRecord,
  skills: stringRecord,
  common: stringRecord,
  signature_phrases: stringRecord,
  /** `甲→乙: X–Y` xưng hô theo cặp; story.json cũ thiếu nhóm này. */
  addressing: stringRecord.default({}),
});
```

và trong `storyConfigSchema` dùng `glossary: glossaryRecordSchema`. Sửa `storyDefaultsSchema`:

```ts
export const BASE_SOURCES = ["builtin", "file"] as const;
export const storyDefaultsSchema = z.object({
  basePrompt: z.string(),
  promptSource: z.enum(BASE_SOURCES),
  promptSuffix: z.string(),
  checkRules: z.array(checkRuleSchema),
  rulesSource: z.enum(BASE_SOURCES),
});
export const BASE_KINDS = ["prompt", "rules", "glossary"] as const;
export const baseViewSchema = z.object({
  kind: z.enum(BASE_KINDS),
  setting: z.enum(GENRE_SETTINGS),
  names: z.enum(GENRE_NAMES).optional(),
  source: z.enum(BASE_SOURCES),
  text: z.string().optional(),
  rules: z.array(checkRuleSchema).optional(),
  glossary: glossaryRecordSchema.optional(),
});
```

`types.ts`: `export type BaseView = z.infer<typeof baseViewSchema>; export type BaseKind = BaseView["kind"]; export type BaseSource = BaseView["source"];` (import `baseViewSchema`).

`api.ts`:

```ts
export interface BasePayload {
  text?: string;
  rules?: CheckRule[];
  glossary?: StoryConfig["glossary"];
}
export const baseGet = (kind: BaseKind, setting: GenreSetting, names?: GenreNames) =>
  call("base_get", { kind, setting, names }, (v) => baseViewSchema.parse(v));
export const baseSave = (kind: BaseKind, setting: GenreSetting, names: GenreNames | undefined, payload: BasePayload) =>
  call("base_save", { kind, setting, names, payload }, (v) => baseViewSchema.parse(v));
export const baseReset = (kind: BaseKind, setting: GenreSetting, names?: GenreNames) =>
  call("base_reset", { kind, setting, names }, (v) => baseViewSchema.parse(v));
```

(`CheckRule`, `GenreSetting`, `GenreNames` là type sẵn có trong `types.ts` — kiểm `grep "export type CheckRule\|GenreSetting\|GenreNames" src/lib/types.ts`; nếu thiếu `CheckRule` thì thêm `export type CheckRule = z.infer<typeof checkRuleSchema>;`).

- [ ] **Step 3: store + hook**

`store/story.ts`: trong `StoryState` thêm `/** Tăng mỗi lần dialog Bản mặc định lưu/xoá — hook defaults nạp lại. */ baseVersion: number; bumpBaseVersion: () => void;`; khởi tạo `baseVersion: 0,` và `bumpBaseVersion: () => set((state) => ({ baseVersion: state.baseVersion + 1 })),`.

`use-story-defaults.ts` thay toàn bộ:

```ts
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { storyDefaults } from "@/lib/api";
import type { StoryDefaults, StoryGenre } from "@/lib/types";
import { useStoryStore } from "@/store/story";

const cache = new Map<string, Promise<StoryDefaults>>();

/**
 * Prompt gốc + rule mặc định theo genre. Cache theo genre + baseVersion: dialog "Bản mặc định" lưu/xoá
 * xong bump version nên tab Prompt/Rule của truyện thấy bản mới ngay, không phải mở lại app.
 */
export function useStoryDefaults(genre: StoryGenre): StoryDefaults | undefined {
  const baseVersion = useStoryStore((s) => s.baseVersion);
  const key = `${genre.setting}/${genre.names}#${baseVersion}`;
  const [state, setState] = useState<{ key: string; value: StoryDefaults } | undefined>();
  useEffect(() => {
    let cancelled = false;
    let pending = cache.get(key);
    if (!pending) {
      pending = storyDefaults(genre);
      cache.set(key, pending);
    }
    pending
      .then((value) => {
        if (!cancelled) setState({ key, value });
      })
      .catch((error: unknown) => {
        cache.delete(key);
        toast.error(error instanceof Error ? error.message : "Không đọc được prompt mặc định");
      });
    return () => {
      cancelled = true;
    };
  }, [key, genre]);
  return state?.key === key ? state.value : undefined;
}
```

Sửa mọi fixture `storyDefaults` trong test hiện có (`story-page.test.tsx`, `prompt-editor.test.tsx` nếu có) thêm `promptSource: "builtin", rulesSource: "builtin"` — grep `promptSuffix` trong `src/**/*.test.tsx`.

- [ ] **Step 4: Chạy check**

Run: `cd apps/qt-ai-gui && npm run check`
Expected: typecheck/lint/test/build xanh.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/lib/schema.ts apps/qt-ai-gui/src/lib/types.ts apps/qt-ai-gui/src/lib/api.ts apps/qt-ai-gui/src/store/story.ts apps/qt-ai-gui/src/store/story.test.ts apps/qt-ai-gui/src/hooks/use-story-defaults.ts apps/qt-ai-gui/src/hooks/use-story-defaults.test.ts apps/qt-ai-gui/src/components/pages/story-page.test.tsx
git commit -m "feat(qt-ai-gui): API base_get/save/reset, storyDefaults kèm nguồn, hook defaults nạp lại theo baseVersion

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 5: GUI — tách `RuleTable` khỏi `CheckRulesEditor`

**Files:**
- Create: `apps/qt-ai-gui/src/components/rule-table.tsx`
- Modify: `apps/qt-ai-gui/src/components/check-rules-editor.tsx`
- Test: `apps/qt-ai-gui/src/components/rule-table.test.tsx` (mới)

**Interfaces:**
- Produces:
  ```ts
  export interface RuleRow { pattern: string; flags: string; message: string }
  export function RuleTable(props: { rows: RuleRow[]; onChange: (rows: RuleRow[]) => void; readOnly?: boolean })
  ```
  Có sửa: mỗi dòng 3 `Input` (aria-label `Regex N`, `Flags N`, `Mô tả rule N`) + nút "Xoá rule"; nút "Thêm" ở cuối. `readOnly`: hiện dạng code/text như bảng mặc định hiện nay.

- [ ] **Step 1: Test đỏ** `rule-table.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { RuleTable } from "@/components/rule-table";

describe("RuleTable", () => {
  it("sửa ô, thêm và xoá dòng gọi onChange với mảng mới", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<RuleTable rows={[{ pattern: "a", flags: "i", message: "m" }]} onChange={onChange} />);
    await user.type(screen.getByLabelText("Mô tả rule 1"), "!");
    expect(onChange).toHaveBeenLastCalledWith([{ pattern: "a", flags: "i", message: "m!" }]);
    await user.click(screen.getByRole("button", { name: "Thêm" }));
    expect(onChange).toHaveBeenLastCalledWith([{ pattern: "a", flags: "i", message: "m" }, { pattern: "", flags: "", message: "" }]);
    await user.click(screen.getByRole("button", { name: "Xoá rule" }));
    expect(onChange).toHaveBeenLastCalledWith([]);
  });

  it("readOnly hiện chữ, không có input", () => {
    render(<RuleTable rows={[{ pattern: "a", flags: "", message: "m" }]} onChange={() => undefined} readOnly />);
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(screen.getByText("m")).toBeInTheDocument();
  });
});
```

Run: `npx vitest run src/components/rule-table.test.tsx` → FAIL (module không tồn tại).

- [ ] **Step 2: `rule-table.tsx`**

```tsx
import { Plus, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export interface RuleRow {
  pattern: string;
  flags: string;
  message: string;
}

interface Props {
  rows: RuleRow[];
  onChange: (rows: RuleRow[]) => void;
  /** Chỉ hiện (bộ mặc định chưa sao chép ra), không có ô nhập. */
  readOnly?: boolean;
}

/** Bảng rule regex thuần props — dùng cho rule riêng của truyện lẫn base rule ở Cài đặt. */
export function RuleTable({ rows, onChange, readOnly }: Props) {
  if (readOnly) {
    return (
      <div className="flex flex-col gap-1">
        {rows.map((rule, index) => (
          <div key={index} className="grid grid-cols-[2fr_60px_2fr] gap-1 text-xs text-muted-foreground">
            <code className="truncate rounded bg-muted px-2 py-1 font-mono" title={rule.pattern}>
              {rule.pattern}
            </code>
            <code className="rounded bg-muted px-2 py-1 font-mono">{rule.flags}</code>
            <span className="truncate px-2 py-1" title={rule.message}>
              {rule.message}
            </span>
          </div>
        ))}
      </div>
    );
  }
  const update = (index: number, patch: Partial<RuleRow>) =>
    onChange(rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  return (
    <div className="flex flex-col gap-1">
      {rows.map((rule, index) => (
        <div key={index} className="grid grid-cols-[2fr_60px_2fr_auto] gap-1">
          <Input
            value={rule.pattern}
            onChange={(e) => update(index, { pattern: e.target.value })}
            placeholder="regex (cú pháp JS)"
            aria-label={`Regex ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            value={rule.flags}
            onChange={(e) => update(index, { flags: e.target.value })}
            placeholder="i"
            aria-label={`Flags ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            value={rule.message}
            onChange={(e) => update(index, { message: e.target.value })}
            placeholder="Mô tả / cách sửa"
            aria-label={`Mô tả rule ${index + 1}`}
            className="h-8"
          />
          <Button type="button" size="icon-sm" variant="ghost" aria-label="Xoá rule" onClick={() => onChange(rows.filter((_, i) => i !== index))}>
            <Trash2 />
          </Button>
        </div>
      ))}
      <Button type="button" size="xs" variant="ghost" className="w-fit" onClick={() => onChange([...rows, { pattern: "", flags: "", message: "" }])}>
        <Plus /> Thêm
      </Button>
    </div>
  );
}
```

- [ ] **Step 3: `check-rules-editor.tsx` dùng `RuleTable`**, giữ nguyên hành vi và mọi aria-label/nhãn hiện có (test `story-page.test.tsx` đang dựa vào "Sửa bộ mặc định", "Về mặc định", "Thêm"):

```tsx
import { Pencil } from "lucide-react";
import { useFieldArray, useFormContext, useWatch } from "react-hook-form";

import { RuleTable, type RuleRow } from "@/components/rule-table";
import { Button } from "@/components/ui/button";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

function defaultRows(defaults: StoryDefaults | undefined): RuleRow[] {
  return (defaults?.checkRules ?? []).map((rule) => ({ pattern: rule.pattern, flags: rule.flags ?? "", message: rule.message }));
}

/**
 * Trống trong story.json = dùng bộ mặc định của app: hiện bộ đó (chỉ đọc); "Sửa bộ mặc định" sao chép ra
 * thành rule riêng; "Về mặc định" xoá bản riêng. Bộ mặc định có thể là bản người dùng đã sửa ở Cài đặt.
 */
export function CheckRulesEditor({ defaults }: { defaults: StoryDefaults | undefined }) {
  const { control } = useFormContext<StoryFormValues>();
  const { fields, replace } = useFieldArray({ control, name: "checkRules" });
  const values = useWatch({ control, name: "checkRules" }) ?? [];
  const usingDefaults = fields.length === 0;
  const rows = usingDefaults ? defaultRows(defaults) : values;
  const edited = defaults?.rulesSource === "file";
  return (
    <fieldset className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <legend className="text-sm font-medium">
          Rule kiểm tra{" "}
          <span className="rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {usingDefaults ? `mặc định của app${edited ? " (đã sửa)" : ""} · ${defaults ? rows.length : "…"}` : `riêng · ${fields.length}`}
          </span>
          <span className="ml-2 text-xs text-muted-foreground">(rule CJK còn sót luôn chạy)</span>
        </legend>
        <div className="flex gap-1">
          {usingDefaults ? (
            <Button type="button" size="xs" variant="secondary" disabled={!defaults} onClick={() => replace(defaultRows(defaults))}>
              <Pencil /> Sửa bộ mặc định
            </Button>
          ) : (
            <Button type="button" size="xs" variant="ghost" onClick={() => replace([])}>
              Về mặc định
            </Button>
          )}
        </div>
      </div>
      <RuleTable
        rows={rows}
        readOnly={usingDefaults}
        onChange={(next) => replace(next)}
      />
    </fieldset>
  );
}
```

Lưu ý: nút "Thêm" khi đang dùng mặc định trước đây sao chép bộ mặc định rồi thêm dòng; nay ở chế độ chỉ đọc không có nút Thêm — người dùng bấm "Sửa bộ mặc định" rồi Thêm. Nếu `story-page.test.tsx` có test bấm "Thêm" khi đang mặc định thì sửa test theo luồng mới (grep `"Thêm"` trong test đó).

- [ ] **Step 4: Chạy check**

Run: `cd apps/qt-ai-gui && npm run check` → xanh.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/components/rule-table.tsx apps/qt-ai-gui/src/components/rule-table.test.tsx apps/qt-ai-gui/src/components/check-rules-editor.tsx apps/qt-ai-gui/src/components/pages/story-page.test.tsx
git commit -m "refactor(qt-ai-gui): tách RuleTable thuần props khỏi CheckRulesEditor, nhãn 'mặc định của app (đã sửa)'

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 6: GUI — `BasePromptDialog`

**Files:**
- Create: `apps/qt-ai-gui/src/components/base-prompt-dialog.tsx`
- Test: `apps/qt-ai-gui/src/components/base-prompt-dialog.test.tsx`

**Interfaces:**
- Consumes: `baseGet/baseSave/baseReset` (Task 4), `useStoryStore.bumpBaseVersion`, `PlatePromptEditor` (lazy, props `{ initialValue, onChange }`), `Choice` — **chuyển `Choice` từ `settings-page.tsx` sang file mới `apps/qt-ai-gui/src/components/choice.tsx`** (export y nguyên, settings-page import lại) để dialog dùng chung.
- Produces: `export function BasePromptDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void })`.

- [ ] **Step 1: Test đỏ** `base-prompt-dialog.test.tsx`:

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BasePromptDialog } from "@/components/base-prompt-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));
// Plate nặng và không cần cho test: editor giả là textarea gọi onChange.
vi.mock("@/components/plate-prompt-editor", () => ({
  PlatePromptEditor: ({ initialValue, onChange }: { initialValue: string; onChange: (v: string) => void }) => (
    <textarea aria-label="Editor" defaultValue={initialValue} onChange={(e) => onChange(e.target.value)} />
  ),
}));

const view = (setting: string, names: string, source: "builtin" | "file", text: string) =>
  ({ kind: "prompt", setting, names, source, text }) as never;

describe("BasePromptDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp genre mặc định, sửa rồi Lưu gọi base_save, nhãn đổi sang đã sửa, baseVersion tăng", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "han", "builtin", "# gốc"));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "han", "file", "# gốc!"));
    render(<BasePromptDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
    expect(baseGet).toHaveBeenCalledWith("prompt", "ancient", "han");
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();

    await user.type(screen.getByLabelText("Editor"), "!");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() => expect(baseSave).toHaveBeenCalledWith("prompt", "ancient", "han", { text: "# gốc!" }));
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("đổi genre nạp lại; Về mặc định gọi base_reset", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet)
      .mockResolvedValueOnce(view("ancient", "han", "builtin", "# gốc"))
      .mockResolvedValueOnce(view("modern", "han", "file", "# hiện đại"));
    vi.mocked(baseReset).mockResolvedValue(view("modern", "han", "builtin", "# cứng"));
    render(<BasePromptDialog open onOpenChange={() => undefined} />);
    await screen.findByText("bản cứng");
    await user.click(screen.getByRole("radio", { name: "Hiện đại" }));
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(baseGet).toHaveBeenLastCalledWith("prompt", "modern", "han");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("prompt", "modern", "han"));
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });
});
```

Run: `npx vitest run src/components/base-prompt-dialog.test.tsx` → FAIL (module không tồn tại).

- [ ] **Step 2: Tách `Choice`** — tạo `src/components/choice.tsx` với đúng hàm `Choice` hiện ở `settings-page.tsx` (export), xoá khỏi settings-page và `import { Choice } from "@/components/choice";`.

- [ ] **Step 3: `base-prompt-dialog.tsx`**

```tsx
import { LoaderCircle, RotateCcw } from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { GENRE_NAMES, GENRE_SETTINGS } from "@/lib/schema";
import { GENRE_NAMES_LABELS, GENRE_SETTING_LABELS, type BaseView, type GenreNames, type GenreSetting } from "@/lib/types";
import { useStoryStore } from "@/store/story";

const PlatePromptEditor = lazy(async () => {
  const module = await import("@/components/plate-prompt-editor");
  return { default: module.PlatePromptEditor };
});

const SETTING_LABELS = Object.fromEntries(GENRE_SETTINGS.map((s) => [s, GENRE_SETTING_LABELS[s].label])) as Record<GenreSetting, string>;
const NAMES_LABELS = Object.fromEntries(GENRE_NAMES.map((n) => [n, GENRE_NAMES_LABELS[n].label])) as Record<GenreNames, string>;

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Sửa base prompt của app theo genre (9 tổ hợp). Lưu = ghi file base/prompts/<setting>-<names>.md;
 * Về mặc định = xoá file. Truyện đang dùng prompt mặc định ăn theo (bumpBaseVersion nạp lại defaults).
 */
export function BasePromptDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  const [names, setNames] = useState<GenreNames>("han");
  const [view, setView] = useState<BaseView | undefined>();
  const [draft, setDraft] = useState<string | undefined>();
  const [version, setVersion] = useState(0);
  const [busy, setBusy] = useState(false);
  const dirty = draft !== undefined && draft !== view?.text;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setView(undefined);
    setDraft(undefined);
    baseGet("prompt", setting, names)
      .then((next) => {
        if (cancelled) return;
        setView(next);
        setVersion((v) => v + 1);
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được base prompt"));
    return () => {
      cancelled = true;
    };
  }, [open, setting, names]);

  function pick<T>(apply: (value: T) => void) {
    return (value: T) => {
      if (dirty && !window.confirm("Bỏ thay đổi chưa lưu?")) return;
      apply(value);
    };
  }

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setView(next);
      setDraft(undefined);
      setVersion((v) => v + 1);
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Prompt mặc định</DialogTitle>
          <DialogDescription>
            Bản dùng cho mọi truyện chưa có prompt riêng. Sửa nguyên văn theo từng tổ hợp bối cảnh × tên riêng.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap items-end gap-4">
          <div className="flex flex-col gap-1.5">
            <Label>Bối cảnh</Label>
            <Choice label="Bối cảnh" value={setting} options={GENRE_SETTINGS} labels={SETTING_LABELS} onChange={pick(setSetting)} disabled={busy} />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>Tên riêng</Label>
            <Choice label="Tên riêng" value={names} options={GENRE_NAMES} labels={NAMES_LABELS} onChange={pick(setNames)} disabled={busy} />
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"}
          </span>
        </div>
        <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
          {view?.text !== undefined ? (
            <Suspense fallback={<div role="status" className="grid min-h-[420px] place-items-center text-sm text-muted-foreground">Đang tải editor…</div>}>
              <PlatePromptEditor key={version} initialValue={view.text} onChange={setDraft} />
            </Suspense>
          ) : (
            <div role="status" className="grid min-h-[420px] place-items-center text-sm text-muted-foreground">
              Đang tải…
            </div>
          )}
        </div>
        <DialogFooter>
          <Button
            type="button"
            variant="ghost"
            disabled={busy || view?.source !== "file"}
            onClick={() => void run(() => baseReset("prompt", setting, names), "Đã về bản cứng")}
          >
            <RotateCcw /> Về mặc định
          </Button>
          <Button
            type="button"
            disabled={busy || !dirty}
            onClick={() => void run(() => baseSave("prompt", setting, names, { text: draft ?? "" }), "Đã lưu prompt mặc định")}
          >
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

`GenreSetting`/`GenreNames` type: kiểm `types.ts` có export (dùng ở `GENRE_SETTING_LABELS: Record<GenreSetting,…>` nên có).

- [ ] **Step 4: Chạy test dialog rồi check toàn bộ**

Run: `npx vitest run src/components/base-prompt-dialog.test.tsx` → PASS. Rồi `npm run check` → xanh.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/components/choice.tsx apps/qt-ai-gui/src/components/pages/settings-page.tsx apps/qt-ai-gui/src/components/base-prompt-dialog.tsx apps/qt-ai-gui/src/components/base-prompt-dialog.test.tsx
git commit -m "feat(qt-ai-gui): dialog Prompt mặc định — sửa base prompt từng genre, lưu/về bản cứng

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 7: GUI — `BaseRulesDialog`

**Files:**
- Create: `apps/qt-ai-gui/src/components/base-rules-dialog.tsx`
- Test: `apps/qt-ai-gui/src/components/base-rules-dialog.test.tsx`

**Interfaces:**
- Consumes: `RuleTable` (Task 5), `baseGet/baseSave/baseReset`, `Choice`.
- Produces: `export function BaseRulesDialog({ open, onOpenChange })`.

- [ ] **Step 1: Test đỏ**

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BaseRulesDialog } from "@/components/base-rules-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));

const view = (setting: string, source: "builtin" | "file", rules: { pattern: string; flags?: string; message: string }[]) =>
  ({ kind: "rules", setting, source, rules }) as never;

describe("BaseRulesDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp bộ cổ đại, sửa dòng và Lưu gửi rules đã chuẩn hoá (flags trống → bỏ), bump version", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "builtin", [{ pattern: "a", flags: "i", message: "m" }]));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "file", [{ pattern: "a", flags: "i", message: "m!" }]));
    render(<BaseRulesDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByLabelText("Mô tả rule 1")).toHaveValue("m");
    expect(baseGet).toHaveBeenCalledWith("rules", "ancient", undefined);
    await user.type(screen.getByLabelText("Mô tả rule 1"), "!");
    await user.click(screen.getByRole("button", { name: "Thêm" }));
    await user.type(screen.getByLabelText("Regex 2"), "b");
    await user.type(screen.getByLabelText("Mô tả rule 2"), "n");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() =>
      expect(baseSave).toHaveBeenCalledWith("rules", "ancient", undefined, {
        rules: [{ pattern: "a", flags: "i", message: "m!" }, { pattern: "b", message: "n" }],
      }),
    );
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("đổi bối cảnh nạp lại; Về mặc định gọi base_reset", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet)
      .mockResolvedValueOnce(view("ancient", "builtin", []))
      .mockResolvedValueOnce(view("modern", "file", [{ pattern: "x", message: "y" }]));
    vi.mocked(baseReset).mockResolvedValue(view("modern", "builtin", []));
    render(<BaseRulesDialog open onOpenChange={() => undefined} />);
    await screen.findByText("bản cứng");
    await user.click(screen.getByRole("radio", { name: "Hiện đại" }));
    expect(await screen.findByLabelText("Regex 1")).toHaveValue("x");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("rules", "modern", undefined));
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: `base-rules-dialog.tsx`**

```tsx
import { LoaderCircle, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { RuleTable, type RuleRow } from "@/components/rule-table";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { GENRE_SETTINGS } from "@/lib/schema";
import { GENRE_SETTING_LABELS, type BaseView, type CheckRule, type GenreSetting } from "@/lib/types";
import { useStoryStore } from "@/store/story";

const SETTING_LABELS = Object.fromEntries(GENRE_SETTINGS.map((s) => [s, GENRE_SETTING_LABELS[s].label])) as Record<GenreSetting, string>;

export function rowsOf(rules: CheckRule[] | undefined): RuleRow[] {
  return (rules ?? []).map((r) => ({ pattern: r.pattern, flags: r.flags ?? "", message: r.message }));
}

/** Bỏ dòng trống pattern; flags trống không gửi (khớp `skip_serializing_if` bên Rust). */
export function rulesOf(rows: RuleRow[]): CheckRule[] {
  return rows
    .filter((r) => r.pattern.trim())
    .map((r) => (r.flags.trim() ? { pattern: r.pattern, flags: r.flags.trim(), message: r.message } : { pattern: r.pattern, message: r.message }));
}

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Sửa bộ rule mặc định theo bối cảnh: file base/rules/<setting>.json thay bộ cứng đã lọc. */
export function BaseRulesDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  const [view, setView] = useState<BaseView | undefined>();
  const [rows, setRows] = useState<RuleRow[]>([]);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setView(undefined);
    baseGet("rules", setting, undefined)
      .then((next) => {
        if (cancelled) return;
        setView(next);
        setRows(rowsOf(next.rules));
        setDirty(false);
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được rule mặc định"));
    return () => {
      cancelled = true;
    };
  }, [open, setting]);

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setView(next);
      setRows(rowsOf(next.rules));
      setDirty(false);
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Rule kiểm tra mặc định</DialogTitle>
          <DialogDescription>
            Bộ rule chạy cho truyện chưa có rule riêng, theo bối cảnh. Rule "còn Hán tự" luôn chạy, không nằm ở đây.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap items-end gap-4">
          <div className="flex flex-col gap-1.5">
            <Label>Bối cảnh</Label>
            <Choice
              label="Bối cảnh"
              value={setting}
              options={GENRE_SETTINGS}
              labels={SETTING_LABELS}
              onChange={(value) => {
                if (dirty && !window.confirm("Bỏ thay đổi chưa lưu?")) return;
                setSetting(value);
              }}
              disabled={busy}
            />
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"} · {rows.length}
          </span>
        </div>
        <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
          {view ? (
            <RuleTable
              rows={rows}
              onChange={(next) => {
                setRows(next);
                setDirty(true);
              }}
            />
          ) : (
            <div role="status" className="grid min-h-[200px] place-items-center text-sm text-muted-foreground">Đang tải…</div>
          )}
        </div>
        <DialogFooter>
          <Button type="button" variant="ghost" disabled={busy || view?.source !== "file"} onClick={() => void run(() => baseReset("rules", setting, undefined), "Đã về bản cứng")}>
            <RotateCcw /> Về mặc định
          </Button>
          <Button type="button" disabled={busy || !dirty} onClick={() => void run(() => baseSave("rules", setting, undefined, { rules: rulesOf(rows) }), "Đã lưu rule mặc định")}>
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

Lint `react-refresh/only-export-components` cấm export helper từ file component: chuyển `rowsOf`/`rulesOf` sang `apps/qt-ai-gui/src/lib/rule-rows.ts` (export từ đó, dialog import).

- [ ] **Step 3: Chạy test + check**

Run: `npx vitest run src/components/base-rules-dialog.test.tsx` → PASS; `npm run check` → xanh.

- [ ] **Step 4: Commit**

```bash
git add apps/qt-ai-gui/src/components/base-rules-dialog.tsx apps/qt-ai-gui/src/components/base-rules-dialog.test.tsx apps/qt-ai-gui/src/lib/rule-rows.ts
git commit -m "feat(qt-ai-gui): dialog Rule mặc định theo bối cảnh

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 8: GUI — `BaseGlossaryDialog`

**Files:**
- Create: `apps/qt-ai-gui/src/components/base-glossary-dialog.tsx`
- Test: `apps/qt-ai-gui/src/components/base-glossary-dialog.test.tsx`

**Interfaces:**
- Consumes: `GlossaryEditor` (RHF `useFormContext<StoryFormValues>`, prop `name: "glossary.<key>"`), `toFormValues`/`fromFormValues` không dùng — dialog có form riêng chỉ với field `glossary` (kiểu `StoryFormValues["glossary"]`); `GlossaryEditor` chỉ đọc `control`/`register` theo path `glossary.*` nên form `{ glossary }` đủ (typing: `useForm<Pick<StoryFormValues, "glossary">>` và ép `FormProvider` bằng generic — nếu TS phàn nàn, khai form là `useForm<StoryFormValues>` với `defaultValues` chỉ có `glossary`; các field khác không bao giờ đọc).
- Produces: `export function BaseGlossaryDialog({ open, onOpenChange })`.

- [ ] **Step 1: Test đỏ**

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BaseGlossaryDialog } from "@/components/base-glossary-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));

const empty = { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} };
const view = (setting: string, source: "builtin" | "file", glossary: Record<string, Record<string, string>>) =>
  ({ kind: "glossary", setting, source, glossary: { ...empty, ...glossary } }) as never;

describe("BaseGlossaryDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp kho cổ đại, thêm mục ở Tên nhân vật rồi Lưu gửi glossary đủ 8 nhóm", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "builtin", { names: { 赵: "Triệu" } }));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "file", { names: { 赵: "Triệu", 钱: "Tiền" } }));
    render(<BaseGlossaryDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByDisplayValue("Triệu")).toBeInTheDocument();
    expect(baseGet).toHaveBeenCalledWith("glossary", "ancient", undefined);
    // Nút "Thêm" của nhóm đầu (Tên nhân vật).
    await user.click(screen.getAllByRole("button", { name: /Thêm/ })[0]!);
    const sources = screen.getAllByPlaceholderText("Hán tự");
    await user.type(sources[sources.length - 1]!, "钱");
    const targets = screen.getAllByPlaceholderText("Tiếng Việt");
    await user.type(targets[targets.length - 1]!, "Tiền");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() =>
      expect(baseSave).toHaveBeenCalledWith("glossary", "ancient", undefined, {
        glossary: { ...empty, names: { 赵: "Triệu", 钱: "Tiền" } },
      }),
    );
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("Về mặc định gọi base_reset cho bối cảnh đang chọn", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "file", { names: { 赵: "Triệu" } }));
    vi.mocked(baseReset).mockResolvedValue(view("ancient", "builtin", {}));
    render(<BaseGlossaryDialog open onOpenChange={() => undefined} />);
    await screen.findByText("đã sửa");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("glossary", "ancient", undefined));
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
  });
});
```

Lint cấm `[0]!` (`no-unnecessary-type-assertion`/`no-non-null-assertion`): trong test dùng `const [addFirst] = screen.getAllByRole(...); if (!addFirst) throw new Error("thiếu nút"); await user.click(addFirst);` và tương tự với `at(-1)`.

- [ ] **Step 2: `base-glossary-dialog.tsx`**

```tsx
import { LoaderCircle, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { GlossaryEditor } from "@/components/glossary-editor";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { glossaryToPairs, pairsToGlossary } from "@/lib/story-form";
import { GENRE_SETTINGS } from "@/lib/schema";
import { GENRE_SETTING_LABELS, GLOSSARY_KEYS, GLOSSARY_LABELS, type BaseView, type GenreSetting, type StoryConfig } from "@/lib/types";
import type { StoryFormValues } from "@/lib/story-form";
import { useStoryStore } from "@/store/story";

const SETTING_LABELS = Object.fromEntries(GENRE_SETTINGS.map((s) => [s, GENRE_SETTING_LABELS[s].label])) as Record<GenreSetting, string>;

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Kho glossary chung theo bối cảnh: nền cho mọi truyện cùng bối cảnh (truyện đè key trùng, vẫn lọc theo
 * chương). Dùng lại GlossaryEditor của truyện qua form RHF riêng chỉ có field `glossary`.
 */
export function BaseGlossaryDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  const [view, setView] = useState<BaseView | undefined>();
  const [busy, setBusy] = useState(false);
  const form = useForm<StoryFormValues>({ defaultValues: { glossary: glossaryToPairs(undefined) } });
  const dirty = form.formState.isDirty;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setView(undefined);
    baseGet("glossary", setting, undefined)
      .then((next) => {
        if (cancelled) return;
        setView(next);
        form.reset({ glossary: glossaryToPairs(next.glossary) });
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được glossary chung"));
    return () => {
      cancelled = true;
    };
  }, [open, setting, form]);

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setView(next);
      form.reset({ glossary: glossaryToPairs(next.glossary) });
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  const save = () => {
    const glossary: StoryConfig["glossary"] = pairsToGlossary(form.getValues("glossary"));
    void run(() => baseSave("glossary", setting, undefined, { glossary }), "Đã lưu glossary chung");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Glossary chung</DialogTitle>
          <DialogDescription>
            Nền cho mọi truyện cùng bối cảnh; glossary riêng của truyện đè mục trùng. Chỉ mục có mặt trong chương mới vào prompt.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap items-end gap-4">
          <div className="flex flex-col gap-1.5">
            <Label>Bối cảnh</Label>
            <Choice
              label="Bối cảnh"
              value={setting}
              options={GENRE_SETTINGS}
              labels={SETTING_LABELS}
              onChange={(value) => {
                if (dirty && !window.confirm("Bỏ thay đổi chưa lưu?")) return;
                setSetting(value);
              }}
              disabled={busy}
            />
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"}
          </span>
        </div>
        <FormProvider {...form}>
          <div className="fine-scrollbar flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
            {view ? (
              GLOSSARY_KEYS.map((key) => <GlossaryEditor key={`${setting}-${key}`} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />)
            ) : (
              <div role="status" className="grid min-h-[200px] place-items-center text-sm text-muted-foreground">Đang tải…</div>
            )}
          </div>
        </FormProvider>
        <DialogFooter>
          <Button type="button" variant="ghost" disabled={busy || view?.source !== "file"} onClick={() => void run(() => baseReset("glossary", setting, undefined), "Đã về bản cứng")}>
            <RotateCcw /> Về mặc định
          </Button>
          <Button type="button" disabled={busy || !dirty} onClick={save}>
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

Thêm vào `src/lib/story-form.ts` hai helper (dùng `toPairs`/`fromPairs` sẵn có trong file):

```ts
/** Glossary record (Rust) → mảng cặp cho GlossaryEditor; `undefined` = 8 nhóm rỗng. */
export function glossaryToPairs(glossary: StoryConfig["glossary"] | undefined): StoryFormValues["glossary"] {
  const g = glossary ?? { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} };
  return {
    names: toPairs(g.names),
    places: toPairs(g.places),
    items: toPairs(g.items),
    creatures: toPairs(g.creatures),
    skills: toPairs(g.skills),
    common: toPairs(g.common),
    signature_phrases: toPairs(g.signature_phrases),
    addressing: toPairs(g.addressing),
  };
}

export function pairsToGlossary(pairs: StoryFormValues["glossary"]): StoryConfig["glossary"] {
  return {
    names: fromPairs(pairs.names),
    places: fromPairs(pairs.places),
    items: fromPairs(pairs.items),
    creatures: fromPairs(pairs.creatures),
    skills: fromPairs(pairs.skills),
    common: fromPairs(pairs.common),
    signature_phrases: fromPairs(pairs.signature_phrases),
    addressing: fromPairs(pairs.addressing),
  };
}
```

và cho `toFormValues`/`fromFormValues` dùng lại hai helper này (bỏ khối 8 dòng lặp).

- [ ] **Step 3: Chạy test + check**

Run: `npx vitest run src/components/base-glossary-dialog.test.tsx` → PASS; `npm run check` → xanh.

- [ ] **Step 4: Commit**

```bash
git add apps/qt-ai-gui/src/components/base-glossary-dialog.tsx apps/qt-ai-gui/src/components/base-glossary-dialog.test.tsx apps/qt-ai-gui/src/lib/story-form.ts
git commit -m "feat(qt-ai-gui): dialog Glossary chung theo bối cảnh

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 9: GUI — card "Bản mặc định" ở Cài đặt, nhãn ở Hồ sơ truyện, README

**Files:**
- Modify: `apps/qt-ai-gui/src/components/pages/settings-page.tsx`
- Modify: `apps/qt-ai-gui/src/components/prompt-editor.tsx` (nhãn), `apps/qt-ai-gui/src/components/pages/story-page.tsx` (dòng kho chung ở tab Glossary)
- Modify: `apps/qt-ai-gui/README.md`
- Test: `apps/qt-ai-gui/src/components/pages/settings-page.test.tsx`, `apps/qt-ai-gui/src/components/pages/story-page.test.tsx`

- [ ] **Step 1: Test đỏ** — `settings-page.test.tsx` thêm mock `baseGet: vi.fn()`, `baseSave: vi.fn()`, `baseReset: vi.fn()` vào `vi.mock("@/lib/api", …)` và test:

```ts
  it("card Bản mặc định có ba nút mở dialog prompt / rule / glossary", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue({ kind: "prompt", setting: "ancient", names: "han", source: "builtin", text: "# x" } as never);
    render(<SettingsPage />);
    expect(screen.getByRole("button", { name: "Rule mặc định" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Glossary chung" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Prompt mặc định" }));
    expect(await screen.findByRole("dialog", { name: "Prompt mặc định" })).toBeInTheDocument();
  });
```

(Thêm `vi.mock("@/components/plate-prompt-editor", …)` giả như Task 6 để dialog không nạp Plate.)

`story-page.test.tsx`: fixture `storyDefaults` trả `promptSource: "file"`, thêm assertion trong test Prompt tab: `expect(screen.getByText("mặc định của app (đã sửa)")).toBeInTheDocument()`; ở tab Glossary: `expect(screen.getByText(/Kho chung/)).toBeInTheDocument()`.

- [ ] **Step 2: settings-page** — sau card "Thư viện" thêm:

```tsx
        <Card title="Bản mặc định" description="Prompt, rule và glossary dùng cho mọi truyện chưa có bản riêng. Sửa ở đây là mọi truyện đang dùng mặc định ăn theo, kể cả phiên agy.">
          <div className="flex flex-wrap gap-2">
            <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("prompt")}>
              <FileText /> Prompt mặc định
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("rules")}>
              <ListChecks /> Rule mặc định
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("glossary")}>
              <BookA /> Glossary chung
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">File nằm trong thư mục cấu hình app, mục <code>base/</code>. Về mặc định = xoá file.</p>
        </Card>
```

State `const [baseDialog, setBaseDialog] = useState<BaseKind | undefined>();`, render ba dialog cuối trang (ngoài footer):

```tsx
      <BasePromptDialog open={baseDialog === "prompt"} onOpenChange={(o) => setBaseDialog(o ? "prompt" : undefined)} />
      <BaseRulesDialog open={baseDialog === "rules"} onOpenChange={(o) => setBaseDialog(o ? "rules" : undefined)} />
      <BaseGlossaryDialog open={baseDialog === "glossary"} onOpenChange={(o) => setBaseDialog(o ? "glossary" : undefined)} />
```

Icon từ `lucide-react` (`FileText`, `ListChecks`, `BookA` — kiểm tồn tại; thay bằng `BookOpen` nếu `BookA` không có).

- [ ] **Step 3: prompt-editor nhãn**: `{usingDefault ? (defaults?.promptSource === "file" ? "mặc định của app (đã sửa)" : "mặc định") : "riêng"}`.

`story-page.tsx` tab Glossary, trước danh sách editor:

```tsx
                <p className="text-xs text-muted-foreground">
                  Kho chung theo bối cảnh (Cài đặt → Bản mặc định → Glossary chung) làm nền; mục ở đây đè khi trùng.
                </p>
```

- [ ] **Step 4: README** — mục mới sau "## Thể loại":

```markdown
## Bản mặc định sửa được

Cài đặt → **Bản mặc định**: sửa base prompt từng genre, bộ rule từng bối cảnh và kho glossary chung từng bối cảnh.
File rời trong thư mục cấu hình app: `base/prompts/<setting>-<names>.md`, `base/rules/<setting>.json`,
`base/glossary/<setting>.json`; không có file = bản cứng trong binary; "Về mặc định" = xoá file. Core đọc qua env
`QT_AI_BASE_DIR` (app đặt lúc khởi động, phiên agy thừa hưởng) nên app, phiên API và `qt-ai next` dùng cùng bản.
Ưu tiên: prompt/rule riêng của truyện > file base > bản cứng; glossary chung làm nền, glossary truyện đè key trùng.
```

- [ ] **Step 5: Chạy check + commit**

Run: `cd apps/qt-ai-gui && npm run check` → xanh.

```bash
git add apps/qt-ai-gui/src/components/pages/settings-page.tsx apps/qt-ai-gui/src/components/pages/settings-page.test.tsx apps/qt-ai-gui/src/components/prompt-editor.tsx apps/qt-ai-gui/src/components/pages/story-page.tsx apps/qt-ai-gui/src/components/pages/story-page.test.tsx apps/qt-ai-gui/README.md
git commit -m "feat(qt-ai-gui): card Bản mặc định ở Cài đặt, nhãn 'mặc định của app (đã sửa)' ở hồ sơ truyện

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

## Self-review

- **Spec coverage**: §1 lưu trữ → Task 1; §2 core đọc + nối ghép → Task 1, 2; §3 command, env, card, ba dialog, nạp lại defaults, nhãn, dòng kho chung → Task 3–9; §4 kiểm thử → test trong từng task (Tauri crate chỉ biên dịch, ghi ở Global Constraints). Khác spec: env đặt ở setup thay vì `SessionConfig.base_dir` — Task 3 Step 4 sửa spec.
- **Placeholder**: không có TBD; mọi bước code có code.
- **Type consistency**: `BaseView.source` là `"builtin" | "file"` ở Rust (String) và zod enum; `baseSave(kind, setting, names, payload)` thống nhất giữa api.ts và ba dialog (`names` = `undefined` cho rules/glossary, test khớp `toHaveBeenCalledWith(..., undefined, …)`); `StoryDefaults.promptSource/rulesSource` dùng ở Task 4, 5, 9; `RuleRow` ở Task 5, 7; `glossaryToPairs/pairsToGlossary` ở Task 8.
