//! qt-core: Quick Translator engine (Rust reimplementation).

// Built incrementally: each module's helpers are consumed by a later task in the
// MVP plan, so intermediate builds have transient unused items. Removed by the
// final MVP cleanup once every module is wired (see docs/plans/2026-07-21-qt-core-mvp.md).
#![allow(dead_code)]

mod dict;
mod han_viet;
mod text;
mod translate;

pub use dict::Dictionaries;

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
    pub scan_range: usize,          // max phrase length scanned per position
}

impl Default for Options {
    fn default() -> Self {
        // Defaults live in QT's binary UI config; these are the working defaults
        // documented in docs/engine/overview.md, to be confirmed by golden tests.
        Options { wrap_type: 0, translation_algorithm: 1, prioritized_name: true, scan_range: 5 }
    }
}

pub struct Engine {
    dicts: Dictionaries,
}

impl Engine {
    pub fn from_dicts(dicts: Dictionaries) -> Engine {
        Engine { dicts }
    }

    pub fn translate(&self, text: &str, mode: Mode, opts: &Options) -> String {
        let chars: Vec<char> = text.chars().collect();
        match mode {
            Mode::HanViet => han_viet::chinese_to_han_viet_string(&chars, &self.dicts.han_viet),
            Mode::VietPhrase => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase, &self.dicts.only_name, &self.dicts.han_viet,
            ),
            Mode::VietPhraseOneMeaning => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase_one_meaning, &self.dicts.only_name, &self.dicts.han_viet,
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
        let d = Dictionaries { han_viet: hv_map(), ..Default::default() };
        Engine::from_dicts(d)
    }

    #[test]
    fn options_default_matches_spec() {
        let o = Options::default();
        assert_eq!(o.scan_range, 5);
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
}
