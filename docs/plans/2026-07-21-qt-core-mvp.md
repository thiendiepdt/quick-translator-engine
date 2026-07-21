# QT-Core MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust `qt-core` engine that reproduces Quick Translator's HanViet, VietPhrase, and VietPhraseOneMeaning modes for text without number/grammar rules, plus a thin `qt` CLI to run it.

**Architecture:** Cargo workspace. `qt-core` is a pure library: parse dictionaries → build lookup maps with QT's priority merge → translate via the `TranslateAll` longest-match loop. Number conversion and Luật Nhân are **stubbed** in this plan (identity/no-op) and land in a later plan. `qt-cli` is a thin binary wrapping `qt-core` over stdin/stdout.

**Tech Stack:** Rust (stable, edition 2021), std-only for `qt-core` (no external crates), `cargo test` for TDD.

## Global Constraints

- Language: Rust edition 2021, must build on stable `cargo build`.
- `qt-core` has **zero external dependencies** in this plan (std only). Do not add crates.
- Faithfulness rule: every algorithm mirrors `docs/engine/` exactly. When in doubt, consult `reference/decompiled/TranslatorEngine.decompiled.cs`.
- `is_chinese(c)` MUST be defined as "`c` is a key in the Han-Việt map" — never Unicode ranges (see `docs/engine/han-viet.md`).
- Input is indexed by Unicode scalar (`Vec<char>`). Chinese text + ASCII punctuation are all BMP, so this matches QT's UTF-16 char indexing for all real input. (Supplementary-plane hanzi can never be Han-Việt keys, so they fall through to raw output either way.)
- Stubs (`number_modifier`, `prescan_numbers`, `handle_nhan_by`) must be real functions with the stub behavior documented in-code, so the later plan only replaces their bodies.
- Commit after every task with the message shown in its final step.

---

## File Structure

```
Cargo.toml                          # workspace root
crates/
  qt-core/
    Cargo.toml
    src/
      lib.rs                        # public API: Engine, Mode, Options, re-exports
      dict.rs                       # parse_dict, Dictionaries, load/merge/priority
      text.rs                       # to_narrow, to_upper_case, append_translated_word, wrap_translation, next_char_is_chinese
      han_viet.rs                   # is_chinese, char_to_han_viet, chinese_to_han_viet_string
      translate.rs                  # is_longest_phrase_in_sentence, contains_name, translate_all, stubs
  qt-cli/
    Cargo.toml
    src/main.rs                     # stdin/stdout, --mode, --data-dir
```

Responsibilities: `dict.rs` owns all loading + the priority merge (the riskiest correctness area, isolated). `text.rs` owns string joining/capitalization (the second-riskiest). `han_viet.rs` and `translate.rs` own the two translation paths. `lib.rs` is the only public surface.

---

### Task 1: Workspace + qt-core skeleton + public types

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `crates/qt-core/Cargo.toml`
- Create: `crates/qt-core/src/lib.rs`
- Test: inline in `lib.rs`

**Interfaces:**
- Produces: `pub enum Mode { HanViet, VietPhrase, VietPhraseOneMeaning }`; `pub struct Options { pub wrap_type: i32, pub translation_algorithm: i32, pub prioritized_name: bool, pub scan_range: usize }` with `Default`; `pub struct Engine { dicts: Dictionaries }` (dicts private, added Task 6 — for now a unit field placeholder).

- [ ] **Step 1: Write workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/qt-core", "crates/qt-cli"]
```

- [ ] **Step 2: Write crates/qt-core/Cargo.toml**

```toml
[package]
name = "qt-core"
version = "0.1.0"
edition = "2021"

[dependencies]
```

- [ ] **Step 3: Write the failing test in lib.rs**

```rust
//! qt-core: Quick Translator engine (Rust reimplementation).

/// Output view, mirrors QT's translation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    HanViet,
    VietPhrase,
    VietPhraseOneMeaning,
}

/// Translation parameters, matching the original engine signature.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub wrap_type: i32,             // 0 = plain, 1 = wrap each phrase in [...]
    pub translation_algorithm: i32, // 0/1/2, controls "longest phrase" rule
    pub prioritized_name: bool,
    pub scan_range: usize,          // max phrase length scanned per position
}

impl Default for Options {
    fn default() -> Self {
        // Defaults decoded from QT2025/Resources/QuickTranslatorMain.config.
        Options { wrap_type: 0, translation_algorithm: 1, prioritized_name: true, scan_range: 30 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_matches_spec() {
        let o = Options::default();
        assert_eq!(o.wrap_type, 0);
        assert_eq!(o.translation_algorithm, 1);
        assert!(o.prioritized_name);
        assert_eq!(o.scan_range, 30);
        assert_eq!(Mode::HanViet, Mode::HanViet);
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qt-core options_default_matches_spec`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/qt-core
git commit -m "feat(qt-core): workspace skeleton + Mode/Options types"
```

---

### Task 2: Dictionary line parser

**Files:**
- Create: `crates/qt-core/src/dict.rs`
- Modify: `crates/qt-core/src/lib.rs` (add `mod dict;`)
- Test: inline in `dict.rs`

**Interfaces:**
- Produces: `pub fn parse_dict(content: &str) -> Vec<(String, String)>` — splits `key=value` lines, mirrors QT: strips a leading UTF-8 BOM, splits on `=` into all parts and keeps a line **only if it yields exactly 2 parts**, skips `#` comment lines.

> **CORRECTION (applied after final review):** the `#`-skip below is WRONG — the engine only skips `#` in the LuatNhan loader, not for HanViet/VietPhrase/Names (see docs/engine/dictionaries.md §2). The shipped code removed the `#`-skip from `parse_dict`; a `#key=val` line is kept. The LuatNhan loader will apply its own `#`-skip in a later plan. The code block below is the original (superseded) version.

- [ ] **Step 1: Write the failing test in dict.rs**

```rust
//! Dictionary loading, merging, and QT's priority rules.

/// Parse `key=value` lines the way TranslatorEngine does:
/// - a leading BOM on the first line is stripped
/// - split on '=' into ALL parts; keep the line only if exactly 2 parts result
///   (so a value containing '=' drops the line, matching QT)
/// - lines starting with '#' are comments (skipped)
pub fn parse_dict(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = if i == 0 { raw.trim_start_matches('\u{feff}') } else { raw };
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            out.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bom_and_skips_bad_lines() {
        let content = "\u{feff}一=nhất\n# comment\n二=nhị\na=b=c\nnoequals\n";
        let got = parse_dict(content);
        assert_eq!(got, vec![
            ("一".to_string(), "nhất".to_string()),
            ("二".to_string(), "nhị".to_string()),
        ]);
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

Add near the top of `crates/qt-core/src/lib.rs`, after the doc comment:

```rust
mod dict;
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test -p qt-core parses_bom_and_skips_bad_lines`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/qt-core/src/dict.rs crates/qt-core/src/lib.rs
git commit -m "feat(qt-core): dictionary line parser (BOM, first-wins split)"
```

---

### Task 3: Han-Việt map, is_chinese, char translation, to_narrow

**Files:**
- Create: `crates/qt-core/src/han_viet.rs`
- Modify: `crates/qt-core/src/lib.rs` (add `mod han_viet;`)
- Test: inline in `han_viet.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub type HanVietMap = std::collections::HashMap<char, String>;`
  `pub fn is_chinese(c: char, han_viet: &HanVietMap) -> bool`;
  `pub fn char_to_han_viet(c: char, han_viet: &HanVietMap) -> String`;
  `pub fn to_narrow(s: &str) -> String`.

- [ ] **Step 1: Write the failing test in han_viet.rs**

```rust
//! Han-Việt phonetic transcription (single-char) and QT's `isChinese` definition.

use std::collections::HashMap;

pub type HanVietMap = HashMap<char, String>;

/// QT's definition: a char is "Chinese" iff it has a Han-Việt reading.
pub fn is_chinese(c: char, han_viet: &HanVietMap) -> bool {
    han_viet.contains_key(&c)
}

/// Full-width `！`..`～` (U+FF01..U+FF5E) → ASCII `!`..`~`; others unchanged.
pub fn to_narrow(s: &str) -> String {
    s.chars()
        .map(|c| {
            if ('\u{FF01}'..='\u{FF5E}').contains(&c) {
                char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

/// Transcribe one char: space → ""; known char → reading; else to_narrow(char).
pub fn char_to_han_viet(c: char, han_viet: &HanVietMap) -> String {
    if c == ' ' {
        return String::new();
    }
    match han_viet.get(&c) {
        Some(v) => v.clone(),
        None => to_narrow(&c.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hv() -> HanVietMap {
        let mut m = HanVietMap::new();
        m.insert('一', "nhất".to_string());
        m
    }

    #[test]
    fn is_chinese_uses_dict_not_unicode() {
        let m = hv();
        assert!(is_chinese('一', &m));
        assert!(!is_chinese('二', &m)); // valid hanzi, but not in dict → not "Chinese"
        assert!(!is_chinese('A', &m));
    }

    #[test]
    fn char_translation_and_to_narrow() {
        let m = hv();
        assert_eq!(char_to_han_viet('一', &m), "nhất");
        assert_eq!(char_to_han_viet(' ', &m), "");
        // full-width '３' U+FF13 → '3'; unknown non-fullwidth passes through
        assert_eq!(char_to_han_viet('３', &m), "3");
        assert_eq!(to_narrow("ＡＢ!"), "AB!");
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

Add after `mod dict;`:

```rust
mod han_viet;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p qt-core --lib han_viet`
Expected: PASS (2 tests in the han_viet module).

- [ ] **Step 4: Commit**

```bash
git add crates/qt-core/src/han_viet.rs crates/qt-core/src/lib.rs
git commit -m "feat(qt-core): han-viet map, is_chinese, char translation, to_narrow"
```

---

### Task 4: Text helpers (join, capitalize, wrap)

**Files:**
- Create: `crates/qt-core/src/text.rs`
- Modify: `crates/qt-core/src/lib.rs` (add `mod text;`)
- Test: inline in `text.rs`

**Interfaces:**
- Consumes: `is_chinese`, `HanVietMap` from `han_viet`.
- Produces:
  `pub fn wrap_translation(t: &str, wrap_type: i32) -> String`;
  `pub fn to_upper_case(text: &str) -> String`;
  `pub fn append_translated_word(result: &mut String, translated: &str, last: &mut String)`;
  `pub fn next_char_is_chinese(chars: &[char], end_idx: usize, han_viet: &HanVietMap) -> bool`.

- [ ] **Step 1: Write the failing test in text.rs**

```rust
//! String assembly: wrapping, sentence-start capitalization, spacing.
//! Mirrors appendTranslatedWord / WrapTranslation / nextCharIsChinese.

use crate::han_viet::{is_chinese, HanVietMap};

pub fn wrap_translation(t: &str, wrap_type: i32) -> String {
    if wrap_type == 0 {
        t.to_string()
    } else {
        format!("[{t}]")
    }
}

/// Capitalize first char; if it starts with '[' (wrapped), capitalize the char after '['.
pub fn to_upper_case(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars[0] != '[' || chars.len() < 2 {
        let head: String = chars[0].to_uppercase().collect();
        let tail: String = chars[1..].iter().collect();
        format!("{head}{tail}")
    } else {
        let head: String = chars[1].to_uppercase().collect();
        let tail: String = chars[2..].iter().collect();
        format!("[{head}{tail}")
    }
}

const SENTENCE_ENDERS: [&str; 11] =
    ["\n", "\t", ". ", "\"", "'", "? ", "! ", ".\" ", "?\" ", "!\" ", ": "];

/// Append `translated` to `result`, tracking `last` (the previously appended chunk).
/// - after a sentence-ender → capitalize first letter
/// - after a space or '(' → join directly
/// - otherwise → insert one leading space
/// Then, if the new chunk starts with , . ? ! and result ends with a space, drop that space.
pub fn append_translated_word(result: &mut String, translated: &str, last: &mut String) {
    let new_last = if SENTENCE_ENDERS.iter().any(|e| last.ends_with(e)) {
        to_upper_case(translated)
    } else if last.ends_with(' ') || last.ends_with('(') {
        translated.to_string()
    } else {
        format!(" {translated}")
    };
    *last = new_last;

    let starts_punct = translated.is_empty()
        || matches!(translated.chars().next(), Some(',') | Some('.') | Some('?') | Some('!'));
    if starts_punct && result.ends_with(' ') {
        result.pop();
    }
    result.push_str(last);
}

pub fn next_char_is_chinese(chars: &[char], end_idx: usize, han_viet: &HanVietMap) -> bool {
    if chars.len() > end_idx + 1 {
        is_chinese(chars[end_idx + 1], han_viet)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_and_upper() {
        assert_eq!(wrap_translation("x", 0), "x");
        assert_eq!(wrap_translation("x", 1), "[x]");
        assert_eq!(to_upper_case("hắn"), "Hắn");
        assert_eq!(to_upper_case("[hắn]"), "[Hắn]");
    }

    #[test]
    fn join_default_inserts_space() {
        let mut r = String::new();
        let mut last = String::new();
        append_translated_word(&mut r, "hắn", &mut last); // first word, last empty → " hắn"
        append_translated_word(&mut r, "rất", &mut last); // last="hắn" → " rất"
        assert_eq!(r, " hắn rất");
    }

    #[test]
    fn join_capitalizes_after_sentence_end() {
        let mut r = String::from("A.");
        let mut last = String::from(". "); // simulate previous ended a sentence
        append_translated_word(&mut r, "hắn", &mut last);
        assert!(r.ends_with("Hắn"));
    }

    #[test]
    fn join_drops_space_before_punct() {
        let mut r = String::from("hắn");
        let mut last = String::from("hắn"); // does NOT end with space
        // ',' start → new_last=" ,"? No: default branch makes " ," then punct rule trims result space.
        append_translated_word(&mut r, ",", &mut last);
        // result had no trailing space, so nothing trimmed; last becomes " ,"
        assert_eq!(r, "hắn ,");
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

Add after `mod han_viet;`:

```rust
mod text;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p qt-core --lib text`
Expected: PASS (4 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/qt-core/src/text.rs crates/qt-core/src/lib.rs
git commit -m "feat(qt-core): text join/capitalize/wrap helpers"
```

---

### Task 5: HanViet string mode + Engine::from_dicts + translate

**Files:**
- Modify: `crates/qt-core/src/han_viet.rs` (add `chinese_to_han_viet_string`)
- Modify: `crates/qt-core/src/dict.rs` (add `Dictionaries` struct, minimal for now)
- Modify: `crates/qt-core/src/lib.rs` (add `Engine`, `translate`)
- Test: inline in `lib.rs`

**Interfaces:**
- Consumes: `append_translated_word`, `is_chinese`, `char_to_han_viet`.
- Produces:
  `pub fn chinese_to_han_viet_string(chars: &[char], han_viet: &HanVietMap) -> String` (han_viet.rs);
  `pub struct Dictionaries { pub han_viet: HanVietMap, pub vietphrase: HashMap<String,String>, pub vietphrase_one_meaning: HashMap<String,String>, pub only_name: HashMap<String,String> }` (dict.rs, fields filled in Task 6; empty maps OK now);
  `impl Engine { pub fn from_dicts(d: Dictionaries) -> Engine; pub fn translate(&self, text: &str, mode: Mode, opts: &Options) -> String }`.

- [ ] **Step 1: Add Dictionaries struct to dict.rs**

Append to `crates/qt-core/src/dict.rs`:

```rust
use std::collections::HashMap;
use crate::han_viet::HanVietMap;

/// All lookup maps used by the engine, after loading + priority merge.
#[derive(Default)]
pub struct Dictionaries {
    pub han_viet: HanVietMap,
    pub only_name: HashMap<String, String>,
    pub vietphrase: HashMap<String, String>,
    pub vietphrase_one_meaning: HashMap<String, String>,
}
```

- [ ] **Step 2: Add chinese_to_han_viet_string to han_viet.rs**

Append to `crates/qt-core/src/han_viet.rs` (above the `#[cfg(test)]` block):

```rust
use crate::text::append_translated_word;

/// Mode HanViet: transcribe each char; a single space separates two consecutive
/// Chinese chars. Uses append_translated_word so sentence-start capitalization applies.
pub fn chinese_to_han_viet_string(chars: &[char], han_viet: &HanVietMap) -> String {
    let mut result = String::new();
    let mut last = String::new();
    let len = chars.len();
    if len == 0 {
        return result;
    }
    for i in 0..len - 1 {
        let c = chars[i];
        if is_chinese(c, han_viet) {
            append_translated_word(&mut result, &char_to_han_viet(c, han_viet), &mut last);
            if is_chinese(chars[i + 1], han_viet) {
                result.push(' ');
                last.push(' ');
            }
        } else {
            result.push(c);
            last.push(c);
        }
    }
    let lc = chars[len - 1];
    if is_chinese(lc, han_viet) {
        append_translated_word(&mut result, &char_to_han_viet(lc, han_viet), &mut last);
    } else {
        result.push(lc);
        last.push(lc);
    }
    result
}
```

- [ ] **Step 3: Add Engine to lib.rs and route HanViet**

Add to `crates/qt-core/src/lib.rs` (after the module declarations and type defs):

```rust
pub use dict::Dictionaries;

pub struct Engine {
    dicts: Dictionaries,
}

impl Engine {
    pub fn from_dicts(dicts: Dictionaries) -> Engine {
        Engine { dicts }
    }

    pub fn translate(&self, text: &str, mode: Mode, opts: &Options) -> String {
        let chars: Vec<char> = text.chars().collect();
        match mode {
            Mode::HanViet => han_viet::chinese_to_han_viet_string(&chars, &self.dicts.han_viet),
            Mode::VietPhrase => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase, &self.dicts.only_name, &self.dicts.han_viet,
            ),
            Mode::VietPhraseOneMeaning => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase_one_meaning, &self.dicts.only_name, &self.dicts.han_viet,
            ),
        }
    }
}
```

Also add `mod translate;` with the other module declarations. `translate::translate_all` does not exist yet — Task 8 creates it. To keep this task compiling and testing HanViet in isolation, temporarily route the two VietPhrase arms to `String::new()` and add a `// TODO(Task 8)` — replace in Task 8:

```rust
            Mode::VietPhrase | Mode::VietPhraseOneMeaning => String::new(), // TODO(Task 8)
```

(Do NOT add `mod translate;` yet — add it in Task 8.)

- [ ] **Step 4: Write the failing test in lib.rs**

Replace the `tests` module in `lib.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn hv_map() -> HashMap<char, String> {
        let mut m = HashMap::new();
        for (k, v) in [('他', "tha"), ('很', "ngận"), ('厉', "lệ"), ('害', "hại")] {
            m.insert(k, v.to_string());
        }
        m
    }

    fn engine_hv_only() -> Engine {
        let d = Dictionaries { han_viet: hv_map(), ..Default::default() };
        Engine::from_dicts(d)
    }

    #[test]
    fn options_default_matches_spec() {
        let o = Options::default();
        assert_eq!(o.scan_range, 30);
    }

    #[test]
    fn hanviet_spaces_between_hanzi() {
        let e = engine_hv_only();
        // The engine initializes lastTranslatedWord="", so output starts with a space.
        let got = e.translate("他很厉害", Mode::HanViet, &Options::default());
        assert_eq!(got, " tha ngận lệ hại");
    }

    #[test]
    fn hanviet_passes_through_non_chinese() {
        let e = engine_hv_only();
        let got = e.translate("他, 好", Mode::HanViet, &Options::default());
        // '他'→tha with a leading space, then raw ", ", then unknown '好'.
        assert_eq!(got, " tha, 好");
    }
}
```

> Note the expected leading space in `" tha, 好"`: `lastTranslatedWord` starts empty, so
> `appendTranslatedWord` takes its default branch. `好` is not in `hv_map`, so
> `is_chinese('好')` is false and it is emitted raw.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p qt-core --lib`
Expected: PASS (all lib tests, including the two new HanViet tests).

- [ ] **Step 6: Commit**

```bash
git add crates/qt-core/src
git commit -m "feat(qt-core): HanViet string mode + Engine::translate"
```

---

### Task 6: Load Names + VietPhrase, priority merge, one-meaning

**Files:**
- Modify: `crates/qt-core/src/dict.rs` (add builders + file loader)
- Test: inline in `dict.rs`

**Interfaces:**
- Consumes: `parse_dict`.
- Produces:
  `impl Dictionaries { pub fn build(han_viet_src: &str, names_src: &str, names2_src: &str, vietphrase_src: &str) -> Dictionaries }` — pure, testable without files;
  `impl Dictionaries { pub fn load(data_dir: &std::path::Path) -> std::io::Result<Dictionaries> }` — reads files per `Dictionaries.config` names.

- [ ] **Step 1: Write the failing test in dict.rs**

Add to the `tests` module in `dict.rs`:

```rust
    #[test]
    fn merge_priority_names_over_vietphrase_and_names2_over_names() {
        // names has 张=Trương ; names2 overrides 张=Trương2 ; vietphrase has 张=(should lose)
        let hv = "";
        let names = "张=TruongName\n李=Ly";
        let names2 = "张=TruongName2";
        let vietphrase = "张=TruongVP\n很好=rất tốt/rất ổn";
        let d = Dictionaries::build(hv, names, names2, vietphrase);

        // Names2 overrode Names in only_name
        assert_eq!(d.only_name.get("张").map(String::as_str), Some("TruongName2"));
        // vietphrase merged: name key wins over vietphrase key
        assert_eq!(d.vietphrase.get("张").map(String::as_str), Some("TruongName2"));
        // pure vietphrase entry present with full multi-meaning value
        assert_eq!(d.vietphrase.get("很好").map(String::as_str), Some("rất tốt/rất ổn"));
        // one-meaning takes first split on '/' or '|'
        assert_eq!(d.vietphrase_one_meaning.get("很好").map(String::as_str), Some("rất tốt"));
    }
```

- [ ] **Step 2: Implement build + load in dict.rs**

Append to `crates/qt-core/src/dict.rs` (before the `#[cfg(test)]` block):

```rust
impl Dictionaries {
    /// Pure builder mirroring loadOnlyNameDictionary → loadVietPhraseDictionary →
    /// vPDictToVPOneMeaningDict. See docs/engine/dictionaries.md §4.
    pub fn build(han_viet_src: &str, names_src: &str, names2_src: &str, vietphrase_src: &str) -> Dictionaries {
        let mut han_viet: HanVietMap = HashMap::new();
        for (k, v) in parse_dict(han_viet_src) {
            let mut it = k.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                han_viet.entry(c).or_insert(v); // first-wins
            }
        }

        // Names: first-wins into only_name; Names2: OVERRIDE (indexer assignment).
        let mut only_name: HashMap<String, String> = HashMap::new();
        for (k, v) in parse_dict(names_src) {
            only_name.entry(k).or_insert(v);
        }
        for (k, v) in parse_dict(names2_src) {
            only_name.insert(k, v); // override
        }

        // only_vietphrase: first-wins
        let mut only_vietphrase: HashMap<String, String> = HashMap::new();
        for (k, v) in parse_dict(vietphrase_src) {
            only_vietphrase.entry(k).or_insert(v);
        }

        // vietphrase = only_name first (all), then only_vietphrase for absent keys.
        let mut vietphrase: HashMap<String, String> = only_name.clone();
        for (k, v) in &only_vietphrase {
            vietphrase.entry(k.clone()).or_insert_with(|| v.clone());
        }

        // one-meaning: first segment split on '/' or '|'.
        let mut vietphrase_one_meaning: HashMap<String, String> = HashMap::new();
        for (k, v) in &vietphrase {
            let first = match v.split(['/', '|']).next() {
                Some(s) => s.to_string(),
                None => v.clone(),
            };
            vietphrase_one_meaning.insert(k.clone(), first);
        }

        Dictionaries { han_viet, only_name, vietphrase, vietphrase_one_meaning }
    }

    /// Load from a data directory using the standard QT filenames.
    pub fn load(data_dir: &std::path::Path) -> std::io::Result<Dictionaries> {
        let read = |rel: &str| -> std::io::Result<String> {
            std::fs::read_to_string(data_dir.join(rel))
        };
        let han_viet = read("Resources/ChinesePhienAmWords.txt")?;
        let names = read("Names.txt")?;
        let names2 = read("Names2/123.txt").unwrap_or_default();
        let vietphrase = read("VietPhrase/VietPhrase.txt")?;
        Ok(Dictionaries::build(&han_viet, &names, &names2, &vietphrase))
    }
}
```

> `only_vietphrase` is a local here (not stored) since only the merged `vietphrase` and its
> one-meaning form are needed by the engine. A later plan (meanings mode) will store it separately.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p qt-core --lib dict`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/qt-core/src/dict.rs
git commit -m "feat(qt-core): load Names/VietPhrase with priority merge + one-meaning"
```

---

### Task 7: Longest-phrase and name-priority predicates

**Files:**
- Create: `crates/qt-core/src/translate.rs`
- Modify: `crates/qt-core/src/lib.rs` (add `mod translate;`)
- Test: inline in `translate.rs`

**Interfaces:**
- Consumes: `std::collections::HashMap`.
- Produces:
  `pub fn is_longest_phrase_in_sentence(chars: &[char], start: usize, phrase_len: usize, dict: &HashMap<String,String>, algo: i32) -> bool`;
  `pub fn contains_name(chars: &[char], start: usize, phrase_len: usize, only_name: &HashMap<String,String>) -> bool`.

- [ ] **Step 1: Write the failing test in translate.rs**

```rust
//! The TranslateAll longest-match loop and its predicates.
//! Number conversion and Luật Nhân are STUBBED in this plan (see docs/engine/).

use std::collections::HashMap;

fn substr(chars: &[char], start: usize, len: usize) -> String {
    chars[start..start + len].iter().collect()
}

/// Mirrors isLongestPhraseInSentence (docs/engine/translation-algorithm.md §4).
pub fn is_longest_phrase_in_sentence(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    dict: &HashMap<String, String>,
    algo: i32,
) -> bool {
    if phrase_len < 2 {
        return true;
    }
    let threshold = if algo == 0 { phrase_len } else { phrase_len.max(3) };
    let end = start + phrase_len - 1; // inclusive
    for i in (start + 1)..=end {
        let mut n = 20usize;
        while n > threshold {
            if chars.len() >= i + n && dict.contains_key(&substr(chars, i, n)) {
                return false;
            }
            n -= 1;
        }
    }
    true
}

/// Mirrors containsName (docs/engine/translation-algorithm.md §5).
pub fn contains_name(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    only_name: &HashMap<String, String>,
) -> bool {
    if phrase_len < 2 || only_name.contains_key(&substr(chars, start, phrase_len)) {
        return false;
    }
    let end = start + phrase_len - 1; // inclusive
    for i in (start + 1)..=end {
        let mut n = 20usize;
        while n >= 2 {
            if chars.len() >= i + n && only_name.contains_key(&substr(chars, i, n)) {
                return true;
            }
            n -= 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn longest_phrase_algo0_rejects_overlapping_longer() {
        let chars: Vec<char> = "ABCD".chars().collect();
        // "AB" at 0 len 2; but "BCD" (len 3) exists overlapping at index 1 → not longest for algo 0
        let d = dict(&[("AB", "x"), ("BCD", "y")]);
        assert!(!is_longest_phrase_in_sentence(&chars, 0, 2, &d, 0));
        // algo 1: threshold = max(2,3)=3, so only len>3 overlaps reject; BCD is len 3 → not > 3 → longest
        assert!(is_longest_phrase_in_sentence(&chars, 0, 2, &d, 1));
    }

    #[test]
    fn contains_name_detects_inner_name() {
        let chars: Vec<char> = "ABCD".chars().collect();
        let names = dict(&[("BC", "Name")]);
        // phrase "ABCD" (len 4) is not itself a name, but "BC" name starts inside → true
        assert!(contains_name(&chars, 0, 4, &names));
        // if the phrase itself is a name, returns false
        let names2 = dict(&[("ABCD", "Name"), ("BC", "Name")]);
        assert!(!contains_name(&chars, 0, 4, &names2));
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

Add `mod translate;` with the other module declarations. Replace the temporary VietPhrase arm from Task 5 is done in Task 8; for now the module just needs to compile (predicates only).

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p qt-core --lib translate`
Expected: PASS (2 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/qt-core/src/translate.rs crates/qt-core/src/lib.rs
git commit -m "feat(qt-core): longest-phrase + name-priority predicates"
```

---

### Task 8: TranslateAll main loop (VietPhrase / OneMeaning)

**Files:**
- Modify: `crates/qt-core/src/translate.rs` (add loop + stubs + process helpers)
- Modify: `crates/qt-core/src/lib.rs` (route VietPhrase arms to `translate_all`)
- Test: inline in `translate.rs`

**Interfaces:**
- Consumes: predicates from Task 7; `text::{append_translated_word, wrap_translation, next_char_is_chinese}`; `han_viet::{is_chinese, char_to_han_viet, HanVietMap}`; `Options`.
- Produces:
  `pub fn translate_all(chars: &[char], opts: &crate::Options, dict: &HashMap<String,String>, only_name: &HashMap<String,String>, han_viet: &HanVietMap) -> String`.

- [ ] **Step 1: Write the failing test in translate.rs**

Add to the `tests` module in `translate.rs`:

```rust
    use crate::han_viet::HanVietMap;
    use crate::Options;

    fn hv(pairs: &[(char, &str)]) -> HanVietMap {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn translates_longest_phrase_then_falls_back_to_hanviet() {
        // dict: 很好=rất tốt ; han-viet: 他=tha
        let chars: Vec<char> = "他很好".chars().collect();
        let d = dict(&[("很好", "rất tốt")]);
        let names = HashMap::new();
        let hanviet = hv(&[('他', "tha"), ('很', "ngận"), ('好', "hảo")]);
        let opts = Options::default();
        // 他 not in dict → HanViet 'tha'; then 很好 phrase → 'rất tốt'.
        // Faithful to the engine: last starts "" so the first word gets a LEADING
        // SPACE and stays lowercase (TranslateAll inits lastTranslatedWord = "").
        let got = translate_all(&chars, &opts, &d, &names, &hanviet);
        assert_eq!(got, " tha rất tốt");
    }

    #[test]
    fn name_priority_skips_phrase_covering_a_name() {
        // phrase 红中人 covers the 2-char name 中人 starting inside it.
        // With prioritized_name, containsName rejects the phrase (inner name has
        // length >= 2), so 红 falls to HanViet and 中人 is translated as the name.
        // (A single-char inner name would NOT trigger containsName, which only
        // scans lengths >= 2 — hence a 2-char inner name here.)
        let chars: Vec<char> = "红中人".chars().collect();
        let d = dict(&[("红中人", "cả cụm"), ("中人", "trung nhân")]);
        let names = dict(&[("中人", "trung nhân")]);
        let hanviet = hv(&[('红', "hồng"), ('中', "trung"), ('人', "nhân")]);
        let opts = Options { prioritized_name: true, ..Options::default() };
        let got = translate_all(&chars, &opts, &d, &names, &hanviet);
        assert_eq!(got, " hồng trung nhân");
    }
```

- [ ] **Step 2: Implement translate_all + stubs in translate.rs**

Append to `crates/qt-core/src/translate.rs` (before the `#[cfg(test)]` block):

```rust
use crate::han_viet::{char_to_han_viet, is_chinese, HanVietMap};
use crate::text::{append_translated_word, next_char_is_chinese, wrap_translation};
use crate::Options;

// ---- Stubs for the later plan (number conversion + Luật Nhân) ----
// Kept as real functions so the later plan only swaps bodies.

/// STUB: real version reorders 余/多 with a following 百/千/万/亿. Identity for MVP.
fn number_modifier(chars: &[char]) -> Vec<char> {
    chars.to_vec()
}

fn process_translation(
    chars: &[char],
    translation: &str,
    start: usize,
    length: usize,
    opts: &Options,
    result: &mut String,
    last: &mut String,
    han_viet: &HanVietMap,
) {
    let text = wrap_translation(translation, opts.wrap_type);
    append_translated_word(result, &text, last);
    if next_char_is_chinese(chars, start + length - 1, han_viet) {
        result.push(' ');
        last.push(' ');
    }
}

fn process_han_viet(
    chars: &[char],
    opts: &Options,
    num2: &mut usize,
    result: &mut String,
    last: &mut String,
    han_viet: &HanVietMap,
) {
    let c = chars[*num2];
    if is_chinese(c, han_viet) {
        let t = wrap_translation(&char_to_han_viet(c, han_viet), opts.wrap_type);
        append_translated_word(result, &t, last);
        if next_char_is_chinese(chars, *num2, han_viet) {
            result.push(' ');
            last.push(' ');
        }
    } else if (c == '"' || c == '\'')
        && !last.ends_with(' ')
        && !last.ends_with('.')
        && !last.ends_with('?')
        && !last.ends_with('!')
        && !last.ends_with('\t')
        && *num2 < chars.len() - 1
        && chars[*num2 + 1] != ' '
        && chars[*num2 + 1] != ','
    {
        result.push(' ');
        result.push(c);
        last.push(' ');
        last.push(c);
    } else {
        result.push(c);
        last.push(c);
    }
    *num2 += 1;
}

/// Main loop, mirrors TranslateAll (docs/engine/translation-algorithm.md §3).
/// Number/Luật-Nhân branches are stubbed: `number_modifier` is identity and there is
/// no PreScanForNumbers / HandleNhanBy, so unmatched positions fall to HanViet.
pub fn translate_all(
    chars: &[char],
    opts: &Options,
    dict: &HashMap<String, String>,
    only_name: &HashMap<String, String>,
    han_viet: &HanVietMap,
) -> String {
    let chars = number_modifier(chars); // stub identity; keeps shape for later plan
    let chars = chars.as_slice();
    let mut result = String::new();
    let mut last = String::new();
    let len = chars.len();
    if len == 0 {
        return result;
    }
    let mut num2 = 0usize;
    while num2 <= len - 1 {
        let mut flag = false;
        let mut num6 = opts.scan_range;
        while num6 > 0 {
            if num2 + num6 <= len {
                let text = substr(chars, num2, num6);
                if let Some(value2) = dict.get(&text) {
                    let is_longest =
                        is_longest_phrase_in_sentence(chars, num2, num6, dict, opts.translation_algorithm);
                    let name_ok = !opts.prioritized_name || !contains_name(chars, num2, num6, only_name);
                    let algo_ok = (opts.translation_algorithm != 0 && opts.translation_algorithm != 2)
                        || is_longest
                        || (opts.prioritized_name && only_name.contains_key(&text));
                    if name_ok && algo_ok {
                        process_translation(chars, value2, num2, num6, opts, &mut result, &mut last, han_viet);
                        flag = true;
                        num2 += num6;
                        break;
                    }
                }
                // Luật Nhân branch (A2) — STUB: not implemented in this plan.
            }
            num6 -= 1;
        }
        if flag {
            continue;
        }
        // Number {s} branch (B) — STUB: no prescanned numbers in MVP.
        process_han_viet(chars, opts, &mut num2, &mut result, &mut last, han_viet);
    }
    result
}
```

- [ ] **Step 3: Route VietPhrase arms in lib.rs**

In `crates/qt-core/src/lib.rs`, replace the temporary arm from Task 5:

```rust
            Mode::VietPhrase | Mode::VietPhraseOneMeaning => String::new(), // TODO(Task 8)
```

with:

```rust
            Mode::VietPhrase => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase, &self.dicts.only_name, &self.dicts.han_viet,
            ),
            Mode::VietPhraseOneMeaning => translate::translate_all(
                &chars, opts, &self.dicts.vietphrase_one_meaning, &self.dicts.only_name, &self.dicts.han_viet,
            ),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p qt-core`
Expected: PASS (all qt-core tests).

- [ ] **Step 5: Add an integration test over a small in-memory engine**

Create `crates/qt-core/tests/vietphrase.rs`:

```rust
use qt_core::{Dictionaries, Engine, Mode, Options};

#[test]
fn end_to_end_vietphrase_and_one_meaning() {
    // build() applies the real merge; multi-meaning value exercises both modes
    let d = Dictionaries::build(
        "他=tha\n很=ngận\n好=hảo",       // han-viet
        "",                               // names
        "",                               // names2
        "很好=rất tốt/rất ổn",            // vietphrase
    );
    let e = Engine::from_dicts(d);
    let o = Options::default();
    // Faithful engine output: leading space, lowercase first word.
    assert_eq!(e.translate("他很好", Mode::VietPhrase, &o), " tha rất tốt/rất ổn");
    assert_eq!(e.translate("他很好", Mode::VietPhraseOneMeaning, &o), " tha rất tốt");
}
```

This requires `Dictionaries::build`, `Engine`, `Mode`, `Options` to be public — they are (`pub`). Run:

Run: `cargo test -p qt-core --test vietphrase`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/qt-core/src crates/qt-core/tests
git commit -m "feat(qt-core): TranslateAll main loop for VietPhrase/OneMeaning"
```

---

### Task 9: qt-cli thin binary

**Files:**
- Create: `crates/qt-cli/Cargo.toml`
- Create: `crates/qt-cli/src/main.rs`
- Test: `crates/qt-cli/tests/cli.rs`

**Interfaces:**
- Consumes: `qt_core::{Engine, Dictionaries, Mode, Options}`.
- Produces: binary `qt` supporting `qt translate --mode <hanviet|vietphrase|vietphrase-one> [--data-dir DIR] [--wrap]`, reading stdin, writing stdout.

- [ ] **Step 1: Write qt-cli/Cargo.toml**

```toml
[package]
name = "qt-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "qt"
path = "src/main.rs"

[dependencies]
qt-core = { path = "../qt-core" }
```

- [ ] **Step 2: Write main.rs**

```rust
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use qt_core::{Dictionaries, Engine, Mode, Options};

fn parse_mode(s: &str) -> Option<Mode> {
    match s {
        "hanviet" => Some(Mode::HanViet),
        "vietphrase" => Some(Mode::VietPhrase),
        "vietphrase-one" => Some(Mode::VietPhraseOneMeaning),
        _ => None,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Expected: translate --mode M [--data-dir D] [--wrap]
    if args.first().map(String::as_str) != Some("translate") {
        eprintln!("usage: qt translate --mode <hanviet|vietphrase|vietphrase-one> [--data-dir DIR] [--wrap]");
        return ExitCode::from(2);
    }
    let mut mode = Mode::VietPhrase;
    let mut data_dir = PathBuf::from("data");
    let mut wrap = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                match args.get(i).and_then(|s| parse_mode(s)) {
                    Some(m) => mode = m,
                    None => {
                        eprintln!("error: invalid or missing --mode value");
                        return ExitCode::from(2);
                    }
                }
            }
            "--data-dir" => {
                i += 1;
                match args.get(i) {
                    Some(d) => data_dir = PathBuf::from(d),
                    None => {
                        eprintln!("error: --data-dir needs a path");
                        return ExitCode::from(2);
                    }
                }
            }
            "--wrap" => wrap = true,
            other => {
                eprintln!("error: unknown argument {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let dicts = match Dictionaries::load(&data_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: failed to load dictionaries from {}: {e}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let engine = Engine::from_dicts(dicts);
    let opts = Options { wrap_type: if wrap { 1 } else { 0 }, ..Options::default() };

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: failed to read stdin");
        return ExitCode::FAILURE;
    }
    let out = engine.translate(&input, mode, &opts);
    let mut stdout = std::io::stdout();
    if stdout.write_all(out.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
```

- [ ] **Step 3: Write the CLI integration test**

Create `crates/qt-cli/tests/cli.rs`:

```rust
use std::io::Write;
use std::process::{Command, Stdio};

// Runs the built `qt` binary with a tiny data dir written to a temp folder.
#[test]
fn cli_hanviet_over_stdin() {
    // Arrange a minimal data dir
    let dir = std::env::temp_dir().join(format!("qtcli-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Resources")).unwrap();
    std::fs::create_dir_all(dir.join("VietPhrase")).unwrap();
    std::fs::create_dir_all(dir.join("Names2")).unwrap();
    std::fs::write(dir.join("Resources/ChinesePhienAmWords.txt"), "他=tha\n好=hảo\n").unwrap();
    std::fs::write(dir.join("Names.txt"), "").unwrap();
    std::fs::write(dir.join("Names2/123.txt"), "").unwrap();
    std::fs::write(dir.join("VietPhrase/VietPhrase.txt"), "").unwrap();

    let bin = env!("CARGO_BIN_EXE_qt");
    let mut child = Command::new(bin)
        .args(["translate", "--mode", "hanviet", "--data-dir", dir.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all("他好".as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();

    assert!(out.status.success());
    // Faithful engine output: leading space, lowercase first word.
    assert_eq!(String::from_utf8_lossy(&out.stdout), " tha hảo");

    let _ = std::fs::remove_dir_all(&dir);
}
```

- [ ] **Step 4: Run the CLI test**

Run: `cargo test -p qt-cli`
Expected: PASS (builds `qt`, runs it, asserts `" tha hảo"`).

- [ ] **Step 5: Smoke-test against the real dictionaries**

Run:
```bash
echo "他很厉害" | cargo run -q -p qt-cli -- translate --mode hanviet --data-dir QT2025
```
Expected: a faithful Han-Việt line (for example ` tha ngận lệ hại`, with a leading space)
printed. Then try `--mode vietphrase`.
This is a manual sanity check over the full 28MB dictionary load; note load time.

- [ ] **Step 6: Commit**

```bash
git add crates/qt-cli
git commit -m "feat(qt-cli): thin CLI over stdin/stdout with --mode/--data-dir/--wrap"
```

---

## Self-Review

**Spec coverage** (against `docs/engine/`):
- HanViet mode → Task 3, 5. ✅
- Dictionaries + priority merge + one-meaning → Task 6 (mirrors dictionaries.md §4). ✅
- TranslateAll loop, longest-phrase, name-priority, ProcessTranslation, ProcessHanViet, append/wrap → Tasks 4, 7, 8 (mirrors translation-algorithm.md §3–9). ✅
- Number conversion (number-conversion.md) → **intentionally stubbed** (`number_modifier` identity, no prescan); documented, deferred to next plan. ✅ (out of MVP scope per architecture.md §8 roadmap)
- Luật Nhân (luat-nhan.md) → **intentionally stubbed** (no HandleNhanBy branch); deferred. ✅
- Meanings/LacViet (meanings-lacviet.md) → not in MVP; deferred. ✅
- CLI stdin/stdout → Task 9. ✅

**Placeholder scan:** stubs are real functions with explicit stub behavior; the one temporary `String::new()` arm (Task 5) is explicitly replaced in Task 8. No TBD/TODO left except the labelled deferral stubs. ✅

**Type consistency:** `Dictionaries` fields (`han_viet`, `only_name`, `vietphrase`, `vietphrase_one_meaning`) are defined in Task 5 and populated in Task 6; `translate_all` signature in Task 7/8 matches the call sites in `lib.rs` (Task 8); `Options` fields (`wrap_type`, `translation_algorithm`, `prioritized_name`, `scan_range`) consistent across Tasks 1, 8, 9. ✅

**Known limitation (documented, not a gap):** without real QT output samples, tests assert algorithm-derived expected values, not byte-diffs against QT. When golden samples arrive, add `tests/golden/` comparisons; the engine API (`Engine::translate`) already supports it.
