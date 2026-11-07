//! Quy Tắc: đảo trật tự và chọn nghĩa trên token sau khi convert.
//! Spec: docs/superpowers/specs/2026-11-06-quy-tac-design.md. Mặc định tắt.

mod char_rules;
mod matcher;
mod numbers;
mod prepare;
mod program;
mod render;
mod tags;

use std::borrow::Cow;
use std::path::Path;

pub(crate) use prepare::Lookups;
use prepare::Raw;

use crate::translate::SourceToken;

/// Lỗi khi đọc file Quy Tắc; `line` là số dòng (tính từ 1), 0 khi không gắn dòng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuyTacError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for QuyTacError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(formatter, "QuyTac dòng {}: {}", self.line, self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for QuyTacError {}

/// Bộ quy tắc đã đọc cùng bảng từ loại.
pub struct QuyTac {
    program: program::Program,
    pos: tags::PosLexicon,
}

impl QuyTac {
    /// Bộ luật mặc định (spec mục 3).
    pub const DEFAULT_RULES: &'static str = include_str!("../../data/QuyTac.txt");
    /// Nhóm `[giọng-mượt]`: chọn nghĩa theo văn phong, chỉ chạy khi được bật.
    pub const GIONG_MUOT_RULES: &'static str = include_str!("../../data/QuyTac-giong-muot.txt");
    /// Nhãn từ loại sửa lại, đè lên `Resources/dict.txt`.
    pub const DEFAULT_TU_LOAI: &'static str = include_str!("../../data/TuLoai.txt");

    pub fn from_sources(rules: &str, pos_dict: &str, tu_loai: &str) -> Result<QuyTac, QuyTacError> {
        Ok(QuyTac {
            program: program::Program::parse(rules)?,
            pos: tags::PosLexicon::build(pos_dict, tu_loai),
        })
    }

    /// `QuyTac.txt` ở gốc thư mục dữ liệu nếu có, không thì bộ mặc định;
    /// `Resources/dict.txt` bắt buộc; `Resources/TuLoai.txt` nếu có. Chỉ file
    /// không có mới dùng bộ mặc định: file có mà không đọc được hay không phải
    /// UTF-8 (UTF-16, ANSI do trình soạn thảo lưu) là lỗi, kèm đường dẫn.
    pub fn load(data_dir: &Path, giong_muot: bool) -> std::io::Result<QuyTac> {
        let mut rules = read_text(&data_dir.join("QuyTac.txt"))?
            .unwrap_or_else(|| Self::DEFAULT_RULES.to_string());
        if giong_muot {
            rules.push('\n');
            rules.push_str(Self::GIONG_MUOT_RULES);
        }
        let pos_path = data_dir.join("Resources/dict.txt");
        let pos_dict = read_text(&pos_path)?.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{}: không có file", pos_path.display()),
            )
        })?;
        let tu_loai = read_text(&data_dir.join("Resources/TuLoai.txt"))?
            .unwrap_or_else(|| Self::DEFAULT_TU_LOAI.to_string());
        Self::from_sources(&rules, &pos_dict, &tu_loai).map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
        })
    }

    /// Áp quy tắc lên output của vòng dịch. Đoạn (ngăn bởi `\n`) nào không có
    /// quy tắc nào chạy thì giữ nguyên từng byte.
    pub(crate) fn apply(
        &self,
        chars: &[char],
        output: &str,
        tokens: &[SourceToken],
        lk: &Lookups,
    ) -> String {
        let mut result = String::with_capacity(output.len() + 64);
        let mut cursor = 0;
        let mut start = 0;
        for (index, token) in tokens.iter().enumerate() {
            if chars[token.start..token.start + token.len].contains(&'\n') {
                self.splice(chars, output, &tokens[start..index], lk, &mut result, &mut cursor);
                start = index + 1;
            }
        }
        self.splice(chars, output, &tokens[start..], lk, &mut result, &mut cursor);
        result.push_str(&output[cursor..]);
        result
    }

    fn splice(
        &self,
        chars: &[char],
        output: &str,
        paragraph: &[SourceToken],
        lk: &Lookups,
        result: &mut String,
        cursor: &mut usize,
    ) {
        // Range hỏng (không được xảy ra, `record` giữ bất biến) thì giữ nguyên
        // đoạn chứ không panic ở bản release; bản debug thì báo ngay.
        let broken = paragraph.iter().find(|t| output.get(t.out_start..t.out_end).is_none());
        let bytes = output.len();
        debug_assert!(broken.is_none(), "range cụm hỏng {broken:?} trên output {bytes} byte");
        if broken.is_some() {
            return;
        }
        let text_of = |t: &SourceToken| output[t.out_start..t.out_end].trim();
        if paragraph.iter().all(|t| text_of(t).is_empty()) {
            return;
        }
        // Chữ Hán cả đoạn một chuỗi; token mượn khúc của nó và nghĩa trong output.
        let source = paragraph.iter().map(|t| &chars[t.start..t.start + t.len]);
        let mut text = String::with_capacity(source.clone().map(<[char]>::len).sum::<usize>() * 3);
        source.clone().for_each(|zh| text.extend(zh));
        let mut at = 0;
        let mut pieces = Vec::with_capacity(paragraph.len());
        let raw: Vec<Raw> = source
            .zip(paragraph)
            .map(|(zh, t)| {
                let start = at;
                at += zh.iter().map(|c| c.len_utf8()).sum::<usize>();
                let vi = text_of(t);
                // Chữ của cụm trong output, bỏ khoảng trắng hai đầu.
                let chunk = &output[t.out_start..t.out_end];
                let out = t.out_start + (chunk.len() - chunk.trim_start().len());
                pieces.push(render::Piece {
                    src: start..at,
                    out: out..out + vi.len(),
                    capital: t.capital,
                });
                Raw::Plain(Cow::Borrowed(&text[start..at]), meaning(vi, t.capital))
            })
            .collect();
        let Some(toks) = self.apply_paragraph(&text, raw, lk) else {
            return;
        };
        render::splice(output, &text, &pieces, &toks, result, cursor);
    }

    /// `text` là chữ Hán cả đoạn, `raw` các token của nó. `None` khi không có
    /// quy tắc nào chạy; không thì token cuối cùng, có `src` và `fired`.
    fn apply_paragraph<'a>(
        &'a self,
        text: &'a str,
        raw: Vec<Raw<'a>>,
        lk: &Lookups,
    ) -> Option<Vec<prepare::Token<'a>>> {
        let (items, char_fired) = char_rules::apply_chars(&self.program, text, raw, lk);
        let toks = prepare::prepare(items, &self.pos, lk, &self.program.classes);
        let covered = toks.last().is_none_or(|t| t.src.end == text.len());
        debug_assert!(covered, "token không phủ trọn đoạn");
        let toks = matcher::join_proper_chains(toks);
        let (toks, fired) = matcher::apply_program(&self.program, toks);
        (char_fired + fired > 0).then_some(toks)
    }
}

/// Nghĩa của cụm như từ điển cho: chữ đầu engine viết hoa vì đứng đầu câu
/// (`capital`) trả về chữ gốc. Khác bản thử (hạ chữ đầu mọi token bị dời, trừ
/// tên): chữ hoa của chính nghĩa ("Ma tộc", tên) giữ nguyên; đầu câu do bước ráp
/// viết hoa lại.
fn meaning(vi: &str, capital: Option<char>) -> Cow<'_, str> {
    match capital {
        Some(c) => {
            let mut out = String::with_capacity(vi.len());
            render::push_uncapitalized(&mut out, vi, c);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(vi),
    }
}

/// Đọc file chữ UTF-8, bỏ BOM đầu file. `Ok(None)` chỉ khi file không có; lỗi
/// khác trả về kèm đường dẫn.
fn read_text(path: &Path) -> std::io::Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(mut text) => {
            if text.starts_with('\u{feff}') {
                text.drain(..'\u{feff}'.len_utf8());
            }
            Ok(Some(text))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(std::io::Error::new(
            error.kind(),
            format!("{}: {error}", path.display()),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const RULES: &str =
        "[lớp]\nvị-trí: 上=trên\n\n[của 1]\n{danh-từ} {vị-trí} = {vị-trí} {danh-từ}\n";

    /// Thư mục dữ liệu tạm có `Resources/dict.txt` mở đầu bằng BOM như QT2025;
    /// xoá khi ra khỏi test.
    struct TempData(PathBuf);

    impl TempData {
        fn new(tag: &str) -> TempData {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("qt-core-quy-tac-{tag}-{unique}"));
            std::fs::create_dir_all(dir.join("Resources")).unwrap();
            let pos_dict = "\u{feff}桌子 10 n\n上 10 f\n";
            std::fs::write(dir.join("Resources/dict.txt"), pos_dict).unwrap();
            TempData(dir)
        }
    }

    impl Drop for TempData {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_user_files_fall_back_to_defaults() {
        let data = TempData::new("missing");
        let quy_tac = QuyTac::load(&data.0, false).unwrap();
        let defaults = program::Program::parse(QuyTac::DEFAULT_RULES).unwrap();
        assert_eq!(quy_tac.program.sections.len(), defaults.sections.len());
        let error = QuyTac::load(&data.0.join("không-có"), false).err().unwrap();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        assert!(error.to_string().contains("dict.txt"), "{error}");
    }

    #[test]
    fn unreadable_user_files_are_errors_not_defaults() {
        let data = TempData::new("invalid");
        // UTF-16 LE có BOM, như Notepad cũ lưu kiểu "Unicode".
        let utf16: Vec<u8> = format!("\u{feff}{RULES}")
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        std::fs::write(data.0.join("QuyTac.txt"), utf16).unwrap();
        let error = QuyTac::load(&data.0, false).err().expect("không được dùng bộ mặc định");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("QuyTac.txt"), "{error}");
        // ANSI (Windows-1258): byte 0xC0 không phải UTF-8.
        std::fs::remove_file(data.0.join("QuyTac.txt")).unwrap();
        std::fs::write(data.0.join("Resources/TuLoai.txt"), b"\xc0\xe1\tn\n").unwrap();
        let error = QuyTac::load(&data.0, false).err().expect("không được dùng bộ mặc định");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("TuLoai.txt"), "{error}");
    }

    #[test]
    fn bom_prefixed_files_parse() {
        let data = TempData::new("bom");
        std::fs::write(data.0.join("QuyTac.txt"), format!("\u{feff}{RULES}")).unwrap();
        std::fs::write(data.0.join("Resources/TuLoai.txt"), "\u{feff}椅子\tn\n").unwrap();
        let quy_tac = QuyTac::load(&data.0, false).unwrap();
        assert_eq!(quy_tac.program.sections.len(), 1);
        // Mục đầu của dict.txt và TuLoai không dính BOM vào key.
        assert_eq!(quy_tac.pos.get("桌子"), Some("n"));
        assert_eq!(quy_tac.pos.get("椅子"), Some("n"));
        assert_eq!(quy_tac.pos.get("上"), Some("f"));
    }
}
