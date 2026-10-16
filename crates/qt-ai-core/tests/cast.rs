//! Bảng nhân vật (cast.json): giới tính, cặp xưng hô theo mốc chương, mục prompt `# Xưng hô`, check.
use qt_ai_core::cast::*;
use qt_ai_core::story::{AutoGlossaryEntry, StringMap};
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
    let listed: Vec<(&str, &str, Option<Gender>)> = got.iter().map(|c| (c.source.as_str(), c.name.as_str(), c.gender)).collect();
    // 莫衡 vắng; 叶清禾 có mặt nhưng chưa rõ giới → không liệt kê.
    assert_eq!(listed, vec![("贺静昭", "Hạ Tĩnh Chiêu", Some(Gender::Female)), ("宋时安", "Tống Thời An", Some(Gender::Male))]);
}

#[test]
fn nhan_ra_nhan_vat_goi_bang_ho_va_chuc_danh_co_trang() {
    let names = map(&[("林衍", "Lâm Diễn"), ("宋青书", "Tống Thanh Thư")]);
    let cast = cast_with(&[("林衍", Gender::Male), ("宋青书", Gender::Male)]);
    let got = present_characters(&names, &cast, "林掌门笑道：“宋少侠请。”");
    let listed: Vec<&str> = got.iter().map(|c| c.source.as_str()).collect();
    assert_eq!(listed, vec!["林衍", "宋青书"]);
}

#[test]
fn chapter_prompt_khong_cast_khong_tu_dien_thi_chi_them_luat_xung_ho_truoc_suffix() {
    let story = story("modern", &[], &[]);
    let source = "许浪醒了。";
    let plain = build_system_prompt(&TranslationGlossary::new(), Some(&story), Some(source));
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0001", source);
    let head = plain.strip_suffix(prompt_suffix()).unwrap();
    assert!(got.starts_with(head) && got.ends_with(prompt_suffix()));
    let section = &got[head.len()..got.len() - prompt_suffix().len()];
    assert!(section.starts_with("\n# Xưng hô\n"), "{section}");
    assert!(section.contains("Danh xưng làm đại từ") && section.contains("Em đừng dọa chị được không"));
    assert!(section.contains("Độc thoại nội tâm") && section.contains("`mình`"));
    assert!(section.contains("老师 ngoài trường học"));
    assert!(section.contains("đạn mạc") && section.contains("Tiêu đề chương là câu nói của nhân vật"));
    assert!(!got.contains("# Từ điển riêng của truyện") && !got.contains("# Nhân vật và xưng hô trong chương"));
}

#[test]
fn chapter_prompt_phan_tinh_dung_truoc_phan_dong_theo_chuong() {
    let mut story = story(
        "modern",
        &[("贺静昭", "Hạ Tĩnh Chiêu"), ("宋时安", "Tống Thời An"), ("莫衡", "Mạc Hành")],
        &[("宋时安→贺静昭", "tôi–cô Hạ"), ("宋时安→莫衡", "em–thầy Mạc")],
    );
    story.summary = "Tóm tắt truyện.".into();
    story.style.voice = "Giọng kể hóm hỉnh.".into();
    let cast = cast_with(&[("贺静昭", Gender::Female), ("宋时安", Gender::Male), ("莫衡", Gender::Male)]);
    let a = build_chapter_prompt(&TranslationGlossary::new(), &story, &cast, "0001", "宋时安看着贺静昭。");
    let b = build_chapter_prompt(&TranslationGlossary::new(), &story, &cast, "0002", "宋时安和莫衡走了。");

    // Thứ tự: thông tin truyện → style → luật xưng hô (tĩnh) → từ điển → nhân vật/cặp trong chương (động) → suffix.
    let at = |text: &str, marker: &str| text.find(marker).unwrap_or_else(|| panic!("thiếu {marker}"));
    let order = [
        "\n# Thông tin truyện\n",
        "\n# Style đặc thù của truyện\n",
        "\n# Xưng hô\n",
        "\n# Từ điển riêng của truyện\n",
        "\n# Nhân vật và xưng hô trong chương\n",
    ];
    let positions: Vec<usize> = order.iter().map(|marker| at(&a, marker)).collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{positions:?}");
    assert!(a.ends_with(prompt_suffix()));

    // Hai chương khác nhau chung y hệt đoạn đầu tới hết phần tĩnh — phần nhà cung cấp cache được.
    let dynamic_start = at(&a, "\n# Từ điển riêng của truyện\n");
    assert_eq!(dynamic_start, at(&b, "\n# Từ điển riêng của truyện\n"));
    assert_eq!(a[..dynamic_start], b[..dynamic_start]);
    // Phần động thì khác: mỗi chương chỉ có người và cặp của chương đó.
    assert!(a[dynamic_start..].contains("贺静昭") && !a[dynamic_start..].contains("莫衡"));
    assert!(b[dynamic_start..].contains("莫衡") && !b[dynamic_start..].contains("贺静昭"));
    // Luật tĩnh không lặp lại ở phần động.
    assert_eq!(a.matches("Danh xưng làm đại từ").count(), 1);
}

#[test]
fn chapter_prompt_co_dai_khong_co_luat_lao_su_va_doc_thoai_la_ta() {
    let story = story("ancient", &[], &[]);
    let prompt = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0001", "他醒了。");
    let got = &prompt[prompt.find("\n# Xưng hô\n").unwrap()..];
    assert!(got.contains("Độc thoại nội tâm") && got.contains("`ta`") && !got.contains("`mình`"));
    assert!(!got.contains("老师 ngoài trường học") && !got.contains("Em đừng dọa chị"));
    assert!(got.contains("Sư đệ đừng dọa sư tỷ"));
    assert!(got.contains("Tiêu đề chương là câu nói của nhân vật") && !got.contains("đạn mạc"));
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
    assert!(got.contains("- 宋时安→贺静昭: em–chị (chốt từ ch.100)") && !got.contains("tôi–cô Hạ"), "{got}");
    assert!(got.contains("Cặp xưng hô đang dùng (chương đang dịch: ch.132)"), "{got}");
    assert!(!got.contains("周涛"), "cặp một bên vắng không gửi");
    assert!(!got.contains("giữ y hệt") && !got.contains("Giữ đúng các cặp này") && got.contains("mặc định giữ"));
    assert!(got.contains("thầy trò, chủ tớ"), "lời dặn nêu cả quan hệ thầy trò, chủ tớ");
    assert!(!got.contains("Cặp cần xét lại"), "chương không có dấu hiệu thân mật → không có khối xét lại");
    // Nhóm addressing không còn nằm trong JSON từ điển (đã chuyển sang mục Xưng hô).
    assert!(!got.contains("\"addressing\""));
    assert!(got.contains("\"贺静昭\": \"Hạ Tĩnh Chiêu\""));
}

#[test]
fn danh_sach_nhan_vat_ghi_dai_tu_loi_ke_theo_gioi_bien_than_va_ghi_rieng() {
    let names = [("贺静昭", "Hạ Tĩnh Chiêu"), ("宋时安", "Tống Thời An"), ("莫衡", "Mạc Hành"), ("叶清禾", "Diệp Thanh Hòa")];
    let mut cast = cast_with(&[("贺静昭", Gender::Female), ("宋时安", Gender::Swap), ("莫衡", Gender::Male)]);
    cast.characters.get_mut("莫衡").unwrap().narration = "lão".into();
    cast.characters.insert("叶清禾".into(), Character { narration: "Ngài".into(), ..Character::default() });
    let source = "宋时安看向贺静昭，莫衡与叶清禾都在。";
    let modern = build_chapter_prompt(&TranslationGlossary::new(), &story("modern", &names, &[]), &cast, "0001", source);
    assert!(modern.contains("- 贺静昭 (Hạ Tĩnh Chiêu): nữ — lời kể: `cô`"), "{modern}");
    assert!(modern.contains("- 宋时安 (Tống Thời An): biến thân — lời kể theo đại từ của raw từng câu (他 → `hắn`, 她 → `cô`)"));
    assert!(modern.contains("- 莫衡 (Mạc Hành): nam — lời kể: `lão` (ghi riêng)"));
    assert!(modern.contains("- 叶清禾 (Diệp Thanh Hòa): lời kể: `Ngài` (ghi riêng)"), "chưa rõ giới mà có đại từ riêng vẫn liệt kê");
    let ancient = build_chapter_prompt(&TranslationGlossary::new(), &story("ancient", &names, &[]), &cast, "0001", source);
    assert!(ancient.contains("- 贺静昭 (Hạ Tĩnh Chiêu): nữ — lời kể: `nàng`"), "{ancient}");
}

#[test]
fn bien_than_khong_bi_check_theo_gioi() {
    let names = map(&[("贺静昭", "Hạ Tĩnh Chiêu")]);
    let cast = cast_with(&[("贺静昭", Gender::Swap)]);
    let raw = vec!["“贺老师很擅长包饺子。”".to_string()];
    let out = vec![Some("“Thầy Hạ rất khéo gói sủi cảo.”".to_string())];
    assert!(gender_violations(&raw, &out, &names, &cast).is_empty());
    assert!(!pair_gender_conflict("宋时安→贺静昭", "tôi–anh", &cast));
    assert_eq!(Gender::parse("Biến thân"), Some(Gender::Swap));
}

#[test]
fn hoc_bien_than_doi_gioi_tu_hoc_nhung_khong_de_gioi_dat_tay() {
    let names: std::collections::HashSet<String> = ["江柠", "白贞羽", "许浪"].iter().map(|s| s.to_string()).collect();
    let mut cast = cast_with(&[("江柠", Gender::Female), ("白贞羽", Gender::Female), ("许浪", Gender::Swap)]);
    cast.characters.get_mut("白贞羽").unwrap().source = EntrySource::User;
    let entries = serde_json::json!([
        {"source": "江柠", "category": "names", "gender": "biến thân"},
        {"source": "白贞羽", "category": "names", "gender": "biến thân"},
        {"source": "许浪", "category": "names", "gender": "nam"},
    ]);
    let raw = "江柠看着白贞羽，他心想许浪又来了，她叹气。";
    let notes = learn_genders(&entries, raw, &names, &mut cast, "0050");
    assert_eq!(cast.gender_of("江柠"), Some(Gender::Swap), "tự học nữ → biến thân");
    assert_eq!(cast.characters["江柠"].chapter, "0050");
    assert_eq!(cast.gender_of("白贞羽"), Some(Gender::Female), "đặt tay thì không đổi");
    assert_eq!(cast.characters["白贞羽"].disputed, vec!["0050".to_string()]);
    assert_eq!(cast.gender_of("许浪"), Some(Gender::Swap), "đã biến thân thì bỏ qua khai nam/nữ");
    assert!(cast.characters["许浪"].disputed.is_empty());
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes[0].contains("江柠 đổi từ nữ sang biến thân"), "{notes:?}");
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
        Ok(Generated::text(text))
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
    assert_eq!(got[0].kind, ReviewKind::Intimacy);
}

#[test]
fn pairs_to_review_bai_su_nhan_chu_ket_nghia_va_cap_to_cau_tao_may() {
    let pairs = map(&[
        ("陈钰→朱媺娖", "ta–ngươi"),
        ("朱媺娖→陈钰", "ta–ngươi"),
        ("陈钰→阿朱", "tớ–cậu"),
        ("阿朱→陈钰", "tớ–cậu"),
    ]);
    let cast = Cast::default();
    // Sự kiện bái sư → xét theo quan hệ mới; danh xưng chung 师父 một mình không bật.
    let got = pairs_to_review(&pairs, &cast, "陈钰跪下拜师，朱媺娖收他为徒。阿朱在旁边看着。");
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got.iter().all(|review| review.kind == ReviewKind::Relation));
    assert_eq!(got[0].markers, vec!["拜师", "为徒"]);
    assert!(pairs_to_review(&pairs, &cast, "陈钰对师父说话。朱媺娖点头。阿朱笑了。").is_empty());
    // Vừa có dấu hiệu thân mật vừa có kết nghĩa → xét theo người yêu/vợ chồng; tớ–cậu cũng là cặp xa cách.
    // Luồng thật lọc cặp theo nhân vật có mặt trước (朱媺娖 vắng → cặp với nàng không xét).
    let source = "阿朱说要和陈钰结婚，又说要结拜。";
    let got = pairs_to_review(&chapter_addressing(&pairs, source), &cast, source);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!((got[0].a.as_str(), got[0].b.as_str(), got[0].kind), ("陈钰", "阿朱", ReviewKind::Intimacy));
}

#[test]
fn chapter_label_va_addressing_since_tu_moc_cast_hoac_log_glossary() {
    assert_eq!(chapter_label("0001_7445623259162542654"), "1");
    assert_eq!(chapter_label("10021482_1040_第一千零四十章"), "1040");
    assert_eq!(chapter_label("chuong-0392"), "392");
    assert_eq!(chapter_label("phan-mo-dau"), "phan-mo-dau");
    let pairs = map(&[("甲→乙", "anh–em"), ("乙→甲", "em–anh"), ("丙→丁", "ta–ngươi")]);
    let mut cast = Cast::default();
    cast.addressing.insert("甲→乙".into(), AddressTimeline { pinned: false, changes: vec![change("0050", "anh–em"), change("0200", "chồng–vợ")] });
    let log = vec![
        AutoGlossaryEntry { source: "乙→甲".into(), target: "tôi–anh".into(), category: "addressing".into(), chapter: "0003".into() },
        AutoGlossaryEntry { source: "甲→乙".into(), target: "tôi–cô".into(), category: "addressing".into(), chapter: "0001".into() },
    ];
    let since = addressing_since(&pairs, &cast, &log, "0120");
    // Mốc cast gần nhất ≤ chương đang dịch thắng log; mốc 0200 chưa tới.
    assert_eq!(since.get("甲→乙").map(String::as_str), Some("0050"));
    assert_eq!(since.get("乙→甲").map(String::as_str), Some("0003"));
    assert_eq!(since.get("丙→丁"), None, "nhập tay, không có trong log");
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
    assert!(keep.contains("- 夏宁→黄怀: chị–em") && !keep.contains("秦寻→夏宁"), "cặp cần xét không nằm trong danh sách phải giữ");
    // Thứ tự theo cặp gặp trước trong glossary (helper `story` xếp key theo mã chữ: 夏宁→秦寻 đứng trước).
    assert!(review.contains("- 夏宁 ↔ 秦寻: đang `夏宁→秦寻: tôi–anh`, `秦寻→夏宁: tôi–cô`; dấu hiệu trong raw: 结婚"), "{review}");
    assert!(review.contains("KHÔNG dùng cặp cũ") && review.contains("người yêu, vợ chồng thường là anh–em"));
}

// ---- Hỗn hợp: tự xưng `ta` mọi cảnh, cặp `tôi–…` cũ phải xét lại ----

#[test]
fn hon_hop_doc_thoai_mac_dinh_ta() {
    let got = addressing_rules_section(qt_ai_core::story::GenreSetting::Mixed);
    assert!(got.contains("mỗi nhân vật chốt MỘT ngôi tự xưng trong cả chương, mặc định `ta`;"), "{got}");
    assert!(!got.contains("`mình` ở cảnh hiện đại"));
}

#[test]
fn hon_hop_cap_con_tu_xung_toi_dua_ra_xet_lai_khong_nam_trong_danh_sach_giu() {
    let story = story(
        "mixed",
        &[("刘源", "Lưu Nguyên"), ("林薇薇", "Lâm Vi Vi"), ("苏清雪", "Tô Thanh Tuyết")],
        &[("刘源→林薇薇", "tôi–cô"), ("林薇薇→刘源", "tôi–trưởng khoa"), ("刘源→苏清雪", "anh–em")],
    );
    let source = "刘源看着林薇薇，又看了看苏清雪。";
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0010", source);
    let section = &got[got.find("\n# Nhân vật và xưng hô trong chương\n").unwrap()..];
    let (keep, legacy) = section.split_once("Cặp còn tự xưng `tôi`").expect("có khối cặp tự xưng tôi");
    assert!(keep.contains("- 刘源→苏清雪: anh–em") && !keep.contains("刘源→林薇薇"), "{keep}");
    assert!(legacy.contains("`林薇薇→刘源: tôi–trưởng khoa`") && legacy.contains("`刘源→林薇薇: tôi–cô`"), "{legacy}");
    assert!(legacy.contains("bề dưới với bề trên") && legacy.contains("BẮT BUỘC khai lại"), "{legacy}");

    // Cặp người dùng đã ghim thì tôn trọng, không đưa ra xét; truyện hiện đại không có khối này.
    let mut pinned = Cast::default();
    pinned.addressing.insert("刘源→林薇薇".into(), AddressTimeline { pinned: true, changes: vec![] });
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &pinned, "0010", source);
    assert!(!got.contains("Cặp còn tự xưng `tôi`"));
    let modern = story_with_setting(&story, "modern");
    let got = build_chapter_prompt(&TranslationGlossary::new(), &modern, &Cast::default(), "0010", source);
    assert!(!got.contains("Cặp còn tự xưng `tôi`"));
}

fn story_with_setting(base: &StoryConfig, setting: &str) -> StoryConfig {
    let mut value = serde_json::to_value(base).unwrap();
    value["genre"]["setting"] = json!(setting);
    StoryConfig::normalize(&value)
}

#[test]
fn hon_hop_khong_hoc_cap_tu_xung_toi_moi() {
    use qt_ai_core::glossary::ExtractedPair;
    use qt_ai_core::story::GenreSetting;
    let names: std::collections::HashSet<String> = ["刘源", "林薇薇", "苏昊"].iter().map(|s| s.to_string()).collect();
    let pair = |source: &str, target: &str| ExtractedPair { source: source.into(), target: target.into(), category: "addressing".into() };
    let pairs = vec![pair("刘源→林薇薇", "tôi–cô"), pair("苏昊→刘源", "vãn bối–tiền bối"), pair("刘源→苏昊", "ta–cậu")];
    let kept: Vec<String> = filter_new_pairs(pairs.clone(), &names, &Cast::default(), GenreSetting::Mixed)
        .into_iter()
        .map(|p| p.source)
        .collect();
    assert_eq!(kept, vec!["苏昊→刘源", "刘源→苏昊"]);
    assert_eq!(filter_new_pairs(pairs, &names, &Cast::default(), GenreSetting::Modern).len(), 3);

    // Đổi cặp cũ sang `tôi–…` cũng không nhận; đổi từ `tôi–…` sang `ta–…` thì nhận.
    let base = map(&[("刘源→林薇薇", "tôi–cô"), ("刘源→苏昊", "tôi–cậu")]);
    let entries = json!([
        { "category": "addressing", "source": "刘源→林薇薇", "target": "ta–cô", "note": "tự xưng theo luật mới" },
        { "category": "addressing", "source": "刘源→苏昊", "target": "tôi–anh", "note": "đổi" },
    ]);
    let mut cast = Cast::default();
    apply_address_changes(&entries, "刘源对林薇薇和苏昊说。", &base, &mut cast, "0010", GenreSetting::Mixed);
    assert_eq!(cast.addressing["刘源→林薇薇"].changes[0].target, "ta–cô");
    assert!(!cast.addressing.contains_key("刘源→苏昊"));
}

#[test]
fn co_dai_khong_hoc_cap_xung_ho_kieu_hien_dai_giu_cap_gia_dinh() {
    use qt_ai_core::glossary::ExtractedPair;
    use qt_ai_core::story::GenreSetting;
    let names: std::collections::HashSet<String> =
        ["许阳", "许晴", "李虎", "周顺昌", "张大牛", "许老二"].iter().map(|s| s.to_string()).collect();
    let pair = |source: &str, target: &str| ExtractedPair { source: source.into(), target: target.into(), category: "addressing".into() };
    let pairs = vec![
        // Kiểu hiện đại (Cẩu Thánh): bị bỏ.
        pair("许晴→许阳", "em–anh"),
        pair("许阳→许晴", "anh–em"),
        pair("张大牛→许阳", "tôi–cậu"),
        pair("李虎→许阳", "tao–mày"),
        pair("许阳→张大牛", "ta–anh Đại Ngưu"),
        pair("许阳→李虎", "ta–cậu"),
        // Cổ trang và từ gia đình: vẫn học.
        pair("许阳→周顺昌", "cháu–cậu cả"),
        pair("周顺昌→许阳", "cậu–cháu"),
        pair("许老二→许晴", "cha–con"),
        pair("李虎→张大牛", "ta–ngươi"),
        pair("许晴→李虎", "muội–Hổ gia"),
        // Tên riêng viết hoa (`Anh Lạc`, `Anh ca`) và `anh hùng` không phải đại từ hiện đại.
        pair("张大牛→李虎", "ta–anh hùng"),
        pair("许老二→许阳", "ta–Anh ca"),
    ];
    let kept: Vec<String> = filter_new_pairs(pairs.clone(), &names, &Cast::default(), GenreSetting::Ancient)
        .into_iter()
        .map(|p| p.source)
        .collect();
    assert_eq!(kept, vec!["许阳→周顺昌", "周顺昌→许阳", "许老二→许晴", "李虎→张大牛", "许晴→李虎", "张大牛→李虎", "许老二→许阳"]);
    assert_eq!(filter_new_pairs(pairs, &names, &Cast::default(), GenreSetting::Modern).len(), 13);

    // Đổi cặp cũ sang kiểu hiện đại cũng không nhận; đổi từ `anh–em` sang cổ trang thì nhận.
    let base = map(&[("许阳→许晴", "anh–em"), ("许晴→许阳", "muội–ca")]);
    let entries = json!([
        { "category": "addressing", "source": "许阳→许晴", "target": "ta–muội", "note": "cổ trang" },
        { "category": "addressing", "source": "许晴→许阳", "target": "em–anh", "note": "thân mật" },
    ]);
    let mut cast = Cast::default();
    apply_address_changes(&entries, "许阳对许晴说。", &base, &mut cast, "0010", GenreSetting::Ancient);
    assert_eq!(cast.addressing["许阳→许晴"].changes[0].target, "ta–muội");
    assert!(!cast.addressing.contains_key("许晴→许阳"));
}

#[test]
fn hon_hop_cap_toi_uu_tien_khoi_tu_xung_moi_hon_khoi_than_mat() {
    // Chương có 老婆 (nói về vợ người khác) — đôi 苏昊↔刘源 vẫn phải nằm ở khối tự xưng mới, không ở khối thân mật
    // (khối đó dặn "dấu hiệu nói về người khác thì giữ cặp cũ", tức giữ `tôi`).
    let story = story(
        "mixed",
        &[("刘源", "Lưu Nguyên"), ("苏昊", "Tô Hạo")],
        &[("苏昊→刘源", "tôi–tiền bối"), ("刘源→苏昊", "tôi–cậu")],
    );
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &Cast::default(), "0105", "苏昊对刘源说：“前辈的老婆真好。”");
    let section = &got[got.find("\n# Nhân vật và xưng hô trong chương\n").unwrap()..];
    let (_, legacy) = section.split_once("Cặp còn tự xưng `tôi`").expect("có khối tự xưng mới");
    assert!(legacy.contains("`苏昊→刘源: tôi–tiền bối`") && legacy.contains("`刘源→苏昊: tôi–cậu`"), "{section}");
    assert!(!section.contains("dấu hiệu quan hệ thân mật"), "{section}");
}

#[test]
fn loi_ke_ngoi_ba_mac_dinh_han_chi_nhan_vat_ghi_rieng_moi_dung_cau() {
    use qt_ai_core::story::GenreSetting;
    for setting in [GenreSetting::Ancient, GenreSetting::Modern, GenreSetting::Mixed] {
        let got = addressing_rules_section(setting);
        assert!(got.contains("- **Đại từ lời kể theo nhân vật.**"), "{got}");
        assert!(got.contains("KHÔNG `cậu`, `cậu ta`, `cậu ấy`"), "{got}");
        assert!(got.contains("trẻ nhỏ (chưa tới tuổi thiếu niên) mặc định `nó`"), "{got}");
        assert!(got.contains("không lẫn `cậu ta` với `hắn` cho cùng một người"), "{got}");
    }
    // Nhân vật được ghi riêng đại từ lời kể thì danh sách có mặt nêu đúng đại từ đó.
    let story = story("modern", &[("唐晨", "Đường Thần")], &[]);
    let mut cast = Cast::default();
    cast.characters.insert(
        "唐晨".into(),
        serde_json::from_value(json!({ "gender": "male", "narration": "cậu", "source": "user" })).unwrap(),
    );
    let got = build_chapter_prompt(&TranslationGlossary::new(), &story, &cast, "0001", "唐晨看着爸爸。");
    assert!(got.contains("- 唐晨 (Đường Thần): nam — lời kể: `cậu` (ghi riêng)"), "{got}");
    assert!(got.contains("trẻ nhỏ được dùng `nó`"), "danh sách có mặt không ép `hắn` cho trẻ nhỏ");
}
