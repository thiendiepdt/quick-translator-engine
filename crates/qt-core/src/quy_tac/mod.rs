//! Quy Tắc: đảo trật tự và chọn nghĩa trên token sau khi convert.
//! Spec: docs/superpowers/specs/2026-11-06-quy-tac-design.md. Mặc định tắt.

mod char_rules;
mod matcher;
mod numbers;
mod prepare;
mod program;
mod render;
mod tags;

use std::path::Path;

pub(crate) use prepare::Lookups;

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
    /// `Resources/dict.txt` bắt buộc; `Resources/TuLoai.txt` nếu có.
    pub fn load(data_dir: &Path, giong_muot: bool) -> std::io::Result<QuyTac> {
        let read_optional = |rel: &str| std::fs::read_to_string(data_dir.join(rel)).ok();
        let mut rules =
            read_optional("QuyTac.txt").unwrap_or_else(|| Self::DEFAULT_RULES.to_string());
        if giong_muot {
            rules.push('\n');
            rules.push_str(Self::GIONG_MUOT_RULES);
        }
        let pos_dict = std::fs::read_to_string(data_dir.join("Resources/dict.txt"))?;
        let tu_loai = read_optional("Resources/TuLoai.txt")
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
        fn text_of<'o>(output: &'o str, t: &SourceToken) -> &'o str {
            output[t.out_start..t.out_end].trim()
        }
        let Some(first) = paragraph.iter().find(|t| !text_of(output, t).is_empty()) else {
            return;
        };
        let last = paragraph
            .iter()
            .rev()
            .find(|t| !text_of(output, t).is_empty())
            .expect("có cụm đầu thì có cụm cuối");
        let raw: Vec<(String, String)> = paragraph
            .iter()
            .map(|t| {
                let source = &chars[t.start..t.start + t.len];
                let mut zh = String::with_capacity(source.iter().map(|c| c.len_utf8()).sum());
                zh.extend(source);
                (zh, text_of(output, t).to_string())
            })
            .collect();
        let Some(rendered) = self.apply_paragraph(raw, lk) else {
            return;
        };
        let head = &output[first.out_start..first.out_end];
        let from = first.out_start + (head.len() - head.trim_start().len());
        let tail = &output[last.out_start..last.out_end];
        let to = last.out_end - (tail.len() - tail.trim_end().len());
        if from < *cursor || to < from {
            return;
        }
        result.push_str(&output[*cursor..from]);
        result.push_str(&rendered);
        *cursor = to;
    }

    /// `None` khi không có quy tắc nào chạy.
    fn apply_paragraph(&self, raw: Vec<(String, String)>, lk: &Lookups) -> Option<String> {
        let (items, char_fired) = char_rules::apply_chars(&self.program, raw, lk);
        let toks = prepare::prepare(items, &self.pos, lk, &self.program.classes);
        let (toks, fired) = matcher::apply_program(&self.program, toks);
        (char_fired + fired > 0).then(|| render::render(&toks))
    }
}
