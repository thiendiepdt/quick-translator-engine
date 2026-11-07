//! Bản Rust phải ra đúng như bản thử Python (docs/superpowers/specs/2026-11-06-quy-tac/)
//! trên các câu mẫu: QT2025 thật, bộ luật mặc định, không names riêng. Trừ chỗ
//! khác cố ý (spec mục 5): chữ đầu của request không viết hoa (như engine,
//! ngoài cửa sổ ráp lại output giữ nguyên), nghĩa viết hoa của từ điển giữ
//! hoa khi bị dời, chữ sau `Chương N:` viết hoa, nhóm `[giọng-mượt]` bật như
//! mặc định (bản thử không có nhóm đó); đáp án đã sửa theo đó.
//! Nạp QT2025 nên chậm: chạy bằng
//! `cargo test --release -p qt-core --test quy_tac_golden -- --ignored`.

use std::path::PathBuf;
use std::sync::LazyLock;

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options, QuyTac,
};
use unicode_normalization::UnicodeNormalization;

/// Nạp một lần cho mọi test trong file.
static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../QT2025");
    Engine::from_dicts(Dictionaries::load(&data).unwrap())
        .with_quy_tac(QuyTac::load(&data, true).unwrap())
});

#[test]
#[ignore = "nạp QT2025 thật, chạy riêng ở bản release"]
fn matches_prototype_on_golden_sentences() {
    let engine = &*ENGINE;
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

/// QT2025 có 29 key nghĩa đầu rỗng (了 的 之 地…) và 172 key nghĩa mở đầu bằng dấu
/// câu, không có trong Hán Việt. Ghép chúng từng làm range cụm bị đảo, bước
/// ghép của Quy Tắc panic.
#[test]
#[ignore = "nạp QT2025 thật, chạy riêng ở bản release"]
fn empty_meanings_then_punctuation_do_not_panic() {
    let engine = &*ENGINE;
    let mode = Mode::VietPhraseOneMeaning;
    let on = Options {
        quy_tac: true,
        ..Options::default()
    };
    for text in ["了的，一般情况下", "的了……等等"] {
        let got = engine.translate(text, mode, &on);
        assert!(!got.trim().is_empty(), "{text}: {got:?}");
    }
}
