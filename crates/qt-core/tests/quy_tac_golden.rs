//! Bản Rust phải ra đúng như bản thử Python (docs/superpowers/specs/2026-11-06-quy-tac/)
//! trên các câu mẫu: QT2025 thật, bộ luật mặc định, không names riêng.
//! Nạp QT2025 nên chậm: chạy bằng
//! `cargo test --release -p qt-core --test quy_tac_golden -- --ignored`.

use std::path::PathBuf;

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options, QuyTac,
};
use unicode_normalization::UnicodeNormalization;

#[test]
#[ignore = "nạp QT2025 thật, chạy riêng ở bản release"]
fn matches_prototype_on_golden_sentences() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../QT2025");
    let engine = Engine::from_dicts(Dictionaries::load(&data).unwrap())
        .with_quy_tac(QuyTac::load(&data, false).unwrap());
    let mode = Mode::VietPhraseOneMeaning;
    let on = Options {
        quy_tac: true,
        ..Options::default()
    };
    let no_luat_nhan = DictionaryOverrides::from_sources(DictionarySourceOverrides {
        luat_nhan: Some(""),
        ..Default::default()
    });
    let mut failures = Vec::new();
    for line in include_str!("data/quy_tac_golden.tsv").lines() {
        let (source, expected) = line.split_once('\t').expect("dòng thiếu TAB");
        let got: String = engine.translate(source, mode, &on).nfc().collect();
        let want: String = if expected == "=" {
            engine
                .translate_with_overrides(source, mode, &Options::default(), &no_luat_nhan)
                .unwrap()
                .nfc()
                .collect()
        } else {
            expected.nfc().collect()
        };
        if got.trim_start() != want.trim_start() {
            failures.push(format!(
                "{source}\n  Rust : {}\n  mẫu  : {}",
                got.trim_start(),
                want.trim_start()
            ));
        }
    }
    assert!(failures.is_empty(), "{} câu lệch:\n{}", failures.len(), failures.join("\n"));
}
