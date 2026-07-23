//! Native AWS Lambda entrypoint dedicated to name filtering.

use std::sync::Arc;

use lambda_http::{run, tracing, Error};
use qt_core::DictionaryDefaults;
use qt_ner_api::AppState;

const CHINESE_PHIEN_AM_WORDS: &str =
    include_str!("../../../QT2025/Resources/ChinesePhienAmWords.txt");
const VIETPHRASE: &str = include_str!("../../../QT2025/VietPhrase/VietPhrase.txt");
const NAMES: &str = include_str!("../../../QT2025/Names.txt");
const NAMES2: &str = include_str!("../../../QT2025/Names2/123.txt");
const LUAT_NHAN: &str = include_str!("../../../QT2025/LuatNhan.txt");
const PRONOUNS: &str = include_str!("../../../QT2025/Resources/Pronouns.txt");
const DANH_TU: &str = include_str!("../../../QT2025/Resources/DanhTu.txt");
const HO_NGUOI: &str = include_str!("../../../QT2025/Resources/HoNguoi.txt");
const HAU_TU: &str = include_str!("../../../QT2025/Resources/HauTu.txt");
const IGNORED_CHINESE_PHRASES: &str = include_str!("../../../QT2025/IgnoredChinesePhrases.txt");

fn dictionary_defaults() -> DictionaryDefaults {
    DictionaryDefaults {
        names: NAMES.to_string(),
        names2: NAMES2.to_string(),
        luat_nhan: LUAT_NHAN.to_string(),
        pronouns: PRONOUNS.to_string(),
        danh_tu: DANH_TU.to_string(),
        ho_nguoi: HO_NGUOI.to_string(),
        hau_tu: HAU_TU.to_string(),
        ignored_chinese_phrases: IGNORED_CHINESE_PHRASES.to_string(),
    }
}

fn build_state() -> Arc<AppState> {
    let defaults = dictionary_defaults();
    let dictionaries = defaults.build_dictionaries(CHINESE_PHIEN_AM_WORDS, VIETPHRASE);
    qt_ner_api::build_state(dictionaries, defaults)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(qt_ner_api::build_router(build_state())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_core::{NameFilterMemory, NameFilterOptions};

    #[test]
    fn embeds_defaults_for_name_filtering() {
        let state = build_state();
        let result = state.engine.filter_names(
            "来人名为萧炎。萧炎走来。",
            &NameFilterOptions {
                min_occurrences: 1,
                ..NameFilterOptions::default()
            },
            &NameFilterMemory::default(),
            None,
        );
        assert!(result
            .candidates
            .iter()
            .any(|candidate| candidate.text == "萧炎"));
        assert!(state.dictionary_defaults.names.len() > 2_000_000);
    }
}
