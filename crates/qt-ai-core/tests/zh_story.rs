//! Chuyển raw phồn thể của truyện sang giản thể, đổi khoá hồ sơ theo, hoàn tác.
use qt_ai_core::cast::load_cast;
use qt_ai_core::story_fs::{load_story_config, story_paths};
use qt_ai_core::titles::load_titles;
use qt_ai_core::zh_story::{convert_story, story_zh_status, undo_story_convert, TRADITIONAL_DIR};
use std::fs;

fn story() -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-zh-").tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::create_dir_all(root.join("out")).unwrap();
    fs::write(root.join("raw/0001.txt"), "第一章 葉翔\n\n葉翔看著後山，說：「走吧。」").unwrap();
    let (big5, _, _) = encoding_rs::BIG5.encode("第二章\n\n葉翔回來了，這裡很安靜。");
    fs::write(root.join("raw/0002.txt"), &big5).unwrap();
    fs::write(root.join("raw/0003.txt"), "第三章\n\n叶翔说：「大乾王朝」不在这里。").unwrap();
    let (gbk, _, _) = encoding_rs::GBK.encode("第四章\n\n叶翔走进了山门，天色已晚。");
    fs::write(root.join("raw/0004.txt"), &gbk).unwrap();
    let story = r#"{"name":"x","glossary":{"names":{"葉翔":"Diệp Tường","叶翔":"Diệp Tường (đã có)","蘇雨":"Tô Vũ"},"addressing":{"葉翔→蘇雨":"ta–muội"}},"style":{"voice":"","toneRules":[],"signaturePhrases":{},"avoid":[]},"autoGlossaryLog":[{"source":"蘇雨","target":"Tô Vũ","category":"names","chapter":"0001"}]}"#;
    fs::write(root.join("story.json"), story).unwrap();
    fs::write(root.join("cast.json"), r#"{"version":1,"characters":{"蘇雨":{"gender":"female","source":"auto"}},"addressing":{}}"#).unwrap();
    fs::write(root.join("titles.json"), r#"{"version":1,"titles":{"葉翔歸來":"Diệp Tường trở về"}}"#).unwrap();
    dir
}

#[test]
fn nhan_chuong_phon_the_va_ma_hoa_khac_utf8() {
    let dir = story();
    let paths = story_paths(dir.path());
    let status = story_zh_status(&paths, None).unwrap();
    assert_eq!((status.scanned, status.traditional, status.not_utf8, status.undo_chapters), (4, 2, 1, None));
    assert_eq!(story_zh_status(&paths, Some(1)).unwrap().scanned, 1);
}

#[test]
fn chuyen_chuong_phon_doi_khoa_ho_so_roi_hoan_tac() {
    let dir = story();
    let root = dir.path();
    let paths = story_paths(root);
    let originals: Vec<Vec<u8>> = (1..=4).map(|n| fs::read(root.join(format!("raw/000{n}.txt"))).unwrap()).collect();
    let profile = ["story.json", "cast.json", "titles.json"].map(|name| fs::read_to_string(root.join(name)).unwrap());

    let done = convert_story(&paths).unwrap();
    assert_eq!(done.chapters, 3, "hai chương phồn + một chương GBK");
    assert_eq!(fs::read_to_string(root.join("raw/0001.txt")).unwrap(), "第一章 叶翔\n\n叶翔看着后山，说：“走吧。”");
    assert_eq!(fs::read_to_string(root.join("raw/0002.txt")).unwrap(), "第二章\n\n叶翔回来了，这里很安静。");
    assert_eq!(fs::read(root.join("raw/0003.txt")).unwrap(), originals[2], "chương giản thể giữ nguyên cả ngoặc 「」");
    assert_eq!(fs::read_to_string(root.join("raw/0004.txt")).unwrap(), "第四章\n\n叶翔走进了山门，天色已晚。", "GBK → UTF-8");
    assert_eq!(fs::read(root.join(TRADITIONAL_DIR).join("0002.txt")).unwrap(), originals[1], "bản gốc Big5 giữ nguyên byte");
    assert!(!root.join(TRADITIONAL_DIR).join("0003.txt").exists());

    let story = load_story_config(&paths).unwrap();
    assert_eq!(story.glossary["names"]["叶翔"], "Diệp Tường (đã có)", "trùng khoá: giữ mục vốn giản thể");
    assert_eq!(story.glossary["names"]["苏雨"], "Tô Vũ");
    assert!(!story.glossary["names"].contains_key("葉翔"));
    assert_eq!(story.glossary["addressing"]["叶翔→苏雨"], "ta–muội");
    assert_eq!(story.auto_glossary_log[0].source, "苏雨");
    assert!(load_cast(&paths).characters.contains_key("苏雨"));
    assert_eq!(load_titles(&paths).titles["叶翔归来"], "Diệp Tường trở về");
    assert_eq!((done.keys, done.merged), (6, 1));
    assert_eq!(story_zh_status(&paths, None).unwrap().undo_chapters, Some(3));
    assert_eq!(convert_story(&paths).unwrap().chapters, 0, "chuyển lại không còn gì");

    fs::write(root.join("raw/0004.txt"), "sửa tay sau đó").unwrap();
    let undone = undo_story_convert(&paths).unwrap();
    assert_eq!((undone.restored, undone.conflicts.clone(), undone.profile_restored), (2, vec!["0004".to_string()], 3));
    assert_eq!(fs::read(root.join("raw/0001.txt")).unwrap(), originals[0]);
    assert_eq!(fs::read(root.join("raw/0002.txt")).unwrap(), originals[1]);
    assert_eq!(fs::read_to_string(root.join("raw/0004.txt")).unwrap(), "sửa tay sau đó", "chương sửa tiếp không bị đè");
    assert_eq!(["story.json", "cast.json", "titles.json"].map(|name| fs::read_to_string(root.join(name)).unwrap()), profile);
    assert!(root.join(TRADITIONAL_DIR).join("0004.txt").exists(), "bản gốc chưa trả về thì giữ lại");
    assert_eq!(story_zh_status(&paths, None).unwrap().undo_chapters, None);
}
