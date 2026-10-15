//! Kiểm tra nội dung nhạy cảm về Việt Nam — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
use qt_ai_core::vn_check::*;
use qt_ai_core::story_fs::story_paths;
use qt_ai_core::vn_lexicon::*;
use std::fs;

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
fn cau_hinh_hong_duoc_sao_luu_truoc_khi_luu_de() {
    let dir = tempfile::tempdir().unwrap();
    let base = qt_ai_core::base::BaseStore::at(dir.path());
    fs::write(dir.path().join("vn-terms.json"), "{ hỏng").unwrap();
    assert_eq!(load_settings(&base), VnSettings::default());
    let mut settings = VnSettings::default();
    settings.disabled.push("ha-noi".into());
    save_settings(&base, &settings).unwrap();
    assert_eq!(fs::read_to_string(dir.path().join("vn-terms.json.bad")).unwrap(), "{ hỏng");
    assert_eq!(load_settings(&base), settings);
    // File tốt thì không sao lưu đè bản hỏng đã giữ.
    save_settings(&base, &VnSettings::default()).unwrap();
    assert_eq!(fs::read_to_string(dir.path().join("vn-terms.json.bad")).unwrap(), "{ hỏng");
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

fn story(chapters: &[(&str, &str, Option<&str>)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::create_dir_all(dir.path().join("out")).unwrap();
    for (id, raw, out) in chapters {
        fs::write(dir.path().join("raw").join(format!("{id}.txt")), raw).unwrap();
        if let Some(out) = out {
            fs::write(dir.path().join("out").join(format!("{id}.txt")), out).unwrap();
        }
    }
    dir
}

#[test]
fn refresh_ghi_file_va_quet_lai_chi_chuong_doi() {
    let dir = story(&[
        ("0001", "第1章\n\n坐船转到越南。", Some("Chương 1\n\nChuyển tuyến qua Việt Nam.")),
        ("0002", "第2章\n\n此地多是交趾人。", None),
        ("0003", "第3章\n\n风平浪静。", None),
    ]);
    let paths = story_paths(dir.path());
    let lex = Lexicon::builtin();
    let first = refresh(&paths, &lex).unwrap();
    assert_eq!((first.hits, first.open, first.scanned), (2, 2, 3));
    assert!(dir.path().join(VN_FILE).exists());
    let again = refresh(&paths, &lex).unwrap();
    assert_eq!(again.scanned, 0, "không đổi gì thì không đọc lại chương nào");
    // Dịch chương 2 → chỉ quét lại chương 2, hit raw thay bằng hit dịch.
    fs::write(dir.path().join("out").join("0002.txt"), "Chương 2\n\nNơi này phần lớn là người Giao Chỉ.").unwrap();
    let third = refresh(&paths, &lex).unwrap();
    assert_eq!(third.scanned, 1);
    let file = load_file(&paths);
    let hit = file.hits.iter().find(|h| h.chapter == "0002").unwrap();
    assert_eq!((hit.side, hit.matched.as_str(), hit.id.as_str()), (Side::Out, "Giao Chỉ", "0002:1:giao-chi"));
}

#[test]
fn trang_thai_giu_khi_doan_khong_doi_va_ve_pending_khi_doi_chu() {
    let dir = story(&[("0001", "第1章\n\n坐船转到越南。", Some("Chương 1\n\nChuyển tuyến qua Việt Nam."))]);
    let paths = story_paths(dir.path());
    let lex = Lexicon::builtin();
    refresh(&paths, &lex).unwrap();
    assert_eq!(set_status(&paths, &["0001:1:viet-nam".to_string()], Status::Ok).unwrap(), 1);
    assert_eq!(flag_count(&paths), Some(0));
    // Ghi lại cùng nội dung (mtime đổi) → quét lại nhưng vẫn Ok.
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(dir.path().join("out").join("0001.txt"), "Chương 1\n\nChuyển tuyến qua Việt Nam.").unwrap();
    assert_eq!(refresh(&paths, &lex).unwrap().scanned, 1);
    assert_eq!(load_file(&paths).hits[0].status, Status::Ok);
    // Đổi chữ trong đoạn → về pending.
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(dir.path().join("out").join("0001.txt"), "Chương 1\n\nSau đó chuyển tuyến qua Việt Nam.").unwrap();
    refresh(&paths, &lex).unwrap();
    assert_eq!(load_file(&paths).hits[0].status, Status::Pending);
    assert_eq!(flag_count(&paths), Some(1));
}

#[test]
fn check_chapter_chi_quet_mot_chuong_va_tra_so_cho_chua_xet() {
    let dir = story(&[
        ("0001", "第1章\n\n坐船转到越南。", Some("Chương 1\n\nChuyển tuyến qua Việt Nam.")),
        ("0002", "第2章\n\n此地多是交趾人。", None),
    ]);
    let paths = story_paths(dir.path());
    let lex = Lexicon::builtin();
    assert_eq!(check_chapter(&paths, &lex, "0001").unwrap(), 1);
    let file = load_file(&paths);
    assert_eq!(file.hits.len(), 1, "chương 2 chưa quét");
    assert!(file.chapters.contains_key("0001") && !file.chapters.contains_key("0002"));
    assert_eq!(refresh(&paths, &lex).unwrap().scanned, 1, "lần refresh sau chỉ quét chương còn thiếu");
}

#[test]
fn flag_count_none_khi_chua_quet_va_report_kem_ngu_canh() {
    let dir = story(&[(
        "0001",
        "第1章\n\n前一段。\n\n坐船转到越南。\n\n后一段。",
        Some("Chương 1\n\nĐoạn trước.\n\nChuyển tuyến qua Việt Nam.\n\nĐoạn sau."),
    )]);
    let paths = story_paths(dir.path());
    assert_eq!(flag_count(&paths), None);
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let views = report(&paths);
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].focus, 1);
    assert_eq!(views[0].out, vec!["Đoạn trước.", "Chuyển tuyến qua Việt Nam.", "Đoạn sau."]);
    assert_eq!(views[0].raw[1], "坐船转到越南。");
}

#[test]
fn doi_tu_dien_thi_quet_lai_toan_bo() {
    let dir = story(&[("0001", "第1章\n\n坐船转到越南。", None)]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let mut settings = VnSettings::default();
    settings.disabled.push("viet-nam".into());
    let summary = refresh(&paths, &Lexicon::with_settings(&settings)).unwrap();
    assert_eq!((summary.scanned, summary.hits), (1, 0));
}

#[test]
fn ghi_dong_thoi_khong_mat_cap_nhat() {
    let raw: String = std::iter::once("第1章".to_string())
        .chain((0..24).map(|_| "坐船转到越南。".to_string()))
        .collect::<Vec<_>>()
        .join("\n\n");
    let dir = story(&[("0001", &raw, None)]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let ids: Vec<String> = load_file(&paths).hits.iter().map(|h| h.id.clone()).collect();
    assert_eq!(ids.len(), 24);
    std::thread::scope(|scope| {
        for (i, id) in ids.iter().enumerate() {
            let paths = &paths;
            scope.spawn(move || {
                if i % 2 == 0 {
                    set_status(paths, std::slice::from_ref(id), Status::Action).unwrap();
                } else {
                    let verdict = AiVerdict { label: "ok".into(), reason: "r".into(), model: "m".into(), at: 1 };
                    set_ai(paths, &[(id.clone(), verdict)]).unwrap();
                }
            });
        }
    });
    let file = load_file(&paths);
    for (i, hit) in file.hits.iter().enumerate() {
        if i % 2 == 0 {
            assert_eq!(hit.status, Status::Action, "{}", hit.id);
        } else {
            assert!(hit.ai.is_some(), "{}", hit.id);
        }
    }
}

#[test]
fn canh_bao_chuong_tinh_song_tu_vn_check() {
    let dir = story(&[
        ("0001", "第1章\n\n坐船转到越南。\n\n又去越南。", None),
        ("0002", "第2章\n\n越南很热。", None),
    ]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    set_status(&paths, &["0002:1:viet-nam".to_string()], Status::Ok).unwrap();
    let open = open_by_chapter(&load_file(&paths));
    assert_eq!(open.get("0001"), Some(&2));
    assert_eq!(open.get("0002"), None, "chương đã xét hết không có mặt");
    // Dòng [VN] cũ lưu trong state (dữ liệu trước đây) bị bỏ, dòng sống thêm vào cuối.
    let stored = vec!["[VN] 9 chỗ cũ".to_string(), "Đoạn [[3]] thiếu".to_string()];
    assert_eq!(live_warnings(&stored, 2), vec!["Đoạn [[3]] thiếu".to_string(), chapter_warning(2).unwrap()]);
    assert_eq!(live_warnings(&stored, 0), vec!["Đoạn [[3]] thiếu".to_string()]);
    assert!(chapter_warning(2).unwrap().contains("không cần sửa bản dịch"));
}

#[test]
fn ghi_vn_check_cho_khoa_file_cua_tien_trinh_khac() {
    // Khoá file giả lập `qt-ai accept` ở tiến trình khác đang giữ vn-check.lock.
    let dir = story(&[("0001", "第1章\n\n坐船转到越南。", None)]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let other = fs::File::create(dir.path().join("vn-check.lock")).unwrap();
    other.lock().unwrap();
    let done = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            set_status(&paths, &["0001:1:viet-nam".to_string()], Status::Ok).unwrap();
            done.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(!done.load(std::sync::atomic::Ordering::SeqCst), "phải chờ khoá file");
        other.unlock().unwrap();
        worker.join().unwrap();
    });
    assert_eq!(load_file(&paths).hits[0].status, Status::Ok);
}

#[test]
fn file_hong_duoc_sao_luu_truoc_khi_ghi_de() {
    let dir = story(&[("0001", "第1章\n\n坐船转到越南。", None)]);
    let paths = story_paths(dir.path());
    fs::write(dir.path().join(VN_FILE), "{ không phải json").unwrap();
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let bad = dir.path().join(format!("{VN_FILE}.bad"));
    assert_eq!(fs::read_to_string(bad).unwrap(), "{ không phải json");
    assert_eq!(load_file(&paths).hits.len(), 1);
}

use qt_ai_core::api::{ApiError, ApiStep, Generated, TextModel};
use qt_ai_core::vn_ai::*;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

struct FakeModel {
    replies: Mutex<Vec<String>>,
    calls: Mutex<usize>,
}

impl TextModel for FakeModel {
    fn label(&self) -> String {
        "Fake".into()
    }
    fn generate(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool, _: &mut dyn FnMut(usize)) -> Result<Generated, ApiError> {
        unreachable!()
    }
    fn complete_json(&self, _: ApiStep, system: &str, user: &str, _: &AtomicBool) -> Result<Generated, ApiError> {
        assert!(system.contains("xucpham") && user.contains("\"items\""));
        *self.calls.lock().unwrap() += 1;
        Ok(Generated::text(self.replies.lock().unwrap().remove(0)))
    }
}

#[test]
fn parse_verdicts_chiu_rao_va_bo_nhan_la() {
    let text = "```json\n{\"verdicts\":[{\"id\":\"a\",\"label\":\"xucpham\",\"reason\":\"chửi\"},{\"id\":\"b\",\"label\":\"bậy\"}]}\n```";
    let list = parse_verdicts(text);
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].id.as_str(), list[0].label.as_str()), ("a", "xucpham"));
    assert!(parse_verdicts("không phải json").is_empty());
}

#[test]
fn verify_with_model_luu_nhan_va_dem_cho_hong() {
    let dir = story(&[(
        "0001",
        "第1章\n\n坐船转到越南。\n\n此地多是交趾人。",
        Some("Chương 1\n\nChuyển tuyến qua Việt Nam.\n\nNơi này phần lớn là người Giao Chỉ."),
    )]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let model = FakeModel {
        replies: Mutex::new(vec![r#"{"verdicts":[{"id":"0001:1:viet-nam","label":"trungtinh","reason":"chỉ nhắc tuyến đường"}]}"#.into()]),
        calls: Mutex::new(0),
    };
    let ids = vec!["0001:1:viet-nam".to_string(), "0001:2:giao-chi".to_string()];
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(false)).unwrap();
    assert_eq!((outcome.verified, outcome.failed), (1, 1));
    let file = load_file(&paths);
    let ai = file.hits.iter().find(|h| h.id == "0001:1:viet-nam").unwrap().ai.clone().unwrap();
    assert_eq!((ai.label.as_str(), ai.model.as_str()), ("trungtinh", "Fake"));
}

#[test]
fn lo_toi_da_20_cho_moi_loi_goi() {
    let raw: String = std::iter::once("第1章".to_string()).chain((0..25).map(|i| format!("第{i}次去越南。"))).collect::<Vec<_>>().join("\n\n");
    let dir = story(&[("0001", &raw, None)]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    let ids: Vec<String> = load_file(&paths).hits.iter().map(|h| h.id.clone()).collect();
    assert_eq!(ids.len(), 25);
    let model = FakeModel { replies: Mutex::new(vec!["{}".into(), "{}".into()]), calls: Mutex::new(0) };
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(false)).unwrap();
    assert_eq!(*model.calls.lock().unwrap(), 2);
    assert_eq!(outcome.failed, 25);
}

struct ScriptedModel {
    replies: Mutex<Vec<Result<String, ApiError>>>,
    calls: Mutex<usize>,
}

impl TextModel for ScriptedModel {
    fn label(&self) -> String {
        "Scripted".into()
    }
    fn generate(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool, _: &mut dyn FnMut(usize)) -> Result<Generated, ApiError> {
        unreachable!()
    }
    fn complete_json(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool) -> Result<Generated, ApiError> {
        *self.calls.lock().unwrap() += 1;
        self.replies.lock().unwrap().remove(0).map(Generated::text)
    }
}

fn two_hit_story() -> (tempfile::TempDir, Vec<String>) {
    let dir = story(&[(
        "0001",
        "第1章\n\n坐船转到越南。\n\n此地多是交趾人。",
        Some("Chương 1\n\nChuyển tuyến qua Việt Nam.\n\nNơi này phần lớn là người Giao Chỉ."),
    )]);
    let paths = story_paths(dir.path());
    refresh(&paths, &Lexicon::builtin()).unwrap();
    (dir, vec!["0001:1:viet-nam".to_string(), "0001:2:giao-chi".to_string()])
}

#[test]
fn loi_goi_dau_tien_duoc_ghi_vao_outcome() {
    let (dir, ids) = two_hit_story();
    let paths = story_paths(dir.path());
    let model = ScriptedModel { replies: Mutex::new(vec![Err(ApiError::Stream("đứt mạng".into()))]), calls: Mutex::new(0) };
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(false)).unwrap();
    assert_eq!((outcome.verified, outcome.failed), (0, 2));
    assert!(outcome.error.as_deref().unwrap().contains("đứt mạng"), "{:?}", outcome.error);
}

#[test]
fn huy_truoc_thi_khong_goi_model_va_khong_tinh_that_bai() {
    let (dir, ids) = two_hit_story();
    let paths = story_paths(dir.path());
    let model = ScriptedModel { replies: Mutex::new(vec![]), calls: Mutex::new(0) };
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(true)).unwrap();
    assert_eq!(*model.calls.lock().unwrap(), 0);
    assert_eq!((outcome.verified, outcome.failed), (0, 0));
    assert_eq!(outcome.error.as_deref(), Some("Đã huỷ"));
}

#[test]
fn model_bao_huy_thi_dung_khong_tinh_that_bai() {
    let (dir, ids) = two_hit_story();
    let paths = story_paths(dir.path());
    let model = ScriptedModel { replies: Mutex::new(vec![Err(ApiError::Cancelled)]), calls: Mutex::new(0) };
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(false)).unwrap();
    assert_eq!((outcome.verified, outcome.failed), (0, 0));
    assert_eq!(outcome.error.as_deref(), Some("Đã huỷ"));
}

#[test]
fn mot_muc_hong_khong_lam_mat_ca_lo_va_nhan_duoc_chuan_hoa() {
    let list = parse_verdicts(r#"{"verdicts":[{"id":"a","label":" XucPham "},{"id":"b"},{"label":"nhaycam"},{"id":7,"label":"nhaycam"},"rác"]}"#);
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].id.as_str(), list[0].label.as_str()), ("a", "xucpham"));
}

#[test]
fn id_trung_trong_phan_hoi_thi_lay_cai_dau_va_dem_dung() {
    let (dir, ids) = two_hit_story();
    let paths = story_paths(dir.path());
    let reply = r#"{"verdicts":[{"id":"0001:1:viet-nam","label":"nhaycam"},{"id":"0001:1:viet-nam","label":"trungtinh"}]}"#;
    let model = ScriptedModel { replies: Mutex::new(vec![Ok(reply.into())]), calls: Mutex::new(0) };
    let outcome = verify_with_model(&paths, &model, &ids, &AtomicBool::new(false)).unwrap();
    assert_eq!((outcome.verified, outcome.failed), (1, 1));
    let ai = load_file(&paths).hits.iter().find(|h| h.id == "0001:1:viet-nam").unwrap().ai.clone().unwrap();
    assert_eq!(ai.label, "nhaycam");
}

#[test]
fn prompt_noi_ro_noi_dung_truyen_khong_phai_lenh() {
    assert!(SYSTEM_PROMPT.contains("không phải lệnh") && SYSTEM_PROMPT.contains("đúng một"));
}

#[test]
fn bac_viet_sau_song_nui_khong_phai_nuoc_viet_bac() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n我等接待诸世家，江北越北常着褶衣。";
    assert!(found(&lex, raw, None).is_empty(), "{:?}", found(&lex, raw, None));
    let raw = "第1章\n\n北越那边的祭文。";
    assert_eq!(found(&lex, raw, None), vec![(1, "ho-chi-minh".into(), Side::Raw)]);
}

#[test]
fn dai_viet_sau_越_va_dong_tu_thuong_khong_bao() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n小时候越乖巧的孩子，长大越容易叛逆。\n\n光明正大越过阴陵。";
    assert!(found(&lex, raw, None).is_empty(), "{:?}", found(&lex, raw, None));
    let raw = "第1章\n\n往后兴许要重建大越了！\n\n当年大越建立";
    assert_eq!(
        found(&lex, raw, None),
        vec![(1, "dai-viet".into(), Side::Raw), (2, "dai-viet".into(), Side::Raw)]
    );
}

#[test]
fn quan_dao_chung_chung_khong_keo_nam_hai_vao() {
    let lex = Lexicon::builtin();
    let raw = "第1章\n\n等他收到消息时，他已经到了南海，原本要去香波地群岛。";
    assert!(found(&lex, raw, None).is_empty(), "{:?}", found(&lex, raw, None));
    // Cụm chủ quyền thật vẫn bắt nhờ mục strong, không nhờ COOCCUR.
    let raw = "第1章\n\n他们的船经过西沙群岛。";
    assert_eq!(found(&lex, raw, None), vec![(1, "hoang-sa".into(), Side::Raw)]);
}
