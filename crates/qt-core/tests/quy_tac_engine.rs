use qt_core::{Dictionaries, DictionaryOverrides, Engine, LiteEngine, Mode, Options, QuyTac};

const RULES: &str = "[lớp]\nvị-trí: 上=trên\n\n[của 1]\n{danh-từ} {vị-trí} = {vị-trí} {danh-từ}\n";
const POS: &str = "桌子 10 n\n上 10 f\n";
const HAN_VIET: &str = "桌=trác\n子=tử\n上=thượng";
const VIETPHRASE: &str = "桌子=cái bàn\n上=bên trên";

fn quy_tac() -> QuyTac {
    QuyTac::from_sources(RULES, POS, "").unwrap()
}

fn on() -> Options {
    Options {
        quy_tac: true,
        ..Options::default()
    }
}

fn engine() -> Engine {
    Engine::from_dicts(Dictionaries::build(HAN_VIET, "", "", VIETPHRASE)).with_quy_tac(quy_tac())
}

#[test]
fn engine_reorders_only_when_enabled() {
    let engine = engine();
    let mode = Mode::VietPhraseOneMeaning;
    assert_eq!(engine.translate("桌子上", mode, &Options::default()), " cái bàn bên trên");
    assert_eq!(engine.translate("桌子上", mode, &on()), " Trên cái bàn");
    assert_eq!(
        engine
            .translate_with_overrides("桌子上", mode, &on(), &DictionaryOverrides::default())
            .unwrap(),
        " Trên cái bàn"
    );
}

#[test]
fn ranges_wrap_and_other_modes_are_untouched() {
    let engine = engine();
    let ranged = engine.translate_with_ranges("桌子上", Mode::VietPhraseOneMeaning, &on());
    assert_eq!(ranged.translated_text, " cái bàn bên trên");
    let wrapped = Options { wrap_type: 1, ..on() };
    assert_eq!(
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &wrapped),
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &Options { wrap_type: 1, ..Options::default() })
    );
    assert_eq!(
        engine.translate("桌子上", Mode::VietPhrase, &on()),
        engine.translate("桌子上", Mode::VietPhrase, &Options::default())
    );
}

#[test]
fn paragraphs_are_handled_separately() {
    let engine = engine();
    assert_eq!(
        engine.translate("桌子上\n上", Mode::VietPhraseOneMeaning, &on()),
        engine
            .translate("桌子上\n上", Mode::VietPhraseOneMeaning, &Options::default())
            .replacen("cái bàn bên trên", "Trên cái bàn", 1)
    );
}

#[test]
fn lite_matches_engine() {
    let lite = LiteEngine::build(HAN_VIET, "", "", VIETPHRASE, "", "", "", "", "")
        .unwrap()
        .with_quy_tac(quy_tac());
    assert_eq!(
        lite.translate("桌子上", &on(), &DictionaryOverrides::default()).unwrap(),
        " Trên cái bàn"
    );
}
