//! Native AWS Lambda entrypoint for Quick Translator Engine.

use std::sync::Arc;

use lambda_http::{run, tracing, Error};
use qt_api::{build_router, AppState, NameFilterServices};
use qt_core::{DictionaryDefaults, Engine};

// Lambda embeds the same QT2025 defaults used by the CLI and HTTP server.
// Request-scoped overrides are parsed separately and never mutate this state.
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
    let dictionary_defaults = dictionary_defaults();
    let dictionaries = dictionary_defaults.build_dictionaries(CHINESE_PHIEN_AM_WORDS, VIETPHRASE);
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(dictionaries)),
        dictionary_defaults: Arc::new(dictionary_defaults),
        name_filter_services: NameFilterServices::from_env(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(build_router(build_state())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_core::{Mode, Options};

    #[test]
    fn embeds_and_loads_all_default_dictionaries() {
        let state = build_state();
        assert_eq!(
            state
                .engine
                .translate("很好", Mode::VietPhraseOneMeaning, &Options::default()),
            " rất tốt"
        );
        assert_eq!(
            state
                .engine
                .translate("他", Mode::HanViet, &Options::default()),
            " tha"
        );
        assert_eq!(
            state
                .engine
                .translate("青阳宗", Mode::VietPhraseOneMeaning, &Options::default()),
            " Thanh Dương tông"
        );
        assert!(state.dictionary_defaults.names.len() > 2_000_000);
        assert!(state.dictionary_defaults.pronouns.contains("他=hắn"));
    }
}
