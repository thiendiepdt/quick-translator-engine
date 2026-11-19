//! Khoá glossary tự học không được đè bảng của prompt; danh xưng X父 / X母 không thành tên.

use qt_ai_core::glossary::{prompt_table_keys, sanitize_extracted};
use qt_ai_core::story::StoryGenre;
use std::collections::HashSet;

#[test]
fn khoa_bang_prompt_lay_tu_o_dau_cua_dong_bang() {
    let keys = prompt_table_keys(qt_ai_core::prompt::base_prompt(&StoryGenre::default()));
    for key in ["坊市", "不过", "委屈", "公公", "公爹", "婆婆"] {
        assert!(keys.contains(key), "thiếu {key}");
    }
    // Ô có ký tự khác chữ Hán (打断…的话) không tách ra mảnh để khỏi chặn nhầm "的话".
    assert!(!keys.contains("的话"));
    assert!(keys.iter().all(|key| key.chars().count() >= 2));
}

#[test]
fn danh_xung_ho_cong_cha_me_khong_vao_names() {
    let parsed = serde_json::json!([
        {"source": "沈父", "target": "Thẩm phụ", "category": "names"},
        {"source": "许母", "target": "Hứa mẫu", "category": "names"},
        {"source": "沈清", "target": "Thẩm Thanh", "category": "names"},
    ]);
    let pairs = sanitize_extracted(&parsed, "沈父与许母看着沈清。", "Thẩm phụ và Hứa mẫu nhìn Thẩm Thanh.", &HashSet::new());
    let sources: Vec<&str> = pairs.iter().map(|pair| pair.source.as_str()).collect();
    assert_eq!(sources, vec!["沈清"]);
}
