//! qt-core: Quick Translator engine (Rust reimplementation).

mod dict;
mod han_viet;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_matches_spec() {
        let o = Options::default();
        assert_eq!(o.wrap_type, 0);
        assert_eq!(o.translation_algorithm, 1);
        assert!(o.prioritized_name);
        assert_eq!(o.scan_range, 5);
        assert_eq!(Mode::HanViet, Mode::HanViet);
    }
}
