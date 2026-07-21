use qt_core::{Dictionaries, Engine, Mode, Options};

#[test]
fn end_to_end_vietphrase_and_one_meaning() {
    // build() applies the real merge; multi-meaning value exercises both modes
    let d = Dictionaries::build(
        "他=tha\n很=ngận\n好=hảo", // han-viet
        "",                        // names
        "",                        // names2
        "很好=rất tốt/rất ổn",     // vietphrase
    );
    let e = Engine::from_dicts(d);
    let o = Options::default();
    // Faithful engine output: leading space, lowercase first word.
    assert_eq!(
        e.translate("他很好", Mode::VietPhrase, &o),
        " tha rất tốt/rất ổn"
    );
    assert_eq!(
        e.translate("他很好", Mode::VietPhraseOneMeaning, &o),
        " tha rất tốt"
    );
}
