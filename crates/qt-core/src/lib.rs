//! qt-core: Quick Translator engine (Rust reimplementation).

mod dict;
mod han_viet;
mod luat_nhan;
mod number;
mod standardize;
mod text;
mod translate;

pub use dict::Dictionaries;

/// A half-open range represented as a UTF-16 start offset and length.
/// UTF-16 matches QT2025/.NET and can be consumed directly by JavaScript UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    pub start: usize,
    pub length: usize,
}

/// Translated text plus parallel source/target phrase ranges.
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
            ),
        }
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
}
