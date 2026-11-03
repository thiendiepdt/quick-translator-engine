//! Engine gọn cho `qt-lite-server`: chỉ dịch VietPhrase một nghĩa.
//!
//! So với [`crate::Engine`], bản này:
//! - giữ VietPhrase và Names **một lần**, chỉ nghĩa đầu, lưu dạng arena
//!   ([`CompactDict`]) thay cho ba bản `FxHashMap<String, String>`;
//! - không nạp Lạc Việt, không giữ chuỗi gốc của các từ điển mặc định, không
//!   có lọc tên (không jieba);
//! - mọi request đi qua cùng đường dịch có override với Engine
//!   ([`crate::LayeredBase`]), nên output trùng từng byte với mode
//!   `VietPhraseOneMeaning` của Engine trên cùng dữ liệu và cùng override.
//!
//! Vì sao chỉ cần nghĩa đầu: ở mode một nghĩa, giá trị chỉ được đọc qua view
//! `OneMeaningDictionary` (map chính và `dictionary_n`); `only_name`,
//! `only_vietphrase` và VietPhrase đầy đủ chỉ được hỏi "có key không".

use crate::compact::CompactDict;
use crate::dict::{
    first_wins_map, parse_dict_borrowed, parse_han_viet, parse_ignored_chinese_phrases,
    parse_luat_nhan, within_scan_window, DictionaryLookup, IndexedLookup, KeyLenIndex,
};
use crate::luat_nhan::LuatNhan;
use crate::standardize::Standardizer;
use crate::{
    first_meaning, DictionaryOverrideError, DictionaryOverrides, HanVietMap, LayeredBase, Mode,
    Options, PriorityDictionary,
};
use rustc_hash::FxHashMap;
use std::path::Path;

pub struct LiteEngine {
    han_viet: HanVietMap,
    standardizer: Standardizer,
    luat_nhan: LuatNhan,
    primary_names: CompactDict,
    secondary_names: CompactDict,
    only_vietphrase: CompactDict,
    pronouns: FxHashMap<String, String>,
    ho_nguoi: FxHashMap<String, String>,
    ho_nguoi_index: KeyLenIndex,
    hau_tu: FxHashMap<String, String>,
    hau_tu_index: KeyLenIndex,
}

/// Số entry đã nạp, để server in ra lúc khởi động.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteEngineStats {
    pub vietphrase_entries: usize,
    pub name_entries: usize,
}

impl LiteEngine {
    /// Nạp từ thư mục dữ liệu QT2025, cùng tên file và cùng quy tắc như
    /// [`crate::Dictionaries::load`]. ChinesePhienAmWords và VietPhrase bắt
    /// buộc; các file còn lại thiếu thì thành từ điển rỗng. Mỗi file được đọc
    /// rồi bỏ ngay sau khi dựng, để đỉnh RAM lúc nạp thấp.
    pub fn load(data_dir: &Path) -> std::io::Result<Self> {
        let read = |rel: &str| std::fs::read_to_string(data_dir.join(rel));
        let read_optional = |rel: &str| read(rel).unwrap_or_default();
        let han_viet = parse_han_viet(&read("Resources/ChinesePhienAmWords.txt")?);
        let only_vietphrase = scan_dictionary(&read("VietPhrase/VietPhrase.txt")?);
        let primary_names = scan_dictionary(&read_optional("Names.txt"));
        let secondary_names = scan_dictionary(&read_optional("Names2/123.txt"));
        Self::assemble(
            han_viet,
            primary_names,
            secondary_names,
            only_vietphrase,
            &read_optional("Resources/Pronouns.txt"),
            &read_optional("Resources/HoNguoi.txt"),
            &read_optional("Resources/HauTu.txt"),
            &read_optional("LuatNhan.txt"),
            &read_optional("IgnoredChinesePhrases.txt"),
        )
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidData, message))
    }

    /// Dựng từ nội dung file — cùng tham số với `DictionaryDefaults` +
    /// hai từ điển cố định. Dùng cho test.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        han_viet_src: &str,
        names_src: &str,
        names2_src: &str,
        vietphrase_src: &str,
        pronouns_src: &str,
        ho_nguoi_src: &str,
        hau_tu_src: &str,
        luat_nhan_src: &str,
        ignored_src: &str,
    ) -> Result<Self, String> {
        Self::assemble(
            parse_han_viet(han_viet_src),
            scan_dictionary(names_src),
            scan_dictionary(names2_src),
            scan_dictionary(vietphrase_src),
            pronouns_src,
            ho_nguoi_src,
            hau_tu_src,
            luat_nhan_src,
            ignored_src,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assemble(
        han_viet: HanVietMap,
        primary_names: (CompactDict, usize),
        secondary_names: (CompactDict, usize),
        only_vietphrase: (CompactDict, usize),
        pronouns_src: &str,
        ho_nguoi_src: &str,
        hau_tu_src: &str,
        luat_nhan_src: &str,
        ignored_src: &str,
    ) -> Result<Self, String> {
        let skipped = primary_names.1 + secondary_names.1 + only_vietphrase.1;
        if skipped > 0 {
            return Err(format!(
                "{skipped} dictionary entries exceed the compact storage limits"
            ));
        }
        let luat_nhan = LuatNhan::try_from_rules(&parse_luat_nhan(luat_nhan_src))
            .map_err(|error| format!("invalid base LuatNhan dictionary: {error}"))?;
        let standardizer =
            Standardizer::new(&han_viet, &parse_ignored_chinese_phrases(ignored_src));
        let ho_nguoi = first_wins_map(ho_nguoi_src);
        let hau_tu = first_wins_map(hau_tu_src);
        let ho_nguoi_index = KeyLenIndex::from_keys(ho_nguoi.keys());
        let hau_tu_index = KeyLenIndex::from_keys(hau_tu.keys());
        Ok(Self {
            han_viet,
            standardizer,
            luat_nhan,
            primary_names: primary_names.0,
            secondary_names: secondary_names.0,
            only_vietphrase: only_vietphrase.0,
            pronouns: first_wins_map(pronouns_src),
            ho_nguoi,
            ho_nguoi_index,
            hau_tu,
            hau_tu_index,
        })
    }

    pub fn stats(&self) -> LiteEngineStats {
        let secondary_only = self
            .secondary_names
            .keys()
            .filter(|key| !self.primary_names.contains_key(key))
            .count();
        LiteEngineStats {
            vietphrase_entries: self.only_vietphrase.len(),
            name_entries: self.primary_names.len() + secondary_only,
        }
    }

    /// Dịch VietPhrase một nghĩa, chỉ trả text (quick convert không cần
    /// range — bỏ chúng đỡ ~45% đỉnh RAM mỗi request). `overrides` rỗng
    /// nghĩa là dùng từ điển mặc định; có override thì xếp lớp y như
    /// [`crate::Engine::translate_with_overrides`].
    pub fn translate(
        &self,
        text: &str,
        opts: &Options,
        overrides: &DictionaryOverrides,
    ) -> Result<String, DictionaryOverrideError> {
        // Names2 đè lên Names.txt; chặn trên độ dài = max của hai index.
        let only_name = PriorityDictionary {
            primary: &self.secondary_names,
            fallback: &self.primary_names,
        };
        let ho_nguoi = IndexedLookup {
            inner: &self.ho_nguoi,
            index: &self.ho_nguoi_index,
        };
        let hau_tu = IndexedLookup {
            inner: &self.hau_tu,
            index: &self.hau_tu_index,
        };
        LayeredBase {
            han_viet: &self.han_viet,
            standardizer: &self.standardizer,
            luat_nhan: &self.luat_nhan,
            primary_names: &self.primary_names,
            secondary_names: &self.secondary_names,
            only_name: &only_name,
            only_vietphrase: &self.only_vietphrase,
            pronouns: &self.pronouns,
            ho_nguoi: &ho_nguoi,
            hau_tu: &hau_tu,
        }
        .translate(text, Mode::VietPhraseOneMeaning, opts, overrides, false)
        .map(|result| result.translated_text)
    }
}

/// Names / Names2 / VietPhrase: cùng quy tắc với `first_wins_scan_map`
/// (bỏ key dài hơn cửa sổ quét, bản đầu thắng) nhưng chỉ giữ nghĩa đầu.
fn scan_dictionary(source: &str) -> (CompactDict, usize) {
    CompactDict::first_wins(
        parse_dict_borrowed(source)
            .filter(|(key, _)| within_scan_window(key))
            .map(|(key, value)| (key, first_meaning(value))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dictionaries, DictionaryPatches, DictionarySourceOverrides, Engine};

    const HAN_VIET: &str =
        "他=tha\n很=ngận\n厉=lệ\n害=hại\n张=trương\n三=tam\n说=thuyết\n的=đích\n好=hảo\n人=nhân";
    const NAMES: &str = "张三=Trương Tam/Trương Ba\n李四=Lý Tứ";
    const NAMES2: &str = "张三=Trương Tam Mới";
    const VIETPHRASE: &str =
        "很好=rất tốt/rất ổn\n厉害=lợi hại|giỏi\n说=nói/bảo\n他=hắn/anh ấy\n的=của\n好人=người tốt";
    const PRONOUNS: &str = "他=hắn";
    const LUAT_NHAN: &str = "{0}的人={0} người";

    fn engines() -> (Engine, LiteEngine) {
        let defaults = crate::DictionaryDefaults {
            names: NAMES.to_string(),
            names2: NAMES2.to_string(),
            luat_nhan: LUAT_NHAN.to_string(),
            pronouns: PRONOUNS.to_string(),
            ..Default::default()
        };
        let dictionaries: Dictionaries = defaults.build_dictionaries(HAN_VIET, VIETPHRASE);
        let lite = LiteEngine::build(
            HAN_VIET, NAMES, NAMES2, VIETPHRASE, PRONOUNS, "", "", LUAT_NHAN, "",
        )
        .unwrap();
        (Engine::from_dicts(dictionaries), lite)
    }

    fn assert_same(
        engine: &Engine,
        lite: &LiteEngine,
        text: &str,
        overrides: &DictionaryOverrides,
    ) {
        let opts = Options::default();
        // So với đường CÓ range của Engine: text của đường không range phải
        // trùng hẳn.
        let expected = engine
            .translate_with_ranges_and_overrides(text, Mode::VietPhraseOneMeaning, &opts, overrides)
            .unwrap()
            .translated_text;
        let got = lite.translate(text, &opts, overrides).unwrap();
        assert_eq!(got, expected, "text: {text}");
    }

    #[test]
    fn matches_engine_one_meaning_without_overrides() {
        let (engine, lite) = engines();
        let empty = DictionaryOverrides::default();
        for text in ["他说很好", "张三很厉害", "李四说的好人", "他很好的人", ""]
        {
            assert_same(&engine, &lite, text, &empty);
        }
        assert_eq!(
            lite.translate("张三很好", &Options::default(), &empty)
                .unwrap(),
            engine.translate("张三很好", Mode::VietPhraseOneMeaning, &Options::default())
        );
    }

    #[test]
    fn matches_engine_with_request_names_and_patches() {
        let (engine, lite) = engines();
        let overrides = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            names: Some("王五=Vương Ngũ/Vương Năm\n张三=Trương Tam Khác"),
            pronouns: Some("他=gã"),
            ..Default::default()
        })
        .with_patches(DictionaryPatches {
            vietphrase: [("很好".to_string(), "tuyệt/được".to_string())]
                .into_iter()
                .collect(),
            chinese_phien_am_words: [('好', "HẢO".to_string())].into_iter().collect(),
        });
        for text in ["王五说很好", "张三很厉害", "他说好人", "好"] {
            assert_same(&engine, &lite, text, &overrides);
        }
        let translated = lite
            .translate("王五说很好", &Options::default(), &overrides)
            .unwrap();
        assert!(translated.contains("Vương Ngũ"), "{translated}");
        assert!(translated.contains("tuyệt"), "{translated}");
        assert!(!translated.contains('/'), "{translated}");
    }

    #[test]
    fn names2_overrides_names_and_stats_count_unique_names() {
        let (_, lite) = engines();
        let translated = lite
            .translate("张三", &Options::default(), &DictionaryOverrides::default())
            .unwrap();
        assert!(translated.contains("Trương Tam Mới"), "{translated}");
        assert_eq!(
            lite.stats(),
            LiteEngineStats {
                vietphrase_entries: 6,
                name_entries: 2,
            }
        );
    }

    #[test]
    fn invalid_base_luat_nhan_is_an_error() {
        let result = LiteEngine::build("", "", "", "", "", "", "", "([{n}=invalid", "");
        assert!(result.is_err());
    }
}
