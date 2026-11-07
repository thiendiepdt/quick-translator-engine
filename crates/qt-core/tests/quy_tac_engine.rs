use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, LiteEngine, Mode, Options,
    QuyTac,
};

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
    let wrapped_off = Options {
        wrap_type: 1,
        ..Options::default()
    };
    assert_eq!(
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &wrapped),
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &wrapped_off)
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

/// Từ điển có dấu câu và nghĩa rỗng như QT2025: 的, 了 rỗng (có Hán Việt nên
/// dấu cách chèn sau chúng), X rỗng và Y mở đầu bằng dấu phẩy (không có Hán Việt).
const HAN_VIET_PUNCT: &str =
    "桌=trác\n子=tử\n上=thượng\n的=đích\n了=liễu\n他=tha\n说=thuyết\n好=hảo";
const VIETPHRASE_PUNCT: &str = "桌子=cái bàn\n上=bên trên\n的=\n了=\nX=\nY=, ý\n说=nói\n好=tốt";

fn engine_punct() -> Engine {
    Engine::from_dicts(Dictionaries::build(HAN_VIET_PUNCT, "", "", VIETPHRASE_PUNCT))
        .with_quy_tac(quy_tac())
}

/// Cụm nghĩa rỗng ở cuối output rồi cụm bỏ dấu cách trước nó: dấu cách đó
/// thuộc cụm trước cả cụm liền trước. Trước đây range bị đảo và panic.
#[test]
fn empty_meanings_then_punctuation_do_not_break_the_splice() {
    let engine = engine_punct();
    let mode = Mode::VietPhraseOneMeaning;
    for text in ["的的XX", "的的Y", "了的X了Y"] {
        let off = engine.translate(text, mode, &Options::default());
        assert_eq!(engine.translate(text, mode, &on()), off, "{text}");
    }
    assert_eq!(engine.translate("的的Y", mode, &on()), " , ý");
    // Quy tắc chạy ngay sau / ngay trước các cụm đó.
    assert_eq!(engine.translate("了的Y桌子上", mode, &on()), " , ý trên cái bàn");
    assert_eq!(engine.translate("桌子上的了Y", mode, &on()), " Trên cái bàn, ý");
    assert_eq!(engine.translate("桌子上的的X", mode, &on()), " Trên cái bàn");
}

/// Đoạn không có quy tắc nào chạy giữ nguyên từng byte, kể cả dấu câu, ngoặc
/// kép, TAB và dòng trống; Quy Tắc tắt thì như engine không gắn Quy Tắc.
#[test]
fn untouched_text_with_punctuation_stays_byte_identical() {
    let engine = engine_punct();
    let bare = Engine::from_dicts(Dictionaries::build(HAN_VIET_PUNCT, "", "", VIETPHRASE_PUNCT));
    let lite = LiteEngine::build(HAN_VIET_PUNCT, "", "", VIETPHRASE_PUNCT, "", "", "", "", "")
        .unwrap()
        .with_quy_tac(quy_tac());
    let names = DictionaryOverrides::from_sources(DictionarySourceOverrides {
        names: Some("李四=Lý Tứ"),
        ..Default::default()
    });
    let mode = Mode::VietPhraseOneMeaning;
    let off = Options::default();
    for text in ["他说：“好，好。”他！\t好…… 'X' (Y)", "“他”说：好！\n\n他了。"] {
        let want = bare.translate(text, mode, &off);
        assert_eq!(engine.translate(text, mode, &off), want);
        assert_eq!(engine.translate_with_overrides(text, mode, &off, &names).unwrap(), want);
        assert_eq!(engine.translate(text, mode, &on()), want);
        assert_eq!(engine.translate_with_overrides(text, mode, &on(), &names).unwrap(), want);
        let empty = DictionaryOverrides::default();
        assert_eq!(lite.translate(text, &on(), &empty).unwrap(), want);
    }
    let text = "他说：“好，好。”他！\t好…… 'X' (Y)\n桌子上";
    let want = bare
        .translate(text, mode, &off)
        .replacen("Cái bàn bên trên", "Trên cái bàn", 1);
    assert_eq!(engine.translate(text, mode, &on()), want);
}
