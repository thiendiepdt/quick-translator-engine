//! Nghĩa từ điển ra dạng NFC dù file nguồn lưu dấu tách rời (QT2025 có hơn
//! 11 nghìn dòng VietPhrase như vậy).

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, LiteEngine, Mode,
    Options,
};

/// "cũng" với dấu ngã tách rời (u + U+0303).
const NFD: &str = "cu\u{0303}ng";
/// "cũng" dựng sẵn (U+0169).
const NFC: &str = "c\u{0169}ng";

#[test]
fn engine_outputs_nfc_meanings() {
    let vietphrase = format!("也={NFD}");
    let engine = Engine::from_dicts(Dictionaries::build("", "", "", &vietphrase));
    assert_eq!(
        engine.translate("也", Mode::VietPhraseOneMeaning, &Options::default()),
        format!(" {NFC}")
    );
}

#[test]
fn lite_outputs_nfc_meanings() {
    let vietphrase = format!("也={NFD}");
    let lite = LiteEngine::build("", "", "", &vietphrase, "", "", "", "", "").unwrap();
    assert_eq!(
        lite.translate("也", &Options::default(), &DictionaryOverrides::default())
            .unwrap(),
        format!(" {NFC}")
    );
}

#[test]
fn request_names_are_normalized_too() {
    let engine = Engine::from_dicts(Dictionaries::build("", "", "", "也=x"));
    let names = format!("也={NFD}");
    let overrides = DictionaryOverrides::from_sources(DictionarySourceOverrides {
        names: Some(&names),
        ..Default::default()
    });
    assert_eq!(
        engine
            .translate_with_overrides("也", Mode::VietPhraseOneMeaning, &Options::default(), &overrides)
            .unwrap(),
        format!(" {NFC}")
    );
}
