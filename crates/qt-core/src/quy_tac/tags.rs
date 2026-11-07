//! Từ loại cho Quy Tắc: bảng kiểu jieba (`Resources/dict.txt`, dòng
//! `từ tần-số nhãn`) đè bằng `TuLoai.txt` (dòng `từ<TAB>nhãn`). Lưu gọn trong
//! bảng băm tự dựng (~350 nghìn mục, ~5,5 MB).

use crate::dict::KeyLenIndex;
use rustc_hash::FxBuildHasher;
use std::hash::BuildHasher;

/// Trợ từ cuối được bỏ đi khi tra một cụm không có trong bảng.
const TRAILERS: &[char] = &['了', '着', '过', '吗', '呢', '啊', '吧', '呀', '嘛', '哦', '哈'];

/// Key dài nhất (byte) lưu được; key dài hơn bị bỏ như ở `CompactDict`.
const MAX_KEY_BYTES: usize = u8::MAX as usize;

/// Bảng key -> nhãn, chỉ đọc. Mỗi bản ghi trong arena là
/// `[độ dài key][byte của key][nhãn 1-2 byte]`; nhãn là chỉ số vào bảng nhãn
/// đã gộp (chỉ vài chục nhãn khác nhau). Bảng băm là mảng `u32` thăm dò
/// tuyến tính, tải ~0,75: mỗi ô gồm `offset + 1` ở các bit thấp và vài bit
/// dấu vân tay của hash ở các bit cao (0 là ô trống), nên hầu hết lần trượt
/// bị loại mà không phải đọc arena.
pub(crate) struct PosLexicon {
    arena: Box<[u8]>,
    slots: Box<[u32]>,
    tags: Box<[Box<str>]>,
    /// 1 hoặc 2 byte chỉ số nhãn trong mỗi bản ghi.
    tag_bytes: usize,
    offset_bits: u32,
    index: KeyLenIndex,
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
        Self::first_wins(fixes.chain(base))
    }

    /// Hai lượt: lượt đầu đếm và gộp nhãn để cấp phát đúng một lần.
    fn first_wins<'a, I>(entries: I) -> PosLexicon
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
    {
        let mut tags: Vec<Box<str>> = Vec::new();
        let (mut count, mut key_bytes) = (0usize, 0usize);
        for (key, tag) in entries.clone() {
            if key.len() > MAX_KEY_BYTES {
                continue;
            }
            if !tags.iter().any(|known| **known == *tag) {
                tags.push(tag.into());
            }
            count += 1;
            key_bytes += key.len();
        }
        let tag_bytes = if tags.len() <= 256 { 1 } else { 2 };
        let bound = key_bytes + count * (1 + tag_bytes);
        let bound = u32::try_from(bound).expect("bảng từ loại vượt 4 GiB");
        let offset_bits = u32::BITS - bound.leading_zeros();
        let slot_count = (count * 4 / 3 + 1).max(1);
        let mut slots = vec![0u32; slot_count];
        let mut arena: Vec<u8> = Vec::with_capacity(bound as usize);
        let hasher = FxBuildHasher;
        for (key, tag) in entries {
            if key.len() > MAX_KEY_BYTES {
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
            let id = tags.iter().position(|known| **known == *tag).expect("nhãn đã gộp");
            slots[slot] = pack(arena.len(), fingerprint, offset_bits);
            arena.push(key.len() as u8);
            arena.extend_from_slice(key.as_bytes());
            arena.push(id as u8);
            if tag_bytes == 2 {
                arena.push((id >> 8) as u8);
            }
        }
        let mut keys = Vec::with_capacity(count);
        let mut start = 0;
        while start < arena.len() {
            let key = record_key(&arena, start);
            keys.push(std::str::from_utf8(key).expect("key lấy từ str"));
            start += 1 + key.len() + tag_bytes;
        }
        let index = KeyLenIndex::from_str_keys(keys);
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
                    let mut id = self.arena[tag_start] as usize;
                    if self.tag_bytes == 2 {
                        id |= (self.arena[tag_start + 1] as usize) << 8;
                    }
                    return Some(&self.tags[id]);
                }
            }
            slot = if slot + 1 == self.slots.len() { 0 } else { slot + 1 };
        }
    }

    fn max_key_len(&self, first: char) -> usize {
        self.index.max_len(first)
    }

    /// Tách tham lam theo cụm dài nhất có trong bảng; ký tự lạ mang nhãn `"?"`.
    pub(crate) fn segment(&self, text: &str) -> Vec<(String, &str)> {
        let chars: Vec<char> = text.chars().collect();
        self.spans(&chars)
            .into_iter()
            .map(|(start, length, tag)| (chars[start..start + length].iter().collect(), tag))
            .collect()
    }

    /// Các khúc của `segment`: (vị trí ký tự đầu, số ký tự, nhãn), khỏi dựng chuỗi.
    fn spans(&self, chars: &[char]) -> Vec<(usize, usize, &str)> {
        let mut out = Vec::new();
        let mut buffer = String::new();
        let mut i = 0;
        while i < chars.len() {
            let cap = self.max_key_len(chars[i]).min(chars.len() - i);
            let mut found = None;
            for length in (1..=cap).rev() {
                buffer.clear();
                buffer.extend(&chars[i..i + length]);
                if let Some(tag) = self.get(&buffer) {
                    found = Some((length, tag));
                    break;
                }
            }
            let (length, tag) = found.unwrap_or((1, "?"));
            out.push((i, length, tag));
            i += length;
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
        let chars: Vec<char> = core.chars().collect();
        let tags: Vec<&str> = self
            .spans(&chars)
            .into_iter()
            .map(|(_, _, tag)| tag)
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

/// Vị trí ô đầu của một hash: nhân-lấy-cao, đều trên `0..count`.
fn slot_of(hash: u64, count: usize) -> usize {
    ((hash as u128 * count as u128) >> 64) as usize
}

/// Ô = `offset + 1` ở `offset_bits` bit thấp, dấu vân tay ở phần cao.
fn pack(offset: usize, fingerprint: u32, offset_bits: u32) -> u32 {
    (offset as u64 + 1 | (fingerprint as u64) << offset_bits) as u32
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

    #[test]
    fn many_tags_and_duplicates_keep_first_and_resolve_all() {
        let mut dict = String::new();
        for i in 0..3000 {
            dict.push_str(&format!("词{i} 1 t{}", i % 300));
            dict.push(char::from(10));
        }
        dict.push_str("词7 1 LOST");
        dict.push(char::from(10));
        let tu_loai = format!("词8{}fix{}", char::from(9), char::from(10));
        let pos = PosLexicon::build(&dict, &tu_loai);
        for i in 0..3000 {
            let expected = if i == 8 { "fix".to_string() } else { format!("t{}", i % 300) };
            assert_eq!(pos.get(&format!("词{i}")), Some(expected.as_str()));
        }
        assert_eq!(pos.get("词3000"), None);
        assert_eq!(pos.get(""), None);
    }
}
