//! Nghĩa từ điển ra dạng NFC dù file nguồn lưu dấu tách rời (QT2025 có hơn
//! 11 nghìn dòng VietPhrase như vậy).

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionaryPatches, DictionarySourceOverrides, Engine,
    LiteEngine, Mode, Options,
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

/// Patch gửi kèm request (`with_patches`, qua `nfc_string`) cũng ra NFC, cả
/// VietPhrase lẫn Hán Việt.
#[test]
fn request_patches_are_normalized_too() {
    let engine = Engine::from_dicts(Dictionaries::build("他=tha", "", "", "也=x"));
    let overrides = DictionaryOverrides::default().with_patches(DictionaryPatches {
        vietphrase: [("也".to_string(), NFD.to_string())].into_iter().collect(),
        chinese_phien_am_words: [('他', NFD.to_string())].into_iter().collect(),
    });
    let mode = Mode::VietPhraseOneMeaning;
    let options = Options::default();
    assert_eq!(
        engine.translate_with_overrides("也", mode, &options, &overrides).unwrap(),
        format!(" {NFC}")
    );
    assert_eq!(
        engine.translate_with_overrides("他", Mode::HanViet, &options, &overrides).unwrap(),
        format!(" {NFC}")
    );
}
