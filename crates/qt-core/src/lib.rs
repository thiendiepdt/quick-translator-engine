//! qt-core: Quick Translator engine (Rust reimplementation).

mod dict;
mod han_viet;
mod luat_nhan;
mod name_filter;
mod number;
mod standardize;
mod text;
mod translate;

pub use dict::{
    parse_dict, Dictionaries, DictionaryDefaults, DictionaryOverrides, DictionaryPatches,
    DictionarySourceOverrides,
};
pub use name_filter::{
    NameCandidate, NameCandidateSource, NameEntityType, NameFilterDocument, NameFilterMemory,
    NameFilterMode, NameFilterOptions, NameFilterResult,
};

use dict::DictionaryLookup;

struct PriorityDictionary<'a> {
    primary: &'a dyn DictionaryLookup,
    fallback: &'a dyn DictionaryLookup,
}

impl DictionaryLookup for PriorityDictionary<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.primary.get(key).or_else(|| self.fallback.get(key))
    }
}

struct OneMeaningDictionary<'a> {
    inner: &'a dyn DictionaryLookup,
}

impl DictionaryLookup for OneMeaningDictionary<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(first_meaning)
    }
}

fn first_meaning(value: &str) -> &str {
    value
        .find(['/', '|'])
        .map_or(value, |index| &value[..index])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryOverrideError {
    message: String,
}

impl std::fmt::Display for DictionaryOverrideError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DictionaryOverrideError {}

/// A half-open range represented as a UTF-16 start offset and length.
/// UTF-16 offsets can be consumed directly by JavaScript UIs. Range entries
/// are phrase/Rust-scalar spans, not necessarily one entry per UTF-16 code
/// unit; for example, a fallback emoji is one range with `length == 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    pub start: usize,
    pub length: usize,
}

/// Translated text plus parallel source/target phrase ranges. HanViet uses
/// this same useful two-sided contract instead of QT2025's target-only,
/// off-by-one-prone `chineseHanVietMappingArray`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationResult {
    pub translated_text: String,
    pub source_ranges: Vec<CharRange>,
    pub target_ranges: Vec<CharRange>,
}

/// Output view, mirrors QT's translation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    HanViet,
    VietPhrase,
    VietPhraseOneMeaning,
}

/// Translation parameters, matching the original engine signature.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub wrap_type: i32,             // 0 = plain, 1 = wrap each phrase in [...]
    pub translation_algorithm: i32, // 0/1/2, controls "longest phrase" rule
    pub prioritized_name: bool,
    pub scan_range: usize, // max phrase length scanned per position
}

impl Default for Options {
    fn default() -> Self {
        // QuickTranslatorMain.config stores scanRange=30,
        // TranslationAlgorithm=1, and PrioritizedName=true.
        Options {
            wrap_type: 0,
            translation_algorithm: 1,
            prioritized_name: true,
            scan_range: 30,
        }
    }
}

pub struct Engine {
    dicts: Dictionaries,
    luat_nhan: luat_nhan::LuatNhan,
    standardizer: standardize::Standardizer,
}

impl Engine {
    pub fn from_dicts(dicts: Dictionaries) -> Engine {
        let luat_nhan = luat_nhan::LuatNhan::new(&dicts);
        let standardizer =
            standardize::Standardizer::new(&dicts.han_viet, &dicts.ignored_chinese_phrases);
        Engine {
            dicts,
            luat_nhan,
            standardizer,
        }
    }

    pub fn translate(&self, text: &str, mode: Mode, opts: &Options) -> String {
        self.translate_with_ranges(text, mode, opts).translated_text
    }

    pub fn translate_with_ranges(
        &self,
        text: &str,
        mode: Mode,
        opts: &Options,
    ) -> TranslationResult {
        let standardized = self.standardizer.standardize(text);
        let chars = standardized.chars;
        let source_ranges = standardized.source_ranges;
        match mode {
            Mode::HanViet => {
                han_viet::chinese_to_han_viet_mapped(&chars, &self.dicts.han_viet, &source_ranges)
            }
            Mode::VietPhrase => translate::translate_all_mapped(
                &chars,
                &source_ranges,
                opts,
                &self.dicts.vietphrase,
                &self.dicts.only_name,
                &self.dicts.only_vietphrase,
                &self.dicts.vietphrase,
                &self.dicts.han_viet,
                &self.luat_nhan,
                None,
                None,
                None,
            ),
            Mode::VietPhraseOneMeaning => translate::translate_all_mapped(
                &chars,
                &source_ranges,
                opts,
                &self.dicts.vietphrase_one_meaning,
                &self.dicts.only_name,
                &self.dicts.only_vietphrase,
                &self.dicts.vietphrase,
                &self.dicts.han_viet,
                &self.luat_nhan,
                None,
                None,
                None,
            ),
        }
    }

    /// Translate with request-scoped file replacements and compact patches
    /// layered over the fixed VietPhrase and ChinesePhienAmWords dictionaries.
    pub fn translate_with_overrides(
        &self,
        text: &str,
        mode: Mode,
        opts: &Options,
        overrides: &DictionaryOverrides,
    ) -> Result<String, DictionaryOverrideError> {
        Ok(self
            .translate_with_ranges_and_overrides(text, mode, opts, overrides)?
            .translated_text)
    }

    pub fn translate_with_ranges_and_overrides(
        &self,
        text: &str,
        mode: Mode,
        opts: &Options,
        overrides: &DictionaryOverrides,
    ) -> Result<TranslationResult, DictionaryOverrideError> {
        if overrides.is_empty() {
            return Ok(self.translate_with_ranges(text, mode, opts));
        }

        let standardized = match &overrides.ignored_chinese_phrases {
            Some(ignored) => self
                .standardizer
                .standardize_with_ignored_source(text, ignored),
            None => self.standardizer.standardize(text),
        };
        let chars = standardized.chars;
        let source_ranges = standardized.source_ranges;
        let custom_han_viet = (!overrides.chinese_phien_am_words_patches.is_empty()).then(|| {
            let mut dictionary = self.dicts.han_viet.clone();
            dictionary.extend(
                overrides
                    .chinese_phien_am_words_patches
                    .iter()
                    .map(|(key, value)| (*key, value.clone())),
            );
            dictionary
        });
        let han_viet = custom_han_viet.as_ref().unwrap_or(&self.dicts.han_viet);
        if mode == Mode::HanViet {
            return Ok(han_viet::chinese_to_han_viet_mapped(
                &chars,
                han_viet,
                &source_ranges,
            ));
        }

        let primary_names = overrides
            .names
            .as_ref()
            .unwrap_or(&self.dicts.primary_names);
        let secondary_names = overrides
            .names2
            .as_ref()
            .unwrap_or(&self.dicts.secondary_names);
        let custom_names = PriorityDictionary {
            primary: secondary_names,
            fallback: primary_names,
        };
        let names: &dyn DictionaryLookup =
            if overrides.names.is_none() && overrides.names2.is_none() {
                &self.dicts.only_name
            } else {
                &custom_names
            };
        let patched_only_vietphrase = PriorityDictionary {
            primary: &overrides.vietphrase_patches,
            fallback: &self.dicts.only_vietphrase,
        };
        let only_vietphrase: &dyn DictionaryLookup = if overrides.vietphrase_patches.is_empty() {
            &self.dicts.only_vietphrase
        } else {
            &patched_only_vietphrase
        };
        let vietphrase = PriorityDictionary {
            primary: names,
            fallback: only_vietphrase,
        };
        let one_meaning_vietphrase = OneMeaningDictionary { inner: &vietphrase };
        let one_meaning_names = OneMeaningDictionary { inner: names };
        let pronouns = overrides.pronouns.as_ref().unwrap_or(&self.dicts.pronouns);
        let dictionary_n = PriorityDictionary {
            primary: pronouns,
            fallback: &one_meaning_names,
        };
        let ho_nguoi = overrides.ho_nguoi.as_ref().unwrap_or(&self.dicts.ho_nguoi);
        let hau_tu = overrides.hau_tu.as_ref().unwrap_or(&self.dicts.hau_tu);

        // DanhTu is loaded by QT2025 and remains part of the public override
        // contract, although the ported translation paths do not consume it.
        let _danh_tu = overrides.danh_tu.as_ref().unwrap_or(&self.dicts.danh_tu);

        let custom_luat_nhan = overrides
            .luat_nhan
            .as_ref()
            .map(|rules| luat_nhan::LuatNhan::try_from_rules(rules))
            .transpose()
            .map_err(|message| DictionaryOverrideError { message })?;
        let luat_nhan = custom_luat_nhan.as_ref().unwrap_or(&self.luat_nhan);
        let dictionary: &dyn DictionaryLookup = match mode {
            Mode::VietPhrase => &vietphrase,
            Mode::VietPhraseOneMeaning => &one_meaning_vietphrase,
            Mode::HanViet => unreachable!("HanViet returned above"),
        };

        Ok(translate::translate_all_mapped(
            &chars,
            &source_ranges,
            opts,
            dictionary,
            names,
            only_vietphrase,
            &vietphrase,
            han_viet,
            luat_nhan,
            Some(&dictionary_n),
            Some(ho_nguoi),
            Some(hau_tu),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn hv_map() -> HashMap<char, String> {
        let mut m = HashMap::new();
        for (k, v) in [('他', "tha"), ('很', "ngận"), ('厉', "lệ"), ('害', "hại")] {
            m.insert(k, v.to_string());
        }
        m
    }

    fn engine_hv_only() -> Engine {
        let d = Dictionaries {
            han_viet: hv_map(),
            ..Default::default()
        };
        Engine::from_dicts(d)
    }

    #[test]
    fn options_default_matches_spec() {
        let o = Options::default();
        assert_eq!(o.wrap_type, 0);
        assert_eq!(o.translation_algorithm, 1);
        assert!(o.prioritized_name);
        assert_eq!(o.scan_range, 30);
    }

    #[test]
    fn hanviet_spaces_between_hanzi() {
        let e = engine_hv_only();
        // 4 consecutive hanzi → readings joined by single spaces; first word keeps a leading space, lowercase (faithful to engine)
        let got = e.translate("他很厉害", Mode::HanViet, &Options::default());
        assert_eq!(got, " tha ngận lệ hại");
    }

    #[test]
    fn hanviet_passes_through_non_chinese() {
        let e = engine_hv_only();
        let got = e.translate("他, 好", Mode::HanViet, &Options::default());
        // '他'→tha (leading space, lowercase), then raw ', ', then unknown '好' passes through
        assert_eq!(got, " tha, 好");
    }

    #[test]
    fn default_scan_range_matches_long_qt_name_entries() {
        let d = Dictionaries::build(
            "丁=đinh\n格=cách\n尔=nhĩ\n斯=tư\n泰=thái\n特=đặc",
            "丁格尔斯泰特=Dingelstedt",
            "",
            "",
        );
        let e = Engine::from_dicts(d);

        assert_eq!(
            e.translate(
                "丁格尔斯泰特",
                Mode::VietPhraseOneMeaning,
                &Options::default()
            ),
            " Dingelstedt"
        );
    }

    #[test]
    fn standardizes_chinese_punctuation_before_translation() {
        let e = engine_hv_only();
        let result = e.translate_with_ranges(
            "他、他。他",
            Mode::VietPhraseOneMeaning,
            &Options::default(),
        );
        assert_eq!(result.translated_text, " tha, tha. Tha");
        assert_eq!(result.source_ranges.len(), result.target_ranges.len());
        assert_eq!(
            result.source_ranges[1],
            CharRange {
                start: 1,
                length: 1
            }
        );
    }

    #[test]
    fn integrates_standardization_with_luat_nhan() {
        let mut dictionaries = Dictionaries::default();
        for (ch, reading) in [('年', "niên"), ('月', "nguyệt"), ('号', "hào")] {
            dictionaries.han_viet.insert(ch, reading.into());
        }
        dictionaries.luat_nhan.push((
            "{s}年{s}月{s}号".into(),
            "ngày {3} tháng {2} năm {1}".into(),
        ));
        let engine = Engine::from_dicts(dictionaries);
        assert_eq!(
            engine.translate(
                "2025年7月21号",
                Mode::VietPhraseOneMeaning,
                &Options::default()
            ),
            " ngày 21 tháng 7 năm 2025"
        );
    }

    #[test]
    fn request_names_replace_base_file_and_keep_vietphrase_fixed() {
        let dictionaries = Dictionaries::build(
            "萧=tiêu\n炎=viêm",
            "萧炎=Base Name",
            "",
            "萧炎=Fixed VietPhrase",
        );
        let engine = Engine::from_dicts(dictionaries);
        let custom = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            names: Some("萧炎=Custom Name/Alternative"),
            ..Default::default()
        });

        assert_eq!(
            engine
                .translate_with_overrides(
                    "萧炎",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom,
                )
                .unwrap(),
            " Custom Name"
        );
        assert_eq!(
            engine.translate("萧炎", Mode::VietPhraseOneMeaning, &Options::default()),
            " Base Name"
        );

        let custom_names2 = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            names: Some("萧炎=Custom Primary"),
            names2: Some("萧炎=Custom Secondary"),
            ..Default::default()
        });
        assert_eq!(
            engine
                .translate_with_overrides(
                    "萧炎",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom_names2,
                )
                .unwrap(),
            " Custom Secondary"
        );

        let empty_names = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            names: Some(""),
            ..Default::default()
        });
        assert_eq!(
            engine
                .translate_with_overrides(
                    "萧炎",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &empty_names,
                )
                .unwrap(),
            " Fixed VietPhrase"
        );
    }

    #[test]
    fn request_patches_layer_over_fixed_vietphrase_and_han_viet() {
        let dictionaries =
            Dictionaries::build("他=tha\n很=ngận\n好=hảo", "", "", "很好=base phrase");
        let engine = Engine::from_dicts(dictionaries);
        let patches = DictionaryPatches {
            vietphrase: HashMap::from([("很好".to_string(), "rất ổn/rất tốt".to_string())]),
            chinese_phien_am_words: HashMap::from([('他', "hắn".to_string())]),
        };
        let custom = DictionaryOverrides::default().with_patches(patches);

        assert_eq!(
            engine
                .translate_with_overrides(
                    "他很好",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom,
                )
                .unwrap(),
            " hắn rất ổn"
        );
        assert_eq!(
            engine.translate("他很好", Mode::VietPhraseOneMeaning, &Options::default()),
            " tha base phrase"
        );
    }

    #[test]
    fn request_vietphrase_patch_prevents_number_prescan() {
        let dictionaries =
            Dictionaries::build("一=nhất\n百=bách\n二=nhị\n十=thập\n三=tam", "", "", "");
        let engine = Engine::from_dicts(dictionaries);
        let custom = DictionaryOverrides::default().with_patches(DictionaryPatches {
            vietphrase: HashMap::from([(
                "一百二十三".to_string(),
                "một trăm hai mươi ba".to_string(),
            )]),
            ..DictionaryPatches::default()
        });

        assert_eq!(
            engine
                .translate_with_overrides(
                    "一百二十三",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom,
                )
                .unwrap(),
            " một trăm hai mươi ba"
        );
    }

    #[test]
    fn request_luat_nhan_uses_request_pronouns() {
        let dictionaries = Dictionaries::build("在=tại\n他=tha\n身=thân\n后=hậu", "", "", "");
        let engine = Engine::from_dicts(dictionaries);
        let custom = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            luat_nhan: Some("在{n}身后=sau lưng {n}"),
            pronouns: Some("他=hắn"),
            ..Default::default()
        });

        assert_eq!(
            engine
                .translate_with_overrides(
                    "在他身后",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom,
                )
                .unwrap(),
            " sau lưng hắn"
        );
    }

    #[test]
    fn request_ignored_phrases_replace_base_file() {
        let dictionaries = Dictionaries::build(
            "他=tha\n本=bản\n章=chương\n完=hoàn",
            "",
            "",
            "本章完=hết chương",
        );
        let engine = Engine::from_dicts(dictionaries);
        let custom = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            ignored_chinese_phrases: Some("本章完"),
            ..Default::default()
        });

        assert_eq!(
            engine
                .translate_with_overrides(
                    "他本章完",
                    Mode::VietPhraseOneMeaning,
                    &Options::default(),
                    &custom,
                )
                .unwrap(),
            " tha"
        );
    }

    #[test]
    fn invalid_request_luat_nhan_returns_an_error() {
        let engine = engine_hv_only();
        let custom = DictionaryOverrides::from_sources(DictionarySourceOverrides {
            luat_nhan: Some("([{n}=invalid"),
            ..Default::default()
        });

        assert!(engine
            .translate_with_overrides(
                "他",
                Mode::VietPhraseOneMeaning,
                &Options::default(),
                &custom,
            )
            .is_err());
    }
}
