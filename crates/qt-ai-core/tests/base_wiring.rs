//! BaseStore nối vào chỗ ghép prompt / rule qua env QT_AI_BASE_DIR. Env là toàn cục nên gộp một test.
use qt_ai_core::base::{BaseStore, BASE_DIR_ENV};
use qt_ai_core::check::check_violations;
use qt_ai_core::prompt::{build_system_prompt, TranslationGlossary};
use qt_ai_core::story::{CheckRule, GenreSetting, StoryConfig, StringMap};

#[test]
fn base_file_thay_ban_cung_nhung_truyen_van_thang() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base");
    std::env::set_var(BASE_DIR_ENV, &base);
    let store = BaseStore::at(&base);
    let mut story = StoryConfig::empty();

    // Prompt: chưa có file → bản cứng; có file → file; truyện có prompt riêng → riêng.
    let builtin = build_system_prompt(&TranslationGlossary::new(), Some(&story), None);
    assert!(builtin.contains("| 我          | **ta**"));
    store.save_prompt(&story.genre, "# BASE CUA TOI\n").unwrap();
    let from_file = build_system_prompt(&TranslationGlossary::new(), Some(&story), None);
    assert!(from_file.starts_with("# BASE CUA TOI"));
    assert!(from_file.contains("Dịch raw text tiếng Trung"), "suffix vẫn nối");
    story.custom_prompt = "# RIENG".to_string();
    assert!(build_system_prompt(&TranslationGlossary::new(), Some(&story), None).starts_with("# RIENG"));
    story.custom_prompt.clear();

    // Glossary nền: caller truyền store.glossary(setting); truyện đè key trùng; lọc theo chương vẫn chạy.
    let mut base_glossary = TranslationGlossary::new();
    base_glossary.insert(
        "names".into(),
        StringMap::from([("赵".to_string(), "Triệu".to_string()), ("钱".to_string(), "Tiền".to_string())]),
    );
    store.save_glossary(GenreSetting::Ancient, &base_glossary).unwrap();
    story.glossary.get_mut("names").unwrap().insert("赵".to_string(), "Triệu (truyện)".to_string());
    let prompt = build_system_prompt(&store.glossary(story.genre.setting), Some(&story), Some("赵 đi chợ"));
    assert!(prompt.contains("\"赵\": \"Triệu (truyện)\""));
    assert!(!prompt.contains("钱"), "钱 không có trong chương → bị lọc");

    // Rule: file thay bộ cứng; rule Hán tự vẫn chạy; truyện có rule riêng thì riêng thắng.
    let ancient_builtin = check_violations("Hừm, vợ hắn", &[], GenreSetting::Ancient);
    assert!(ancient_builtin.iter().any(|v| v.message.contains("thê tử/phu quân")));
    store
        .save_rules(
            GenreSetting::Ancient,
            &[CheckRule { pattern: "Hừm".into(), flags: None, message: "Hừm → Ân (base)".into() }],
        )
        .unwrap();
    let ancient_file = check_violations("Hừm, vợ hắn 赵", &[], GenreSetting::Ancient);
    assert!(ancient_file.iter().any(|v| v.message == "Hừm → Ân (base)"));
    assert!(!ancient_file.iter().any(|v| v.message.contains("thê tử/phu quân")));
    assert!(ancient_file.iter().any(|v| v.message.contains("CJK còn sót")));
    let own = vec![CheckRule { pattern: "vợ".into(), flags: None, message: "riêng".into() }];
    let with_own = check_violations("Hừm, vợ hắn", &own, GenreSetting::Ancient);
    assert!(with_own.iter().any(|v| v.message == "riêng"));
    assert!(!with_own.iter().any(|v| v.message == "Hừm → Ân (base)"));

    std::env::remove_var(BASE_DIR_ENV);
}
