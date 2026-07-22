//! Native AWS Lambda entrypoint for Quick Translator Engine.

use std::sync::Arc;

use lambda_http::{run, tracing, Error};
use qt_api::{build_router, AppState};
use qt_core::{Dictionaries, Engine};

// Lambda deliberately embeds only the two immutable dictionaries. Every
// other runtime dictionary is supplied through the request-scoped API
// contract, so a warm execution environment can safely share this engine.
const CHINESE_PHIEN_AM_WORDS: &str =
    include_str!("../../../QT2025/Resources/ChinesePhienAmWords.txt");
const VIETPHRASE: &str = include_str!("../../../QT2025/VietPhrase/VietPhrase.txt");

fn build_state() -> Arc<AppState> {
    let dictionaries = Dictionaries::build(CHINESE_PHIEN_AM_WORDS, "", "", VIETPHRASE);
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(dictionaries)),
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
    fn embeds_and_loads_the_fixed_dictionaries() {
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
    }
}
