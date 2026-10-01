//! Bảng nhân vật (cast.json): giới tính, cặp xưng hô theo mốc chương, mục prompt `# Xưng hô`, check.
use qt_ai_core::cast::*;
use qt_ai_core::story::StringMap;
use qt_ai_core::story_fs::story_paths;
use std::fs;

fn map(pairs: &[(&str, &str)]) -> StringMap {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn cast_with(genders: &[(&str, Gender)]) -> Cast {
    let mut cast = Cast::default();
    for (name, gender) in genders {
        cast.characters.insert(name.to_string(), Character { gender: Some(*gender), ..Character::default() });
    }
    cast
}

fn change(from: &str, target: &str) -> AddressChange {
    AddressChange { from: from.into(), target: target.into(), note: "thành người yêu".into(), source: EntrySource::Auto }
}

#[test]
fn cast_thieu_file_hoac_hong_thi_rong_va_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let paths = story_paths(dir.path());
    assert_eq!(load_cast(&paths), Cast::default());
    fs::write(&paths.cast_json, "không phải json").unwrap();
    assert_eq!(load_cast(&paths), Cast::default());

    let mut cast = cast_with(&[("贺静昭", Gender::Female)]);
    cast.addressing.insert(
        "许浪→曹雅旋".into(),
        AddressTimeline { pinned: true, changes: vec![change("0005", "em–chị")] },
    );
    save_cast(&paths, &cast).unwrap();
    assert_eq!(load_cast(&paths), cast);
    let text = fs::read_to_string(&paths.cast_json).unwrap();
    assert!(text.contains("\"gender\": \"female\"") && text.contains("\"from\": \"0005\""), "{text}");
}

#[test]
fn cast_bo_muc_sai_schema_giu_muc_dung() {
    let dir = tempfile::tempdir().unwrap();
    let paths = story_paths(dir.path());
    fs::write(
        &paths.cast_json,
        r#"{"version":1,"characters":{"贺静昭":{"gender":"female"},"莫衡":{"gender":"robot"},"x":5},
            "addressing":{"a→b":{"changes":[{"from":"1","target":"anh–em"},{"from":2}]},"hỏng":"x"}}"#,
    )
    .unwrap();
    let cast = load_cast(&paths);
    assert_eq!(cast.gender_of("贺静昭"), Some(Gender::Female));
    assert_eq!(cast.gender_of("莫衡"), None);
    assert_eq!(cast.addressing["a→b"].changes.len(), 1);
    assert!(!cast.addressing.contains_key("hỏng"));
}

#[test]
fn term_gender_nhan_tu_mang_gioi_o_dau_hoac_cuoi() {
    for term in ["thầy Hạ", "anh", "ông Lý", "Tống công tử", "sư huynh", "chàng", "chú Diệp", "đại ca"] {
        assert_eq!(term_gender(term), Some(Gender::Male), "{term}");
    }
    for term in ["cô Hạ", "chị", "bà chủ", "Ninh sư tỷ", "tiểu thư", "nàng", "dì", "Chung phu nhân", "chị đại"] {
        assert_eq!(term_gender(term), Some(Gender::Female), "{term}");
    }
    for term in ["tôi", "cậu", "em", "đệ tử", "ngươi", "mày", "bác", "sếp Tống", "Thời An", "con", "cháu"] {
        assert_eq!(term_gender(term), None, "{term}");
    }
}

#[test]
fn normalize_pair_dao_cap_viet_nguoc() {
    assert_eq!(normalize_pair("tao–mày"), Some(("tao".into(), "mày".into())));
    assert_eq!(normalize_pair("mày–tao"), Some(("tao".into(), "mày".into())));
    assert_eq!(normalize_pair("cậu–tớ"), Some(("tớ".into(), "cậu".into())));
    assert_eq!(normalize_pair("tôi–cậu"), Some(("tôi".into(), "cậu".into())));
    // Cả hai đều có thể là ngôi một (tôi–tôi hiếm) hoặc không rõ → giữ nguyên.
    assert_eq!(normalize_pair("em–chị"), Some(("em".into(), "chị".into())));
    assert_eq!(normalize_pair("không có gạch"), None);
}

#[test]
fn pair_gender_conflict_theo_bang_nhan_vat() {
    let cast = cast_with(&[("贺静昭", Gender::Female), ("莫衡", Gender::Male)]);
    assert!(pair_gender_conflict("宋时安→贺静昭", "tôi–thầy Hạ", &cast));
    assert!(!pair_gender_conflict("莫衡→贺静昭", "tôi–cô", &cast));
    // Vế tự xưng của 甲 cũng phải hợp giới 甲.
    assert!(pair_gender_conflict("贺静昭→莫衡", "anh–em", &cast));
    assert!(!pair_gender_conflict("贺静昭→莫衡", "chị–em", &cast));
    // Chưa rõ giới → không coi là mâu thuẫn.
    assert!(!pair_gender_conflict("宋时安→何允恬", "em–thầy Hà", &cast));
}

#[test]
fn effective_addressing_lay_moc_dung_truoc_chuong() {
    let base = map(&[("许浪→苏雨", "tôi–cô"), ("周涛→许浪", "mày–tao")]);
    let mut cast = Cast::default();
    cast.addressing.insert(
        "许浪→苏雨".into(),
        AddressTimeline { pinned: false, changes: vec![change("0010", "anh–em"), change("0200", "chồng–vợ")] },
    );
    // Dịch (lại) chương trước mốc: cặp cũ. Từ chính chương có mốc trở đi: cặp mới — đổi cặp xét theo trạng thái
    // quan hệ nên chương khai đổi đã dùng cặp mới cho cả chương.
    assert_eq!(effective_addressing(&base, &cast, "0003")["许浪→苏雨"], "tôi–cô");
    assert_eq!(effective_addressing(&base, &cast, "0010")["许浪→苏雨"], "anh–em");
    assert_eq!(effective_addressing(&base, &cast, "0011")["许浪→苏雨"], "anh–em");
    assert_eq!(effective_addressing(&base, &cast, "0999")["许浪→苏雨"], "chồng–vợ");
    // Cặp viết ngược trong story.json được đảo khi đưa ra.
    assert_eq!(effective_addressing(&base, &cast, "0003")["周涛→许浪"], "tao–mày");
}

#[test]
fn effective_addressing_nhan_cap_chi_co_trong_cast() {
    let mut cast = Cast::default();
    cast.addressing.insert("甲甲→乙乙".into(), AddressTimeline { pinned: true, changes: vec![change("0001", "em–anh")] });
    let got = effective_addressing(&StringMap::new(), &cast, "0005");
    assert_eq!(got["甲甲→乙乙"], "em–anh");
    assert!(effective_addressing(&StringMap::new(), &cast, "0000").is_empty());
}

#[test]
fn chapter_addressing_chi_giu_cap_ca_hai_ben_co_mat() {
    let pairs = map(&[("许浪→曹雅旋", "tôi–chị"), ("许浪→童绮", "tôi–cô"), ("曹雅旋→许浪", "tôi–cậu")]);
    let got = chapter_addressing(&pairs, "许浪看着雅旋，没说话。");
    // 曹雅旋 có mặt ở dạng bỏ họ; 童绮 vắng.
    assert_eq!(got.keys().collect::<Vec<_>>(), vec!["许浪→曹雅旋", "曹雅旋→许浪"]);
}

// ---- Mục prompt `# Xưng hô` ----

use qt_ai_core::prompt::{build_chapter_prompt, build_system_prompt, prompt_suffix, TranslationGlossary};
use qt_ai_core::story::StoryConfig;
use serde_json::json;

fn story(setting: &str, names: &[(&str, &str)], addressing: &[(&str, &str)]) -> StoryConfig {
    StoryConfig::normalize(&json!({
        "name": "Truyện thử",
        "genre": { "setting": setting, "names": "han" },
        "glossary": {
            "names": names.iter().cloned().collect::<std::collections::BTreeMap<_, _>>(),
            "addressing": addressing.iter().cloned().collect::<std::collections::BTreeMap<_, _>>(),
        },
    }))
}

#[test]
fn present_characters_theo_ten_ten_bo_ho_va_ho_kem_chuc_danh() {
    let names = map(&[("贺静昭", "Hạ Tĩnh Chiêu"), ("莫衡", "Mạc Hành"), ("宋时安", "Tống Thời An"), ("叶清禾", "Diệp Thanh Hòa")]);
    let cast = cast_with(&[("贺静昭", Gender::Female), ("莫衡", Gender::Male), ("宋时安", Gender::Male)]);
    let got = present_characters(&names, &cast, "时安笑道：“贺老师很擅长包饺子。”叶清禾点头。");
    let listed: Vec<(&str, &str, Gender)> = got.iter().map(|c| (c.source.as_str(), c.name.as_str(), c.gender)).collect();
    // 莫衡 vắng; 叶清禾 có mặt nhưng chưa rõ giới → không liệt kê.
    assert_eq!(listed, vec![("贺静昭", "Hạ Tĩnh Chiêu", Gender::Female), ("宋时安", "Tống Thời An", Gender::Male)]);
}

#[test]
fn chapter_prompt_khong_cast_van_co_luat_chung_va_giu_nguyen_phan_con_lai() {
    let story = story("modern", &[("许浪", "Hứa Lãng")], &[]);
    let source = "许浪醒了。";
    let plain = build_system_prompt(&TranslationGlossary::new(), Some(&story), Some(source));
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0001", source);
    let head = plain.strip_suffix(prompt_suffix()).unwrap();
    assert!(got.starts_with(head) && got.ends_with(prompt_suffix()), "mục mới chèn ngay trước suffix");
    let section = &got[head.len()..got.len() - prompt_suffix().len()];
    assert!(section.starts_with("\n# Xưng hô\n"), "{section}");
    assert!(section.contains("Danh xưng làm đại từ") && section.contains("Em đừng dọa chị được không"));
    assert!(section.contains("Độc thoại nội tâm") && section.contains("`mình`"));
    assert!(section.contains("老师 ngoài trường học"));
    assert!(!section.contains("Nhân vật có mặt") && !section.contains("Cặp xưng hô đang dùng"));
}

#[test]
fn chapter_prompt_co_dai_khong_co_luat_lao_su_va_doc_thoai_la_ta() {
    let story = story("ancient", &[], &[]);
    let prompt = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0001", "他醒了。");
    let got = &prompt[prompt.find("\n# Xưng hô\n").unwrap()..];
    assert!(got.contains("Độc thoại nội tâm") && got.contains("`ta`") && !got.contains("`mình`"));
    assert!(!got.contains("老师 ngoài trường học") && !got.contains("Em đừng dọa chị"));
    assert!(got.contains("Sư đệ đừng dọa sư tỷ"));
}

#[test]
fn chapter_prompt_liet_ke_nhan_vat_va_cap_hieu_luc_bo_ghi_chu_khoa_cung() {
    let story = story(
        "modern",
        &[("贺静昭", "Hạ Tĩnh Chiêu"), ("宋时安", "Tống Thời An"), ("莫衡", "Mạc Hành")],
        &[("宋时安→贺静昭", "tôi–cô Hạ"), ("宋时安→莫衡", "em–thầy Mạc"), ("周涛→宋时安", "mày–tao")],
    );
    let mut cast = cast_with(&[("贺静昭", Gender::Female), ("宋时安", Gender::Male)]);
    cast.addressing.insert("宋时安→贺静昭".into(), AddressTimeline { pinned: false, changes: vec![change("0100", "em–chị")] });
    let source = "宋时安挽起袖子：“贺老师很擅长包饺子。”贺静昭没想到他知道。";
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &cast, "0132", source);

    assert!(got.contains("Nhân vật có mặt trong chương"));
    assert!(got.contains("- 贺静昭 (Hạ Tĩnh Chiêu): nữ") && got.contains("- 宋时安 (Tống Thời An): nam"));
    assert!(!got.contains("莫衡"), "nhân vật vắng không vào prompt: cả tên lẫn cặp xưng hô");
    // Cặp hiệu lực tại chương 0132 là mốc 0100, không phải giá trị gốc.
    assert!(got.contains("\"宋时安→贺静昭\": \"em–chị\"") && !got.contains("tôi–cô Hạ"), "{got}");
    assert!(!got.contains("周涛"), "cặp một bên vắng không gửi");
    assert!(!got.contains("giữ y hệt") && got.contains("Giữ đúng các cặp này"));
    assert!(!got.contains("Cặp cần xét lại"), "chương không có dấu hiệu thân mật → không có khối xét lại");
    // Nhóm addressing không còn nằm trong JSON từ điển (đã chuyển sang mục Xưng hô).
    assert!(!got.contains("\"addressing\""));
    assert!(got.contains("\"贺静昭\": \"Hạ Tĩnh Chiêu\""));
}

// ---- Check: sai giới + lời gọi thô ----

fn paragraphs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn translated(items: &[&str]) -> Vec<Option<String>> {
    items.iter().map(|s| Some(s.to_string())).collect()
}

#[test]
fn gender_violations_bat_tu_trai_gioi_kem_ho() {
    let names = map(&[("贺静昭", "Hạ Tĩnh Chiêu"), ("莫衡", "Mạc Hành")]);
    let cast = cast_with(&[("贺静昭", Gender::Female), ("莫衡", Gender::Male)]);
    let raw = paragraphs(&["贺静昭帮忙洗菜。", "“如果我没记错的话，贺老师很擅长包饺子。”", "“莫老师说的没错。”"]);
    let out = translated(&[
        "Hạ Tĩnh Chiêu giúp rửa rau.",
        "“Nếu tôi nhớ không nhầm thì thầy Hạ rất khéo gói sủi cảo.”",
        "“Thầy Mạc nói không sai.”",
    ]);
    let got = gender_violations(&raw, &out, &names, &cast);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].paragraph, 1);
    assert_eq!(got[0].message, "Sai giới: 贺老师 là 贺静昭 (nữ) → `cô Hạ`, bản dịch viết `thầy Hạ`");
}

#[test]
fn gender_violations_bo_qua_khi_ho_ung_nhieu_nguoi_hoac_chua_ro_gioi() {
    let names = map(&[("贺静昭", "Hạ Tĩnh Chiêu"), ("贺明楼", "Hạ Minh Lâu"), ("叶清禾", "Diệp Thanh Hòa")]);
    let cast = cast_with(&[("贺静昭", Gender::Female), ("贺明楼", Gender::Male)]);
    // Hai người họ 贺 cùng có mặt trong chương → không biết 贺老师 là ai.
    let raw = paragraphs(&["贺静昭和贺明楼都来了。", "“贺老师好。”", "“叶老师好。”"]);
    let out = translated(&["Hạ Tĩnh Chiêu và Hạ Minh Lâu đều tới.", "“Chào thầy Hạ.”", "“Chào thầy Diệp.”"]);
    assert!(gender_violations(&raw, &out, &names, &cast).is_empty());
    // Chỉ một người họ 贺 có mặt trong chương → xét được dù glossary có hai người họ 贺.
    let raw = paragraphs(&["贺静昭来了。", "“贺老师好。”"]);
    let out = translated(&["Hạ Tĩnh Chiêu tới.", "“Chào Thầy Hạ.”"]);
    assert_eq!(gender_violations(&raw, &out, &names, &cast).len(), 1);
}

#[test]
fn gender_violations_khong_khop_nua_chu_va_nhan_ca_chieu_nam() {
    let names = map(&[("何泽宇", "Hà Trạch Vũ")]);
    let cast = cast_with(&[("何泽宇", Gender::Male)]);
    let raw = paragraphs(&["“何老师，你还得练。”", "“何老师在吗？”"]);
    // "cô Hàn" không phải "cô Hà"; đoạn 2 mới sai thật.
    let out = translated(&["“Thầy Hà, cô Hàn bảo cậu còn phải luyện.”", "“Cô Hà có đây không?”"]);
    let got = gender_violations(&raw, &out, &names, &cast);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].message, "Sai giới: 何老师 là 何泽宇 (nam) → `thầy Hà`, bản dịch viết `Cô Hà`");
}

#[test]
fn vocative_violations_bat_dan_em_dan_chi_lam_loi_goi_trong_thoai() {
    let raw = paragraphs(&[
        "“学弟，你不要吓学姐好不好！”曹雅旋强作镇定。",
        "许浪看着学姐，没说话。",
        "“不许说话，学姐，我相信你是个聪明人。”",
        "“好。”",
    ]);
    let out = translated(&[
        "“Đàn em, cậu đừng dọa đàn chị có được không!” Tào Nhã Tuyền cố giữ bình tĩnh.",
        "Hứa Lãng nhìn đàn chị, không nói gì.",
        "“Không được lên tiếng, đàn chị, tôi tin chị là người thông minh.”",
        "“Được.”",
    ]);
    let got = vocative_violations(&raw, &out);
    assert_eq!(got.iter().map(|issue| issue.paragraph).collect::<Vec<_>>(), vec![0, 2]);
    assert_eq!(got[0].message, "Danh xưng thô trong thoại: `Đàn em` → dùng em/chị/anh làm đại từ, bỏ `đàn em/đàn chị`");
}

// ---- Quét bù giới tính ----

use qt_ai_core::api::{ApiError, ApiStep, Generated, TextModel};
use qt_ai_core::cast_scan::{collect_snippets, scan_genders, ScanCandidate, SCAN_BATCH};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

struct ScanModel {
    replies: Mutex<Vec<Result<String, ApiError>>>,
    prompts: Mutex<Vec<String>>,
}

impl ScanModel {
    fn new(replies: Vec<Result<String, ApiError>>) -> Self {
        ScanModel { replies: Mutex::new(replies.into_iter().rev().collect()), prompts: Mutex::new(vec![]) }
    }
}

impl TextModel for ScanModel {
    fn label(&self) -> String {
        "scan".into()
    }
    fn generate(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool, _: &mut dyn FnMut(usize)) -> Result<Generated, ApiError> {
        unreachable!("quét giới chỉ dùng complete_json")
    }
    fn complete_json(&self, step: ApiStep, _: &str, user: &str, _: &AtomicBool) -> Result<Generated, ApiError> {
        assert_eq!(step, ApiStep::Fill);
        self.prompts.lock().unwrap().push(user.to_string());
        let text = self.replies.lock().unwrap().pop().unwrap_or_else(|| Ok("{\"genders\":{}}".into()))?;
        Ok(Generated { text, usage: None })
    }
}

fn scan_story() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::write(dir.path().join("raw/0001.txt"), "贺静昭没想到这个年轻人连这个也知道，她是北方人。\n\n莫衡指着自己的白头发说：“你们管这叫男生？”").unwrap();
    fs::write(dir.path().join("raw/0002.txt"), "贺静昭帮忙洗菜。\n\n叶清禾走到台下。").unwrap();
    dir
}

#[test]
fn collect_snippets_gom_cau_chua_ten_uu_tien_cau_co_dau_hieu_gioi() {
    let dir = scan_story();
    let names = map(&[("贺静昭", "Hạ Tĩnh Chiêu"), ("莫衡", "Mạc Hành"), ("不存在", "Không Có"), ("叶清禾", "Diệp Thanh Hòa")]);
    let cast = cast_with(&[("莫衡", Gender::Male)]);
    let got = collect_snippets(&story_paths(dir.path()), &names, &cast).unwrap();
    // 莫衡 đã rõ giới, 不存在 không có trong raw → không hỏi.
    assert_eq!(got.iter().map(|c| c.source.as_str()).collect::<Vec<_>>(), vec!["贺静昭", "叶清禾"]);
    assert_eq!(got[0].name, "Hạ Tĩnh Chiêu");
    assert_eq!(got[0].snippets.len(), 2, "mỗi chương một câu trích");
    assert!(got[0].snippets[0].contains("她是北方人"), "{:?}", got[0].snippets);
}

#[test]
fn scan_genders_chi_dien_cho_trong_co_can_cu_va_bo_lo_json_hong() {
    let candidates: Vec<ScanCandidate> = (0..SCAN_BATCH + 2)
        .map(|i| ScanCandidate { source: format!("人{i}"), name: format!("Người {i}"), snippets: vec![format!("人{i}笑了，她点头。")] })
        .collect();
    let mut cast = Cast::default();
    let model = ScanModel::new(vec![
        Ok("```json\n{\"genders\":{\"人0\":\"nữ\",\"人1\":\"nam\",\"人2\":\"\",\"không hỏi\":\"nữ\"}}\n```".into()),
        Ok("không phải json".into()),
    ]);
    let outcome = scan_genders(&candidates, &mut cast, &model, &AtomicBool::new(false)).unwrap();
    assert_eq!(model.prompts.lock().unwrap().len(), 2, "chia lô {SCAN_BATCH} tên");
    assert!(model.prompts.lock().unwrap()[0].contains("### 人0 (Người 0)\n- 人0笑了，她点头。"));
    assert_eq!((outcome.asked, outcome.filled, outcome.failed_batches), (SCAN_BATCH + 2, 1, 1));
    assert_eq!(cast.gender_of("人0"), Some(Gender::Female));
    assert_eq!(cast.gender_of("人1"), None, "câu trích chỉ có 她, không có dấu hiệu nam → không nhận");
    assert!(!cast.characters.contains_key("không hỏi"));
}

#[test]
fn scan_genders_loi_api_thi_tra_loi_khong_nuot() {
    let candidates = vec![ScanCandidate { source: "人".into(), name: "Người".into(), snippets: vec!["人，她。".into()] }];
    let model = ScanModel::new(vec![Err(ApiError::Blocked("x".into()))]);
    assert!(scan_genders(&candidates, &mut Cast::default(), &model, &AtomicBool::new(false)).is_err());
}

// ---- Bề mặt cho GUI ----

#[test]
fn cast_normalize_tu_json_cua_gui_bo_muc_sai() {
    let cast = Cast::normalize(&json!({
        "characters": { "贺静昭": { "gender": "female", "source": "user" }, "x": { "gender": "?" } },
        "addressing": { "a→b": { "pinned": true, "changes": [{ "from": "0003", "target": "em–anh", "source": "user" }] } }
    }));
    assert_eq!(cast.characters["贺静昭"].source, EntrySource::User);
    assert_eq!(cast.characters.len(), 1);
    assert!(cast.addressing["a→b"].pinned);
    assert_eq!(cast.addressing["a→b"].changes[0].source, EntrySource::User);
}

#[test]
fn conflicting_addressing_liet_ke_cap_cu_trai_gioi() {
    let cast = cast_with(&[("贺静昭", Gender::Female), ("莫衡", Gender::Male)]);
    let pairs = map(&[("宋时安→贺静昭", "tôi–thầy Hạ"), ("莫衡→贺静昭", "tôi–cô"), ("宋时安→莫衡", "em–thầy Mạc")]);
    assert_eq!(conflicting_addressing(&pairs, &cast), vec!["宋时安→贺静昭"]);
}

#[test]
fn pin_edited_pairs_ghim_cap_nguoi_dung_sua_hoac_them_va_go_cap_bi_xoa() {
    let old = map(&[("a→b", "tôi–cô"), ("b→a", "tôi–anh"), ("c→d", "tôi–cậu")]);
    let new = map(&[("a→b", "anh–em"), ("b→a", "tôi–anh"), ("e→f", "chị–em")]);
    let mut cast = Cast::default();
    cast.addressing.insert("a→b".into(), AddressTimeline { pinned: false, changes: vec![change("0010", "anh–em")] });
    cast.addressing.insert("b→a".into(), AddressTimeline { pinned: false, changes: vec![change("0010", "em–anh")] });
    cast.addressing.insert("c→d".into(), AddressTimeline { pinned: false, changes: vec![change("0010", "x–y")] });
    assert!(pin_edited_pairs(&old, &new, &mut cast));
    // Sửa tay → ghim, giá trị người dùng áp cho mọi chương nên mốc cũ bị xoá.
    assert_eq!(cast.addressing["a→b"], AddressTimeline { pinned: true, changes: vec![] });
    // Không đụng → giữ mốc tự động.
    assert_eq!(cast.addressing["b→a"].changes.len(), 1);
    assert!(!cast.addressing["b→a"].pinned);
    assert!(!cast.addressing.contains_key("c→d"), "cặp bị xoá khỏi glossary thì gỡ luôn mốc");
    assert!(cast.addressing["e→f"].pinned, "cặp người dùng tự thêm cũng ghim");
    // Lưu lại y nguyên → không đổi gì.
    let mut same = cast.clone();
    assert!(!pin_edited_pairs(&new, &new, &mut same));
    assert_eq!(same, cast);
}

#[test]
fn scan_genders_loi_giua_chung_van_giu_cac_lo_da_xong() {
    let candidates: Vec<ScanCandidate> = (0..SCAN_BATCH + 1)
        .map(|i| ScanCandidate { source: format!("人{i}"), name: format!("Người {i}"), snippets: vec![format!("人{i}，她。")] })
        .collect();
    let mut cast = Cast::default();
    let model = ScanModel::new(vec![Ok("{\"genders\":{\"人0\":\"nữ\"}}".into()), Err(ApiError::Blocked("x".into()))]);
    assert!(scan_genders(&candidates, &mut cast, &model, &AtomicBool::new(false)).is_err());
    assert_eq!(cast.gender_of("人0"), Some(Gender::Female), "lô 1 đã điền vào bảng trước khi lô 2 lỗi — caller ghi lại được");
}

// ---- Cặp cần xét lại: harness phát hiện, không trông vào model tự nhận ra ----

#[test]
fn pairs_to_review_chi_lay_cap_kieu_nguoi_la_khi_chuong_co_dau_hieu_than_mat() {
    let pairs = map(&[
        ("秦寻→夏宁", "tôi–cô"),
        ("夏宁→秦寻", "tôi–anh"),
        ("夏宁→黄怀", "chị–em"),
        ("黄怀→夏宁", "em–chị Hạ Ninh"),
        ("秦寻→安可", "tôi–cô"),
    ]);
    let mut cast = Cast::default();
    cast.addressing.insert("秦寻→安可".into(), AddressTimeline { pinned: true, changes: vec![] });
    let source = "夏宁说：“以后我们结婚之后一定会因为这个吵架的。”秦寻怔住了。";
    let got = pairs_to_review(&pairs, &cast, source);
    // chị–em không phải cặp kiểu người lạ; 秦寻→安可 đã ghim → không xét.
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!((got[0].a.as_str(), got[0].b.as_str()), ("秦寻", "夏宁"));
    assert_eq!(got[0].markers, vec!["结婚"]);
    assert!(pairs_to_review(&pairs, &cast, "夏宁握着方向盘。秦寻怔住了。").is_empty(), "không có dấu hiệu → không xét");
}

#[test]
fn chapter_prompt_tach_cap_can_xet_lai_khoi_danh_sach_phai_giu() {
    let story = story(
        "modern",
        &[("秦寻", "Tần Tầm"), ("夏宁", "Hạ Ninh"), ("黄怀", "Hoàng Hoài")],
        &[("秦寻→夏宁", "tôi–cô"), ("夏宁→秦寻", "tôi–anh"), ("夏宁→黄怀", "chị–em")],
    );
    let source = "夏宁说：“以后我们结婚之后一定会吵架的。”秦寻怔住了。黄怀笑了。";
    let prompt = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0893", source);
    let got = &prompt[prompt.find("\n# Xưng hô\n").unwrap()..];
    let (keep, review) = got.split_once("Cặp cần xét lại trong chương này").expect("có khối xét lại");
    assert!(keep.contains("\"夏宁→黄怀\": \"chị–em\"") && !keep.contains("秦寻→夏宁"), "cặp cần xét không nằm trong danh sách phải giữ");
    // Thứ tự theo cặp gặp trước trong glossary (helper `story` xếp key theo mã chữ: 夏宁→秦寻 đứng trước).
    assert!(review.contains("- 夏宁 ↔ 秦寻: đang `夏宁→秦寻: tôi–anh`, `秦寻→夏宁: tôi–cô`; dấu hiệu trong raw: 结婚"), "{review}");
    assert!(review.contains("KHÔNG dùng cặp cũ") && review.contains("người yêu, vợ chồng thường là anh–em"));
}
