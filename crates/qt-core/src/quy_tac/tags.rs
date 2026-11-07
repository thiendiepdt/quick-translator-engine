//! Từ loại cho Quy Tắc: bảng kiểu jieba (`Resources/dict.txt`, dòng
//! `từ tần-số nhãn`) đè bằng `TuLoai.txt` (dòng `từ<TAB>nhãn`). Lưu gọn trong
//! bảng băm tự dựng (~350 nghìn mục, ~5,5 MB).
//!
//! BOM UTF-8 đầu hai file được bỏ (`dict.txt` của QT2025 có). Dòng TuLoai
//! không có TAB, ví dụ ngăn từ và nhãn bằng dấu cách, bị bỏ qua không báo lỗi.

use crate::dict::KeyLenIndex;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::hash::BuildHasher;

/// Trợ từ cuối được bỏ đi khi tra một cụm không có trong bảng.
const TRAILERS: &[char] = &['了', '着', '过', '吗', '呢', '啊', '吧', '呀', '嘛', '哦', '哈'];

/// Key dài nhất (byte) lưu được; key dài hơn bị bỏ như ở `CompactDict`.
const MAX_KEY_BYTES: usize = u8::MAX as usize;

/// Nhãn dài nhất (byte) lưu được, như giới hạn value của `CompactDict`.
const MAX_TAG_BYTES: usize = (1 << 24) - 1;

/// Bảng key -> nhãn, chỉ đọc. Mỗi bản ghi trong arena là
/// `[độ dài key][byte của key][chỉ số nhãn 1-4 byte]`; nhãn là chỉ số vào bảng nhãn
/// đã gộp (chỉ vài chục nhãn khác nhau). Bảng băm là mảng `u32` thăm dò
/// tuyến tính, tải ~0,75: mỗi ô gồm `offset + 1` ở các bit thấp và vài bit
/// dấu vân tay của hash ở các bit cao (0 là ô trống), nên hầu hết lần trượt
/// bị loại mà không phải đọc arena.
pub(crate) struct PosLexicon {
    arena: Box<[u8]>,
    slots: Box<[u32]>,
    tags: Box<[Box<str>]>,
    /// 1 đến 4 byte (little-endian) chỉ số nhãn trong mỗi bản ghi.
    tag_bytes: usize,
    offset_bits: u32,
    index: KeyLenIndex,
}

impl PosLexicon {
    pub(crate) fn build(jieba_dict: &str, tu_loai: &str) -> PosLexicon {
        // BOM dính vào key đầu thì mục đầu file không bao giờ tra trúng.
        let jieba_dict = jieba_dict.strip_prefix('\u{feff}').unwrap_or(jieba_dict);
        let tu_loai = tu_loai.strip_prefix('\u{feff}').unwrap_or(tu_loai);
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
        Self::first_wins(fixes.chain(base))
    }

    /// Hai lượt: lượt đầu đếm và gộp nhãn để cấp phát đúng một lần.
    fn first_wins<'a, I>(entries: I) -> PosLexicon
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
    {
        let storable =
            |key: &str, tag: &str| key.len() <= MAX_KEY_BYTES && tag.len() <= MAX_TAG_BYTES;
        let mut ids: FxHashMap<&str, usize> = FxHashMap::default();
        let mut tags: Vec<Box<str>> = Vec::new();
        let (mut count, mut key_bytes) = (0usize, 0usize);
        for (key, tag) in entries.clone() {
            if !storable(key, tag) {
                continue;
            }
            ids.entry(tag).or_insert_with(|| {
                tags.push(tag.into());
                tags.len() - 1
            });
            count += 1;
            key_bytes += key.len();
        }
        // Số byte vừa đủ cho chỉ số nhãn lớn nhất (1 đến 4).
        let tag_bytes = (usize::BITS - tags.len().saturating_sub(1).leading_zeros())
            .div_ceil(8)
            .max(1) as usize;
        let bound = key_bytes + count * (1 + tag_bytes);
        let bound = u32::try_from(bound).expect("bảng từ loại vượt 4 GiB");
        let offset_bits = u32::BITS - bound.leading_zeros();
        let slot_count = (count * 4 / 3 + 1).max(1);
        let mut slots = vec![0u32; slot_count];
        let mut arena: Vec<u8> = Vec::with_capacity(bound as usize);
        let hasher = FxBuildHasher;
        for (key, tag) in entries {
            if !storable(key, tag) {
                continue;
            }
            let hash = hasher.hash_one(key);
            let fingerprint = fingerprint(hash, offset_bits);
            let mut slot = slot_of(hash, slot_count);
            let mut present = false;
            while slots[slot] != 0 {
                let value = slots[slot];
                if fingerprint_of(value, offset_bits) == fingerprint
                    && record_key(&arena, offset_of(value, offset_bits)) == key.as_bytes()
                {
                    present = true;
                    break;
                }
                slot = if slot + 1 == slot_count { 0 } else { slot + 1 };
            }
            if present {
                continue;
            }
            let id = ids[tag];
            slots[slot] = pack(arena.len(), fingerprint, offset_bits);
            arena.push(key.len() as u8);
            arena.extend_from_slice(key.as_bytes());
            arena.extend_from_slice(&id.to_le_bytes()[..tag_bytes]);
        }
        drop(ids);
        // Chỉ mục độ dài key đọc thẳng từ arena, khỏi dựng Vec key tạm.
        let mut start = 0;
        let index = KeyLenIndex::from_str_keys(std::iter::from_fn(|| {
            if start >= arena.len() {
                return None;
            }
            let key = record_key(&arena, start);
            start += 1 + key.len() + tag_bytes;
            Some(std::str::from_utf8(key).expect("key lấy từ str"))
        }));
        PosLexicon {
            arena: arena.into_boxed_slice(),
            slots: slots.into_boxed_slice(),
            tags: tags.into_boxed_slice(),
            tag_bytes,
            offset_bits,
            index,
        }
    }

    pub(crate) fn get(&self, word: &str) -> Option<&str> {
        let hash = FxBuildHasher.hash_one(word);
        let fingerprint = fingerprint(hash, self.offset_bits);
        let mut slot = slot_of(hash, self.slots.len());
        loop {
            let value = self.slots[slot];
            if value == 0 {
                return None;
            }
            if fingerprint_of(value, self.offset_bits) == fingerprint {
                let start = offset_of(value, self.offset_bits);
                let key = record_key(&self.arena, start);
                if key == word.as_bytes() {
                    let tag_start = start + 1 + key.len();
                    let mut raw = [0u8; 8];
                    raw[..self.tag_bytes]
                        .copy_from_slice(&self.arena[tag_start..tag_start + self.tag_bytes]);
                    return Some(&self.tags[usize::from_le_bytes(raw)]);
                }
            }
            slot = if slot + 1 == self.slots.len() { 0 } else { slot + 1 };
        }
    }

    fn max_key_len(&self, first: char) -> usize {
        self.index.max_len(first)
    }

    /// Tách tham lam theo cụm dài nhất có trong bảng; ký tự lạ mang nhãn `"?"`.
    #[cfg(test)]
    pub(crate) fn segment(&self, text: &str) -> Vec<(String, &str)> {
        let mut out = Vec::new();
        self.spans(text, false, |piece, tag| out.push((piece.to_string(), tag)));
        out
    }

    /// Như `segment`, khi người gọi vừa tra `text` không thấy: khỏi tra lại cả chuỗi.
    pub(crate) fn segment_missed(&self, text: &str) -> Vec<(String, &str)> {
        debug_assert!(self.get(text).is_none(), "{text:?} có trong bảng");
        let mut out = Vec::new();
        self.spans(text, true, |piece, tag| out.push((piece.to_string(), tag)));
        out
    }

    /// Lần lượt các khúc của `segment` cùng nhãn, cắt thẳng trên `text`: tại mỗi
    /// chỗ thử khúc dài nhất trước (tối đa `max_key_len` chữ), bớt dần từng chữ
    /// cuối; khúc đầu tiên có trong bảng thắng, không có thì một chữ mang nhãn
    /// `"?"`. `missed`: cả `text` đã biết là không có trong bảng, bỏ lần tra đó.
    fn spans<'s, 't>(
        &'s self,
        text: &'t str,
        missed: bool,
        mut each: impl FnMut(&'t str, &'s str),
    ) {
        let mut at = 0;
        while let Some(first) = text[at..].chars().next() {
            let rest = &text[at..];
            let cap = self.max_key_len(first);
            let mut end = rest.char_indices().nth(cap).map_or(rest.len(), |(i, _)| i);
            let mut found = None;
            while end > 0 {
                if !(missed && at == 0 && end == rest.len()) {
                    if let Some(tag) = self.get(&rest[..end]) {
                        found = Some((end, tag));
                        break;
                    }
                }
                end = rest[..end].char_indices().next_back().map_or(0, |(i, _)| i);
            }
            let (length, tag) = found.unwrap_or((first.len_utf8(), "?"));
            each(&rest[..length], tag);
            at += length;
        }
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
        // `core` bằng `zh` thì vừa tra không thấy, khỏi tra lại.
        if core.len() != zh.len() {
            if let Some(tag) = self.get(core) {
                return tag;
            }
        }
        // `core` chắc chắn không có trong bảng. Trong các nhãn khác `"?"` của các
        // khúc chỉ cần nhãn đầu, hai nhãn cuối, số nhãn và có động từ hay không.
        let (mut first, mut last, mut before_last) = (None, None, None);
        let (mut count, mut verb) = (0, false);
        self.spans(core, true, |_, tag| {
            if tag == "?" {
                return;
            }
            first.get_or_insert(tag);
            before_last = last.replace(tag);
            count += 1;
            verb |= tag.starts_with('v') && tag != "vn";
        });
        let (Some(first), Some(last)) = (first, last) else {
            return "?";
        };
        if first.starts_with('p') && count > 1 {
            return "pp";
        }
        if verb {
            return "v";
        }
        match before_last {
            Some(before_last) if last.starts_with('u') && count > 1 => before_last,
            _ => last,
        }
    }
}

/// Vị trí ô đầu của một hash: nhân-lấy-cao, đều trên `0..count`.
fn slot_of(hash: u64, count: usize) -> usize {
    ((hash as u128 * count as u128) >> 64) as usize
}

/// Ô = `offset + 1` ở `offset_bits` bit thấp, dấu vân tay ở phần cao.
fn pack(offset: usize, fingerprint: u32, offset_bits: u32) -> u32 {
    ((offset as u64 + 1) | ((fingerprint as u64) << offset_bits)) as u32
}

fn offset_of(value: u32, offset_bits: u32) -> usize {
    ((value as u64 & ((1u64 << offset_bits) - 1)) - 1) as usize
}

fn fingerprint_of(value: u32, offset_bits: u32) -> u32 {
    (value as u64 >> offset_bits) as u32
}

/// Dấu vân tay: các bit thấp của hash vừa đủ lấp phần cao còn lại của ô
/// (0 bit khi offset chiếm cả 32 bit).
fn fingerprint(hash: u64, offset_bits: u32) -> u32 {
    ((hash as u32 as u64) & ((1u64 << (32 - offset_bits)) - 1)) as u32
}

/// Byte key của bản ghi bắt đầu tại `start`.
fn record_key(arena: &[u8], start: usize) -> &[u8] {
    let length = arena[start] as usize;
    &arena[start + 1..start + 1 + length]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compact::CompactDict;
    use crate::dict::DictionaryLookup;

    const DICT: &str = "美丽 10 ns\n少女 5 n\n看 3 v\n在 9 p\n家 4 n\n的 9 uj\n";

    #[test]
    fn tu_loai_overrides_jieba_tags() {
        let pos = PosLexicon::build(DICT, "美丽\ta\n");
        assert_eq!(pos.get("美丽"), Some("a"));
        assert_eq!(pos.get("少女"), Some("n"));
        assert_eq!(pos.get("没有"), None);
    }

    /// QT2025 `Resources/dict.txt` mở đầu bằng EF BB BF.
    #[test]
    fn leading_bom_does_not_hide_the_first_entry() {
        let pos = PosLexicon::build("\u{feff}美丽 10 ns\n少女 5 n\n", "\u{feff}看\tv\n家 n\n");
        assert_eq!(pos.get("美丽"), Some("ns"));
        assert_eq!(pos.get("看"), Some("v"));
        // Dòng TuLoai ngăn bằng dấu cách bị bỏ (ghi trong doc module).
        assert_eq!(pos.get("家"), None);
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

    /// Bản cũ của `segment` và `phrase` (mảng ký tự, dựng chuỗi từng lần tra),
    /// để so với bản cắt thẳng trên chuỗi.
    fn segment_slow<'s>(pos: &'s PosLexicon, text: &str) -> Vec<(String, &'s str)> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let cap = pos.max_key_len(chars[i]).min(chars.len() - i);
            let found = (1..=cap).rev().find_map(|length| {
                let piece: String = chars[i..i + length].iter().collect();
                pos.get(&piece).map(|tag| (length, tag))
            });
            let (length, tag) = found.unwrap_or((1, "?"));
            out.push((chars[i..i + length].iter().collect(), tag));
            i += length;
        }
        out
    }

    fn phrase_slow<'s>(pos: &'s PosLexicon, zh: &str) -> &'s str {
        if let Some(tag) = pos.get(zh) {
            return tag;
        }
        let core = zh.trim_end_matches(TRAILERS);
        let core = if core.is_empty() { zh } else { core };
        if let Some(tag) = pos.get(core) {
            return tag;
        }
        let tags: Vec<&str> = segment_slow(pos, core)
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

    #[test]
    fn string_spans_match_the_char_array_version() {
        let alphabet: Vec<char> = "在家看着了的们少女美丽吗x1".chars().collect();
        let tags = ["p", "v", "vn", "uj", "ul", "n", "ns", "a", "?", "d"];
        let mut seed: u64 = 5;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        let word = |next: &mut dyn FnMut() -> usize, max: usize| -> String {
            (0..next() % (max + 1)).map(|_| alphabet[next() % alphabet.len()]).collect()
        };
        for _ in 0..200 {
            let mut dict = String::new();
            for _ in 0..1 + next() % 40 {
                let key = word(&mut next, 4);
                if !key.is_empty() {
                    dict.push_str(&format!("{key} 1 {}\n", tags[next() % tags.len()]));
                }
            }
            let pos = PosLexicon::build(&dict, "");
            for _ in 0..50 {
                let text = word(&mut next, 7);
                assert_eq!(pos.segment(&text), segment_slow(&pos, &text), "{dict:?} {text:?}");
                if pos.get(&text).is_none() {
                    assert_eq!(pos.segment_missed(&text), segment_slow(&pos, &text), "{text:?}");
                }
                assert_eq!(pos.phrase(&text), phrase_slow(&pos, &text), "{dict:?} {text:?}");
            }
        }
    }

    #[test]
    fn many_tags_and_duplicates_keep_first_and_resolve_all() {
        let mut dict = String::new();
        for i in 0..3000 {
            dict.push_str(&format!("词{i} 1 t{}\n", i % 300));
        }
        dict.push_str("词7 1 LOST\n");
        let pos = PosLexicon::build(&dict, "词8\tfix\n");
        for i in 0..3000 {
            let expected = if i == 8 { "fix".to_string() } else { format!("t{}", i % 300) };
            assert_eq!(pos.get(&format!("词{i}")), Some(expected.as_str()));
        }
        assert_eq!(pos.get("词3000"), None);
        assert_eq!(pos.get(""), None);
    }

    #[test]
    fn tag_ids_beyond_two_bytes_do_not_wrap() {
        let entries: Vec<(String, String)> =
            (0..70_000).map(|i| (format!("k{i}"), format!("tag{i}"))).collect();
        let pos = PosLexicon::first_wins(entries.iter().map(|(k, t)| (k.as_str(), t.as_str())));
        assert_eq!(pos.tag_bytes, 3);
        for (key, tag) in &entries {
            assert_eq!(pos.get(key), Some(tag.as_str()));
        }
    }

    #[test]
    fn matches_compact_dict_on_synthetic_keys() {
        let long = "长".repeat(100); // 300 byte, quá giới hạn key
        let mut entries: Vec<(String, String)> = Vec::new();
        // Nhiều key chung chữ đầu: chuỗi thăm dò dài, nhiều ô cùng vân tay.
        for i in 0..4000 {
            entries.push((format!("我{i}"), format!("t{}", i % 400)));
        }
        for i in 0..500 {
            entries.push((format!("你{}{}", i, "们".repeat(i % 7)), format!("u{}", i % 3)));
        }
        entries.push(("我7".into(), "LOST".into())); // trùng: bản đầu thắng
        entries.push((long.clone(), "n".into()));
        entries.push((String::new(), "empty-key".into()));
        entries.push(("空".into(), String::new()));
        entries.push(("a".into(), "x".repeat(100)));
        let pairs = || entries.iter().map(|(k, t)| (k.as_str(), t.as_str()));
        let (reference, skipped) = CompactDict::first_wins(pairs());
        assert_eq!(skipped, 1);
        let pos = PosLexicon::first_wins(pairs());
        for (key, _) in &entries {
            assert_eq!(pos.get(key), reference.get(key), "key {key:?}");
        }
        for miss in ["我", "我4000", "他", "你们", "b", "空空"] {
            assert_eq!(pos.get(miss), reference.get(miss), "miss {miss:?}");
        }
        for first in ['我', '你', '空', 'a', 'b', '长', '他'] {
            assert_eq!(pos.max_key_len(first), reference.max_key_len(first), "{first}");
        }
    }
}
