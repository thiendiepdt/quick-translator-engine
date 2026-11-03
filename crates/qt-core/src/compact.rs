//! Từ điển chỉ-đọc lưu gọn: mọi key/value nằm liền trong một chuỗi arena,
//! bảng băm chỉ giữ offset + độ dài (8 byte/entry).
//!
//! `FxHashMap<String, String>` tốn hai `String` header (48 B) + hai lần cấp
//! phát riêng cho mỗi entry. VietPhrase có ~764k entry nhưng chữ thật chỉ
//! ~22 MB (key + nghĩa đầu), nên overhead mới là phần ăn RAM: cùng dữ liệu
//! đó dạng map ~95 MB, dạng arena ~33 MB. Dùng cho server lite, nơi từ điển
//! nạp một lần và không bao giờ sửa.

use crate::dict::{DictionaryLookup, KeyLenIndex};
use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;
use std::hash::BuildHasher;

/// Key dài nhất (byte) lưu được: 15 ký tự Hán × 4 byte vẫn còn dư xa.
const MAX_KEY_BYTES: usize = u8::MAX as usize;
/// Value dài nhất (byte) lưu được trong 24 bit độ dài.
const MAX_VALUE_BYTES: usize = (1 << 24) - 1;

#[derive(Clone, Copy)]
struct Entry {
    offset: u32,
    /// `value_len << 8 | key_len`.
    lengths: u32,
}

impl Entry {
    fn key_len(self) -> usize {
        (self.lengths & 0xff) as usize
    }

    fn value_len(self) -> usize {
        (self.lengths >> 8) as usize
    }
}

#[derive(Default)]
pub(crate) struct CompactDict {
    arena: Box<str>,
    table: HashTable<Entry>,
    index: KeyLenIndex,
}

impl CompactDict {
    /// Dựng từ các cặp (key, value) theo thứ tự nguồn; key trùng thì bản đầu
    /// thắng (first-wins như các loader QT). Cặp vượt giới hạn độ dài bị bỏ
    /// qua và được đếm vào giá trị trả về thứ hai.
    ///
    /// Duyệt nguồn hai lượt: lượt đầu chỉ đếm để cấp phát arena và bảng băm
    /// đúng một lần. Để chúng tự phình (nhân đôi rồi chép) thì các bản cũ đã
    /// free vẫn bị allocator giữ trong RSS sau khi nạp.
    pub(crate) fn first_wins<'a, I>(entries: I) -> (Self, usize)
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
    {
        let (count, bytes) = entries
            .clone()
            .fold((0usize, 0usize), |(count, bytes), (key, value)| {
                (count + 1, bytes + key.len() + value.len())
            });
        let hasher = FxBuildHasher;
        let mut arena = String::with_capacity(bytes);
        let mut table: HashTable<Entry> = HashTable::with_capacity(count);
        let mut skipped = 0usize;
        for (key, value) in entries {
            if key.len() > MAX_KEY_BYTES || value.len() > MAX_VALUE_BYTES {
                skipped += 1;
                continue;
            }
            let hash = hasher.hash_one(key);
            if table
                .find(hash, |entry| entry_key(&arena, *entry) == key)
                .is_some()
            {
                continue;
            }
            let offset =
                u32::try_from(arena.len()).expect("compact dictionary arena exceeds 4 GiB");
            arena.push_str(key);
            arena.push_str(value);
            let entry = Entry {
                offset,
                lengths: ((value.len() as u32) << 8) | key.len() as u32,
            };
            table.insert_unique(hash, entry, |entry| {
                hasher.hash_one(entry_key(&arena, *entry))
            });
        }
        table.shrink_to_fit(|entry| hasher.hash_one(entry_key(&arena, *entry)));
        let arena = arena.into_boxed_str();
        let index = KeyLenIndex::from_str_keys(table.iter().map(|entry| entry_key(&arena, *entry)));
        (
            Self {
                arena,
                table,
                index,
            },
            skipped,
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.table.len()
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &str> {
        self.table
            .iter()
            .map(|entry| entry_key(&self.arena, *entry))
    }
}

fn entry_key(arena: &str, entry: Entry) -> &str {
    let start = entry.offset as usize;
    &arena[start..start + entry.key_len()]
}

fn entry_value(arena: &str, entry: Entry) -> &str {
    let start = entry.offset as usize + entry.key_len();
    &arena[start..start + entry.value_len()]
}

impl DictionaryLookup for CompactDict {
    fn get(&self, key: &str) -> Option<&str> {
        let hash = FxBuildHasher.hash_one(key);
        self.table
            .find(hash, |entry| entry_key(&self.arena, *entry) == key)
            .map(|entry| entry_value(&self.arena, *entry))
    }

    fn max_key_len(&self, first: char) -> usize {
        self.index.max_len(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_wins_lookup_and_key_length_index() {
        let (dict, skipped) = CompactDict::first_wins(
            [
                ("很好", "rất tốt"),
                ("很", "rất"),
                ("很好", "LOST"),
                ("一二三", ""),
            ]
            .into_iter(),
        );
        assert_eq!(skipped, 0);
        assert_eq!(dict.len(), 3);
        assert_eq!(dict.get("很好"), Some("rất tốt"));
        assert_eq!(dict.get("很"), Some("rất"));
        assert_eq!(dict.get("一二三"), Some(""));
        assert_eq!(dict.get("好"), None);
        assert_eq!(dict.max_key_len('很'), 2);
        assert_eq!(dict.max_key_len('一'), 3);
        assert_eq!(dict.max_key_len('好'), 0);
        let mut keys: Vec<_> = dict.keys().collect();
        keys.sort_unstable();
        assert_eq!(keys, ["一二三", "很", "很好"]);
    }

    #[test]
    fn oversized_entries_are_skipped_and_counted() {
        let long_key = "k".repeat(MAX_KEY_BYTES + 1);
        let (dict, skipped) =
            CompactDict::first_wins([(long_key.as_str(), "x"), ("a", "b")].into_iter());
        assert_eq!(skipped, 1);
        assert_eq!(dict.get("a"), Some("b"));
        assert_eq!(dict.get(&long_key), None);
    }
}
