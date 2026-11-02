//! BaseStore nối vào chỗ ghép prompt / rule qua env QT_AI_BASE_DIR. Env là toàn cục nên gộp một test.
use qt_ai_core::base::{BaseStore, BASE_DIR_ENV};
use qt_ai_core::check::check_violations;
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::idioms::{IdiomTable, IDIOM_ISSUE};
use qt_ai_core::prompt::{build_system_prompt, TranslationGlossary};
use qt_ai_core::story::{CheckRule, GenreSetting, StoryConfig, StringMap};
use qt_ai_core::story_fs::{story_paths, work_file, WorkKind};

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

    // Thành ngữ: bảng người dùng đè bảng chung; chương có câu nào thì prompt `qt-ai next` có câu đó, bản dịch không
    // theo thì accept ghi cảnh báo (vẫn chốt).
    let user = IdiomTable::from([("打草惊蛇".to_string(), "rút dây động rừng".to_string())]);
    store.save_idioms_user(&user).unwrap();
    let story_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(story_dir.path().join("raw")).unwrap();
    std::fs::write(story_dir.path().join("raw").join("0001.txt"), "第一章\n\n三十年河东，三十年河西，你别打草惊蛇。").unwrap();
    run_init(story_dir.path(), "qt-ai").unwrap();
    let next = run_next(story_dir.path()).unwrap();
    let prompt = std::fs::read_to_string(&next.prompt_path).unwrap();
    assert!(prompt.contains("# Thành ngữ trong chương"));
    assert!(prompt.contains("\"打草惊蛇\": \"rút dây động rừng\""), "bảng người dùng đè");
    assert!(prompt.contains("\"三十年河东，三十年河西\": \"ba mươi năm hà đông, ba mươi năm hà tây\""), "bảng đóng sẵn");
    let paths = story_paths(story_dir.path());
    std::fs::write(
        work_file(&paths, "0001", WorkKind::Draft),
        "[[1]] Chương 1\n\n[[2]] Ba mươi năm sông Đông, ba mươi năm sông Tây, ngươi đừng rút dây động rừng.\n",
    )
    .unwrap();
    let accepted = run_accept(story_dir.path(), "0001", false).unwrap();
    let idiom_warnings: Vec<&String> = accepted.warnings.iter().filter(|w| w.contains(IDIOM_ISSUE)).collect();
    assert_eq!(idiom_warnings.len(), 1, "{:?}", accepted.warnings);
    assert!(idiom_warnings[0].contains("三十年河东"));

    std::env::remove_var(BASE_DIR_ENV);
}
