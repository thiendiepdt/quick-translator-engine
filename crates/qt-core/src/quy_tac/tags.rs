//! Từ loại cho Quy Tắc: bảng kiểu jieba (`Resources/dict.txt`, dòng
//! `từ tần-số nhãn`) đè bằng `TuLoai.txt` (dòng `từ<TAB>nhãn`). Lưu gọn trong
//! `CompactDict` (~350 nghìn mục, vài MB).

use crate::compact::CompactDict;
use crate::dict::DictionaryLookup;

/// Trợ từ cuối được bỏ đi khi tra một cụm không có trong bảng.
const TRAILERS: &[char] = &['了', '着', '过', '吗', '呢', '啊', '吧', '呀', '嘛', '哦', '哈'];

pub(crate) struct PosLexicon {
    table: CompactDict,
}

impl PosLexicon {
    pub(crate) fn build(jieba_dict: &str, tu_loai: &str) -> PosLexicon {
        let fixes = tu_loai.lines().filter_map(|line| {
            let (word, tag) = line.trim_end_matches('\r').split_once('\t')?;
            (!word.is_empty() && !tag.is_empty()).then_some((word, tag))
        });
        let base = jieba_dict.lines().filter_map(|line| {
            let mut parts = line.split_whitespace();
            let (word, _frequency, tag) = (parts.next()?, parts.next()?, parts.next()?);
            parts.next().is_none().then_some((word, tag))
        });
        // TuLoai đứng trước nên thắng (first-wins).
        let (table, _) = CompactDict::first_wins(fixes.chain(base));
        PosLexicon { table }
    }

    pub(crate) fn get(&self, word: &str) -> Option<&str> {
        self.table.get(word)
    }

    /// Tách tham lam theo cụm dài nhất có trong bảng; ký tự lạ mang nhãn `"?"`.
    pub(crate) fn segment(&self, text: &str) -> Vec<(String, &str)> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = Vec::new();
        let mut buffer = String::new();
        let mut i = 0;
        while i < chars.len() {
            let cap = self.table.max_key_len(chars[i]).min(chars.len() - i);
            let mut found = None;
            for length in (1..=cap).rev() {
                buffer.clear();
                buffer.extend(&chars[i..i + length]);
                if let Some(tag) = self.table.get(&buffer) {
                    found = Some((length, tag));
                    break;
                }
            }
            match found {
                Some((length, tag)) => {
                    out.push((chars[i..i + length].iter().collect(), tag));
                    i += length;
                }
                None => {
                    out.push((chars[i].to_string(), "?"));
                    i += 1;
                }
            }
        }
        out
    }

    /// Nhãn của một token: tra thẳng; bỏ trợ từ cuối rồi tra; không có thì suy
    /// từ các từ con (giới từ đầu → "pp", có động từ → "v", đuôi trợ từ → từ
    /// trước, còn lại → từ cuối). Giống `Pos.phrase` của bản thử.
    pub(crate) fn phrase(&self, zh: &str) -> &str {
        if let Some(tag) = self.get(zh) {
            return tag;
        }
        let core = zh.trim_end_matches(TRAILERS);
        let core = if core.is_empty() { zh } else { core };
        if let Some(tag) = self.get(core) {
            return tag;
        }
        let tags: Vec<&str> = self
            .segment(core)
            .into_iter()
            .map(|(_, tag)| tag)
            .filter(|tag| *tag != "?")
            .collect();
        let Some(&last) = tags.last() else {
            return "?";
        };
        if tags[0].starts_with('p') && tags.len() > 1 {
            return "pp";
        }
        if tags.iter().any(|tag| tag.starts_with('v') && *tag != "vn") {
            return "v";
        }
        if last.starts_with('u') && tags.len() > 1 {
            return tags[tags.len() - 2];
        }
        last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DICT: &str = "美丽 10 ns\n少女 5 n\n看 3 v\n在 9 p\n家 4 n\n的 9 uj\n";

    #[test]
    fn tu_loai_overrides_jieba_tags() {
        let pos = PosLexicon::build(DICT, "美丽\ta\n");
        assert_eq!(pos.get("美丽"), Some("a"));
        assert_eq!(pos.get("少女"), Some("n"));
        assert_eq!(pos.get("没有"), None);
    }

    #[test]
    fn segments_longest_first() {
        let pos = PosLexicon::build(DICT, "");
        let parts: Vec<(String, &str)> = pos.segment("少女们");
        assert_eq!(
            parts,
            vec![("少女".to_string(), "n"), ("们".to_string(), "?")]
        );
    }

    #[test]
    fn phrase_infers_tags_like_the_prototype() {
        let pos = PosLexicon::build(DICT, "");
        assert_eq!(pos.phrase("看着"), "v"); // bỏ đuôi 着 rồi tra
        assert_eq!(pos.phrase("少女们"), "n"); // từ cuối biết được
        assert_eq!(pos.phrase("在家"), "pp"); // giới từ + từ khác
        assert_eq!(pos.phrase("少女的"), "n"); // đuôi trợ từ: lấy từ trước
        assert_eq!(pos.phrase("们们"), "?");
    }
}
