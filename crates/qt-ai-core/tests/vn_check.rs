//! Kiểm tra nội dung nhạy cảm về Việt Nam — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
use qt_ai_core::vn_lexicon::*;

fn ids(lex: &Lexicon, hits: &[Hit]) -> Vec<String> {
    hits.iter().map(|h| lex.entries[h.entry].id.clone()).collect()
}

#[test]
fn lexicon_tim_chu_han_theo_cum_dai_nhat() {
    let lex = Lexicon::builtin();
    let hits = lex.find_zh("他骂道：越南猴子！还有越南。");
    assert_eq!(ids(&lex, &hits), vec!["khi-viet", "viet-nam"]);
}

#[test]
fn lexicon_tieng_viet_khop_nguyen_tu() {
    let lex = Lexicon::builtin();
    assert!(lex.find_vi("Hoàng Sam nữ tử bước tới.").is_empty(), "Hoàng Sam không phải Hoàng Sa");
    assert_eq!(ids(&lex, &lex.find_vi("quần đảo Hoàng Sa")), vec!["hoang-sa"]);
    assert_eq!(ids(&lex, &lex.find_vi("người Việt Nam, khỉ Việt Nam")), vec!["viet-nam", "khi-viet"]);
}

#[test]
fn lexicon_loai_tru_trong_cua_so_quanh_cho_khop() {
    let lex = Lexicon::builtin();
    let text = "他越战越勇，越大越好，天南海北。";
    let kept: Vec<String> = lex.find_zh(text).into_iter().filter(|h| !lex.excluded(*h, text)).map(|h| lex.entries[h.entry].id.clone()).collect();
    assert!(kept.is_empty(), "{kept:?}");
    let real = "大越国的使者到了。";
    assert!(lex.find_zh(real).iter().any(|h| lex.entries[h.entry].id == "dai-viet" && !lex.excluded(*h, real)));
}

#[test]
fn lexicon_cau_hinh_nguoi_dung_them_va_tat_muc() {
    let mut settings = VnSettings::default();
    settings.disabled.push("nam-hai".into());
    settings.extra.push(Entry {
        id: "rieng".into(),
        group: Group::Ten,
        level: Level::Canxem,
        weak: false,
        zh: vec!["猴子国".into()],
        vi: vec!["nước khỉ con".into()],
        exclude: vec![],
    });
    let lex = Lexicon::with_settings(&settings);
    assert!(lex.entries.iter().all(|e| e.id != "nam-hai"));
    assert_eq!(ids(&lex, &lex.find_zh("那个猴子国")), vec!["rieng"]);
    assert_ne!(lex.hash, Lexicon::builtin().hash, "đổi từ điển → hash đổi để quét lại");
}

#[test]
fn bang_thay_mac_dinh_co_chiem_thanh_va_thai_lan() {
    let defaults = default_replacements();
    assert!(defaults.iter().any(|r| r.from == "Giao Chỉ" && r.to == "Chiêm Thành" && r.setting == ReplaceSetting::Ancient));
    assert!(defaults.iter().any(|r| r.from == "Việt Nam" && r.to == "Thái Lan" && r.setting == ReplaceSetting::Modern));
    let roundtrip: VnSettings = serde_json::from_str(r#"{"disabled":["x"]}"#).unwrap();
    assert_eq!(roundtrip.replacements, defaults, "file thiếu bảng thay → dùng mặc định");
}

#[test]
fn luu_va_doc_cau_hinh_o_thu_muc_ban_mac_dinh() {
    let dir = tempfile::tempdir().unwrap();
    let base = qt_ai_core::base::BaseStore::at(dir.path());
    assert_eq!(load_settings(&base), VnSettings::default());
    let mut settings = VnSettings::default();
    settings.disabled.push("ha-noi".into());
    save_settings(&base, &settings).unwrap();
    assert_eq!(load_settings(&base), settings);
    assert!(dir.path().join("vn-terms.json").exists());
}
