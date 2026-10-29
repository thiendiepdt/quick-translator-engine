//! Chương ngắn cho nền tảng đăng truyện: cài đặt bù chữ (base/padding.json, mọi truyện), danh sách chương dưới
//! ngưỡng, chèn thủ công cho một chương. Logic ở `qt_ai_core::padding`.

use crate::error::{blocking, CmdResult, CommandError};
use qt_ai_core::backfill::translated_chapters;
use qt_ai_core::base::BaseStore;
use qt_ai_core::padding::{pad_chapter, published_words, PaddingMode, PaddingSettings};
use qt_ai_core::story_fs::{read_text, story_paths, write_atomic};
use serde::Serialize;
use std::path::Path;

#[tauri::command]
pub async fn padding_get() -> CmdResult<PaddingSettings> {
    blocking(|| Ok(BaseStore::from_env().padding())).await
}

#[tauri::command]
pub async fn padding_save(settings: PaddingSettings) -> CmdResult<PaddingSettings> {
    blocking(move || {
        if settings.threshold == 0 {
            return Err(CommandError::new("invalid_input", "Ngưỡng chữ phải lớn hơn 0."));
        }
        BaseStore::from_env().save_padding(&settings)?;
        Ok(settings)
    })
    .await
}

/// Về mặc định: xoá base/padding.json (tắt, 500 chữ, văn mẫu gốc).
#[tauri::command]
pub async fn padding_reset() -> CmdResult<PaddingSettings> {
    blocking(|| {
        let store = BaseStore::from_env();
        store.reset_padding()?;
        Ok(store.padding())
    })
    .await
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortChapter {
    pub id: String,
    pub words: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortChapters {
    pub threshold: usize,
    pub chapters: Vec<ShortChapter>,
}

/// Chương đã dịch dưới ngưỡng tính như nền tảng đếm (cả phần chèn, không tính tên chương), theo thứ tự chương.
pub fn short_chapters_inner(root: &Path, threshold: usize) -> ShortChapters {
    let chapters = translated_chapters(&story_paths(root))
        .into_iter()
        .filter_map(|(id, path)| {
            let words = published_words(&read_text(&path).ok()?);
            (words < threshold).then_some(ShortChapter { id, words })
        })
        .collect();
    ShortChapters { threshold, chapters }
}

/// Đọc mọi out/ của truyện (truyện 2.500 chương ~0,1–0,2 s) — chạy ngoài luồng IPC.
#[tauri::command]
pub async fn short_chapters(root: String) -> CmdResult<ShortChapters> {
    blocking(move || Ok(short_chapters_inner(Path::new(&root), BaseStore::from_env().padding().threshold))).await
}

/// Chèn cho đủ ngưỡng một chương theo cách bù trong cài đặt (đang tắt thì dùng văn mẫu). false khi chương đã đủ,
/// không có gì đổi.
pub fn chapter_pad_inner(root: &Path, id: &str, settings: &PaddingSettings) -> CmdResult<bool> {
    let path = story_paths(root).out_dir.join(format!("{id}.txt"));
    if !path.is_file() {
        return Err(CommandError::new("invalid_state", format!("Chương {id} chưa có bản dịch trong out/.")));
    }
    let mode = if settings.mode == PaddingMode::Off { PaddingMode::Filler } else { settings.mode };
    match pad_chapter(&read_text(&path)?, mode, settings) {
        Some(padded) => {
            write_atomic(&path, &padded)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn chapter_pad(root: String, id: String) -> CmdResult<bool> {
    blocking(move || chapter_pad_inner(Path::new(&root), &id, &BaseStore::from_env().padding())).await
}

/// Chèn cho mọi chương ngắn của truyện (nút "Chèn cho cả N chương" trên toolbar). Trả số chương đã chèn.
pub fn chapters_pad_all_inner(root: &Path, settings: &PaddingSettings) -> CmdResult<usize> {
    let mut padded = 0;
    for chapter in short_chapters_inner(root, settings.threshold).chapters {
        if chapter_pad_inner(root, &chapter.id, settings)? {
            padded += 1;
        }
    }
    Ok(padded)
}

#[tauri::command]
pub async fn chapters_pad_all(root: String) -> CmdResult<usize> {
    blocking(move || chapters_pad_all_inner(Path::new(&root), &BaseStore::from_env().padding())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::padding::PADDING_MARKER;
    use std::fs;

    #[test]
    fn liet_ke_chuong_ngan_va_chen_thu_cong() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("0001.txt"), "Chương 1\n\nmột hai ba bốn năm sáu\n").unwrap();
        fs::write(out.join("0002.txt"), "Chương 2\n\nmột hai\n").unwrap();
        assert_eq!(short_chapters_inner(dir.path(), 5).chapters, vec![ShortChapter { id: "0002".into(), words: 2 }]);
        let settings = PaddingSettings { mode: PaddingMode::Off, threshold: 5, filler: "a b c d\n\nKết.".into() };
        assert!(chapter_pad_inner(dir.path(), "0002", &settings).unwrap());
        assert!(fs::read_to_string(out.join("0002.txt")).unwrap().contains(PADDING_MARKER), "tắt thì nút thủ công dùng văn mẫu");
        assert!(short_chapters_inner(dir.path(), 5).chapters.is_empty());
        assert!(!chapter_pad_inner(dir.path(), "0001", &settings).unwrap(), "đủ chữ thì không đổi");
        assert!(chapter_pad_inner(dir.path(), "0009", &settings).is_err());
    }

    #[test]
    fn chen_ca_truyen_chi_dung_chuong_ngan() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("0001.txt"), "Chương 1

một hai ba bốn năm sáu
").unwrap();
        fs::write(out.join("0002.txt"), "Chương 2

một hai
").unwrap();
        fs::write(out.join("0003.txt"), "Chương 3

một
").unwrap();
        let settings = PaddingSettings { mode: PaddingMode::Off, threshold: 5, filler: "a b c d

Kết.".into() };
        assert_eq!(chapters_pad_all_inner(dir.path(), &settings).unwrap(), 2);
        assert_eq!(fs::read_to_string(out.join("0001.txt")).unwrap(), "Chương 1

một hai ba bốn năm sáu
");
        assert!(short_chapters_inner(dir.path(), 5).chapters.is_empty());
        assert_eq!(chapters_pad_all_inner(dir.path(), &settings).unwrap(), 0);
    }
}
