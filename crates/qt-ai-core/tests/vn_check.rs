//! Kiểm tra nội dung nhạy cảm về Việt Nam — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
use qt_ai_core::vn_check::*;
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

#[test]
fn khop_chong_nhau_strong_thang_weak_phia_trung() {
    let lex = Lexicon::builtin();
    assert!(ids(&lex, &lex.find_zh("他去过云南越南边境")).contains(&"viet-nam".to_string()));
    let hu_nam = ids(&lex, &lex.find_zh("湖南越南"));
    assert!(hu_nam.contains(&"viet-nam".to_string()), "{hu_nam:?}");
    assert!(!hu_nam.contains(&"nam-viet".to_string()), "{hu_nam:?}");
}

#[test]
fn khop_chong_nhau_cum_ngan_hop_le_ben_trong_cum_dai_truot_nguyen_tu() {
    let lex = Lexicon::builtin();
    let found = ids(&lex, &lex.find_vi("người Việt Namxyz đến"));
    assert!(!found.contains(&"viet-nam".to_string()), "{found:?}");
    assert!(found.contains(&"nguoi-viet".to_string()), "{found:?}");
    // cụm dài nhất vẫn thắng khi cùng độ mạnh
    assert_eq!(ids(&lex, &lex.find_zh("越南猴子")), vec!["khi-viet"]);
    assert_eq!(ids(&lex, &lex.find_vi("người Việt Nam")), vec!["viet-nam"]);
}

fn found(lex: &Lexicon, raw: &str, out: Option<&str>) -> Vec<(usize, String, Side)> {
    scan_chapter(lex, raw, out).into_iter().map(|f| (f.paragraph, lex.entries[f.entry].id.clone(), f.side)).collect()
}

#[test]
fn quet_raw_khi_chua_dich_va_bo_cac_cum_trung_chu() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n他越战越勇。\n\n天南海北都有人。\n\n坐船转到越南。\n\n此地多是交趾人。";
    assert_eq!(
        found(&lex, raw, None),
        vec![(3, "viet-nam".into(), Side::Raw), (4, "giao-chi".into(), Side::Raw)]
    );
}

#[test]
fn cac_cum_trung_chu_trong_spec_khong_bao() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n他越战了很久越快。\n\n河内郡王到了。\n\n越王勾践卧薪尝胆，吴越之争。\n\n他头顿时一痛。\n\n莫西沙走来。\n\n京城很大。";
    assert!(found(&lex, raw, None).is_empty(), "{:?}", found(&lex, raw, None));
    // "người Kinh thành" dịch từ 京城 → đối chiếu raw không có 越人/京族 → bỏ.
    let out = "Chương 1\n\nHắn càng đánh lâu càng nhanh.\n\nHà Nội quận vương tới.\n\nViệt vương Câu Tiễn nằm gai nếm mật.\n\nĐầu hắn lập tức đau.\n\nMạc Tây Sa đi tới.\n\nNgười Kinh thành rất đông.";
    assert!(found(&lex, raw, Some(out)).is_empty(), "{:?}", found(&lex, raw, Some(out)));
}

#[test]
fn ban_dich_la_tin_hieu_chinh_va_phai_doi_chieu_raw() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n一路黄沙。\n\n西沙群岛自古以来。\n\n越战越勇。";
    let out = "Chương 1\n\nMột đường Hoàng Sa cuồn cuộn.\n\nQuần đảo Hoàng Sa từ xưa tới nay.\n\nCàng đánh càng hăng.";
    // Đoạn 1: "Hoàng Sa" dịch từ 黄沙 (cát vàng) → bỏ. Đoạn 2: raw có 西沙 → báo. Đoạn 3: bản dịch không có từ → bỏ dù raw có 越战.
    assert_eq!(found(&lex, raw, Some(out)), vec![(2, "hoang-sa".into(), Side::Out)]);
}

#[test]
fn tu_yeu_can_tu_di_kem_cung_doan() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n他去了南海。\n\n南海诸岛与西沙之争。";
    let out = "Chương 1\n\nHắn đi Nam Hải.\n\nTranh chấp quần đảo Nam Hải và Tây Sa.";
    // Đoạn 1 "Hắn đi Nam Hải" không có từ đi kèm → bỏ. Đoạn 2 có "quần đảo Nam Hải" (strong) + "Tây Sa".
    // Thứ tự theo (đoạn, chỉ số mục trong builtin_entries): hoang-sa (10) trước nam-hai-chu-dao (15).
    let ids: Vec<String> = found(&lex, raw, Some(out)).into_iter().map(|(p, id, _)| format!("{p}:{id}")).collect();
    assert_eq!(ids, vec!["2:hoang-sa", "2:nam-hai-chu-dao"]);
}

#[test]
fn so_doan_lech_thi_bo_doi_chieu_nhung_van_khop_nguyen_tu() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n坐船转到越南。";
    let out = "Chương 1\n\nĐoạn thêm.\n\nChuyển tuyến qua Việt Nam.";
    assert_eq!(found(&lex, raw, Some(out)), vec![(2, "viet-nam".into(), Side::Out)]);
}

#[test]
fn nhieu_cho_cung_muc_trong_doan_gop_mot_kem_so_lan() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n越南，越南，还是越南。";
    let out = "Chương 1\n\nViệt Nam, Việt Nam, vẫn là Việt Nam.";
    let list = scan_chapter(&lex, raw, Some(out));
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].count, list[0].matched.as_str()), (3, "Việt Nam"));
}

#[test]
fn quet_raw_van_thay_viet_nam_sau_tu_dia_danh_trung_chu_nam_viet() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n他从湖南越南边境回来。";
    assert!(found(&lex, raw, None).contains(&(1, "viet-nam".into(), Side::Raw)));
}
