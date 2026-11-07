# Quy Tắc — kế hoạch 1: chuẩn hoá NFC + bộ máy trong qt-core (tắt mặc định)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Làm bước 0 và bước 1 của spec Quy Tắc: nghĩa từ điển ra NFC, và bộ máy Quy Tắc chạy được trong qt-core, tắt mặc định, ra đúng như bản thử Python và qua cổng tốc độ / RAM.

**Architecture:** Vòng dịch hiện có (`translate_all_mapped`) ghi thêm danh sách cụm đã dịch (`SourceToken`: chỉ số ký tự nguồn + byte trong output) khi Quy Tắc bật. Với mỗi đoạn (ngăn bởi `\n`), `QuyTac::apply` dựng token `(chữ Hán, nghĩa)`, chạy nhóm `[số]` trên ký tự, tách hư từ, gắn từ loại, chạy các nhóm quy tắc rồi ráp lại. Đoạn nào không có quy tắc nào chạy thì giữ nguyên từng byte. Đường Quy Tắc đi qua `LayeredBase` với bộ Luật Nhân rỗng (vẫn giữ `{h}{t}`), chỉ cho text, mode `vietphrase-one`, `wrap_type == 0`.

**Tech Stack:** Rust 2021 (qt-core), crate `regex` 1.12 (đã có), `unicode-normalization` 0.1.24 (mới), `serde_json` 1 (dev, cho example).

Spec: `docs/superpowers/specs/2026-11-06-quy-tac-design.md` (mục 4–9, 12–14). Bản thử tham chiếu: `docs/superpowers/specs/2026-11-06-quy-tac/quytac.py`, `qtlib.py`. Kế hoạch sau (bước 2–5: CLI, `dao`, bật mặc định + API/app/range, xoá Luật Nhân) viết khi kế hoạch này xong và có số đo thật.

## Global Constraints

- `Options::quy_tac` mặc định `false`. Khi tắt, output mọi đường dịch giữ nguyên từng byte như trước, chỉ khác bước NFC ở Task 1.
- Quy Tắc chỉ chạy khi: `opts.quy_tac == true`, engine có `QuyTac`, mode `VietPhraseOneMeaning`, `opts.wrap_type == 0`, và đường chỉ trả text. Đường có range không đổi trong kế hoạch này.
- Ngữ nghĩa port nguyên từ bản thử Python, kể cả chỗ kỳ quặc (ghi rõ trong code). Chỗ duy nhất sửa khác bản thử: ở nhóm `[số]`, phần tách lại sau khúc số không được lấn sang khúc số kế tiếp (bản thử có thể nhân đôi chữ).
- Bộ luật mặc định nhúng trong binary: `crates/qt-core/data/QuyTac.txt`, `QuyTac-giong-muot.txt`, `TuLoai.txt`. Có file cùng tên trong thư mục dữ liệu thì dùng file đó (`QuyTac.txt` ở gốc, `TuLoai.txt` trong `Resources/`).
- Không ghi tên của bộ luật số thứ hai (hai file trong `corpus/rules/` ngoài `rules.txt`) ở bất kỳ file, comment hay message commit nào; khi cần nhắc thì gọi là "bộ luật số thứ hai". Trước mỗi commit, grep không phân biệt hoa thường tên đó trên các file định stage.
- Không chạy `cargo fmt` cho cả workspace. Viết code đã định dạng sẵn.
- Comment và message tiếng Việt, theo giọng code hiện có.
- Commit thẳng `main`, mỗi task một commit, **chỉ khi user đã cho phép commit**. Không push.
- Lệnh chạy ở gốc repo `D:\Work\projects\quick-translator-engine` (Git Bash).
- Cổng chất lượng (Task 10): cụm 3 từ trùng bản biên tập **≥ +10,0%** so với convert hiện tại, tỉ lệ thắng **≥ 80%**. Cổng tốc độ: thời gian đường Quy Tắc **≤ 70%** đường hiện tại. Cổng RAM: `QuyTac` giữ **≤ 10 MiB**. Cổng hỏng thì dừng, báo số cho user, không chỉnh luật theo bộ test.

## File Structure

| File | Trách nhiệm |
| --- | --- |
| `crates/qt-core/Cargo.toml` | thêm `unicode-normalization`, dev `serde_json` |
| `crates/qt-core/src/dict.rs` | `nfc_values`, `nfc_string`; chuẩn hoá ở mọi chỗ nạp nguồn |
| `crates/qt-core/src/lite.rs` | chuẩn hoá NFC khi nạp; giữ `Option<QuyTac>`, `with_quy_tac` |
| `crates/qt-core/data/QuyTac.txt` (mới) | bộ luật mặc định (chép từ thư mục spec) |
| `crates/qt-core/data/QuyTac-giong-muot.txt` (mới) | nhóm giọng mượt |
| `crates/qt-core/data/TuLoai.txt` (mới) | sửa nhãn từ loại (chép `tag_fix.tsv`) |
| `crates/qt-core/src/quy_tac/mod.rs` (mới) | `QuyTac`, `QuyTacError`, nạp file, `apply` (chia đoạn, ghép lại output) |
| `crates/qt-core/src/quy_tac/numbers.rs` (mới) | đọc giá trị số Hán, đọc số ra chữ, `num_text` |
| `crates/qt-core/src/quy_tac/program.rs` (mới) | đọc file luật: lớp, `[số]`, nhóm quy tắc, kiểm khuôn |
| `crates/qt-core/src/quy_tac/tags.rs` (mới) | bảng từ loại `PosLexicon` |
| `crates/qt-core/src/quy_tac/prepare.rs` (mới) | `Token`, `Raw`, `Lookups`, tách hư từ, gắn nhãn, tách số + lượng từ |
| `crates/qt-core/src/quy_tac/char_rules.rs` (mới) | chạy nhóm `[số]` trên ký tự |
| `crates/qt-core/src/quy_tac/matcher.rs` (mới) | khớp mẫu, điền khuôn, chạy nhóm |
| `crates/qt-core/src/quy_tac/render.rs` (mới) | ráp token thành câu |
| `crates/qt-core/src/translate.rs` | `SourceToken`; ghi cụm đã dịch khi được yêu cầu |
| `crates/qt-core/src/lib.rs` | `mod quy_tac`, `Options::quy_tac`, `Engine::with_quy_tac`, nối `LayeredBase` |
| `apps/qt-gui/src-tauri/src/lib.rs` | thêm `quy_tac: false` vào literal `Options` |
| `crates/qt-core/tests/nfc.rs` (mới) | NFC qua Engine, Lite, override |
| `crates/qt-core/tests/quy_tac_engine.rs` (mới) | Quy Tắc qua API công khai với từ điển nhỏ |
| `crates/qt-core/tests/data/quy_tac_golden.tsv` (mới) | 60 câu mẫu, đáp án từ bản thử |
| `crates/qt-core/tests/quy_tac_golden.rs` (mới) | so Rust với bản thử trên QT2025 thật |
| `crates/qt-core/examples/quy_tac_eval.rs` (mới) | cổng chất lượng / tốc độ / RAM trên bookshelf |

---

### Task 1: Chuẩn hoá NFC nghĩa từ điển khi nạp

**Files:**
- Modify: `crates/qt-core/Cargo.toml`
- Modify: `crates/qt-core/src/dict.rs` (đầu file; `DictionaryOverrides::from_sources`, `with_patches`; `Dictionaries::build`, `with_lac_viet`, `build_full`)
- Modify: `crates/qt-core/src/lite.rs` (`load`, `build`)
- Test: `crates/qt-core/tests/nfc.rs`

**Interfaces:**
- Produces: `pub(crate) fn nfc_values(source: &str) -> std::borrow::Cow<'_, str>`, `pub(crate) fn nfc_string(value: String) -> String` trong `crate::dict`. Crate `unicode_normalization` dùng được trong toàn qt-core (Task 5, 10 dùng).

- [ ] **Step 1: Thêm dependency**

Trong `crates/qt-core/Cargo.toml`, mục `[dependencies]` thêm một dòng (giữ thứ tự chữ cái), và thêm mục dev ở cuối file:

```toml
unicode-normalization = "0.1.24"
```

```toml
[dev-dependencies]
serde_json = "1"
```

- [ ] **Step 2: Viết test hỏng**

Tạo `crates/qt-core/tests/nfc.rs`:

```rust
//! Nghĩa từ điển ra dạng NFC dù file nguồn lưu dấu tách rời (QT2025 có hơn
//! 11 nghìn dòng VietPhrase như vậy).

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, LiteEngine, Mode,
    Options,
};

/// "cũng" với dấu ngã tách rời (u + U+0303).
const NFD: &str = "cu\u{0303}ng";
/// "cũng" dựng sẵn (U+0169).
const NFC: &str = "c\u{0169}ng";

#[test]
fn engine_outputs_nfc_meanings() {
    let vietphrase = format!("也={NFD}");
    let engine = Engine::from_dicts(Dictionaries::build("", "", "", &vietphrase));
    assert_eq!(
        engine.translate("也", Mode::VietPhraseOneMeaning, &Options::default()),
        format!(" {NFC}")
    );
}

#[test]
fn lite_outputs_nfc_meanings() {
    let vietphrase = format!("也={NFD}");
    let lite = LiteEngine::build("", "", "", &vietphrase, "", "", "", "", "").unwrap();
    assert_eq!(
        lite.translate("也", &Options::default(), &DictionaryOverrides::default())
            .unwrap(),
        format!(" {NFC}")
    );
}

#[test]
fn request_names_are_normalized_too() {
    let engine = Engine::from_dicts(Dictionaries::build("", "", "", "也=x"));
    let names = format!("也={NFD}");
    let overrides = DictionaryOverrides::from_sources(DictionarySourceOverrides {
        names: Some(&names),
        ..Default::default()
    });
    assert_eq!(
        engine
            .translate_with_overrides("也", Mode::VietPhraseOneMeaning, &Options::default(), &overrides)
            .unwrap(),
        format!(" {NFC}")
    );
}
```

- [ ] **Step 3: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core --test nfc`
Expected: 3 test FAIL, output chứa `"cu\u{303}ng"` thay vì `"c\u{169}ng"`.

- [ ] **Step 4: Viết hàm chuẩn hoá**

Trong `crates/qt-core/src/dict.rs`, thêm vào khối `use` đầu file:

```rust
use std::borrow::Cow;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
```

Thêm ngay sau `impl DictionaryDefaults { … }` (trước `pub struct DictionarySourceOverrides`):

```rust
/// Chuẩn hoá NFC phần nghĩa (sau dấu `=` đầu tiên) của từng dòng `key=value`.
///
/// QT2025 lưu hơn 11 nghìn dòng VietPhrase với dấu tách rời (NFD): `也` ra
/// "cũng" nhìn giống hệt nhưng tìm, thay và so chuỗi đều hụt. Key giữ nguyên
/// byte: NFC đổi chữ Hán tương thích (U+F900…) sang chữ thường, làm key không
/// còn khớp văn bản nguồn.
pub(crate) fn nfc_values(source: &str) -> Cow<'_, str> {
    if is_nfc_quick(source.chars()) == IsNormalized::Yes {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        match line.split_once('=') {
            Some((key, value)) if is_nfc_quick(value.chars()) != IsNormalized::Yes => {
                out.push_str(key);
                out.push('=');
                out.extend(value.nfc());
            }
            _ => out.push_str(line),
        }
    }
    Cow::Owned(out)
}

/// NFC cho một nghĩa đơn lẻ (patch gửi kèm request).
pub(crate) fn nfc_string(value: String) -> String {
    if is_nfc_quick(value.chars()) == IsNormalized::Yes {
        value
    } else {
        value.nfc().collect()
    }
}
```

- [ ] **Step 5: Chuẩn hoá ở mọi chỗ nạp**

Trong `DictionaryOverrides::from_sources`, thay phần thân `Self { … }` bằng:

```rust
        Self {
            names: sources.names.map(|source| first_wins_scan_map(&nfc_values(source))),
            names2: sources
                .names2
                .map(|source| first_wins_scan_map(&nfc_values(source))),
            luat_nhan: sources
                .luat_nhan
                .map(|source| parse_luat_nhan(&nfc_values(source))),
            pronouns: sources.pronouns.map(|source| first_wins_map(&nfc_values(source))),
            danh_tu: sources.danh_tu.map(|source| first_wins_map(&nfc_values(source))),
            ho_nguoi: sources.ho_nguoi.map(|source| first_wins_map(&nfc_values(source))),
            hau_tu: sources.hau_tu.map(|source| first_wins_map(&nfc_values(source))),
            ignored_chinese_phrases: sources
                .ignored_chinese_phrases
                .map(parse_ignored_chinese_phrases),
            vietphrase_patches: FxHashMap::default(),
            chinese_phien_am_words_patches: HanVietMap::default(),
        }
```

Trong `with_patches`, thay hai dòng gán bằng:

```rust
        self.vietphrase_patches = patches
            .vietphrase
            .into_iter()
            .map(|(key, value)| (key, nfc_string(value)))
            .collect();
        self.chinese_phien_am_words_patches = patches
            .chinese_phien_am_words
            .into_iter()
            .map(|(key, value)| (key, nfc_string(value)))
            .collect();
```

Trong `Dictionaries::build`, thêm ở đầu thân hàm (trước `let han_viet = …`):

```rust
        let han_viet_src = nfc_values(han_viet_src);
        let names_src = nfc_values(names_src);
        let names2_src = nfc_values(names2_src);
        let vietphrase_src = nfc_values(vietphrase_src);
```

rồi sửa các chỗ dùng trong hàm thành `parse_han_viet(&han_viet_src)`, `first_wins_scan_map(&names_src)`, `first_wins_scan_map(&names2_src)`, `parse_dict(&vietphrase_src)`.

Trong `with_lac_viet`: `self.lac_viet = first_wins_map(&nfc_values(lac_viet_src));`

Trong `build_full`, thay các dòng gán bằng:

```rust
        dictionaries.pronouns = first_wins_map(&nfc_values(pronouns_src));
        dictionaries.danh_tu = first_wins_map(&nfc_values(danh_tu_src));
        dictionaries.ho_nguoi = first_wins_map(&nfc_values(ho_nguoi_src));
        dictionaries.hau_tu = first_wins_map(&nfc_values(hau_tu_src));
        dictionaries.luat_nhan = parse_luat_nhan(&nfc_values(luat_nhan_src));
```

Trong `crates/qt-core/src/lite.rs`, thêm `nfc_values` vào `use crate::dict::{…}`. Trong `load`, thay bốn dòng nạp đầu bằng:

```rust
        let han_viet = parse_han_viet(&nfc_values(&read("Resources/ChinesePhienAmWords.txt")?));
        let only_vietphrase = scan_dictionary(&nfc_values(&read("VietPhrase/VietPhrase.txt")?));
        let primary_names = scan_dictionary(&nfc_values(&read_optional("Names.txt")));
        let secondary_names = scan_dictionary(&nfc_values(&read_optional("Names2/123.txt")));
```

và ba tham số text kế tiếp thành `&nfc_values(&read_optional("Resources/Pronouns.txt"))`, `&nfc_values(&read_optional("Resources/HoNguoi.txt"))`, `&nfc_values(&read_optional("Resources/HauTu.txt"))`, `&nfc_values(&read_optional("LuatNhan.txt"))`. Trong `build`, bọc tương tự: `parse_han_viet(&nfc_values(han_viet_src))`, `scan_dictionary(&nfc_values(names_src))`, `scan_dictionary(&nfc_values(names2_src))`, `scan_dictionary(&nfc_values(vietphrase_src))`, và `&nfc_values(pronouns_src)`, `&nfc_values(ho_nguoi_src)`, `&nfc_values(hau_tu_src)`, `&nfc_values(luat_nhan_src)`.

Thêm unit test ở cuối `crates/qt-core/src/dict.rs`:

```rust
#[cfg(test)]
mod nfc_tests {
    use super::*;

    #[test]
    fn nfc_values_normalizes_meanings_but_not_keys() {
        // U+F928 là chữ Hán tương thích: NFC sẽ đổi nó, nên key phải giữ nguyên.
        let source = "也=cu\u{0303}ng\n\u{F928}=x\r\n";
        assert_eq!(nfc_values(source), "也=c\u{0169}ng\n\u{F928}=x\r\n");
    }

    #[test]
    fn nfc_values_borrows_when_already_nfc() {
        assert!(matches!(nfc_values("也=cũng\n"), Cow::Borrowed(_)));
    }
}
```

- [ ] **Step 6: Chạy test**

Run: `cargo test -p qt-core --test nfc && cargo test -p qt-core nfc_tests`
Expected: PASS cả 5 test.

- [ ] **Step 7: Chạy toàn bộ test các crate dùng engine**

Run: `cargo test -p qt-core -p qt-api -p qt-cli`
Expected: PASS. Test nào hỏng vì đáp án cũ ở dạng NFD thì sửa đáp án sang NFC và ghi tên test vào message commit. Không sửa test vì lý do khác.

- [ ] **Step 8: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/Cargo.toml Cargo.lock crates/qt-core/src/dict.rs crates/qt-core/src/lite.rs crates/qt-core/tests/nfc.rs
git commit -m "fix(core): chuẩn hoá NFC nghĩa từ điển khi nạp — QT2025 có 11.071 dòng VietPhrase dấu tách rời"
```

---

### Task 2: Dữ liệu mặc định, khung module, phần số

**Files:**
- Create: `crates/qt-core/data/QuyTac.txt`, `crates/qt-core/data/QuyTac-giong-muot.txt`, `crates/qt-core/data/TuLoai.txt`
- Create: `crates/qt-core/src/quy_tac/mod.rs`, `crates/qt-core/src/quy_tac/numbers.rs`
- Modify: `crates/qt-core/src/lib.rs` (khối `mod`)

**Interfaces:**
- Produces (trong `crate::quy_tac::numbers`): `pub(crate) const NUM_CHARS: &str`, `pub(crate) const DIGIT_WORDS: [&str; 10]`, `pub(crate) fn zh_value(s: &str) -> Option<i64>`, `pub(crate) fn number_words(n: i64) -> String`, `pub(crate) fn number_ordinal(n: i64) -> String`, `pub(crate) fn num_text(value: i64, digit: bool, how: &str) -> String`.

- [ ] **Step 1: Chép dữ liệu**

```bash
mkdir -p crates/qt-core/data
cp docs/superpowers/specs/2026-11-06-quy-tac/QuyTac.txt crates/qt-core/data/QuyTac.txt
cp docs/superpowers/specs/2026-11-06-quy-tac/QuyTac-giong-muot.txt crates/qt-core/data/QuyTac-giong-muot.txt
cp docs/superpowers/specs/2026-11-06-quy-tac/tag_fix.tsv crates/qt-core/data/TuLoai.txt
grep -v "^\s*#" crates/qt-core/data/QuyTac.txt | grep -c " = "
```

Expected: `204` (ba dòng comment cũng chứa " = " nên phải lọc `#` trước khi đếm).

- [ ] **Step 2: Khung module**

Tạo `crates/qt-core/src/quy_tac/mod.rs`:

```rust
//! Quy Tắc: đảo trật tự và chọn nghĩa trên token sau khi convert.
//! Spec: docs/superpowers/specs/2026-11-06-quy-tac-design.md. Mặc định tắt.
#![allow(dead_code)] // Task 8 nối vào engine rồi gỡ dòng này.

mod numbers;
```

Trong `crates/qt-core/src/lib.rs`, thêm `mod quy_tac;` vào khối `mod` (sau `mod number;`).

- [ ] **Step 3: Viết test hỏng cho phần số**

Tạo `crates/qt-core/src/quy_tac/numbers.rs` chỉ với phần test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_chinese_numbers_like_the_prototype() {
        assert_eq!(zh_value("三千五"), Some(3500));
        assert_eq!(zh_value("二〇一九"), Some(2019));
        assert_eq!(zh_value("三五"), Some(35));
        assert_eq!(zh_value("十五"), Some(15));
        assert_eq!(zh_value("二十四"), Some(24));
        assert_eq!(zh_value("三百五十"), Some(350));
        assert_eq!(zh_value("两百"), Some(200));
        assert_eq!(zh_value("十万"), Some(100_000));
        assert_eq!(zh_value("一万三千"), Some(13_000));
        assert_eq!(zh_value("2015"), Some(2015));
        assert_eq!(zh_value("三点"), None);
    }

    #[test]
    fn reads_numbers_in_vietnamese_words() {
        assert_eq!(number_words(0), "không");
        assert_eq!(number_words(15), "mười lăm");
        assert_eq!(number_words(21), "hai mươi mốt");
        assert_eq!(number_words(24), "hai mươi tư");
        assert_eq!(number_words(105), "một trăm lẻ năm");
        assert_eq!(number_words(350), "ba trăm năm mươi");
        assert_eq!(number_words(2019), "hai ngàn mười chín");
        assert_eq!(number_words(13_000), "một vạn ba ngàn");
        assert_eq!(number_ordinal(1), "nhất");
        assert_eq!(number_ordinal(4), "tư");
        assert_eq!(number_ordinal(3), "ba");
    }

    #[test]
    fn num_text_follows_filters() {
        assert_eq!(num_text(4, false, "thứ"), "tư");
        assert_eq!(num_text(4, true, "thứ"), "4");
        assert_eq!(num_text(3, false, "ngày"), "mùng ba");
        assert_eq!(num_text(15, false, "ngày"), "mười lăm");
        assert_eq!(num_text(50, false, "số"), "50");
        assert_eq!(num_text(50, false, ""), "năm mươi");
        assert_eq!(num_text(50, true, ""), "50");
    }
}
```

- [ ] **Step 4: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::numbers`
Expected: FAIL biên dịch, báo `cannot find function zh_value`.

- [ ] **Step 5: Viết phần số**

Thêm vào đầu `crates/qt-core/src/quy_tac/numbers.rs` (trên khối test):

```rust
//! Số cho Quy Tắc: giá trị số Hán và cách đọc ra chữ.
//!
//! Port nguyên từ bản thử (`qtlib.py`) để output khớp từng chữ. Không dùng
//! chung `crate::number`: cách đọc ở đây là vạn / ức / ngàn, "lẻ", "mốt",
//! "lăm", "tư", đúng như bản biên tập viết.

/// Ký tự được coi là chữ số Hán trong mẫu `[số]`.
pub(crate) const NUM_CHARS: &str = "零〇一二三四五六七八九十百千万亿两";

const WORDS: [&str; 11] = [
    "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín", "mười",
];

/// Đọc từng chữ số (phần thập phân, năm đọc rời).
pub(crate) const DIGIT_WORDS: [&str; 10] =
    ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];

fn digit_value(c: char) -> Option<i64> {
    Some(match c {
        '零' | '〇' => 0,
        '一' => 1,
        '二' | '两' => 2,
        '三' => 3,
        '四' => 4,
        '五' => 5,
        '六' => 6,
        '七' => 7,
        '八' => 8,
        '九' => 9,
        _ => return None,
    })
}

fn unit_value(c: char) -> Option<i64> {
    Some(match c {
        '十' => 10,
        '百' => 100,
        '千' => 1_000,
        '万' => 10_000,
        '亿' => 100_000_000,
        _ => return None,
    })
}

/// Giá trị của số Hán hoặc số Ả Rập. Chuỗi toàn chữ số đơn (二〇一九) đọc rời
/// từng chữ. `三千五` = 3500: chữ số cuối đứng sau đơn vị mang đơn vị kế dưới.
pub(crate) fn zh_value(s: &str) -> Option<i64> {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        return s.parse().ok();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > 1 && chars.iter().all(|&c| digit_value(c).is_some()) {
        let mut value: i64 = 0;
        for &c in &chars {
            value = value.checked_mul(10)?.checked_add(digit_value(c)?)?;
        }
        return Some(value);
    }
    let (mut total, mut section, mut num, mut last_unit) = (0i64, 0i64, 0i64, 1i64);
    for &c in &chars {
        if let Some(digit) = digit_value(c) {
            num = digit;
        } else if c == '万' || c == '亿' {
            let unit = unit_value(c)?;
            section = section.checked_add(num)?.checked_mul(unit)?;
            total = total.checked_add(section)?;
            section = 0;
            num = 0;
            last_unit = unit;
        } else if let Some(unit) = unit_value(c) {
            section = section.checked_add(if num == 0 { 1 } else { num }.checked_mul(unit)?)?;
            num = 0;
            last_unit = unit;
        } else {
            return None;
        }
    }
    if num != 0
        && last_unit >= 100
        && chars.len() >= 2
        && unit_value(chars[chars.len() - 2]).is_some()
    {
        num = num.checked_mul(last_unit / 10)?;
    }
    total.checked_add(section)?.checked_add(num)
}

fn below_thousand(n: i64) -> String {
    let (hundreds, rest) = (n / 100, n % 100);
    let (tens, ones) = (rest / 10, rest % 10);
    let mut parts: Vec<&str> = Vec::new();
    if hundreds > 0 {
        parts.push(WORDS[hundreds as usize]);
        parts.push("trăm");
        if rest > 0 && tens == 0 {
            parts.push("lẻ");
        }
    }
    if tens == 1 {
        parts.push("mười");
    } else if tens > 1 {
        parts.push(WORDS[tens as usize]);
        parts.push("mươi");
    }
    if ones > 0 {
        parts.push(match ones {
            1 if tens >= 2 => "mốt",
            5 if tens >= 1 => "lăm",
            4 if tens >= 2 => "tư",
            _ => WORDS[ones as usize],
        });
    }
    parts.join(" ")
}

/// Đọc số ra chữ: ức / vạn / ngàn như bản biên tập.
pub(crate) fn number_words(n: i64) -> String {
    if n <= 0 {
        return "không".to_string();
    }
    let mut n = n;
    let mut parts: Vec<String> = Vec::new();
    for (unit, name) in [(100_000_000, "ức"), (10_000, "vạn"), (1_000, "ngàn")] {
        if n >= unit {
            parts.push(number_words(n / unit));
            parts.push(name.to_string());
            n %= unit;
        }
    }
    if n > 0 {
        parts.push(below_thousand(n));
    }
    parts.join(" ")
}

/// Số thứ tự: nhất, hai, tư; còn lại đọc như số đếm.
pub(crate) fn number_ordinal(n: i64) -> String {
    match n {
        1 => "nhất".to_string(),
        2 => "hai".to_string(),
        4 => "tư".to_string(),
        _ => number_words(n),
    }
}

/// Giá trị số ra chữ theo bộ lọc khuôn: `thứ`, `ngày`, `số`, hoặc rỗng (giữ kiểu nguồn).
pub(crate) fn num_text(value: i64, digit: bool, how: &str) -> String {
    match how {
        "thứ" if digit => value.to_string(),
        "thứ" => number_ordinal(value),
        "ngày" => {
            let text = if digit { value.to_string() } else { number_words(value) };
            if value <= 10 {
                format!("mùng {text}")
            } else {
                text
            }
        }
        "số" => value.to_string(),
        _ if digit => value.to_string(),
        _ => number_words(value),
    }
}
```

Trong `mod.rs` thì `mod numbers;` đã có.

- [ ] **Step 6: Chạy test**

Run: `cargo test -p qt-core quy_tac::numbers`
Expected: PASS 3 test.

- [ ] **Step 7: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/data crates/qt-core/src/quy_tac crates/qt-core/src/lib.rs
git commit -m "feat(core): khung Quy Tắc — dữ liệu mặc định, đọc số Hán và đọc số ra chữ"
```

---

### Task 3: Đọc file luật (`program.rs`)

**Files:**
- Create: `crates/qt-core/src/quy_tac/program.rs`
- Modify: `crates/qt-core/src/quy_tac/mod.rs`

**Interfaces:**
- Consumes: `numbers::NUM_CHARS`.
- Produces:
  - `pub struct QuyTacError { pub line: usize, pub message: String }` (`Display`: `QuyTac dòng {line}: {message}`) trong `mod.rs`.
  - Trong `program.rs`: `pub(crate) type Class = FxHashMap<String, String>`; `enum LeftKind { Ve, VeDong, MenhDe }` với `fn name(self) -> &'static str`; `enum ElKind { Start, End, Left(LeftKind), Gap, Many(String), Slot(String), Lit(Vec<String>) }`; `struct El { kind, opt, neg }`; `enum RefKey { Index(usize), Name(String) }`; `enum Part { Text(String), Ref { key: RefKey, filter: Option<String> } }`; `struct Rule { line, pre, pat, post, tmpl: Vec<Part>, result: String, needs: Vec<Vec<String>> }`; `struct Section { name, rounds, result, rules: Vec<Rule>, first: FxHashMap<char, Vec<usize>>, left: FxHashMap<char, Vec<usize>>, generic: Vec<usize> }`; `struct CharRule { line, regex: Regex, slots: Vec<String>, tmpl: Vec<Part>, result: String }`; `struct Program { classes: FxHashMap<String, Class>, char_rules: Vec<CharRule>, sections: Vec<Section> }`; `Program::parse(&str) -> Result<Program, QuyTacError>`. Mọi trường `pub(crate)`.

- [ ] **Step 1: Thêm lỗi vào `mod.rs`**

Thêm vào `crates/qt-core/src/quy_tac/mod.rs` (sau `mod numbers;`):

```rust
mod program;

/// Lỗi khi đọc file Quy Tắc; `line` là số dòng (tính từ 1), 0 khi không gắn dòng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuyTacError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for QuyTacError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(formatter, "QuyTac dòng {}: {}", self.line, self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for QuyTacError {}
```

- [ ] **Step 2: Viết test hỏng**

Tạo `crates/qt-core/src/quy_tac/program.rs` chỉ với phần test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[lớp]
vị-trí: 上=trên 中央=giữa
lượng-từ: 层=tầng 个=

[số]
{số}点半 = {1} giờ rưỡi -> thời-gian
第{số}(章|回) = Chương {1|số}: -> lit

[của 3]
{danh-từ} {vị-trí} | !{danh-từ} = {vị-trí} {danh-từ} -> nơi-cụm
这个 {danh-từ}+ | !的 = {1} này
{vế-động} 的时候 = khi {vế-động}

[nghĩa]
^ | 可是 | = Nhưng -> lit
";

    #[test]
    fn parses_classes_sections_and_indexes() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.classes["vị-trí"]["中央"], "giữa");
        assert_eq!(program.classes["lượng-từ"]["个"], "");
        assert_eq!(program.sections.len(), 2);
        let cua = &program.sections[0];
        assert_eq!((cua.name.as_str(), cua.rounds), ("của", 3));
        assert_eq!(cua.generic, vec![0]);
        assert_eq!(cua.first[&'这'], vec![1]);
        assert_eq!(cua.left[&'的'], vec![2]);
        assert_eq!(cua.rules[0].result, "nơi-cụm");
        assert_eq!(cua.rules[1].result, "cụm");
        assert_eq!(cua.rules[0].post.len(), 1);
        assert!(cua.rules[0].post[0].neg);
        let nghia = &program.sections[1];
        assert_eq!(nghia.rules[0].pre[0].kind, ElKind::Start);
        assert_eq!(nghia.rules[0].result, "lit");
    }

    #[test]
    fn char_rules_sort_by_fixed_characters_and_match() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.char_rules.len(), 2);
        // " 第(章|回)" dài hơn " 点半" nên đứng trước.
        let chapter = &program.char_rules[0];
        assert_eq!(chapter.result, "lit");
        let caps = chapter.regex.captures("第 12 章").unwrap();
        assert_eq!(&caps[1], "12");
        assert!(program.char_rules[1].regex.is_match("三点半"));
    }

    #[test]
    fn templates_are_split_into_parts() {
        let program = Program::parse(SAMPLE).unwrap();
        let rule = &program.sections[0].rules[1];
        assert_eq!(
            rule.tmpl,
            vec![
                Part::Ref { key: RefKey::Index(1), filter: None },
                Part::Text(" này".to_string()),
            ]
        );
        let chapter = &program.char_rules[0];
        assert!(chapter.tmpl.contains(&Part::Ref {
            key: RefKey::Index(1),
            filter: Some("số".to_string()),
        }));
    }

    #[test]
    fn rejects_bad_lines_with_line_numbers() {
        let error = Program::parse("[của]\n{danh-từ} = {2}\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("{2}"), "{}", error.message);
        let error = Program::parse("[của]\n{không-có} = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
        let error = Program::parse("[của]\nabc\n").unwrap_err();
        assert!(error.message.contains("thiếu ' = '"), "{}", error.message);
        let error = Program::parse("abc = x\n").unwrap_err();
        assert_eq!(error.line, 1);
        let error = Program::parse("[số]\n{lạ}点 = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
    }

    #[test]
    fn default_rule_files_parse() {
        let base = include_str!("../../data/QuyTac.txt");
        let program = Program::parse(base).unwrap();
        let rules: usize = program.sections.iter().map(|s| s.rules.len()).sum();
        assert_eq!(rules + program.char_rules.len(), 204);
        let extra = include_str!("../../data/QuyTac-giong-muot.txt");
        Program::parse(&format!("{base}\n{extra}")).unwrap();
    }
}
```

- [ ] **Step 3: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::program`
Expected: FAIL biên dịch, báo `cannot find type Program`.

- [ ] **Step 4: Viết bộ đọc**

Thêm vào đầu `crates/qt-core/src/quy_tac/program.rs` (trên khối test):

```rust
//! Đọc file Quy Tắc: lớp từ (`[lớp]`), nhóm `[số]` quét ký tự, và các nhóm quy
//! tắc chạy trên token. Cú pháp ở spec mục 5. Ngữ nghĩa theo bản thử
//! `quytac.py` (`Program`, `compile_char_rules`).

use std::cmp::Reverse;
use std::sync::LazyLock;

use regex::{Captures, Regex};
use rustc_hash::FxHashMap;

use super::numbers::NUM_CHARS;
use super::QuyTacError;

/// Một lớp từ: chữ Hán → nghĩa thay trong khuôn (có thể rỗng).
pub(crate) type Class = FxHashMap<String, String>;

/// Nhãn khoá có sẵn, ngoài các lớp từ khai báo trong `[lớp]`.
pub(crate) const BUILTIN_SLOTS: [&str; 21] = [
    "đại-từ",
    "tên",
    "người",
    "danh-từ",
    "cụm",
    "động-từ",
    "danh-động",
    "hệ-từ",
    "có-đích",
    "tính-từ",
    "thành-ngữ",
    "số",
    "số-lượng",
    "thời-gian",
    "nơi",
    "phó-từ",
    "dấu",
    "chỉ-định",
    "nơi-cụm",
    "nơi-từ",
    "động-từ-tại",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LeftKind {
    Ve,
    VeDong,
    MenhDe,
}

impl LeftKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            LeftKind::Ve => "vế",
            LeftKind::VeDong => "vế-động",
            LeftKind::MenhDe => "mệnh-đề",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ElKind {
    Start,
    End,
    Left(LeftKind),
    Gap,
    Many(String),
    Slot(String),
    Lit(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct El {
    pub(crate) kind: ElKind,
    pub(crate) opt: bool,
    pub(crate) neg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefKey {
    /// `{1}`, `{2}`… theo thứ tự khoá trong mẫu (tính từ 1).
    Index(usize),
    /// `{danh-từ}`: khoá đầu tiên mang tên đó.
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Part {
    Text(String),
    Ref { key: RefKey, filter: Option<String> },
}

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) line: usize,
    pub(crate) pre: Vec<El>,
    pub(crate) pat: Vec<El>,
    pub(crate) post: Vec<El>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    /// Chữ cố định bắt buộc của mẫu: đoạn không chứa thì khỏi thử.
    pub(crate) needs: Vec<Vec<String>>,
}

#[derive(Debug)]
pub(crate) struct Section {
    pub(crate) name: String,
    pub(crate) rounds: usize,
    pub(crate) result: String,
    pub(crate) rules: Vec<Rule>,
    /// Quy tắc mở đầu bằng chữ cố định, theo chữ đầu.
    pub(crate) first: FxHashMap<char, Vec<usize>>,
    /// Quy tắc mở đầu bằng `{vế}`…, theo chữ đầu của chữ cố định ngay sau.
    pub(crate) left: FxHashMap<char, Vec<usize>>,
    pub(crate) generic: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct CharRule {
    pub(crate) line: usize,
    pub(crate) regex: Regex,
    pub(crate) slots: Vec<String>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    sort_key: usize,
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) classes: FxHashMap<String, Class>,
    pub(crate) char_rules: Vec<CharRule>,
    pub(crate) sections: Vec<Section>,
}

static CONTEXT_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)\|(?:\s|$)").unwrap());
static TEMPLATE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([^{}|]+)(?:\|([^{}]+))?\}").unwrap());
static CHAR_SLOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{([^{}]+)\}").unwrap());
/// Độ ưu tiên luật `[số]` như bản thử: bỏ khoá, tiêu đề, khuôn, ký tự tuỳ chọn
/// rồi đếm ký tự còn lại.
static SORT_STRIP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}]*\}|\[số\]| = .*|.\?").unwrap());

fn err(line: usize, message: impl Into<String>) -> QuyTacError {
    QuyTacError {
        line,
        message: message.into(),
    }
}

impl Program {
    pub(crate) fn parse(source: &str) -> Result<Program, QuyTacError> {
        #[derive(PartialEq)]
        enum Mode {
            Outside,
            Classes,
            Chars,
            Rules,
        }
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        let mut char_lines: Vec<(usize, String)> = Vec::new();
        let mut sections: Vec<Section> = Vec::new();
        let mut mode = Mode::Outside;
        for (index, raw) in source.lines().enumerate() {
            let line_no = index + 1;
            let raw = if index == 0 {
                raw.trim_start_matches('\u{feff}')
            } else {
                raw
            };
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                let header: Vec<&str> = line
                    .trim_matches(|c| c == '[' || c == ']')
                    .split_whitespace()
                    .collect();
                let Some(&name) = header.first() else {
                    return Err(err(line_no, "tiêu đề nhóm rỗng"));
                };
                mode = match name {
                    "lớp" => Mode::Classes,
                    "số" => Mode::Chars,
                    _ => {
                        let rounds = match header.get(1) {
                            Some(value) => value.parse::<usize>().map_err(|_| {
                                err(line_no, format!("số lượt không hợp lệ: {value}"))
                            })?,
                            None => 1,
                        };
                        let result = header.get(2).copied().unwrap_or("cụm").to_string();
                        sections.push(Section::new(name, rounds.max(1), result));
                        Mode::Rules
                    }
                };
                continue;
            }
            match mode {
                Mode::Outside => return Err(err(line_no, "dòng nằm ngoài mọi nhóm")),
                Mode::Classes => parse_class(line, line_no, &mut classes)?,
                Mode::Chars => char_lines.push((line_no, line.to_string())),
                Mode::Rules => {
                    let section = sections.last_mut().expect("Mode::Rules luôn có nhóm");
                    let rule = parse_rule(line, line_no, &section.result, &classes)?;
                    section.add(rule);
                }
            }
        }
        let mut char_rules = char_lines
            .iter()
            .map(|(line_no, line)| compile_char_rule(line, *line_no, &classes))
            .collect::<Result<Vec<_>, _>>()?;
        // Nhiều chữ cố định hơn đi trước; hoà thì giữ thứ tự file (sort ổn định).
        char_rules.sort_by_key(|rule| Reverse(rule.sort_key));
        for section in &mut sections {
            section.finish();
        }
        Ok(Program {
            classes,
            char_rules,
            sections,
        })
    }
}

fn parse_class(
    line: &str,
    line_no: usize,
    classes: &mut FxHashMap<String, Class>,
) -> Result<(), QuyTacError> {
    let (name, members) = line
        .split_once(':')
        .ok_or_else(|| err(line_no, "dòng lớp thiếu ':'"))?;
    let table = classes.entry(name.trim().to_string()).or_default();
    for member in members.split_whitespace() {
        let (zh, vi) = member.split_once('=').unwrap_or((member, ""));
        table.insert(zh.to_string(), vi.replace('_', " "));
    }
    Ok(())
}

fn slot_name(
    name: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<String, QuyTacError> {
    if classes.contains_key(name) || BUILTIN_SLOTS.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(err(line_no, format!("khoá không có: {{{name}}}")))
    }
}

fn parse_elements(
    text: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<Vec<El>, QuyTacError> {
    let mut out = Vec::new();
    for part in text.split_whitespace() {
        let neg = part.starts_with('!');
        let mut part = if neg { &part[1..] } else { part };
        let opt = part.ends_with('?');
        if opt {
            part = &part[..part.len() - 1];
        }
        let kind = match part {
            "^" => ElKind::Start,
            "$" => ElKind::End,
            "{vế}" => ElKind::Left(LeftKind::Ve),
            "{vế-động}" => ElKind::Left(LeftKind::VeDong),
            "{mệnh-đề}" => ElKind::Left(LeftKind::MenhDe),
            "{đoạn}" => ElKind::Gap,
            _ if part.starts_with('{') && part.ends_with("}+") => {
                ElKind::Many(slot_name(&part[1..part.len() - 2], line_no, classes)?)
            }
            _ if part.starts_with('{') && part.ends_with('}') => {
                ElKind::Slot(slot_name(&part[1..part.len() - 1], line_no, classes)?)
            }
            _ => {
                let mut lits: Vec<String> = Vec::new();
                for lit in part.trim_matches(|c| c == '(' || c == ')').split('|') {
                    if lit.is_empty() {
                        return Err(err(line_no, format!("chữ cố định rỗng trong {part}")));
                    }
                    if !lits.iter().any(|known| known == lit) {
                        lits.push(lit.to_string());
                    }
                }
                ElKind::Lit(lits)
            }
        };
        out.push(El { kind, opt, neg });
    }
    Ok(out)
}

fn parse_template(
    source: &str,
    cap_names: &[String],
    line_no: usize,
) -> Result<Vec<Part>, QuyTacError> {
    let mut parts = Vec::new();
    let mut last = 0;
    for caps in TEMPLATE_REF.captures_iter(source) {
        let whole = caps.get(0).expect("nhóm 0 luôn có");
        if whole.start() > last {
            parts.push(Part::Text(source[last..whole.start()].to_string()));
        }
        let key = &caps[1];
        let key = match key.parse::<usize>() {
            Ok(index) => {
                if index == 0 || index > cap_names.len() {
                    return Err(err(
                        line_no,
                        format!(
                            "khuôn tham chiếu {{{index}}} nhưng mẫu chỉ có {} khoá",
                            cap_names.len()
                        ),
                    ));
                }
                RefKey::Index(index)
            }
            Err(_) => {
                if !cap_names.iter().any(|name| name == key) {
                    return Err(err(
                        line_no,
                        format!("khuôn tham chiếu {{{key}}} nhưng mẫu không có khoá đó"),
                    ));
                }
                RefKey::Name(key.to_string())
            }
        };
        parts.push(Part::Ref {
            key,
            filter: caps.get(2).map(|m| m.as_str().to_string()),
        });
        last = whole.end();
    }
    if last < source.len() {
        parts.push(Part::Text(source[last..].to_string()));
    }
    Ok(parts)
}

fn parse_rule(
    line: &str,
    line_no: usize,
    default_result: &str,
    classes: &FxHashMap<String, Class>,
) -> Result<Rule, QuyTacError> {
    let (lhs, rhs) = line
        .split_once(" = ")
        .ok_or_else(|| err(line_no, "thiếu ' = '"))?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, default_result.to_string()),
    };
    let pieces: Vec<&str> = CONTEXT_SPLIT.split(lhs).collect();
    let (pre, pat, post) = match pieces.as_slice() {
        [pat] => ("", *pat, ""),
        [pat, post] => ("", *pat, *post),
        [pre, pat, post] => (*pre, *pat, *post),
        _ => return Err(err(line_no, "quá nhiều dấu '|' ngữ cảnh")),
    };
    let pre = parse_elements(pre, line_no, classes)?;
    let pat = parse_elements(pat, line_no, classes)?;
    let post = parse_elements(post, line_no, classes)?;
    if pat.is_empty() {
        return Err(err(line_no, "mẫu rỗng"));
    }
    let mut cap_names: Vec<String> = Vec::new();
    for (index, el) in pat.iter().enumerate() {
        let next_is_lit = matches!(pat.get(index + 1).map(|e| &e.kind), Some(ElKind::Lit(_)));
        match &el.kind {
            ElKind::Left(kind) => {
                if index != 0 || !next_is_lit {
                    return Err(err(
                        line_no,
                        format!(
                            "{{{}}} phải đứng đầu mẫu và theo sau là chữ cố định",
                            kind.name()
                        ),
                    ));
                }
                cap_names.push(kind.name().to_string());
            }
            ElKind::Gap => {
                if !next_is_lit {
                    return Err(err(line_no, "{đoạn} phải theo sau là chữ cố định"));
                }
                cap_names.push("đoạn".to_string());
            }
            ElKind::Slot(name) | ElKind::Many(name) => cap_names.push(name.clone()),
            _ => {}
        }
    }
    let tmpl = parse_template(tmpl_src.trim(), &cap_names, line_no)?;
    let needs = pat
        .iter()
        .filter(|el| !el.opt && !el.neg)
        .filter_map(|el| match &el.kind {
            ElKind::Lit(lits) => Some(lits.clone()),
            _ => None,
        })
        .collect();
    Ok(Rule {
        line: line_no,
        pre,
        pat,
        post,
        tmpl,
        result,
        needs,
    })
}

fn compile_char_rule(
    line: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<CharRule, QuyTacError> {
    let (lhs, rhs) = line
        .split_once(" = ")
        .ok_or_else(|| err(line_no, "thiếu ' = '"))?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, "số-cụm".to_string()),
    };
    // Nhóm (a|b) của người viết không chiếm chỗ đánh số của khoá.
    let mut grouped = String::with_capacity(lhs.len() + 8);
    let mut chars = lhs.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' && chars.peek() != Some(&'?') {
            grouped.push_str("(?:");
        } else {
            grouped.push(c);
        }
    }
    let mut slots: Vec<String> = Vec::new();
    let mut failure: Option<String> = None;
    let pattern = CHAR_SLOT.replace_all(&grouped, |caps: &Captures| {
        let spec = &caps[1];
        let (name, bounds) = spec.split_once(':').unwrap_or((spec, ""));
        let (lo, hi) = bounds.split_once('-').unwrap_or((bounds, ""));
        slots.push(name.to_string());
        match (name, bounds.is_empty()) {
            ("số", false) => format!(r"\s*((?:[{NUM_CHARS}]|\d){{{lo},{hi}}})\s*"),
            ("số", true) => format!(r"\s*([{NUM_CHARS}]+|\d+(?:\.\d+)?)\s*"),
            // Năm lịch: chữ số Hán đọc từng chữ, hoặc số Ả Rập trông như năm.
            // 500年, 5000年 là "500 năm", không phải năm lịch.
            ("năm", false) if lo == "3" => {
                r"\s*([零〇一二三四五六七八九]{3,4}|1[5-9]\d\d|20\d\d)\s*".to_string()
            }
            ("năm", false) => format!(r"\s*([零〇一二三四五六七八九\d]{{{lo},{hi}}})\s*"),
            ("năm", true) => r"\s*([零〇一二三四五六七八九\d]+)\s*".to_string(),
            _ => match classes.get(name) {
                Some(class) => {
                    let mut keys: Vec<&String> = class.keys().collect();
                    keys.sort_by(|a, b| {
                        b.chars()
                            .count()
                            .cmp(&a.chars().count())
                            .then_with(|| a.cmp(b))
                    });
                    let alternatives: Vec<String> =
                        keys.iter().map(|key| regex::escape(key)).collect();
                    format!("({})", alternatives.join("|"))
                }
                None => {
                    failure.get_or_insert_with(|| format!("khoá không có: {{{name}}}"));
                    String::new()
                }
            },
        }
    });
    if let Some(message) = failure {
        return Err(err(line_no, message));
    }
    let regex = Regex::new(&pattern)
        .map_err(|error| err(line_no, format!("mẫu số không hợp lệ: {error}")))?;
    let tmpl = parse_template(tmpl_src.trim(), &slots, line_no)?;
    let sort_key = SORT_STRIP
        .replace_all(&format!("[số] {line}"), "")
        .chars()
        .count();
    Ok(CharRule {
        line: line_no,
        regex,
        slots,
        tmpl,
        result,
        sort_key,
    })
}

fn first_chars(lits: &[String]) -> Vec<char> {
    let mut out = Vec::new();
    for c in lits.iter().filter_map(|lit| lit.chars().next()) {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

impl Section {
    fn new(name: &str, rounds: usize, result: String) -> Section {
        Section {
            name: name.to_string(),
            rounds,
            result,
            rules: Vec::new(),
            first: FxHashMap::default(),
            left: FxHashMap::default(),
            generic: Vec::new(),
        }
    }

    fn add(&mut self, rule: Rule) {
        let index = self.rules.len();
        let head = &rule.pat[0];
        match &head.kind {
            ElKind::Left(_) => {
                if let ElKind::Lit(lits) = &rule.pat[1].kind {
                    for first in first_chars(lits) {
                        self.left.entry(first).or_default().push(index);
                    }
                }
            }
            ElKind::Lit(lits) if !head.opt => {
                for first in first_chars(lits) {
                    self.first.entry(first).or_default().push(index);
                }
            }
            _ => self.generic.push(index),
        }
        self.rules.push(rule);
    }

    /// Trong mỗi danh sách: mẫu dài hơn trước, rồi nhiều chữ cố định hơn; hoà giữ thứ tự file.
    fn finish(&mut self) {
        let rules = &self.rules;
        let key = |index: &usize| {
            let rule = &rules[*index];
            let lits = rule
                .pat
                .iter()
                .filter(|el| matches!(el.kind, ElKind::Lit(_)))
                .count();
            (Reverse(rule.pat.len()), Reverse(lits))
        };
        for list in self.first.values_mut().chain(self.left.values_mut()) {
            list.sort_by_key(key);
        }
        self.generic.sort_by_key(key);
    }
}
```

- [ ] **Step 5: Chạy test**

Run: `cargo test -p qt-core quy_tac::program`
Expected: PASS 5 test. Nếu `default_rule_files_parse` báo lỗi một dòng của `QuyTac.txt` thì đó là lỗi dữ liệu thật (khuôn tham chiếu khoá không có). Sửa dòng đó trong **cả** `crates/qt-core/data/QuyTac.txt` và `docs/superpowers/specs/2026-11-06-quy-tac/QuyTac.txt`, rồi ghi vào message commit.

- [ ] **Step 6: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src/quy_tac
git commit -m "feat(core): đọc file Quy Tắc — lớp từ, nhóm [số], nhóm quy tắc, kiểm khuôn theo số dòng"
```

---

### Task 4: Bảng từ loại (`tags.rs`)

**Files:**
- Create: `crates/qt-core/src/quy_tac/tags.rs`
- Modify: `crates/qt-core/src/quy_tac/mod.rs` (`mod tags;`)

**Interfaces:**
- Consumes: `crate::compact::CompactDict` (`first_wins`, `DictionaryLookup::{get, max_key_len}`).
- Produces: `pub(crate) struct PosLexicon` với `build(jieba_dict: &str, tu_loai: &str) -> PosLexicon`, `get(&self, &str) -> Option<&str>`, `segment(&self, &str) -> Vec<(String, &str)>` (nhãn `"?"` khi không biết), `phrase(&self, &str) -> &str`.

- [ ] **Step 1: Viết test hỏng**

Tạo `crates/qt-core/src/quy_tac/tags.rs` chỉ với phần test, và thêm `mod tags;` vào `mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const DICT: &str = "美丽 10 ns\n少女 5 n\n看 3 v\n在 9 p\n家 4 n\n的 9 uj\n";

    #[test]
    fn tu_loai_overrides_jieba_tags() {
        let pos = PosLexicon::build(DICT, "美丽\ta\n");
        assert_eq!(pos.get("美丽"), Some("a"));
        assert_eq!(pos.get("少女"), Some("n"));
        assert_eq!(pos.get("没有"), None);
    }

    #[test]
    fn segments_longest_first() {
        let pos = PosLexicon::build(DICT, "");
        let parts: Vec<(String, &str)> = pos.segment("少女们");
        assert_eq!(
            parts,
            vec![("少女".to_string(), "n"), ("们".to_string(), "?")]
        );
    }

    #[test]
    fn phrase_infers_tags_like_the_prototype() {
        let pos = PosLexicon::build(DICT, "");
        assert_eq!(pos.phrase("看着"), "v"); // bỏ đuôi 着 rồi tra
        assert_eq!(pos.phrase("少女们"), "n"); // từ cuối biết được
        assert_eq!(pos.phrase("在家"), "pp"); // giới từ + từ khác
        assert_eq!(pos.phrase("少女的"), "n"); // đuôi trợ từ: lấy từ trước
        assert_eq!(pos.phrase("们们"), "?");
    }
}
```

- [ ] **Step 2: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::tags`
Expected: FAIL biên dịch, báo `cannot find type PosLexicon`.

- [ ] **Step 3: Viết bảng từ loại**

Thêm vào đầu `crates/qt-core/src/quy_tac/tags.rs`:

```rust
//! Từ loại cho Quy Tắc: bảng kiểu jieba (`Resources/dict.txt`, dòng
//! `từ tần-số nhãn`) đè bằng `TuLoai.txt` (dòng `từ<TAB>nhãn`). Lưu gọn trong
//! `CompactDict` (~350 nghìn mục, vài MB).

use crate::compact::CompactDict;
use crate::dict::DictionaryLookup;

/// Trợ từ cuối được bỏ đi khi tra một cụm không có trong bảng.
const TRAILERS: &[char] = &['了', '着', '过', '吗', '呢', '啊', '吧', '呀', '嘛', '哦', '哈'];

pub(crate) struct PosLexicon {
    table: CompactDict,
}

impl PosLexicon {
    pub(crate) fn build(jieba_dict: &str, tu_loai: &str) -> PosLexicon {
        let fixes = tu_loai.lines().filter_map(|line| {
            let (word, tag) = line.trim_end_matches('\r').split_once('\t')?;
            (!word.is_empty() && !tag.is_empty()).then_some((word, tag))
        });
        let base = jieba_dict.lines().filter_map(|line| {
            let mut parts = line.split_whitespace();
            let (word, _frequency, tag) = (parts.next()?, parts.next()?, parts.next()?);
            parts.next().is_none().then_some((word, tag))
        });
        // TuLoai đứng trước nên thắng (first-wins).
        let (table, _) = CompactDict::first_wins(fixes.chain(base));
        PosLexicon { table }
    }

    pub(crate) fn get(&self, word: &str) -> Option<&str> {
        self.table.get(word)
    }

    /// Tách tham lam theo cụm dài nhất có trong bảng; ký tự lạ mang nhãn `"?"`.
    pub(crate) fn segment(&self, text: &str) -> Vec<(String, &str)> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = Vec::new();
        let mut buffer = String::new();
        let mut i = 0;
        while i < chars.len() {
            let cap = self.table.max_key_len(chars[i]).min(chars.len() - i);
            let mut found = None;
            for length in (1..=cap).rev() {
                buffer.clear();
                buffer.extend(&chars[i..i + length]);
                if let Some(tag) = self.table.get(&buffer) {
                    found = Some((length, tag));
                    break;
                }
            }
            match found {
                Some((length, tag)) => {
                    out.push((chars[i..i + length].iter().collect(), tag));
                    i += length;
                }
                None => {
                    out.push((chars[i].to_string(), "?"));
                    i += 1;
                }
            }
        }
        out
    }

    /// Nhãn của một token: tra thẳng; bỏ trợ từ cuối rồi tra; không có thì suy
    /// từ các từ con (giới từ đầu → "pp", có động từ → "v", đuôi trợ từ → từ
    /// trước, còn lại → từ cuối). Giống `Pos.phrase` của bản thử.
    pub(crate) fn phrase(&self, zh: &str) -> &str {
        if let Some(tag) = self.get(zh) {
            return tag;
        }
        let core = zh.trim_end_matches(TRAILERS);
        let core = if core.is_empty() { zh } else { core };
        if let Some(tag) = self.get(core) {
            return tag;
        }
        let tags: Vec<&str> = self
            .segment(core)
            .into_iter()
            .map(|(_, tag)| tag)
            .filter(|tag| *tag != "?")
            .collect();
        let Some(&last) = tags.last() else {
            return "?";
        };
        if tags[0].starts_with('p') && tags.len() > 1 {
            return "pp";
        }
        if tags.iter().any(|tag| tag.starts_with('v') && *tag != "vn") {
            return "v";
        }
        if last.starts_with('u') && tags.len() > 1 {
            return tags[tags.len() - 2];
        }
        last
    }
}
```

- [ ] **Step 4: Chạy test**

Run: `cargo test -p qt-core quy_tac::tags`
Expected: PASS 3 test.

- [ ] **Step 5: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src/quy_tac
git commit -m "feat(core): bảng từ loại Quy Tắc — jieba dict.txt đè bằng TuLoai.txt, suy nhãn cụm"
```

---

### Task 5: Chuẩn bị token (`prepare.rs`)

**Files:**
- Create: `crates/qt-core/src/quy_tac/prepare.rs`
- Modify: `crates/qt-core/src/quy_tac/mod.rs` (`mod prepare;`)

**Interfaces:**
- Consumes: `numbers::{zh_value, number_words, NUM_CHARS}`, `program::Class`, `tags::PosLexicon`, `crate::dict::DictionaryLookup`, `crate::first_meaning`.
- Produces:
  - `pub(crate) struct Token<'q> { zh: String, vi: String, tag: &'q str, val: Option<i64>, digit: bool }` với `Token::new(zh, vi, tag)`.
  - `pub(crate) enum Raw<'q> { Plain(String, String), Tagged(String, String, &'q str) }`.
  - `pub(crate) struct Lookups<'a> { names: &'a dyn DictionaryLookup, lexicon: &'a dyn DictionaryLookup }` với `name(&str) -> Option<&str>`, `word(&str) -> Option<&str>` (nghĩa đầu, đã trim), `is_name(&str) -> bool`.
  - `pub(crate) fn seg(text: &str, lk: &Lookups) -> Vec<(String, String)>`.
  - `pub(crate) fn prepare<'q>(raw: Vec<Raw<'q>>, pos: &'q PosLexicon, lk: &Lookups, classes: &FxHashMap<String, Class>) -> Vec<Token<'q>>`.

- [ ] **Step 1: Viết test hỏng**

Tạo `crates/qt-core/src/quy_tac/prepare.rs` chỉ với phần test, thêm `mod prepare;` vào `mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> FxHashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn plain(pairs: &[(&str, &str)]) -> Vec<Raw<'static>> {
        pairs.iter().map(|(z, v)| Raw::Plain(z.to_string(), v.to_string())).collect()
    }

    #[test]
    fn untie_splits_punctuation_and_trailing_de() {
        let pos = PosLexicon::build("她 9 r\n", "");
        let names = map(&[]);
        let lexicon = map(&[]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(
            untie(plain(&[("他走了。", "hắn đi nha.")]), &pos, &lk),
            plain(&[("他走了", "hắn đi nha"), ("。", ".")])
        );
        assert_eq!(
            untie(plain(&[("她的", "nàng"), ("手", "tay")]), &pos, &lk),
            plain(&[("她", "nàng"), ("的", ""), ("手", "tay")])
        );
    }

    #[test]
    fn unfreeze_only_when_meaning_keeps_source_order() {
        let pos = PosLexicon::build("", "");
        let names = map(&[]);
        let lexicon = map(&[
            ("美丽", "mỹ lệ"),
            ("少女", "thiếu nữ"),
            ("不好", "không tốt"),
            ("预感", "dự cảm"),
        ]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(
            untie(plain(&[("美丽的少女", "mỹ lệ thiếu nữ")]), &pos, &lk),
            plain(&[("美丽", "mỹ lệ"), ("的", ""), ("少女", "thiếu nữ")])
        );
        // Đã dịch tay theo trật tự tiếng Việt: giữ nguyên.
        assert_eq!(
            untie(plain(&[("不好的预感", "dự cảm không tốt")]), &pos, &lk),
            plain(&[("不好的预感", "dự cảm không tốt")])
        );
    }

    #[test]
    fn prepare_tags_names_numbers_and_measure_words() {
        let pos = PosLexicon::build("桌子 9 n\n", "");
        let names = map(&[("叶妩", "Diệp Vũ")]);
        let lexicon = map(&[]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        classes.insert("lượng-từ".to_string(), map(&[("个", "cái")]));
        let toks = prepare(
            plain(&[("叶妩", "Diệp Vũ"), ("三个", "ba cái"), ("三百五十", "350"), ("12", "12"), ("桌子", "cái bàn"), ("，", ",")]),
            &pos,
            &lk,
            &classes,
        );
        let view: Vec<(&str, &str, &str, Option<i64>)> = toks
            .iter()
            .map(|t| (t.zh.as_str(), t.vi.as_str(), t.tag, t.val))
            .collect();
        assert_eq!(
            view,
            vec![
                ("叶妩", "Diệp Vũ", "nr", None),
                ("三", "ba", "số", Some(3)),
                ("个", "cái", "q", None),
                ("三百五十", "350", "số", Some(350)),
                ("12", "12", "số", Some(12)),
                ("桌子", "cái bàn", "n", None),
                ("，", ",", "w", None),
            ]
        );
        assert!(toks[4].digit && !toks[1].digit);
    }

    #[test]
    fn seg_prefers_names_then_lexicon() {
        let names = map(&[("出发", "Xuất Phát")]);
        let lexicon = map(&[("出发", "xuất phát"), ("出", "ra")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(seg("出发", &lk), vec![("出发".to_string(), "Xuất Phát".to_string())]);
        let empty = map(&[]);
        let lk = Lookups { names: &empty, lexicon: &lexicon };
        assert_eq!(seg("出发了", &lk), vec![
            ("出发".to_string(), "xuất phát".to_string()),
            ("了".to_string(), "了".to_string()),
        ]);
    }
}
```

- [ ] **Step 2: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::prepare`
Expected: FAIL biên dịch, báo `cannot find type Raw`.

- [ ] **Step 3: Viết phần chuẩn bị**

Thêm vào đầu `crates/qt-core/src/quy_tac/prepare.rs`:

```rust
//! Chuẩn bị token cho Quy Tắc (spec mục 6–7): tách dấu câu và `的` dính vào
//! cụm, tách cụm từ điển đóng băng trật tự convert (chỉ khi nghĩa của nó đúng
//! bằng nghĩa từng phần ghép theo thứ tự nguồn), gắn từ loại, tách số + lượng
//! từ. Port từ `untie`, `unfreeze`, `prepare` của bản thử.

use std::sync::LazyLock;

use regex::Regex;
use rustc_hash::FxHashMap;
use unicode_normalization::UnicodeNormalization;

use super::numbers::{number_words, zh_value, NUM_CHARS};
use super::program::Class;
use super::tags::PosLexicon;
use crate::dict::DictionaryLookup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token<'q> {
    pub(crate) zh: String,
    pub(crate) vi: String,
    pub(crate) tag: &'q str,
    /// Giá trị khi `tag == "số"`.
    pub(crate) val: Option<i64>,
    /// Số viết bằng chữ số Ả Rập (giữ nguyên khi ra).
    pub(crate) digit: bool,
}

impl<'q> Token<'q> {
    pub(crate) fn new(zh: impl Into<String>, vi: impl Into<String>, tag: &'q str) -> Self {
        Token {
            zh: zh.into(),
            vi: vi.into(),
            tag,
            val: None,
            digit: false,
        }
    }

    fn number(zh: impl Into<String>, vi: impl Into<String>, val: i64, digit: bool) -> Self {
        Token {
            zh: zh.into(),
            vi: vi.into(),
            tag: "số",
            val: Some(val),
            digit,
        }
    }
}

/// Token thô: `(chữ Hán, nghĩa)` từ engine, hoặc khúc nhóm `[số]` đã dịch kèm nhãn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Raw<'q> {
    Plain(String, String),
    Tagged(String, String, &'q str),
}

impl Raw<'_> {
    fn zh(&self) -> &str {
        match self {
            Raw::Plain(zh, _) | Raw::Tagged(zh, _, _) => zh,
        }
    }
}

/// Từ điển mà Quy Tắc đọc: tên (Names hoặc names của request) và VietPhrase.
pub(crate) struct Lookups<'a> {
    pub(crate) names: &'a dyn DictionaryLookup,
    pub(crate) lexicon: &'a dyn DictionaryLookup,
}

impl Lookups<'_> {
    pub(crate) fn name(&self, word: &str) -> Option<&str> {
        self.names.get(word).map(|value| crate::first_meaning(value).trim())
    }

    pub(crate) fn word(&self, word: &str) -> Option<&str> {
        self.lexicon.get(word).map(|value| crate::first_meaning(value).trim())
    }

    pub(crate) fn is_name(&self, word: &str) -> bool {
        self.names.contains_key(word)
    }

    pub(crate) fn has_word(&self, word: &str) -> bool {
        self.lexicon.contains_key(word)
    }
}

const KEEP_DE: [&str; 6] = ["的时候", "的话", "的确", "的是", "的吗", "的呢"];
const KEEP_TAIL_DE: [&str; 9] = ["有的", "别的", "目的", "是的", "真的", "好的", "似的", "假的", "对的"];
const LOCS: [&str; 15] = [
    "之上", "之中", "之下", "之内", "之外", "上面", "下面", "里面", "外面", "上", "里", "中", "下",
    "内", "外",
];
/// Cụm số + lượng từ có nghĩa riêng, không tách.
const MEASURE_EXCLUDED: [&str; 26] = [
    "一点", "一下", "一些", "一样", "一直", "一定", "一起", "一切", "一般", "一边", "一旦", "一时",
    "一半", "一丝", "一身", "一面", "一口", "一眼", "一声", "一阵", "一番", "一片", "一场", "一股",
    "一道", "一天",
];

static PUNCT_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.*?[^，。！？、；：,.!?…])([，。！？、；：,.!?…]+)$").unwrap()
});
static WRONG_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\s*(bên trên|phía trên|bên trong|phía trong|bên dưới|phía dưới|bên ngoài|phía ngoài)\s*$",
    )
    .unwrap()
});
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\w+").unwrap());
static HAN_OR_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\x{4e00}-\x{9fff}\d]").unwrap());
static NUM_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"^(?:[{NUM_CHARS}]+|\d+(?:\.\d+)?)$")).unwrap()
});
static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)?$").unwrap());
static MEASURE_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^(第?)([{NUM_CHARS}]+)(.)$")).unwrap());

/// Chữ thường, NFC, chỉ giữ chữ: để so nghĩa không vướng dấu câu, hoa thường.
pub(crate) fn plain_words(text: &str) -> Vec<String> {
    let lowered = text.nfc().collect::<String>().to_lowercase();
    WORD.find_iter(&lowered).map(|m| m.as_str().to_string()).collect()
}

fn same_order(parts: &[(String, String)], vi: &str) -> bool {
    let joined: Vec<&str> = parts.iter().map(|(_, v)| v.as_str()).collect();
    plain_words(&joined.join(" ")) == plain_words(vi)
}

/// Tách lại một khúc bằng từ điển, cụm dài trước (tối đa 15 ký tự); tên đi trước VietPhrase.
pub(crate) fn seg(text: &str, lk: &Lookups) -> Vec<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut i = 0;
    'outer: while i < chars.len() {
        for length in (1..=15.min(chars.len() - i)).rev() {
            buffer.clear();
            buffer.extend(&chars[i..i + length]);
            if let Some(value) = lk.name(&buffer).or_else(|| lk.word(&buffer)) {
                out.push((buffer.clone(), value.to_string()));
                i += length;
                continue 'outer;
            }
        }
        out.push((chars[i].to_string(), chars[i].to_string()));
        i += 1;
    }
    out
}

/// Cụm từ điển giữ trật tự convert (`美丽的少女`, `高台上`, `都不知道`): tách ra
/// để quy tắc đảo được. Cụm có nghĩa khác phép ghép từng phần (đã dịch tay)
/// thì giữ nguyên. Thành ngữ không tách.
fn unfreeze(zh: &str, vi: &str, pos: &PosLexicon, lk: &Lookups) -> Option<Vec<(String, String)>> {
    let tag = pos.get(zh).unwrap_or("");
    if tag == "i" || tag == "l" {
        return None;
    }
    let chars: Vec<char> = zh.chars().collect();
    if chars.len() >= 3 && chars[1..chars.len() - 1].contains(&'的') {
        let parts: Vec<&str> = zh.split('的').collect();
        if parts.iter().all(|part| !part.is_empty()) {
            let mut out = Vec::new();
            for (index, part) in parts.iter().enumerate() {
                if index > 0 {
                    out.push(("的".to_string(), String::new()));
                }
                out.extend(seg(part, lk));
            }
            return same_order(&out, vi).then_some(out);
        }
    }
    for loc in LOCS {
        if chars.len() > loc.chars().count() && zh.ends_with(loc) && WRONG_TAIL.is_match(vi) {
            let mut parts = seg(&zh[..zh.len() - loc.len()], lk);
            parts.push((loc.to_string(), lk.word(loc).unwrap_or(loc).to_string()));
            return same_order(&parts, vi).then_some(parts);
        }
    }
    if chars.len() > 2 && (chars[0] == '都' || chars[0] == '也') {
        let rest = &zh[chars[0].len_utf8()..];
        if let Some(meaning) = lk.word(rest) {
            let head = chars[0].to_string();
            let head_vi = lk.word(&head).unwrap_or("").to_string();
            return Some(vec![(head, head_vi), (rest.to_string(), meaning.to_string())]);
        }
    }
    None
}

fn vi_punct(tail: &str) -> String {
    tail.chars()
        .map(|c| match c {
            '，' | '、' => ',',
            '。' => '.',
            '！' => '!',
            '？' => '?',
            '；' => ';',
            '：' => ':',
            other => other,
        })
        .collect()
}

/// Tách dấu câu cuối, `的` đầu/cuối, cụm đóng băng. Nghĩa giữ ở phần chữ.
pub(crate) fn untie<'q>(raw: Vec<Raw<'q>>, pos: &PosLexicon, lk: &Lookups) -> Vec<Raw<'q>> {
    let mut out = Vec::with_capacity(raw.len() + 4);
    for index in 0..raw.len() {
        let (zh, vi) = match &raw[index] {
            Raw::Tagged(..) => {
                out.push(raw[index].clone());
                continue;
            }
            Raw::Plain(zh, vi) => (zh.clone(), vi.clone()),
        };
        let next_de = raw.get(index + 1).is_some_and(|next| next.zh().starts_with('的'));
        // Cụm động từ + danh từ đứng ngay trước 的: tách theo từ để 的 bám đúng định ngữ.
        if next_de && zh.chars().count() >= 3 && pos.get(&zh).is_none() && !lk.is_name(&zh) {
            let words = pos.segment(&zh);
            if words.len() >= 2 && words[0].1.starts_with('v') {
                let pieces: Vec<(String, String)> = words
                    .iter()
                    .map(|(word, _)| (word.clone(), lk.word(word).unwrap_or(word).to_string()))
                    .collect();
                if same_order(&pieces, &vi) {
                    out.extend(pieces.into_iter().map(|(z, v)| Raw::Plain(z, v)));
                    continue;
                }
            }
        }
        // Tính xong trong một câu lệnh để `Captures` (mượn `zh`) hết hạn trước khi `zh` bị move.
        let split = PUNCT_TAIL
            .captures(&zh)
            .filter(|_| zh.chars().count() > 1)
            .map(|caps| (caps[1].to_string(), caps[2].to_string()));
        let (mut zh, vi, tail) = match split {
            Some((body, tail)) => {
                let vi = vi
                    .trim_end_matches([' ', ',', '.', '!', '?', ';', ':', '…'])
                    .to_string();
                (body, vi, Some(tail))
            }
            None => (zh, vi, None),
        };
        if let Some(frozen) = unfreeze(&zh, &vi, pos, lk) {
            out.extend(frozen.into_iter().map(|(z, v)| Raw::Plain(z, v)));
            if let Some(tail) = tail {
                let tail_vi = vi_punct(&tail);
                out.push(Raw::Plain(tail, tail_vi));
            }
            continue;
        }
        if zh.chars().count() >= 2
            && zh.starts_with('的')
            && !KEEP_DE.contains(&zh.as_str())
            && pos.get(&zh['的'.len_utf8()..]).is_some()
        {
            out.push(Raw::Plain("的".to_string(), String::new()));
            zh = zh['的'.len_utf8()..].to_string();
        }
        let split_tail = zh.chars().count() >= 2
            && !KEEP_TAIL_DE.contains(&zh.as_str())
            && zh
                .strip_suffix('的')
                .is_some_and(|head| pos.get(head).is_some() || lk.has_word(head));
        if split_tail {
            let head = zh[..zh.len() - '的'.len_utf8()].to_string();
            out.push(Raw::Plain(head, vi));
            out.push(Raw::Plain("的".to_string(), String::new()));
        } else {
            out.push(Raw::Plain(zh, vi));
        }
        if let Some(tail) = tail {
            let tail_vi = vi_punct(&tail);
            out.push(Raw::Plain(tail, tail_vi));
        }
    }
    out
}

/// Tách hư từ, gắn nhãn, nhận số và tách số + lượng từ.
pub(crate) fn prepare<'q>(
    raw: Vec<Raw<'q>>,
    pos: &'q PosLexicon,
    lk: &Lookups,
    classes: &FxHashMap<String, Class>,
) -> Vec<Token<'q>> {
    let measure = classes.get("lượng-từ");
    let raw = untie(raw, pos, lk);
    let mut out = Vec::with_capacity(raw.len() + 4);
    for item in raw {
        let (zh, vi) = match item {
            Raw::Tagged(zh, vi, tag) => {
                out.push(Token::new(zh, vi, tag));
                continue;
            }
            Raw::Plain(zh, vi) => (zh, vi),
        };
        if !HAN_OR_DIGIT.is_match(&zh) {
            out.push(Token::new(zh, vi, "w"));
            continue;
        }
        if NUM_TOKEN.is_match(&zh) && !["一", "万", "千", "百", "十"].contains(&zh.as_str()) {
            if DIGITS.is_match(&zh) {
                // Như bản thử: int(float(zh)), phần thập phân bị cắt.
                match zh.parse::<f64>() {
                    Ok(value) => out.push(Token::number(zh, vi, value as i64, true)),
                    Err(_) => out.push(Token::new(zh, vi, "m")),
                }
            } else if let Some(value) = zh_value(&zh) {
                out.push(Token::number(zh, vi, value, false));
            } else {
                out.push(Token::new(zh, vi, "m"));
            }
            continue;
        }
        if zh == "一" {
            out.push(Token::number(zh, vi, 1, false));
            continue;
        }
        if let (Some(measure), Some(caps)) = (measure, MEASURE_SPLIT.captures(&zh)) {
            let (ordinal, number, unit) = (&caps[1], &caps[2], &caps[3]);
            if measure.contains_key(unit)
                && !["万", "千", "百"].contains(&number)
                && !MEASURE_EXCLUDED.contains(&zh.as_str())
            {
                if let Some(value) = zh_value(number) {
                    let unit_vi = measure
                        .get(unit)
                        .filter(|v| !v.is_empty())
                        .cloned()
                        .unwrap_or_else(|| vi.clone());
                    if !ordinal.is_empty() {
                        out.push(Token::new("第", "thứ", "lit"));
                    }
                    out.push(Token::number(number, number_words(value), value, false));
                    out.push(Token::new(unit, unit_vi, "q"));
                    continue;
                }
            }
        }
        let tag = if lk.is_name(&zh) {
            "nr"
        } else if matches!(zh.as_str(), "时候" | "时" | "时间点") {
            "khung"
        } else if zh.ends_with('是') {
            "hệ-từ"
        } else {
            pos.phrase(&zh)
        };
        out.push(Token::new(zh, vi, tag));
    }
    out
}
```

- [ ] **Step 4: Chạy test**

Run: `cargo test -p qt-core quy_tac::prepare`
Expected: PASS 4 test.

- [ ] **Step 5: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src/quy_tac
git commit -m "feat(core): chuẩn bị token Quy Tắc — tách hư từ có điều kiện, gắn nhãn, số + lượng từ"
```

---

### Task 6: Nhóm `[số]` trên ký tự (`char_rules.rs`)

**Files:**
- Create: `crates/qt-core/src/quy_tac/char_rules.rs`
- Modify: `crates/qt-core/src/quy_tac/mod.rs` (`mod char_rules;`)

**Interfaces:**
- Consumes: `program::{Program, CharRule, Part, RefKey}`, `prepare::{seg, Lookups, Raw}`, `numbers::{num_text, zh_value, DIGIT_WORDS, NUM_CHARS}`.
- Produces: `pub(crate) fn apply_chars<'q>(program: &'q Program, raw: &[(String, String)], lk: &Lookups) -> (Vec<Raw<'q>>, usize)` — số khúc `[số]` đã thay ở phần tử thứ hai.

- [ ] **Step 1: Viết test hỏng**

Tạo `crates/qt-core/src/quy_tac/char_rules.rs` chỉ với phần test, thêm `mod char_rules;` vào `mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    const RULES: &str = "[lớp]
buổi: 晚上=tối

[số]
{buổi}{số:1-2}点半 = {2} giờ rưỡi {1} -> thời-gian
{số}点半 = {1} giờ rưỡi -> thời-gian
{số:1-2}点{số:1-2}分 = {1} giờ {2} phút -> thời-gian
{năm:3-4}年 = năm {1} -> thời-gian
{số:1-2}点{năm:1-2} = {1} phẩy {2|từng-chữ} -> m
";

    fn map(pairs: &[(&str, &str)]) -> FxHashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn raw(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(z, v)| (z.to_string(), v.to_string())).collect()
    }

    #[test]
    fn replaces_number_spans_with_templates() {
        let program = Program::parse(RULES).unwrap();
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (out, fired) = apply_chars(&program, &raw(&[("晚上", "buổi tối"), ("九点", "chín điểm"), ("半", "bán")]), &lk);
        assert_eq!(fired, 1);
        assert_eq!(out, vec![Raw::Tagged("晚上九点半".into(), "chín giờ rưỡi tối".into(), "thời-gian")]);
        let (out, _) = apply_chars(&program, &raw(&[("二〇一九年", "x")]), &lk);
        assert_eq!(out, vec![Raw::Tagged("二〇一九年".into(), "năm 2019".into(), "thời-gian")]);
        let (out, _) = apply_chars(&program, &raw(&[("三点一四", "x")]), &lk);
        assert_eq!(out, vec![Raw::Tagged("三点一四".into(), "ba phẩy một bốn".into(), "m")]);
    }

    #[test]
    fn longer_dictionary_phrase_vetoes_the_rule() {
        let program = Program::parse(RULES).unwrap();
        let names = map(&[]);
        let lexicon = map(&[("三点半钟", "ba giờ rưỡi đúng")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (out, fired) = apply_chars(&program, &raw(&[("三点半钟", "ba giờ rưỡi đúng")]), &lk);
        assert_eq!(fired, 0);
        assert_eq!(out, vec![Raw::Plain("三点半钟".into(), "ba giờ rưỡi đúng".into())]);
    }

    #[test]
    fn span_cut_inside_a_token_resegments_the_rest() {
        let program = Program::parse(RULES).unwrap();
        let names = map(&[]);
        let lexicon = map(&[("出发", "xuất phát")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (out, fired) = apply_chars(
            &program,
            &raw(&[("三点", "ba điểm"), ("十五", "mười lăm"), ("分出", "phân ra"), ("发", "phát")]),
            &lk,
        );
        assert_eq!(fired, 1);
        assert_eq!(
            out,
            vec![
                Raw::Tagged("三点十五分".into(), "ba giờ mười lăm phút".into(), "thời-gian"),
                Raw::Plain("出发".into(), "xuất phát".into()),
            ]
        );
    }

    #[test]
    fn duration_is_not_a_calendar_year() {
        let program = Program::parse(RULES).unwrap();
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (_, fired) = apply_chars(&program, &raw(&[("500年", "500 năm")]), &lk);
        assert_eq!(fired, 0);
    }
}
```

- [ ] **Step 2: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::char_rules`
Expected: FAIL biên dịch, báo `cannot find function apply_chars`.

- [ ] **Step 3: Viết nhóm `[số]`**

Thêm vào đầu `crates/qt-core/src/quy_tac/char_rules.rs`:

```rust
//! Nhóm `[số]` (spec mục 8): quét ký tự trên cả đoạn, trước bước tách hư từ.
//!
//! Một khúc được nhận khi nó bắt đầu ở ranh giới token và từ điển không có cụm
//! dài hơn bắt đầu cùng chỗ. Đuôi khúc được cắt giữa một token; phần dư tách lại
//! bằng từ điển cùng token kế tiếp (`分出|发` → `出发`). Khác bản thử một chỗ:
//! phần tách lại không lấn sang khúc số kế tiếp.

use std::sync::LazyLock;

use regex::Regex;

use super::numbers::{num_text, zh_value, DIGIT_WORDS, NUM_CHARS};
use super::prepare::{seg, Lookups, Raw};
use super::program::{CharRule, Part, Program, RefKey};

static HAS_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"[{NUM_CHARS}\d]")).unwrap());
static MIXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(?:\.\d+)?)([十百千万亿]+)$").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

struct Span<'q> {
    a: usize,
    b: usize,
    vi: String,
    result: &'q str,
}

fn digit_char(c: char) -> Option<char> {
    if c.is_ascii_digit() {
        return Some(c);
    }
    let index = if c == '〇' {
        0
    } else {
        "零一二三四五六七八九".chars().position(|d| d == c)?
    };
    char::from_digit(index as u32, 10)
}

/// Giá trị của một khoá trong khuôn `[số]`, theo bộ lọc (`số`, `thứ`, `năm`, `từng-chữ`…).
fn char_value(program: &Program, name: &str, text: &str, how: Option<&str>) -> String {
    if name == "năm" {
        let digits: String = text.chars().filter_map(digit_char).collect();
        if how == Some("từng-chữ") {
            let words: Vec<&str> = digits
                .chars()
                .filter_map(|d| d.to_digit(10))
                .map(|d| DIGIT_WORDS[d as usize])
                .collect();
            return words.join(" ");
        }
        return digits;
    }
    if name != "số" {
        return program
            .classes
            .get(name)
            .and_then(|class| class.get(text))
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| text.to_string());
    }
    if let Some(caps) = MIXED.captures(text) {
        let units: Vec<&str> = caps[2]
            .chars()
            .map(|c| match c {
                '十' => "mươi",
                '百' => "trăm",
                '千' => "ngàn",
                '万' => "vạn",
                _ => "ức",
            })
            .collect();
        return format!("{} {}", &caps[1], units.join(" "));
    }
    if text.contains('.') {
        return text.to_string();
    }
    let digit = text.chars().next().is_some_and(|c| c.is_ascii_digit());
    let value = if digit {
        text.parse::<i64>().ok()
    } else {
        zh_value(text)
    };
    let Some(value) = value else {
        return text.to_string();
    };
    if how == Some("năm") && !digit && text.chars().all(|c| "零〇一二三四五六七八九".contains(c)) {
        return text.chars().filter_map(digit_char).collect();
    }
    num_text(value, digit, how.unwrap_or(""))
}

fn fill_char(program: &Program, rule: &CharRule, values: &[Option<&str>]) -> String {
    let mut out = String::new();
    for part in &rule.tmpl {
        match part {
            Part::Text(text) => out.push_str(text),
            Part::Ref { key, filter } => {
                let index = match key {
                    RefKey::Index(index) => Some(index - 1),
                    RefKey::Name(name) => rule.slots.iter().position(|slot| slot == name),
                };
                let Some(index) = index else { continue };
                if let (Some(name), Some(Some(text))) = (rule.slots.get(index), values.get(index)) {
                    out.push_str(&char_value(program, name, text, filter.as_deref()));
                }
            }
        }
    }
    SPACES.replace_all(&out, " ").trim().to_string()
}

pub(crate) fn apply_chars<'q>(
    program: &'q Program,
    raw: &[(String, String)],
    lk: &Lookups,
) -> (Vec<Raw<'q>>, usize) {
    let plain = || {
        raw.iter()
            .map(|(zh, vi)| Raw::Plain(zh.clone(), vi.clone()))
            .collect::<Vec<_>>()
    };
    let text: String = raw.iter().map(|(zh, _)| zh.as_str()).collect();
    if !HAS_NUMBER.is_match(&text) {
        return (plain(), 0);
    }
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut char_of_byte = vec![usize::MAX; text.len() + 1];
    for (char_index, (byte_index, _)) in text.char_indices().enumerate() {
        char_of_byte[byte_index] = char_index;
    }
    char_of_byte[text.len()] = n;
    // bounds[vị trí ký tự] = chỉ số token bắt đầu ở đó.
    let mut bounds: Vec<Option<usize>> = vec![None; n + 1];
    let mut starts = Vec::with_capacity(raw.len() + 1);
    let mut position = 0;
    for (index, (zh, _)) in raw.iter().enumerate() {
        bounds[position] = Some(index);
        starts.push(position);
        position += zh.chars().count();
    }
    bounds[n] = Some(raw.len());
    starts.push(n);

    let mut taken = vec![false; n];
    let mut spans: Vec<Span<'q>> = Vec::new();
    let mut buffer = String::new();
    for rule in &program.char_rules {
        for caps in rule.regex.captures_iter(&text) {
            let whole = caps.get(0).expect("nhóm 0 luôn có");
            let (a, b) = (char_of_byte[whole.start()], char_of_byte[whole.end()]);
            if a == b || bounds[a].is_none() || taken[a..b].iter().any(|&t| t) {
                continue;
            }
            // Từ điển có cụm dài hơn bắt đầu cùng chỗ thì nhường (两点一线, 一个人…).
            let vetoed = ((b - a + 1)..=15.min(n - a)).any(|length| {
                buffer.clear();
                buffer.extend(&chars[a..a + length]);
                lk.has_word(&buffer)
            });
            if vetoed {
                continue;
            }
            let values: Vec<Option<&str>> = (0..rule.slots.len())
                .map(|i| caps.get(i + 1).map(|m| m.as_str()))
                .collect();
            spans.push(Span {
                a,
                b,
                vi: fill_char(program, rule, &values),
                result: rule.result.as_str(),
            });
            taken[a..b].iter_mut().for_each(|t| *t = true);
        }
    }
    if spans.is_empty() {
        return (plain(), 0);
    }
    spans.sort_by_key(|span| span.a);
    let fired = spans.len();
    let mut out = Vec::with_capacity(raw.len() + fired);
    let mut index = 0;
    for (k, span) in spans.iter().enumerate() {
        let first = bounds[span.a].expect("khúc luôn bắt đầu ở ranh giới token");
        while index < first {
            out.push(Raw::Plain(raw[index].0.clone(), raw[index].1.clone()));
            index += 1;
        }
        out.push(Raw::Tagged(
            chars[span.a..span.b].iter().collect(),
            span.vi.clone(),
            span.result,
        ));
        if let Some(next) = bounds[span.b] {
            index = next;
            continue;
        }
        // Đuôi khúc cắt giữa một token: tách lại phần dư cùng token kế tiếp.
        let mut end = *starts.iter().find(|&&p| p > span.b).expect("luôn có vị trí n");
        let next_span = spans.get(k + 1).map(|s| s.a);
        if let Some(&later) = starts.iter().find(|&&p| p > end) {
            if later - span.b <= 6 && next_span.is_none_or(|a| later <= a) {
                end = later;
            }
        }
        let rest: String = chars[span.b..end].iter().collect();
        out.extend(seg(&rest, lk).into_iter().map(|(z, v)| Raw::Plain(z, v)));
        index = bounds[end].expect("end là ranh giới token");
    }
    while index < raw.len() {
        out.push(Raw::Plain(raw[index].0.clone(), raw[index].1.clone()));
        index += 1;
    }
    (out, fired)
}
```

- [ ] **Step 4: Chạy test**

Run: `cargo test -p qt-core quy_tac::char_rules`
Expected: PASS 4 test.

- [ ] **Step 5: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src/quy_tac
git commit -m "feat(core): nhóm [số] của Quy Tắc — quét ký tự, nhường cụm từ điển dài hơn, tách lại phần dư"
```

---

### Task 7: Khớp mẫu, điền khuôn, ráp câu (`matcher.rs`, `render.rs`)

**Files:**
- Create: `crates/qt-core/src/quy_tac/matcher.rs`, `crates/qt-core/src/quy_tac/render.rs`
- Modify: `crates/qt-core/src/quy_tac/mod.rs` (`mod matcher; mod render;`)

**Interfaces:**
- Consumes: `program::{Program, Section, Rule, El, ElKind, LeftKind, Part, RefKey}`, `prepare::Token`, `numbers::num_text`.
- Produces: `pub(crate) fn apply_program<'q>(program: &'q Program, toks: Vec<Token<'q>>) -> (Vec<Token<'q>>, usize)` (số lần quy tắc chạy); `pub(crate) fn render(toks: &[Token]) -> String`.

- [ ] **Step 1: Viết test hỏng**

Tạo `crates/qt-core/src/quy_tac/render.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::prepare::Token;

    #[test]
    fn joins_fixes_spacing_and_capitalizes() {
        let toks: Vec<Token> = ["hắn", "đi", ",", "“", "ta", "nói", "”", "."]
            .iter()
            .map(|vi| Token::new("x", *vi, "n"))
            .collect();
        assert_eq!(render(&toks), "Hắn đi, “Ta nói”.");
    }
}
```

Tạo `crates/qt-core/src/quy_tac/matcher.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::program::Program;

    fn tok(zh: &str, vi: &str, tag: &'static str) -> Token<'static> {
        Token::new(zh, vi, tag)
    }

    fn run(rules: &str, toks: Vec<Token<'static>>) -> (String, usize) {
        let program = Box::leak(Box::new(Program::parse(rules).unwrap()));
        let (toks, fired) = apply_program(program, toks);
        (crate::quy_tac::render::render(&toks), fired)
    }

    #[test]
    fn reorders_locative_with_class_value() {
        let rules = "[lớp]\nvị-trí: 上=trên\n\n[của 1]\n{danh-từ} {vị-trí} = {vị-trí} {danh-từ}\n";
        let (text, fired) = run(rules, vec![tok("桌子", "cái bàn", "n"), tok("上", "bên trên", "f")]);
        assert_eq!((text.as_str(), fired), ("Trên cái bàn", 1));
    }

    #[test]
    fn left_clause_before_de_shi_hou() {
        let rules = "[khung 1]\n{vế-động} 的时候 = khi {vế-động}\n";
        let (text, _) = run(
            rules,
            vec![tok("他", "Hắn", "r"), tok("走进来", "đi tới", "v"), tok("的时候", "thời điểm", "n")],
        );
        assert_eq!(text, "Khi hắn đi tới");
    }

    #[test]
    fn context_start_and_number_filter() {
        let rules = "[lớp]\nlượng-từ: 层=tầng\n\n[lượng 1]\n第 {số} {lượng-từ} = {lượng-từ} thứ {số|thứ}\n\n[nghĩa 1]\n^ | 可是 | = Nhưng -> lit\n";
        let mut four = tok("四", "bốn", "số");
        four.val = Some(4);
        let (text, _) = run(rules, vec![tok("第", "thứ", "lit"), four, tok("层", "tầng", "q")]);
        assert_eq!(text, "Tầng thứ tư");
        let (text, _) = run(rules, vec![tok("可是", "có thể là", "c"), tok("他", "hắn", "r")]);
        assert_eq!(text, "Nhưng hắn");
        let (_, fired) = run(rules, vec![tok("他", "hắn", "r"), tok("可是", "có thể là", "c")]);
        assert_eq!(fired, 0);
    }

    #[test]
    fn names_keep_capitals_and_de_reorders() {
        let rules = "[của 3]\n{tên} 的 {danh-từ}+ = {2} của {1}\n";
        let (text, _) = run(
            rules,
            vec![tok("叶妩", "Diệp Vũ", "nr"), tok("的", "", "uj"), tok("手", "Tay", "n")],
        );
        assert_eq!(text, "Tay của Diệp Vũ");
    }
}
```

Thêm `mod matcher;` và `mod render;` vào `mod.rs`.

- [ ] **Step 2: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core quy_tac::matcher quy_tac::render`
Expected: FAIL biên dịch, báo `cannot find function apply_program` / `render`.

- [ ] **Step 3: Viết phần ráp câu**

Thêm vào đầu `crates/qt-core/src/quy_tac/render.rs`:

```rust
//! Ráp token thành câu cho đoạn có quy tắc chạy: nối bằng dấu cách, bỏ cách
//! trước dấu câu / sau ngoặc mở, viết hoa đầu câu. Giống `render` của bản thử.

use std::sync::LazyLock;

use regex::{Captures, Regex};

use super::prepare::Token;

static SPACE_BEFORE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+([,.!?:;…”’)\]】])").unwrap());
static SPACE_AFTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([“‘(\[【])\s+").unwrap());
static CAPITAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^|[.!?…]\s+|[“‘]\s*)(\w)").unwrap());

pub(crate) fn render(toks: &[Token]) -> String {
    let pieces: Vec<&str> = toks
        .iter()
        .map(|t| t.vi.trim())
        .filter(|piece| !piece.is_empty())
        .collect();
    let text = pieces.join(" ");
    let text = SPACE_BEFORE.replace_all(&text, "$1");
    let text = SPACE_AFTER.replace_all(&text, "$1");
    CAPITAL
        .replace_all(&text, |caps: &Captures| {
            format!("{}{}", &caps[1], caps[2].to_uppercase())
        })
        .into_owned()
}
```

- [ ] **Step 4: Viết phần khớp mẫu**

Thêm vào đầu `crates/qt-core/src/quy_tac/matcher.rs`:

```rust
//! Chạy các nhóm quy tắc trên token: khớp mẫu, điền khuôn, gộp khúc thành một
//! token mang nhãn kết quả. Port từ `match`, `fill`, `run_section` của bản thử,
//! giữ cả chỗ kỳ quặc: token gộp chỉ giữ chữ Hán của phần bên phải; quy tắc
//! khớp mà không tiêu token nào thì token hiện tại bị bỏ.

use std::sync::LazyLock;

use regex::Regex;

use super::numbers::num_text;
use super::prepare::Token;
use super::program::{El, ElKind, LeftKind, Part, Program, RefKey, Rule, Section};

const PRONOUNS: &[&str] = &[
    "我", "你", "他", "她", "它", "您", "咱", "我们", "你们", "他们", "她们", "它们", "咱们", "大家",
    "大伙", "自己", "人家", "别人", "对方", "本座", "本王", "在下", "老夫", "朕", "贫道", "小女子",
    "谁", "彼此", "众人",
];
const DEMONSTRATIVE: &[&str] = &[
    "这", "那", "此", "该", "这个", "那个", "这些", "那些", "这种", "那种", "这样", "那样", "这位",
    "那位", "这名", "那名",
];
const CLAUSE_OPEN: &[&str] = &[
    "但", "但是", "可是", "可", "而", "却", "然后", "于是", "所以", "因为", "如果", "虽然", "不过",
    "只是", "就", "便", "才", "并", "并且", "而且", "或者", "还是", "甚至", "只要", "要是",
];
const AUX: &[&str] = &[
    "不", "没", "没有", "未", "能", "会", "要", "想", "可以", "应该", "已经", "曾", "曾经", "正",
    "正在", "最", "很", "更", "所", "都", "也", "还", "再", "又", "才", "就", "刚", "刚刚", "总",
    "总是", "一直", "不会", "不能", "不想", "不敢", "能够", "可能", "将要", "快要", "敢", "肯",
    "愿意", "需要", "必须", "该", "得",
];
const STOP_VERB: &[&str] = &["是", "说", "道", "问", "笑道", "说道", "喊道", "叫道"];
const NOUN_TAGS: &[&str] = &["n", "ng", "nz", "vn", "an", "j", "cụm", "nrfg"];
const ADJ_TAGS: &[&str] = &["a", "ad", "ag", "z", "b"];

static BO_TAI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s*(tại|ở|trên|tới|vào|đến)\s*$").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

fn has(list: &[&str], word: &str) -> bool {
    list.contains(&word)
}

fn is_person(t: &Token) -> bool {
    has(PRONOUNS, &t.zh) || t.tag == "người"
}

fn is_name(t: &Token) -> bool {
    t.tag.starts_with("nr") || matches!(t.tag, "ns" | "nt" | "tên")
}

fn is_noun(t: &Token) -> bool {
    has(NOUN_TAGS, t.tag) && !has(PRONOUNS, &t.zh)
}

fn nominal(t: &Token) -> bool {
    is_noun(t)
        || is_person(t)
        || is_name(t)
        || matches!(t.tag, "s" | "t" | "nơi" | "cụm" | "số-lượng" | "thời-gian")
}

fn verbal(t: &Token) -> bool {
    t.tag.starts_with('v') && !has(STOP_VERB, &t.zh) && !t.zh.starts_with('是')
}

fn slot_ok(program: &Program, name: &str, t: &Token) -> bool {
    if let Some(class) = program.classes.get(name) {
        return class.contains_key(&t.zh);
    }
    match name {
        "đại-từ" => has(PRONOUNS, &t.zh),
        "tên" => is_name(t),
        "người" => is_person(t) || is_name(t),
        "danh-từ" => is_noun(t),
        "cụm" => is_noun(t) || is_person(t) || is_name(t) || t.tag == "s",
        "động-từ" => t.tag.starts_with('v'),
        "danh-động" => t.tag == "vn",
        "hệ-từ" => t.tag == "hệ-từ",
        "có-đích" => {
            t.zh.chars().count() >= 2 && t.zh.ends_with('的') && !t.vi.trim().is_empty()
        }
        "tính-từ" => has(ADJ_TAGS, t.tag) || t.tag == "tính-cụm",
        "thành-ngữ" => matches!(t.tag, "i" | "l"),
        "số" => t.tag == "số",
        "số-lượng" => t.tag == "số-lượng",
        "thời-gian" => matches!(t.tag, "t" | "tg" | "thời-gian"),
        "nơi" => matches!(t.tag, "s" | "nơi"),
        "phó-từ" => t.tag.starts_with('d') && t.tag != "dấu",
        "dấu" => t.tag == "w",
        "chỉ-định" => has(DEMONSTRATIVE, &t.zh),
        "nơi-cụm" => t.tag == "nơi-cụm",
        "nơi-từ" => matches!(t.tag, "nơi-cụm" | "s" | "f") && t.zh.chars().count() > 1,
        "động-từ-tại" => {
            t.zh.chars().count() >= 2 && t.zh.ends_with('在') && t.tag.starts_with('v')
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy)]
enum Cap {
    Empty,
    One(usize),
    Toks(usize, usize),
    Out(usize, usize),
}

struct Capture<'p> {
    name: &'p str,
    cap: Cap,
}

/// Nghĩa của token khi đứng giữa câu: tên giữ hoa, còn lại hạ chữ đầu.
fn low(t: &Token) -> String {
    if is_name(t) || t.vi.is_empty() {
        return t.vi.trim().to_string();
    }
    let mut chars = t.vi.chars();
    let first: String = chars.next().map(|c| c.to_lowercase().collect()).unwrap_or_default();
    format!("{first}{}", chars.as_str()).trim().to_string()
}

/// Số token liền nhau ghép đúng thành một chữ cố định; 0 nếu không khớp.
fn lit_span(toks: &[Token], j: usize, lits: &[String]) -> usize {
    let Some(first) = toks.get(j) else { return 0 };
    let mut acc = first.zh.clone();
    if lits.iter().any(|lit| *lit == acc) {
        return 1;
    }
    let longest = lits.iter().map(|lit| lit.chars().count()).max().unwrap_or(0);
    let mut k = j + 1;
    while k < toks.len() && acc.chars().count() < longest {
        acc.push_str(&toks[k].zh);
        k += 1;
        if lits.iter().any(|lit| *lit == acc) {
            return k - j;
        }
    }
    0
}

/// Mệnh đề định ngữ kết thúc ở cuối `out`: `[chủ ngữ] [phó từ] động từ [tân ngữ]`.
fn clause_span(program: &Program, out: &[Token]) -> Option<usize> {
    let main = program.classes.get("động-từ-chính");
    let in_main = |word: &str| main.is_some_and(|m| m.contains_key(word));
    let starts_main = |word: &str| {
        main.is_some_and(|m| {
            m.keys().any(|key| {
                word.starts_with(key.as_str()) && word.chars().count() > key.chars().count()
            })
        })
    };
    let mut k = out.len();
    let start_object = k;
    while k > 0 && start_object - k < 2 && nominal(&out[k - 1]) {
        k -= 1;
    }
    let has_object = k != start_object;
    if k == 0 || !verbal(&out[k - 1]) {
        return None;
    }
    let verb = &out[k - 1];
    if (in_main(&verb.zh) && has_object) || starts_main(&verb.zh) {
        return None;
    }
    k -= 1;
    while k > 0 && (has(AUX, &out[k - 1].zh) || matches!(out[k - 1].tag, "d" | "ad")) {
        k -= 1;
    }
    if k > 0 && (is_person(&out[k - 1]) || is_name(&out[k - 1]) || is_noun(&out[k - 1])) {
        k -= 1;
    }
    Some(k)
}

/// Khúc bên trái lùi tới ranh giới vế (tối đa 16 token).
fn left_span(program: &Program, out: &[Token], kind: LeftKind) -> usize {
    if kind == LeftKind::MenhDe {
        return clause_span(program, out).unwrap_or(out.len());
    }
    let mut i = out.len();
    while i > 0 && out.len() - i < 16 {
        let t = &out[i - 1];
        if t.tag == "w" || has(CLAUSE_OPEN, &t.zh) {
            break;
        }
        if t.tag == "hệ-từ" || matches!(t.zh.as_str(), "到" | "等" | "等到") {
            break;
        }
        i -= 1;
    }
    i
}

fn ctx_ok(program: &Program, el: &El, tok: Option<&Token>, at_edge: bool) -> bool {
    let caret = matches!(&el.kind, ElKind::Lit(lits) if lits.iter().any(|lit| lit == "^"));
    let mut hit = match (&el.kind, tok) {
        (ElKind::Start | ElKind::End, _) => at_edge,
        (_, None) => caret,
        (ElKind::Lit(lits), Some(t)) => lits.iter().any(|lit| *lit == t.zh),
        (ElKind::Slot(name) | ElKind::Many(name), Some(t)) => slot_ok(program, name, t),
        (_, Some(_)) => false,
    };
    if caret && at_edge {
        hit = true;
    }
    hit != el.neg
}

/// Khớp `rule` tại `toks[i]`. Trả về (số token tiêu bên phải, số token lấy từ `out`, khoá).
fn match_rule<'p>(
    program: &Program,
    rule: &'p Rule,
    out: &[Token],
    toks: &[Token],
    i: usize,
) -> Option<(usize, usize, Vec<Capture<'p>>)> {
    let n = toks.len();
    let mut caps: Vec<Capture<'p>> = Vec::new();
    let mut j = i;
    let mut take = 0;
    let mut idx = 0;
    if let ElKind::Left(kind) = &rule.pat[0].kind {
        let kind = *kind;
        let start = left_span(program, out, kind);
        let span = &out[start..];
        if span.is_empty() {
            return None;
        }
        if kind == LeftKind::VeDong && !span.iter().any(|t| t.tag.starts_with('v') && t.tag != "vn")
        {
            return None;
        }
        caps.push(Capture {
            name: kind.name(),
            cap: Cap::Out(start, out.len()),
        });
        take = span.len();
        idx = 1;
    }
    while idx < rule.pat.len() {
        let el = &rule.pat[idx];
        match &el.kind {
            ElKind::Lit(lits) => {
                let span = lit_span(toks, j, lits);
                if (span > 0) != el.neg {
                    if !el.neg {
                        j += span;
                    }
                } else if !el.opt {
                    return None;
                }
            }
            ElKind::Slot(name) => {
                if j < n && slot_ok(program, name, &toks[j]) {
                    caps.push(Capture { name: name.as_str(), cap: Cap::One(j) });
                    j += 1;
                } else if !el.opt {
                    return None;
                } else {
                    caps.push(Capture { name: name.as_str(), cap: Cap::Empty });
                }
            }
            ElKind::Many(name) => {
                let mut k = j;
                while k < n && k - j < 3 && slot_ok(program, name, &toks[k]) {
                    k += 1;
                }
                if k == j {
                    return None;
                }
                caps.push(Capture { name: name.as_str(), cap: Cap::Toks(j, k) });
                j = k;
            }
            ElKind::Gap => {
                let ElKind::Lit(next) = &rule.pat[idx + 1].kind else {
                    return None;
                };
                let mut k = j;
                let mut found = false;
                while k < n && k - j <= 12 && toks[k].tag != "w" {
                    if k > j && lit_span(toks, k, next) > 0 {
                        found = true;
                        break;
                    }
                    k += 1;
                }
                if !found {
                    return None;
                }
                caps.push(Capture { name: "đoạn", cap: Cap::Toks(j, k) });
                j = k;
            }
            ElKind::Start | ElKind::End | ElKind::Left(_) => {}
        }
        idx += 1;
    }
    for el in &rule.post {
        let tok = toks.get(j);
        if !ctx_ok(program, el, tok, tok.is_none_or(|t| t.tag == "w")) {
            return None;
        }
    }
    if !rule.pre.is_empty() {
        let tok = out.len().checked_sub(take + 1).map(|back| &out[back]);
        for el in &rule.pre {
            if !ctx_ok(program, el, tok, tok.is_none_or(|t| t.tag == "w")) {
                return None;
            }
        }
    }
    Some((j - i, take, caps))
}

fn value(program: &Program, hit: &Capture, out: &[Token], toks: &[Token]) -> String {
    let join = |slice: &[Token]| {
        slice
            .iter()
            .map(low)
            .filter(|piece| !piece.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };
    match hit.cap {
        Cap::Empty => String::new(),
        Cap::Toks(a, b) => join(&toks[a..b]),
        Cap::Out(a, b) => join(&out[a..b]),
        Cap::One(index) => {
            let t = &toks[index];
            if let Some(v) = program
                .classes
                .get(hit.name)
                .and_then(|class| class.get(&t.zh))
                .filter(|v| !v.is_empty())
            {
                return v.clone();
            }
            if t.tag == "số" {
                if let Some(v) = t.val {
                    return num_text(v, t.digit, "");
                }
            }
            low(t)
        }
    }
}

fn fill(program: &Program, rule: &Rule, caps: &[Capture], out: &[Token], toks: &[Token]) -> String {
    let mut text = String::new();
    for part in &rule.tmpl {
        match part {
            Part::Text(piece) => text.push_str(piece),
            Part::Ref { key, filter } => {
                let hit = match key {
                    RefKey::Index(index) => caps.get(index - 1),
                    RefKey::Name(name) => caps.iter().find(|c| c.name == name.as_str()),
                };
                let Some(hit) = hit else { continue };
                if let (Some(how), Cap::One(index)) = (filter.as_deref(), hit.cap) {
                    let t = &toks[index];
                    if t.tag == "số" {
                        text.push_str(&match t.val {
                            Some(v) => num_text(v, t.digit, how),
                            None => low(t),
                        });
                        continue;
                    }
                }
                let piece = value(program, hit, out, toks);
                if filter.as_deref() == Some("bỏ-tại") {
                    text.push_str(&BO_TAI.replace(&piece, ""));
                } else {
                    text.push_str(&piece);
                }
            }
        }
    }
    SPACES.replace_all(&text, " ").trim().to_string()
}

fn run_section<'q>(
    program: &'q Program,
    section: &'q Section,
    mut toks: Vec<Token<'q>>,
) -> (Vec<Token<'q>>, usize) {
    let text: String = toks.iter().map(|t| t.zh.as_str()).collect();
    let mut fired = 0;
    for _ in 0..section.rounds {
        let mut out: Vec<Token<'q>> = Vec::with_capacity(toks.len());
        let mut i = 0;
        let mut changed = false;
        while i < toks.len() {
            let head = toks[i].zh.chars().next();
            let groups: [&[usize]; 3] = [
                head.and_then(|c| section.left.get(&c)).map_or(&[][..], Vec::as_slice),
                head.and_then(|c| section.first.get(&c)).map_or(&[][..], Vec::as_slice),
                section.generic.as_slice(),
            ];
            let mut done = false;
            'groups: for group in groups {
                for &index in group {
                    let rule = &section.rules[index];
                    if !rule
                        .needs
                        .iter()
                        .all(|lits| lits.iter().any(|lit| text.contains(lit.as_str())))
                    {
                        continue;
                    }
                    let Some((used, take, caps)) = match_rule(program, rule, &out, &toks, i) else {
                        continue;
                    };
                    let vi = fill(program, rule, &caps, &out, &toks);
                    let zh: String = toks[i..i + used].iter().map(|t| t.zh.as_str()).collect();
                    out.truncate(out.len() - take);
                    out.push(Token::new(zh, vi, rule.result.as_str()));
                    fired += 1;
                    i += used.max(1);
                    done = true;
                    changed = true;
                    break 'groups;
                }
            }
            if !done {
                out.push(toks[i].clone());
                i += 1;
            }
        }
        toks = out;
        if !changed {
            break;
        }
    }
    (toks, fired)
}

/// Chạy lần lượt mọi nhóm quy tắc; trả về token mới và số lần có quy tắc chạy.
pub(crate) fn apply_program<'q>(
    program: &'q Program,
    mut toks: Vec<Token<'q>>,
) -> (Vec<Token<'q>>, usize) {
    let mut fired = 0;
    for section in &program.sections {
        let (next, count) = run_section(program, section, toks);
        toks = next;
        fired += count;
    }
    (toks, fired)
}
```

- [ ] **Step 5: Chạy test**

Run: `cargo test -p qt-core quy_tac::matcher quy_tac::render`
Expected: PASS 5 test.

- [ ] **Step 6: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src/quy_tac
git commit -m "feat(core): khớp mẫu và điền khuôn Quy Tắc — ngữ cảnh, vế bên trái, mệnh đề, bộ lọc số"
```

---

### Task 8: `QuyTac` công khai và nối vào engine

**Files:**
- Modify: `crates/qt-core/src/quy_tac/mod.rs`
- Modify: `crates/qt-core/src/translate.rs` (`TranslationOutput`, `process_translation`, `process_han_viet`, `translate_all`, `translate_all_mapped`)
- Modify: `crates/qt-core/src/lib.rs` (`mod`/`pub use`, `Options`, `Engine`, `LayeredBase`)
- Modify: `crates/qt-core/src/lite.rs`
- Modify: `apps/qt-gui/src-tauri/src/lib.rs:434`
- Test: `crates/qt-core/tests/quy_tac_engine.rs`

**Interfaces:**
- Consumes: mọi thứ từ Task 2–7.
- Produces:
  - `pub struct QuyTac` với `DEFAULT_RULES`, `GIONG_MUOT_RULES`, `DEFAULT_TU_LOAI` (`&'static str`), `from_sources(rules, pos_dict, tu_loai) -> Result<QuyTac, QuyTacError>`, `load(data_dir: &Path, giong_muot: bool) -> io::Result<QuyTac>`, `pub(crate) fn apply(&self, chars: &[char], output: &str, tokens: &[SourceToken], lk: &Lookups) -> String`.
  - `pub(crate) struct SourceToken { start, len, out_start, out_end: usize }` trong `crate::translate`.
  - `Options::quy_tac: bool` (mặc định `false`); `Engine::with_quy_tac(self, QuyTac) -> Engine`; `LiteEngine::with_quy_tac(self, QuyTac) -> LiteEngine`; `qt_core::{QuyTac, QuyTacError}`.

- [ ] **Step 1: Viết test hỏng**

Tạo `crates/qt-core/tests/quy_tac_engine.rs`:

```rust
use qt_core::{Dictionaries, DictionaryOverrides, Engine, LiteEngine, Mode, Options, QuyTac};

const RULES: &str = "[lớp]\nvị-trí: 上=trên\n\n[của 1]\n{danh-từ} {vị-trí} = {vị-trí} {danh-từ}\n";
const POS: &str = "桌子 10 n\n上 10 f\n";
const HAN_VIET: &str = "桌=trác\n子=tử\n上=thượng";
const VIETPHRASE: &str = "桌子=cái bàn\n上=bên trên";

fn quy_tac() -> QuyTac {
    QuyTac::from_sources(RULES, POS, "").unwrap()
}

fn on() -> Options {
    Options {
        quy_tac: true,
        ..Options::default()
    }
}

fn engine() -> Engine {
    Engine::from_dicts(Dictionaries::build(HAN_VIET, "", "", VIETPHRASE)).with_quy_tac(quy_tac())
}

#[test]
fn engine_reorders_only_when_enabled() {
    let engine = engine();
    let mode = Mode::VietPhraseOneMeaning;
    assert_eq!(engine.translate("桌子上", mode, &Options::default()), " cái bàn bên trên");
    assert_eq!(engine.translate("桌子上", mode, &on()), " Trên cái bàn");
    assert_eq!(
        engine
            .translate_with_overrides("桌子上", mode, &on(), &DictionaryOverrides::default())
            .unwrap(),
        " Trên cái bàn"
    );
}

#[test]
fn ranges_wrap_and_other_modes_are_untouched() {
    let engine = engine();
    let ranged = engine.translate_with_ranges("桌子上", Mode::VietPhraseOneMeaning, &on());
    assert_eq!(ranged.translated_text, " cái bàn bên trên");
    let wrapped = Options { wrap_type: 1, ..on() };
    assert_eq!(
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &wrapped),
        engine.translate("桌子上", Mode::VietPhraseOneMeaning, &Options { wrap_type: 1, ..Options::default() })
    );
    assert_eq!(
        engine.translate("桌子上", Mode::VietPhrase, &on()),
        engine.translate("桌子上", Mode::VietPhrase, &Options::default())
    );
}

#[test]
fn paragraphs_are_handled_separately() {
    let engine = engine();
    assert_eq!(
        engine.translate("桌子上\n上", Mode::VietPhraseOneMeaning, &on()),
        engine
            .translate("桌子上\n上", Mode::VietPhraseOneMeaning, &Options::default())
            .replacen("cái bàn bên trên", "Trên cái bàn", 1)
    );
}

#[test]
fn lite_matches_engine() {
    let lite = LiteEngine::build(HAN_VIET, "", "", VIETPHRASE, "", "", "", "", "")
        .unwrap()
        .with_quy_tac(quy_tac());
    assert_eq!(
        lite.translate("桌子上", &on(), &DictionaryOverrides::default()).unwrap(),
        " Trên cái bàn"
    );
}
```

- [ ] **Step 2: Chạy test, xác nhận hỏng**

Run: `cargo test -p qt-core --test quy_tac_engine`
Expected: FAIL biên dịch, báo không có `QuyTac` trong `qt_core` và không có trường `quy_tac`.

- [ ] **Step 3: Ghi cụm đã dịch trong vòng dịch**

Trong `crates/qt-core/src/translate.rs`:

Thêm ngay trên `#[derive(Default)] struct TranslationOutput`:

```rust
/// Một cụm đã ghi ra output: `start..start + len` là chỉ số ký tự nguồn (sau
/// chuẩn hoá và `number_modifier`), `out_start..out_end` là byte trong
/// `translated_text` (có thể dính dấu cách chèn trước). Quy Tắc đọc lại từ đây.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceToken {
    pub(crate) start: usize,
    pub(crate) len: usize,
    pub(crate) out_start: usize,
    pub(crate) out_end: usize,
}
```

Đổi `TranslationOutput` thành có lifetime và trường ghi cụm:

```rust
#[derive(Default)]
struct TranslationOutput<'t> {
    result: String,
    last: String,
    source_ranges: Vec<CharRange>,
    target_ranges: Vec<CharRange>,
    target_length: usize,
    tokens: Option<&'t mut Vec<SourceToken>>,
}

impl TranslationOutput<'_> {
```

(giữ nguyên các hàm cũ trong `impl`) và thêm vào `impl`:

```rust
    /// Ghi cụm vừa nối (`out_start` tới cuối output). Dấu cách cuối của cụm
    /// trước có thể đã bị `append_translated_word` bỏ, nên cắt lại cụm trước.
    fn record(&mut self, start: usize, len: usize, out_start: usize) {
        let out_end = self.result.len();
        if let Some(tokens) = self.tokens.as_deref_mut() {
            if let Some(previous) = tokens.last_mut() {
                previous.out_end = previous.out_end.min(out_start);
            }
            tokens.push(SourceToken {
                start,
                len,
                out_start,
                out_end,
            });
        }
    }
```

Trong `process_translation`, sau dòng `output.append_word(&text);` thêm:

```rust
    let out_start = output.result.len() - output.last.len();
    output.record(start, length, out_start);
```

Trong `process_han_viet`: nhánh `if is_chinese(c, han_viet)`, sau `output.append_word(&t);` thêm:

```rust
        let out_start = output.result.len() - output.last.len();
        output.record(*num2, 1, out_start);
```

Nhánh dấu nháy: sau `output.append_char(c);` thêm `output.record(*num2, 1, output.result.len() - c.len_utf8());`. Nhánh `else`: sau `output.append_char(c);` (trước `if needs_space_after_sentence_punctuation`) thêm cùng dòng đó.

Đổi chữ ký các hàm nhận output: `output: &mut TranslationOutput` giữ nguyên (lifetime được suy ra).

Trong `translate_all_mapped`, thêm tham số cuối và dùng nó:

```rust
    hau_tu: Option<&dyn DictionaryLookup>,
    tokens: Option<&mut Vec<SourceToken>>,
) -> TranslationResult {
```

và thay `let mut output = TranslationOutput::default();` bằng:

```rust
    let mut output = TranslationOutput {
        tokens,
        ..TranslationOutput::default()
    };
```

Trong `translate_all` (`#[cfg(test)]`), thêm đối số `None` vào cuối lời gọi `translate_all_mapped`.

- [ ] **Step 4: `QuyTac` công khai**

Thay toàn bộ `crates/qt-core/src/quy_tac/mod.rs` bằng:

```rust
//! Quy Tắc: đảo trật tự và chọn nghĩa trên token sau khi convert.
//! Spec: docs/superpowers/specs/2026-11-06-quy-tac-design.md. Mặc định tắt.

mod char_rules;
mod matcher;
mod numbers;
mod prepare;
mod program;
mod render;
mod tags;

use std::path::Path;

pub(crate) use prepare::Lookups;

use crate::translate::SourceToken;

/// Lỗi khi đọc file Quy Tắc; `line` là số dòng (tính từ 1), 0 khi không gắn dòng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuyTacError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for QuyTacError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(formatter, "QuyTac dòng {}: {}", self.line, self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for QuyTacError {}

/// Bộ quy tắc đã đọc cùng bảng từ loại.
pub struct QuyTac {
    program: program::Program,
    pos: tags::PosLexicon,
}

impl QuyTac {
    /// Bộ luật mặc định (spec mục 3).
    pub const DEFAULT_RULES: &'static str = include_str!("../../data/QuyTac.txt");
    /// Nhóm `[giọng-mượt]`: chọn nghĩa theo văn phong, chỉ chạy khi được bật.
    pub const GIONG_MUOT_RULES: &'static str = include_str!("../../data/QuyTac-giong-muot.txt");
    /// Nhãn từ loại sửa lại, đè lên `Resources/dict.txt`.
    pub const DEFAULT_TU_LOAI: &'static str = include_str!("../../data/TuLoai.txt");

    pub fn from_sources(rules: &str, pos_dict: &str, tu_loai: &str) -> Result<QuyTac, QuyTacError> {
        Ok(QuyTac {
            program: program::Program::parse(rules)?,
            pos: tags::PosLexicon::build(pos_dict, tu_loai),
        })
    }

    /// `QuyTac.txt` ở gốc thư mục dữ liệu nếu có, không thì bộ mặc định;
    /// `Resources/dict.txt` bắt buộc; `Resources/TuLoai.txt` nếu có.
    pub fn load(data_dir: &Path, giong_muot: bool) -> std::io::Result<QuyTac> {
        let read_optional = |rel: &str| std::fs::read_to_string(data_dir.join(rel)).ok();
        let mut rules =
            read_optional("QuyTac.txt").unwrap_or_else(|| Self::DEFAULT_RULES.to_string());
        if giong_muot {
            rules.push('\n');
            rules.push_str(Self::GIONG_MUOT_RULES);
        }
        let pos_dict = std::fs::read_to_string(data_dir.join("Resources/dict.txt"))?;
        let tu_loai = read_optional("Resources/TuLoai.txt")
            .unwrap_or_else(|| Self::DEFAULT_TU_LOAI.to_string());
        Self::from_sources(&rules, &pos_dict, &tu_loai).map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
        })
    }

    /// Áp quy tắc lên output của vòng dịch. Đoạn (ngăn bởi `\n`) nào không có
    /// quy tắc nào chạy thì giữ nguyên từng byte.
    pub(crate) fn apply(
        &self,
        chars: &[char],
        output: &str,
        tokens: &[SourceToken],
        lk: &Lookups,
    ) -> String {
        let mut result = String::with_capacity(output.len() + 64);
        let mut cursor = 0;
        let mut start = 0;
        for (index, token) in tokens.iter().enumerate() {
            if chars[token.start..token.start + token.len].contains(&'\n') {
                self.splice(chars, output, &tokens[start..index], lk, &mut result, &mut cursor);
                start = index + 1;
            }
        }
        self.splice(chars, output, &tokens[start..], lk, &mut result, &mut cursor);
        result.push_str(&output[cursor..]);
        result
    }

    fn splice(
        &self,
        chars: &[char],
        output: &str,
        paragraph: &[SourceToken],
        lk: &Lookups,
        result: &mut String,
        cursor: &mut usize,
    ) {
        fn text_of<'o>(output: &'o str, t: &SourceToken) -> &'o str {
            output[t.out_start..t.out_end].trim()
        }
        let Some(first) = paragraph.iter().find(|t| !text_of(output, t).is_empty()) else {
            return;
        };
        let last = paragraph
            .iter()
            .rev()
            .find(|t| !text_of(output, t).is_empty())
            .expect("có cụm đầu thì có cụm cuối");
        let raw: Vec<(String, String)> = paragraph
            .iter()
            .map(|t| {
                (
                    chars[t.start..t.start + t.len].iter().collect(),
                    text_of(output, t).to_string(),
                )
            })
            .collect();
        let Some(rendered) = self.apply_paragraph(raw, lk) else {
            return;
        };
        let head = &output[first.out_start..first.out_end];
        let from = first.out_start + (head.len() - head.trim_start().len());
        let tail = &output[last.out_start..last.out_end];
        let to = last.out_end - (tail.len() - tail.trim_end().len());
        if from < *cursor || to < from {
            return;
        }
        result.push_str(&output[*cursor..from]);
        result.push_str(&rendered);
        *cursor = to;
    }

    /// `None` khi không có quy tắc nào chạy.
    fn apply_paragraph(&self, raw: Vec<(String, String)>, lk: &Lookups) -> Option<String> {
        let (items, char_fired) = char_rules::apply_chars(&self.program, &raw, lk);
        let toks = prepare::prepare(items, &self.pos, lk, &self.program.classes);
        let (toks, fired) = matcher::apply_program(&self.program, toks);
        (char_fired + fired > 0).then(|| render::render(&toks))
    }
}
```

- [ ] **Step 5: Nối vào `lib.rs`**

Trong `crates/qt-core/src/lib.rs`:

1. Thêm `pub use quy_tac::{QuyTac, QuyTacError};` sau `pub use name_filter::{…};`.
2. Trong `struct Options`, thêm trường cuối:

```rust
    /// Chạy Quy Tắc sau khi convert (chỉ `VietPhraseOneMeaning`, `wrap_type == 0`,
    /// đường trả text, và engine có `QuyTac`). Mặc định tắt.
    pub quy_tac: bool,
```

   và trong `impl Default for Options` thêm `quy_tac: false,`.
3. Trong `struct Engine`, thêm trường cuối `quy_tac: Option<QuyTac>,`; trong `from_dicts` thêm `quy_tac: None,` vào literal.
4. Thêm vào `impl Engine` (sau `from_dicts`):

```rust
    /// Gắn bộ Quy Tắc; chỉ chạy khi `Options::quy_tac` bật.
    pub fn with_quy_tac(mut self, quy_tac: QuyTac) -> Engine {
        self.quy_tac = Some(quy_tac);
        self
    }

    fn quy_tac_active(&self, mode: Mode, opts: &Options) -> bool {
        opts.quy_tac
            && opts.wrap_type == 0
            && mode == Mode::VietPhraseOneMeaning
            && self.quy_tac.is_some()
    }
```

5. Đầu thân `translate_mapped` thêm:

```rust
        if !with_ranges && self.quy_tac_active(mode, opts) {
            return self
                .translate_mapped_with_overrides(text, mode, opts, &DictionaryOverrides::default(), false)
                .expect("không có override thì không lỗi");
        }
```

   và thêm `None` làm đối số cuối của hai lời gọi `translate::translate_all_mapped` trong hàm đó.
6. Trong `translate_mapped_with_overrides`, đổi điều kiện trả sớm thành:

```rust
        if overrides.is_empty() && !(!with_ranges && self.quy_tac_active(mode, opts)) {
            return Ok(self.translate_mapped(text, mode, opts, with_ranges));
        }
```

   và thêm `quy_tac: self.quy_tac.as_ref(),` vào literal `LayeredBase { … }`.
7. Trong `struct LayeredBase<'a>`, thêm trường cuối `quy_tac: Option<&'a QuyTac>,`.
8. Trong `LayeredBase::translate`, thay đoạn từ `let custom_luat_nhan = …` tới hết hàm bằng:

```rust
        let use_quy_tac = !with_ranges
            && opts.quy_tac
            && opts.wrap_type == 0
            && mode == Mode::VietPhraseOneMeaning
            && self.quy_tac.is_some();
        let custom_luat_nhan = overrides
            .luat_nhan
            .as_ref()
            .map(|rules| luat_nhan::LuatNhan::try_from_rules(rules))
            .transpose()
            .map_err(|message| DictionaryOverrideError { message })?;
        // Quy Tắc thay Luật Nhân: không luật nào, nhưng `{h}{t}` vẫn chạy nhờ
        // Họ người / Hậu tố truyền vào bên dưới.
        let no_rules = luat_nhan::LuatNhan::default();
        let luat_nhan = if use_quy_tac {
            &no_rules
        } else {
            custom_luat_nhan.as_ref().unwrap_or(self.luat_nhan)
        };
        let dictionary: &dyn DictionaryLookup = match mode {
            Mode::VietPhrase => &vietphrase,
            Mode::VietPhraseOneMeaning => &one_meaning_vietphrase,
            Mode::HanViet => unreachable!("HanViet returned above"),
        };
        let mut tokens = Vec::new();
        let result = translate::translate_all_mapped(
            &chars,
            with_ranges.then_some(source_ranges.as_slice()),
            opts,
            dictionary,
            names,
            only_vietphrase,
            &vietphrase,
            han_viet,
            luat_nhan,
            Some(&dictionary_n),
            Some(ho_nguoi),
            Some(hau_tu),
            use_quy_tac.then_some(&mut tokens),
        );
        match self.quy_tac {
            Some(quy_tac) if use_quy_tac => {
                let lexicon = OneMeaningDictionary {
                    inner: only_vietphrase,
                };
                let lookups = quy_tac::Lookups {
                    names,
                    lexicon: &lexicon,
                };
                // Token chỉ vào chuỗi đã qua number_modifier, như vòng dịch.
                let modified = number::number_modifier(&chars);
                let text = quy_tac.apply(&modified, &result.translated_text, &tokens, &lookups);
                Ok(TranslationResult {
                    translated_text: text,
                    source_ranges: Vec::new(),
                    target_ranges: Vec::new(),
                })
            }
            _ => Ok(result),
        }
```

Trong `crates/qt-core/src/lite.rs`: thêm `use crate::QuyTac;` (hoặc thêm vào khối `use crate::{…}`), trường cuối `quy_tac: Option<QuyTac>,` trong `struct LiteEngine`, `quy_tac: None,` trong literal của `assemble`, hàm:

```rust
    /// Gắn bộ Quy Tắc; chỉ chạy khi `Options::quy_tac` bật.
    pub fn with_quy_tac(mut self, quy_tac: QuyTac) -> Self {
        self.quy_tac = Some(quy_tac);
        self
    }
```

và `quy_tac: self.quy_tac.as_ref(),` trong literal `LayeredBase { … }` của `translate`.

Trong `apps/qt-gui/src-tauri/src/lib.rs`, literal `Ok(Options { … })` (dòng ~434) thêm `quy_tac: false,` sau `scan_range: request.scan_range,`.

- [ ] **Step 6: Chạy test**

Run: `cargo test -p qt-core --test quy_tac_engine && cargo test -p qt-core`
Expected: PASS. Không còn cảnh báo `dead_code` trong `quy_tac`; nếu còn hàm không dùng thì xoá, không tắt cảnh báo.

- [ ] **Step 7: Biên dịch các crate phụ thuộc**

Run: `cargo build -p qt-api -p qt-cli -p qt-lambda && cargo check -p qt-gui`
Expected: build OK. Nếu `qt-gui` hỏng vì lý do không liên quan `Options` (thiếu frontend…), ghi lại lỗi và báo user, không sửa.

- [ ] **Step 8: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/src crates/qt-core/tests/quy_tac_engine.rs apps/qt-gui/src-tauri/src/lib.rs
git commit -m "feat(core): nối Quy Tắc vào engine và lite — Options::quy_tac (tắt mặc định), Luật Nhân rỗng khi bật"
```

---

### Task 9: So Rust với bản thử trên câu mẫu

**Files:**
- Create: `crates/qt-core/tests/data/quy_tac_golden.tsv`
- Create: `crates/qt-core/tests/quy_tac_golden.rs`

**Interfaces:**
- Consumes: `Engine::with_quy_tac`, `QuyTac::load`, `Options::quy_tac`, `Dictionaries::load`, `DictionarySourceOverrides { luat_nhan: Some("") }`.

- [ ] **Step 1: Tạo file đáp án**

Tạo `crates/qt-core/tests/data/quy_tac_golden.tsv` (UTF-8, LF, cột phân cách bằng TAB). Cột 2 là `=` khi bản thử không có quy tắc nào chạy, tức output phải bằng convert khi tắt Luật Nhân:

```text
他站在高台上。	Hắn đứng trên đài cao.
这丹药比普通的驻颜丹好。	Đan dược này tốt hơn trú Nhan đan bình thường.
在音波的冲击下。	Dưới xung kích của sóng âm.
这是掌门的命令。	Đây là mệnh lệnh của chưởng môn.
美丽的少女看着远处的山峰。	Thiếu nữ mỹ lệ nhìn phía xa ngọn núi.
他走进来的时候，大家都愣住了。	Khi hắn đi tới, tất cả mọi người sửng sốt.
很有责任感的男人。	Nam nhân rất có tinh thần trách nhiệm.
属于修行者的气息。	Khí tức thuộc về người tu hành.
你连这种事都不知道？	Ngươi ngay cả loại sự tình này cũng không biết?
他在吃饭。	=
这个世界的规则。	Quy tắc của thế giới này.
站在她身后的女孩。	Nữ hài đứng sau lưng nàng.
2015年6月11日，晚上九点半。	Ngày 11 tháng 6 năm 2015, chín giờ rưỡi tối.
下午三点二十分。	Ba giờ hai mươi phút chiều.
第三个人走了进来。	=
三十里外有一座城。	Ngoài ba mươi dặm có một tòa thành.
他活了三百五十岁。	Hắn sống ba trăm năm mươi tuổi.
百分之五十的把握。	Năm mươi phần trăm nắm chắc.
三分之一的人死了。	=
星期三见。	=
第二层的第四道门。	Cửa thứ tư của tầng thứ hai.
第 12 章 重逢	Chương 12: trùng phùng
三点十五分出发。	Ba giờ mười lăm phút xuất phát.
三点一刻到了。	Ba giờ mười lăm phút đến.
圆周率是三点一四。	Số Pi là ba phẩy một bốn.
百分之三点五的概率。	Ba phẩy năm phần trăm xác suất.
二〇一九年冬天。	Năm 2019 mùa đông.
他拿出三万元。	Hắn lấy ra ba vạn tệ.
五百块下品灵石。	Năm trăm khối hạ phẩm linh thạch.
十年过去了。	=
有一天他来了。	=
他一个人走了。	=
两点一线的生活。	=
三十六岁的男人。	Ba mươi sáu tuổi nam nhân.
他是三阶武者。	=
三个小时后。	Ba tiếng sau.
第四层有人。	Tầng thứ tư có người.
第一次见面。	Lần đầu gặp mặt.
第二天早上。	=
十多里外。	Hơn mười dặm bên ngoài.
几十个人。	=
他在这里活了五百年。	Hắn tại chỗ này sống năm trăm năm.
三天前他来过。	Ba ngày trước hắn tới qua.
五分钟后出发。	Năm phút sau xuất phát.
三年之内必有大成。	Trong vòng ba năm nhất định có đại thành.
一个半小时过去了。	Một tiếng rưỡi đi qua.
两个星期后。	Hai tuần sau.
三点到五点在家。	Từ ba giờ đến năm giờ ở nhà.
公元前二百年。	Năm hai trăm trước Công nguyên.
晚上九点多钟。	Hơn chín giờ tối.
城墙周围都是人。	Xung quanh tường thành đều là người.
三天左右。	Khoảng ba ngày.
他刚刚回来。	Hắn vừa trở về.
她正在吃饭。	=
他变得很强。	Hắn trở nên rất mạnh.
既然来了，就别走了。	Đã đến, cũng đừng đi nha.
谁在那里？	Ai ở đâu?
他看着她。	Hắn nhìn nàng.
她身上有伤。	=
他似乎很累。	=
```

Kiểm tra: `grep -c . crates/qt-core/tests/data/quy_tac_golden.tsv` ra `60`, và `grep -cP '\t' crates/qt-core/tests/data/quy_tac_golden.tsv` ra `60`.

- [ ] **Step 2: Viết test**

Tạo `crates/qt-core/tests/quy_tac_golden.rs`:

```rust
//! Bản Rust phải ra đúng như bản thử Python (docs/superpowers/specs/2026-11-06-quy-tac/)
//! trên các câu mẫu: QT2025 thật, bộ luật mặc định, không names riêng.
//! Nạp QT2025 nên chậm: chạy bằng
//! `cargo test --release -p qt-core --test quy_tac_golden -- --ignored`.

use std::path::PathBuf;

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options, QuyTac,
};
use unicode_normalization::UnicodeNormalization;

#[test]
#[ignore = "nạp QT2025 thật, chạy riêng ở bản release"]
fn matches_prototype_on_golden_sentences() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../QT2025");
    let engine = Engine::from_dicts(Dictionaries::load(&data).unwrap())
        .with_quy_tac(QuyTac::load(&data, false).unwrap());
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
```

- [ ] **Step 3: Chạy test**

Run: `cargo test --release -p qt-core --test quy_tac_golden -- --ignored`
Expected: PASS. Nếu có câu lệch, với từng câu:
- tìm nguyên nhân bằng cách so token Rust với bản thử. Chạy bản thử theo `docs/superpowers/specs/2026-11-06-quy-tac/` với `tokdump.rs` và `quytac.py`;
- lệch do port sai thì sửa port, viết thêm unit test cho chỗ đó ở module tương ứng;
- lệch có lý do chính đáng (tên trong `Names2/123.txt` mà bản thử không nạp, `first_meaning` cắt cả `|`) thì giữ output Rust, sửa dòng đáp án và ghi lý do vào message commit.

Không được sửa đáp án chỉ để test qua.

- [ ] **Step 4: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/tests/data/quy_tac_golden.tsv crates/qt-core/tests/quy_tac_golden.rs
git commit -m "test(core): Quy Tắc khớp bản thử trên 60 câu mẫu (QT2025 thật)"
```

---

### Task 10: Cổng chất lượng, tốc độ, RAM trên bookshelf

**Files:**
- Create: `crates/qt-core/examples/quy_tac_eval.rs`
- Modify: `docs/superpowers/specs/2026-11-06-quy-tac-design.md` (mục 12, ghi số đo thật)

**Interfaces:**
- Consumes: API công khai của Task 8, `serde_json` (dev), `regex`, `unicode_normalization`.

- [ ] **Step 1: Viết example**

Tạo `crates/qt-core/examples/quy_tac_eval.rs`:

```rust
//! Cổng bước 1 của spec Quy Tắc (mục 12): chất lượng, tốc độ, RAM trên bộ test bookshelf.
//!
//! cargo run --release -p qt-core --example quy_tac_eval -- <QT2025> <books/common> [chương-mỗi-truyện=12] [lệch=7]
//!
//! Bộ test giống bản thử: mỗi truyện lấy rải đều 12 chương có bản biên tập
//! (`out/` cùng tên với `raw/`, cùng số đoạn), bắt đầu từ chương thứ 8. Mỗi
//! đoạn dịch hai lần: convert hiện tại (có Luật Nhân) và Quy Tắc. Chấm bằng số
//! cụm 2 và 3 từ trùng với bản biên tập (chữ thường, NFC).

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Duration, Instant};

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options, QuyTac,
};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_add(new_size as isize - layout.size() as isize, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn words(text: &str, word: &Regex) -> Vec<String> {
    let lowered = text.nfc().collect::<String>().to_lowercase();
    word.find_iter(&lowered).map(|m| m.as_str().to_string()).collect()
}

fn grams(words: &[String], n: usize) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for window in words.windows(n) {
        *out.entry(window.join("\u{1}")).or_insert(0) += 1;
    }
    out
}

fn overlap(text: &HashMap<String, usize>, reference: &HashMap<String, usize>) -> u64 {
    text.iter()
        .map(|(gram, count)| (*count).min(*reference.get(gram).unwrap_or(&0)) as u64)
        .sum()
}

fn non_blank_lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).collect()
}

fn story_names(book: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(book.join("story.json")) else {
        return String::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return String::new();
    };
    value
        .get("glossary")
        .and_then(|g| g.get("names"))
        .and_then(|n| n.as_object())
        .map(|names| {
            names
                .iter()
                .map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: quy_tac_eval <QT2025> <books/common> [chương-mỗi-truyện=12] [lệch=7]");
        return ExitCode::from(2);
    }
    let data = PathBuf::from(&args[0]);
    let books = PathBuf::from(&args[1]);
    let per_book: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(12);
    let offset: usize = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(7);

    let engine = Engine::from_dicts(Dictionaries::load(&data).expect("nạp QT2025"));
    let before = LIVE.load(Ordering::Relaxed);
    let quy_tac = QuyTac::load(&data, false).expect("nạp Quy Tắc");
    let quy_tac_bytes = LIVE.load(Ordering::Relaxed) - before;
    let engine = engine.with_quy_tac(quy_tac);
    let word = Regex::new(r"\w+").unwrap();
    let off = Options::default();
    let on = Options {
        quy_tac: true,
        ..Options::default()
    };
    let mode = Mode::VietPhraseOneMeaning;

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&books)
        .expect("đọc thư mục truyện")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("raw").is_dir() && path.join("out").is_dir())
        .collect();
    dirs.sort();

    let (mut cur2, mut cur3, mut new2, mut new3) = (0u64, 0u64, 0u64, 0u64);
    let (mut paragraphs, mut changed, mut better, mut worse) = (0usize, 0usize, 0usize, 0usize);
    let (mut cur_time, mut new_time) = (Duration::ZERO, Duration::ZERO);
    let mut chars = 0usize;
    for book in &dirs {
        let names = story_names(book);
        let overrides = if names.is_empty() {
            DictionaryOverrides::default()
        } else {
            DictionaryOverrides::from_sources(DictionarySourceOverrides {
                names: Some(&names),
                ..Default::default()
            })
        };
        let mut files: Vec<PathBuf> = std::fs::read_dir(book.join("raw"))
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "txt")
                    && book.join("out").join(path.file_name().unwrap()).is_file()
            })
            .collect();
        files.sort();
        if files.is_empty() {
            continue;
        }
        let step = (files.len() / per_book).max(1);
        for file in files.iter().skip(offset.min(files.len() - 1)).step_by(step).take(per_book) {
            let (Ok(raw), Ok(reference)) = (
                std::fs::read_to_string(file),
                std::fs::read_to_string(book.join("out").join(file.file_name().unwrap())),
            ) else {
                continue;
            };
            let (raw, reference) = (non_blank_lines(&raw), non_blank_lines(&reference));
            if raw.len() != reference.len() {
                continue;
            }
            for (source, edited) in raw.iter().zip(&reference) {
                chars += source.chars().count();
                let started = Instant::now();
                let current = engine
                    .translate_with_overrides(source, mode, &off, &overrides)
                    .unwrap();
                cur_time += started.elapsed();
                let started = Instant::now();
                let next = engine
                    .translate_with_overrides(source, mode, &on, &overrides)
                    .unwrap();
                new_time += started.elapsed();
                let reference_words = words(edited, &word);
                let (ref2, ref3) = (grams(&reference_words, 2), grams(&reference_words, 3));
                let current_words = words(&current, &word);
                let next_words = words(&next, &word);
                let (c2, c3) = (
                    overlap(&grams(&current_words, 2), &ref2),
                    overlap(&grams(&current_words, 3), &ref3),
                );
                let (n2, n3) = (
                    overlap(&grams(&next_words, 2), &ref2),
                    overlap(&grams(&next_words, 3), &ref3),
                );
                cur2 += c2;
                cur3 += c3;
                new2 += n2;
                new3 += n3;
                paragraphs += 1;
                if next != current {
                    changed += 1;
                    if n2 + n3 > c2 + c3 {
                        better += 1;
                    } else if n2 + n3 < c2 + c3 {
                        worse += 1;
                    }
                }
            }
        }
    }

    let gain = |new: u64, cur: u64| (new as f64 - cur as f64) * 100.0 / cur.max(1) as f64;
    let win_rate = better as f64 * 100.0 / (better + worse).max(1) as f64;
    let speed = new_time.as_secs_f64() / cur_time.as_secs_f64().max(1e-9);
    let ns = |time: Duration| time.as_secs_f64() * 1e9 / chars.max(1) as f64;
    let mib = quy_tac_bytes as f64 / (1024.0 * 1024.0);
    println!(
        "{paragraphs} đoạn, {chars} ký tự; đổi {changed} ({:.1}%); tốt hơn {better}, kém hơn {worse}; thắng {win_rate:.1}%",
        changed as f64 * 100.0 / paragraphs.max(1) as f64
    );
    println!("cụm 2 từ: hiện tại {cur2}, Quy Tắc {new2} ({:+.2}%)", gain(new2, cur2));
    println!("cụm 3 từ: hiện tại {cur3}, Quy Tắc {new3} ({:+.2}%)", gain(new3, cur3));
    println!(
        "thời gian: hiện tại {:.0} ns/ký tự, Quy Tắc {:.0} ns/ký tự ({:.0}%)",
        ns(cur_time),
        ns(new_time),
        speed * 100.0
    );
    println!("RAM giữ của QuyTac: {mib:.1} MiB");
    let gates = [
        ("chất lượng: cụm 3 từ >= +10,0%", gain(new3, cur3) >= 10.0),
        ("chất lượng: thắng >= 80%", win_rate >= 80.0),
        ("tốc độ: <= 70% hiện tại", speed <= 0.70),
        ("RAM: <= 10 MiB", mib <= 10.0),
    ];
    let mut ok = true;
    for (name, pass) in gates {
        println!("{} {name}", if pass { "ĐẠT " } else { "HỎNG" });
        ok &= pass;
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
```

- [ ] **Step 2: Chạy**

Run: `cargo run --release -p qt-core --example quy_tac_eval -- QT2025 D:/Work/projects/bookshelf/books/common`
Expected: in số đo, bốn dòng `ĐẠT`, exit 0. Tham chiếu của bản thử trên cùng bộ test: cụm 3 từ +10,79%, thắng 82,8%. Bản thử dịch cả chương một lần còn example dịch từng đoạn, nên số lệch một chút là bình thường.

- [ ] **Step 3: Xử lý kết quả**

- Bốn cổng đều đạt: ghi số đo thật (đoạn, cụm 2 / 3 từ, thắng, ns/ký tự hai đường, MiB) vào spec mục 12, dưới bảng cổng, kèm ngày.
- Có cổng hỏng: **dừng**, không chỉnh bộ luật theo bộ test. Ghi số vào spec mục 12 kèm chữ "chưa đạt", báo user con số, rồi đề xuất hướng sửa và chờ user chốt. Cổng tốc độ hỏng thì đo thêm thời gian từng phần (vòng dịch, `apply_chars`, `prepare`, `apply_program`) bằng `Instant` tạm thời trong example để chỉ đúng chỗ.

- [ ] **Step 4: Commit** (khi user đã cho phép)

```bash
git add crates/qt-core/examples/quy_tac_eval.rs docs/superpowers/specs/2026-11-06-quy-tac-design.md
git commit -m "test(core): cổng Quy Tắc trên bookshelf — chất lượng, tốc độ, RAM (số đo trong spec mục 12)"
```

---

## Self-Review

**Độ phủ spec (bước 0–1):**
- Mục 10 (NFC khi nạp): Task 1.
- Mục 4 (luồng: `[số]` → tách từ → tách hư từ → gắn nhãn → nhóm → ráp; đoạn không đổi giữ nguyên byte): Task 6, 5, 7, 8.
- Mục 5 (cú pháp: lớp, chữ cố định nhiều token, `{x}+`, `{vế}`, `{mệnh-đề}`, `{đoạn}`, ngữ cảnh `^ $ !`, khuôn theo số / tên, bộ lọc, ưu tiên, `lặp=N`, lỗi theo số dòng): Task 3, 7.
- Mục 6 (từ loại, `TuLoai.txt` đè `dict.txt`, tên luôn là `tên`, `是` → `hệ-từ`): Task 4, 5.
- Mục 7 (tách hư từ có điều kiện cùng trật tự): Task 5.
- Mục 8 (số giữ kiểu nguồn, `{số:a-b}`, `{năm}`, thắng tách từ nhưng nhường cụm dài hơn, năm lịch ≠ khoảng thời gian): Task 2, 3, 6.
- Mục 9 (chọn nghĩa): nằm trong bộ luật mặc định, chạy qua Task 7. Nhóm giọng mượt nạp được qua `QuyTac::load(dir, true)`.
- Mục 12 (cổng hiệu năng) và mục 13 (khớp bản thử, câu mẫu): Task 9, 10.

**Để sang kế hoạch sau:**
- Cờ `tắt` của nhóm và `Options::bat_nhom`/`tat_nhom`. Kế hoạch này chỉ có cách nạp nhóm giọng mượt bằng tham số.
- Quy tắc chống khớp nửa chuỗi số `三十二点`, vì bản thử chưa có và thêm vào sẽ lệch đáp án mẫu.
- Range trên đường Quy Tắc; CLI `qt quy-tac …`; `dao`; API / lite server / Lambda / app; xoá Luật Nhân.

**Kiểm tên và kiểu:**
- `Raw`, `Token`, `Lookups` (`prepare`) dùng đúng tên ở `char_rules`, `matcher`, `mod`.
- `SourceToken` (`translate`) dùng ở `mod::apply` và `LayeredBase`.
- `apply_chars` trả `(Vec<Raw>, usize)` và `apply_program` trả `(Vec<Token>, usize)`, `apply_paragraph` dùng đúng hai kiểu đó.
- `Program.classes: FxHashMap<String, Class>` khớp chữ ký `prepare`.
