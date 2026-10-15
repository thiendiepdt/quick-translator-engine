# Kiểm tra nội dung nhạy cảm về Việt Nam — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Phát hiện sớm chỗ nhạy cảm về Việt Nam trong raw và bản dịch, cho người dùng xác minh (có AI sơ loại) và thay thế cụm từ trong bản dịch theo phạm vi tuỳ chọn.

**Architecture:** qt-ai-core có từ điển (`vn_lexicon`), bộ quét + file `vn-check.json` quét tăng dần (`vn_check`), AI xác minh theo lô (`vn_ai`); móc vào `accept` (cảnh báo chương) và lệnh `qt-ai vn-check`. qt-ai-gui có lệnh Tauri `vn_cmds`, quét khi quét chương mới / thư viện, huy hiệu ở màn chọn truyện, tab "Kiểm tra VN" trên trang Truyện, hộp thoại thay thế (bọc engine `replace.rs`) và thẻ cài đặt bảng thay / từ điển.

**Tech Stack:** Rust (qt-ai-core, Tauri 2), crate `aho-corasick` 1.1 (đã có trong Cargo.lock), `regex`; React 19 + TypeScript + zod + vitest + Testing Library.

Spec: `docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md`.

## Global Constraints

- Chỉ cảnh báo: không chặn dịch, chốt, export.
- Bản dịch là tín hiệu chính ở chương đã dịch; raw chỉ dùng cho chương chưa dịch.
- Tiếng Việt khớp nguyên từ (hai bên không dính chữ/số).
- Đối chiếu hai bên: chỗ khớp ở bản dịch chỉ tính khi đoạn raw cùng nhãn có một dạng `zh` của chính mục đó (khi số đoạn raw = số đoạn dịch).
- Từ yếu chỉ báo khi cùng đoạn có từ đi kèm.
- Loại trừ xét trong cửa sổ ±12 ký tự quanh chỗ khớp.
- Khớp chồng nhau theo cụm dài nhất (`MatchKind::LeftmostLongest`).
- File kết quả: `<truyện>/vn-check.json`; cấu hình người dùng: `<thư mục bản mặc định>/vn-terms.json` (`BaseStore::dir()`).
- Hiệu năng: lần đầu truyện 2.500 chương ≤ vài giây; quét lại không đổi gì ≤ 100 ms; ≤ 8 luồng.
- Trạng thái chỗ khớp: `pending` / `ok` / `action`; nhãn AI: `xucpham` / `nhaycam` / `trungtinh` / `khonglienquan`.
- Lô AI ≤ 20 chỗ.
- Thay chỉ trong `out/`; dùng `replace_in_story` để có hoàn tác.
- Test viết trước (TDD), commit sau mỗi task. Build Rust với `CARGO_INCREMENTAL=0` (đĩa máy dev gần đầy).
- Lệnh test Rust chạy ở gốc repo `/home/hungnp/projects/quick-translator-engine`; test frontend chạy trong `apps/qt-ai-gui`.

## File Structure

| File | Trách nhiệm |
| --- | --- |
| `crates/qt-ai-core/src/vn_lexicon.rs` (mới) | Mục từ điển, bản cứng, cấu hình người dùng (`VnSettings`), `Lexicon` (Aho-Corasick + loại trừ) |
| `crates/qt-ai-core/src/vn_check.rs` (mới) | Quét một chương (luật khớp), `vn-check.json`, quét tăng dần song song, trạng thái, báo cáo kèm ngữ cảnh |
| `crates/qt-ai-core/src/vn_ai.rs` (mới) | Prompt + lô AI, đọc kết quả, lưu nhãn |
| `crates/qt-ai-core/src/lib.rs` | khai báo 3 module |
| `crates/qt-ai-core/Cargo.toml` | thêm `aho-corasick = "1"` |
| `crates/qt-ai-core/src/commands/accept.rs` | thêm cảnh báo `[VN]` khi chốt chương |
| `crates/qt-ai-core/src/bin/qt-ai.rs` | lệnh `qt-ai vn-check <root>` |
| `crates/qt-ai-core/tests/vn_check.rs` (mới) | test lõi |
| `apps/qt-ai-gui/src-tauri/src/vn_cmds.rs` (mới) | lệnh Tauri: refresh, report, status, AI, settings, replace |
| `apps/qt-ai-gui/src-tauri/src/lib.rs` | đăng ký module + lệnh |
| `apps/qt-ai-gui/src-tauri/src/library_cmds.rs` | quét VN sau `rescan_story`, `library_rescan`, `import_chapters` |
| `apps/qt-ai-gui/src-tauri/src/story_cmds.rs`, `summary_cache.rs` | `RecentSummary.vn_flags` |
| `apps/qt-ai-gui/src/lib/schema.ts`, `types.ts`, `api.ts` | schema + gọi lệnh |
| `apps/qt-ai-gui/src/components/vn-check-panel.tsx` (+test) | tab Kiểm tra VN |
| `apps/qt-ai-gui/src/components/vn-replace-dialog.tsx` (+test) | hộp thoại thay thế |
| `apps/qt-ai-gui/src/components/vn-settings-editor.tsx` (+test) | thẻ Cài đặt: bật/tắt mục có sẵn, mục riêng, bảng thay |
| `apps/qt-ai-gui/src/components/pages/story-page.tsx`, `settings-page.tsx`, `story-picker.tsx` | gắn tab, thẻ, huy hiệu |

---

### Task 1: Từ điển `vn_lexicon`

**Files:**
- Create: `crates/qt-ai-core/src/vn_lexicon.rs`
- Modify: `crates/qt-ai-core/src/lib.rs`, `crates/qt-ai-core/Cargo.toml`
- Test: `crates/qt-ai-core/tests/vn_check.rs`

**Interfaces:**
- Produces: `Group {Ten,Cq,Ls,Hd}`, `Level {Mietthi,Canxem}`, `Entry {id, group, level, weak, zh, vi, exclude}`, `ReplaceSetting {Ancient,Modern,Any}`, `Replacement {from,to,setting}`, `VnSettings {extra, disabled, replacements}`, `default_replacements()`, `builtin_entries()`, `Lexicon::{new, builtin, with_settings, load, find_zh, find_vi, excluded, mentions_zh}` + `Lexicon.entries`, `Lexicon.hash`, `Hit {entry,start,end}`, `load_settings(&BaseStore)`, `save_settings(&BaseStore,&VnSettings)`, `pub(crate) fn fnv64(&str) -> u64`.

- [ ] **Step 1: Thêm dependency và module rỗng để test biên dịch được**

`crates/qt-ai-core/Cargo.toml`, mục `[dependencies]`, thêm dòng:

```toml
aho-corasick = "1"
```

`crates/qt-ai-core/src/lib.rs`, thêm sau `pub mod titles;` (giữ thứ tự hiện có, chèn ba dòng cuối danh sách `pub mod`):

```rust
pub mod vn_lexicon;
pub mod vn_check;
pub mod vn_ai;
```

Tạo tạm `crates/qt-ai-core/src/vn_check.rs` và `crates/qt-ai-core/src/vn_ai.rs` với một dòng `//! (Task 2/4)` để crate biên dịch.

- [ ] **Step 2: Viết test thất bại**

Tạo `crates/qt-ai-core/tests/vn_check.rs`:

```rust
//! Kiểm tra nội dung nhạy cảm về Việt Nam — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
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
```

- [ ] **Step 3: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error" | sort | uniq -c`
Expected: lỗi biên dịch "cannot find … in this scope" (chưa có `Lexicon`, `Entry`…).

- [ ] **Step 4: Viết `vn_lexicon.rs`**

```rust
//! Từ điển kiểm tra nội dung nhạy cảm về Việt Nam (spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md).
//! Bản cứng trong binary; phần người dùng (mục thêm, mục tắt, bảng thay thế) ở `<base>/vn-terms.json` — cùng thư mục
//! bản mặc định nên cả GUI lẫn CLI (agy gọi `qt-ai accept`) đều đọc được.

use crate::base::BaseStore;
use crate::error::{CoreError, Result};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SETTINGS_FILE: &str = "vn-terms.json";
/// Số ký tự mỗi bên chỗ khớp để xét mẫu loại trừ.
const WINDOW: usize = 12;
/// `越…越` (越战越勇, 越大越好, 他越战…越快): 越 là phó từ "càng", không phải nước Việt.
const CANG: &str = r"越[^，。！？、\n]{0,8}越";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    /// Tên gọi VN, người Việt, kể cả miệt thị.
    Ten,
    /// Chủ quyền biển đảo.
    Cq,
    /// Chiến tranh, lịch sử nhạy cảm.
    Ls,
    /// Địa danh, nhân vật VN hiện đại.
    Hd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Mietthi,
    Canxem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub group: Group,
    pub level: Level,
    /// Từ yếu: chỉ báo khi cùng đoạn có từ đi kèm.
    #[serde(default)]
    pub weak: bool,
    #[serde(default)]
    pub zh: Vec<String>,
    #[serde(default)]
    pub vi: Vec<String>,
    /// Regex loại trừ, xét trong cửa sổ ±12 ký tự quanh chỗ khớp (raw lẫn bản dịch).
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplaceSetting {
    Ancient,
    Modern,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replacement {
    pub from: String,
    pub to: String,
    pub setting: ReplaceSetting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VnSettings {
    pub extra: Vec<Entry>,
    pub disabled: Vec<String>,
    pub replacements: Vec<Replacement>,
}

impl Default for VnSettings {
    fn default() -> Self {
        VnSettings { extra: vec![], disabled: vec![], replacements: default_replacements() }
    }
}

pub fn default_replacements() -> Vec<Replacement> {
    use ReplaceSetting::*;
    let r = |from: &str, to: &str, setting| Replacement { from: from.into(), to: to.into(), setting };
    vec![
        r("Giao Chỉ", "Chiêm Thành", Ancient),
        r("An Nam", "Chiêm Thành", Ancient),
        r("Giao Châu", "Chiêm Thành", Ancient),
        r("Đại Việt", "Chiêm Thành", Ancient),
        r("Việt Nam", "Thái Lan", Modern),
        r("người Việt", "người Thái", Modern),
        r("tiếng Việt", "tiếng Thái", Modern),
        r("Hà Nội", "Băng Cốc", Modern),
        r("Sài Gòn", "Chiang Mai", Modern),
        r("TP Hồ Chí Minh", "Chiang Mai", Modern),
    ]
}

fn e(id: &str, group: Group, level: Level, weak: bool, zh: &[&str], vi: &[&str], exclude: &[&str]) -> Entry {
    let own = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();
    Entry { id: id.into(), group, level, weak, zh: own(zh), vi: own(vi), exclude: own(exclude) }
}

/// Mọi cụm chứa một dạng strong phải nằm ở mục strong (khớp theo cụm dài nhất — cụm dài ở mục weak sẽ nuốt chỗ khớp thật).
pub fn builtin_entries() -> Vec<Entry> {
    use Group::*;
    use Level::*;
    vec![
        e("viet-nam", Ten, Canxem, false, &["越南", "越南人", "越南籍", "越南语", "越语"], &["Việt Nam", "người Việt Nam", "tiếng Việt"], &[]),
        e("khi-viet", Ten, Mietthi, false, &["越南猴子", "越南猴", "越猴", "猴国", "安南猴", "交趾蛮", "小越南"], &["khỉ Việt Nam", "khỉ Việt", "nước khỉ"], &[]),
        e("co-dau-viet", Ten, Mietthi, false, &["越南新娘", "越南媳妇", "越南老婆", "越南妹"], &["cô dâu Việt", "vợ Việt"], &[]),
        e("an-nam", Ten, Canxem, false, &["安南", "安南国", "安南人"], &["An Nam"], &[r"[平西淮延泰雅长兴永吉]安南", r"安南(?:[东西北]|方向|边|面|门|部)"]),
        e("an-nam-do-ho", Ten, Canxem, false, &["安南都护", "安南都护府"], &["An Nam đô hộ"], &[]),
        e("giao-chi", Ten, Canxem, false, &["交趾", "交阯", "交趾人", "交趾国"], &["Giao Chỉ"], &[]),
        e("dai-viet", Ten, Canxem, false, &["大越"], &["Đại Việt"], &[CANG]),
        e("giao-chau", Ten, Canxem, true, &["交州"], &["Giao Châu"], &[]),
        e("nam-viet", Ten, Canxem, true, &["南越", "南越国", "南越王"], &["Nam Việt"], &[r"[江湖河]南越", CANG]),
        e("nguoi-viet", Ten, Canxem, true, &["越人", "京族"], &["người Việt", "người Kinh"], &[CANG, r"(?:吴|於|百|山|闽|瓯)越人"]),
        e("hoang-sa", Cq, Canxem, false, &["西沙", "西沙群岛"], &["Hoàng Sa", "Tây Sa"], &["莫西沙", "Mạc Tây Sa"]),
        e("truong-sa", Cq, Canxem, false, &["南沙群岛"], &["quần đảo Trường Sa", "quần đảo Nam Sa"], &[]),
        e("truong-sa-yeu", Cq, Canxem, true, &["南沙"], &["Trường Sa", "Nam Sa"], &[]),
        e("trung-sa", Cq, Canxem, false, &["中沙群岛", "三沙市"], &["Trung Sa", "Tam Sa"], &[]),
        e("duong-chin-doan", Cq, Canxem, false, &["九段线", "十一段线", "南海断续线"], &["đường chín đoạn", "đường lưỡi bò", "đường mười một đoạn"], &[]),
        e("nam-hai-chu-dao", Cq, Canxem, false, &["南海诸岛", "南中国海"], &["biển Nam Trung Hoa", "quần đảo Nam Hải"], &[]),
        e("dao-da", Cq, Canxem, false, &["永兴岛", "永暑礁", "美济礁", "赤瓜礁", "黄岩岛"], &["đảo Phú Lâm", "đá Chữ Thập", "đá Vành Khăn", "Gạc Ma", "đảo Hoàng Nham"], &[]),
        e(
            "nam-hai",
            Cq,
            Canxem,
            true,
            &["南海"],
            &["Nam Hải", "Biển Đông"],
            &[
                "天南海北",
                "南海北",
                "海北天南",
                r"南海(?:派|剑|神尼|鳄神|老怪|郡|县|观音|普陀|龙王|龙宫)",
                r"Nam Hải (?:phái|kiếm|thần ni|Ngạc Thần|lão quái|quận|Long Vương|Quan Âm)",
            ],
        ),
        e("vinh-bac-bo", Cq, Canxem, true, &["北部湾", "东京湾"], &["Vịnh Bắc Bộ"], &[]),
        e("chien-tranh-1979", Ls, Canxem, false, &["对越自卫反击战", "对越反击战", "中越战争", "对越作战"], &["chiến tranh biên giới Việt", "chiến tranh Trung - Việt", "phản kích tự vệ"], &[]),
        e("lao-son", Ls, Canxem, false, &["老山战役", "两山轮战", "法卡山", "者阴山"], &["Lão Sơn", "Pháp Tạp Sơn"], &[]),
        e("hai-ba-trung", Ls, Canxem, false, &["征侧", "征贰"], &["Hai Bà Trưng", "Trưng Trắc", "Trưng Nhị"], &[r"征侧[翼面身]", r"[出远亲长]征侧"]),
        e("binh-dinh-giao-chi", Ls, Canxem, false, &["平定交趾", "郡县安南", "交趾郡", "日南郡", "九真郡"], &["bình định Giao Chỉ", "quận Giao Chỉ", "quận Nhật Nam", "quận Cửu Chân"], &[]),
        e("nhan-vat-lich-su", Ls, Canxem, false, &["李常杰", "黎利", "陈兴道", "陈国峻", "李公蕴", "阮惠"], &["Lý Thường Kiệt", "Lê Lợi", "Trần Hưng Đạo", "Trần Quốc Tuấn", "Lý Công Uẩn", "Nguyễn Huệ"], &[]),
        e("ho-chi-minh", Ls, Canxem, false, &["胡志明", "越共", "越盟", "北越"], &["Hồ Chí Minh", "Việt Cộng", "Việt Minh", "Bắc Việt"], &[]),
        e("chien-tranh-viet-nam", Ls, Canxem, false, &["越南战争"], &["chiến tranh Việt Nam"], &[]),
        e("viet-chien", Ls, Canxem, true, &["越战"], &[], &[CANG]),
        e("ma-vien", Ls, Canxem, true, &["马援", "伏波将军"], &["Mã Viện", "Phục Ba tướng quân"], &[]),
        e("bach-dang", Ls, Canxem, true, &["白藤江"], &["Bạch Đằng"], &[]),
        e("dia-danh", Hd, Canxem, false, &["西贡", "胡志明市", "岘港", "芽庄", "下龙湾", "湄公河", "顺化", "头顿市"], &["Sài Gòn", "TP Hồ Chí Minh", "thành phố Hồ Chí Minh", "Đà Nẵng", "Nha Trang", "Hạ Long", "Mê Kông", "Huế", "Vũng Tàu"], &[]),
        e("ha-noi", Hd, Canxem, true, &["河内"], &["Hà Nội"], &["河内郡", r"quận Hà Nội", r"Hà Nội (?:quận|vương)"]),
        e("hai-phong", Hd, Canxem, true, &["海防市"], &["Hải Phòng"], &[]),
    ]
}

/// Một chỗ khớp: chỉ số mục trong `Lexicon::entries` + vị trí byte trong văn bản.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub entry: usize,
    pub start: usize,
    pub end: usize,
}

pub struct Lexicon {
    pub entries: Vec<Entry>,
    /// Đổi khi từ điển đổi — `vn-check.json` khác hash thì quét lại cả truyện.
    pub hash: String,
    zh: Option<AhoCorasick>,
    zh_owner: Vec<usize>,
    vi: Option<AhoCorasick>,
    vi_owner: Vec<usize>,
    exclude: Vec<Vec<Regex>>,
}

pub(crate) fn fnv64(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn automaton(patterns: &[String], ascii_case_insensitive: bool) -> Option<AhoCorasick> {
    if patterns.is_empty() {
        return None;
    }
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .ascii_case_insensitive(ascii_case_insensitive)
        .build(patterns)
        .ok()
}

impl Lexicon {
    pub fn new(entries: Vec<Entry>) -> Lexicon {
        let (mut zh, mut zh_owner, mut vi, mut vi_owner) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (index, entry) in entries.iter().enumerate() {
            for pattern in entry.zh.iter().filter(|p| !p.trim().is_empty()) {
                zh.push(pattern.clone());
                zh_owner.push(index);
            }
            for pattern in entry.vi.iter().filter(|p| !p.trim().is_empty()) {
                vi.push(pattern.clone());
                vi_owner.push(index);
            }
        }
        let exclude = entries.iter().map(|entry| entry.exclude.iter().filter_map(|p| Regex::new(p).ok()).collect()).collect();
        let hash = format!("{:016x}", fnv64(&serde_json::to_string(&entries).unwrap_or_default()));
        Lexicon { zh: automaton(&zh, false), zh_owner, vi: automaton(&vi, true), vi_owner, exclude, hash, entries }
    }

    pub fn builtin() -> Lexicon {
        Lexicon::new(builtin_entries())
    }

    pub fn with_settings(settings: &VnSettings) -> Lexicon {
        let mut entries: Vec<Entry> =
            builtin_entries().into_iter().filter(|entry| !settings.disabled.contains(&entry.id)).collect();
        entries.extend(settings.extra.iter().cloned());
        Lexicon::new(entries)
    }

    /// Từ điển có hiệu lực: bản cứng + `<base>/vn-terms.json`. File hỏng/thiếu → bản cứng.
    pub fn load(base: &BaseStore) -> Lexicon {
        Lexicon::with_settings(&load_settings(base))
    }

    pub fn find_zh(&self, text: &str) -> Vec<Hit> {
        let Some(ac) = &self.zh else { return vec![] };
        ac.find_iter(text)
            .map(|m| Hit { entry: self.zh_owner[m.pattern().as_usize()], start: m.start(), end: m.end() })
            .collect()
    }

    /// Tiếng Việt khớp nguyên từ: ký tự liền trước/sau không phải chữ/số.
    pub fn find_vi(&self, text: &str) -> Vec<Hit> {
        let Some(ac) = &self.vi else { return vec![] };
        ac.find_iter(text)
            .filter(|m| {
                !text[..m.start()].chars().next_back().is_some_and(char::is_alphanumeric)
                    && !text[m.end()..].chars().next().is_some_and(char::is_alphanumeric)
            })
            .map(|m| Hit { entry: self.vi_owner[m.pattern().as_usize()], start: m.start(), end: m.end() })
            .collect()
    }

    /// Có mẫu loại trừ của mục khớp một đoạn chồng lên chỗ khớp, trong cửa sổ ±12 ký tự.
    pub fn excluded(&self, hit: Hit, text: &str) -> bool {
        let rules = &self.exclude[hit.entry];
        if rules.is_empty() {
            return false;
        }
        let lo = text[..hit.start].char_indices().rev().nth(WINDOW - 1).map_or(0, |(i, _)| i);
        let hi = text[hit.end..].char_indices().nth(WINDOW).map_or(text.len(), |(i, _)| hit.end + i);
        let window = &text[lo..hi];
        let (start, end) = (hit.start - lo, hit.end - lo);
        rules.iter().any(|rule| rule.find_iter(window).any(|m| m.start() < end && m.end() > start))
    }

    /// Đoạn raw có một dạng chữ Hán của mục — luật đối chiếu hai bên.
    pub fn mentions_zh(&self, entry: usize, text: &str) -> bool {
        self.entries[entry].zh.iter().any(|zh| !zh.is_empty() && text.contains(zh.as_str()))
    }
}

pub fn settings_path(base: &BaseStore) -> Option<PathBuf> {
    base.dir().map(|dir| dir.join(SETTINGS_FILE))
}

pub fn load_settings(base: &BaseStore) -> VnSettings {
    settings_path(base)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_settings(base: &BaseStore, settings: &VnSettings) -> Result<()> {
    let path = settings_path(base)
        .ok_or_else(|| CoreError::Internal("Không có thư mục dữ liệu app để lưu cấu hình kiểm tra VN.".into()))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(CoreError::io(dir))?;
    }
    let text = serde_json::to_string_pretty(settings).expect("VnSettings luôn serialize được");
    crate::story_fs::write_atomic(&path, &format!("{text}\n"))
}
```

- [ ] **Step 5: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error|panicked|test result"`
Expected: `test result: ok. 6 passed`

- [ ] **Step 6: Commit**

```bash
git add crates/qt-ai-core/Cargo.toml Cargo.lock crates/qt-ai-core/src/lib.rs crates/qt-ai-core/src/vn_lexicon.rs crates/qt-ai-core/src/vn_check.rs crates/qt-ai-core/src/vn_ai.rs crates/qt-ai-core/tests/vn_check.rs
git commit -m "feat(qt-ai): từ điển kiểm tra nội dung nhạy cảm về Việt Nam"
```

---

### Task 2: Quét một chương — luật khớp

**Files:**
- Modify: `crates/qt-ai-core/src/vn_check.rs`
- Test: `crates/qt-ai-core/tests/vn_check.rs`

**Interfaces:**
- Consumes: `Lexicon`, `Hit` (Task 1).
- Produces: `Side {Raw,Out}`, `Finding {paragraph, entry, side, matched, count}`, `pub fn out_paragraphs(&str) -> Vec<String>`, `pub fn scan_chapter(&Lexicon, raw: &str, out: Option<&str>) -> Vec<Finding>`.

- [ ] **Step 1: Viết test thất bại** — thêm vào cuối `crates/qt-ai-core/tests/vn_check.rs`:

```rust
use qt_ai_core::vn_check::*;

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
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error" | sort | uniq -c`
Expected: lỗi biên dịch, chưa có `scan_chapter`, `Side`.

- [ ] **Step 3: Viết phần quét trong `vn_check.rs`** (thay nội dung tạm của file):

```rust
//! Quét nội dung nhạy cảm về Việt Nam trong một truyện — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
//! Luật khớp ở `scan_chapter`; lưu + quét tăng dần ở `refresh` (Task 3).

use crate::paragraphs::paragraphs_of;
use crate::vn_lexicon::Lexicon;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Từ đi kèm cho từ yếu (ngoài một mục strong bất kỳ cùng đoạn).
const COOCCUR: &[&str] = &[
    "主权", "领土", "领海", "岛礁", "群岛", "越南", "安南", "交趾", "chủ quyền", "lãnh thổ", "lãnh hải", "quần đảo",
    "Việt Nam", "An Nam", "Giao Chỉ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Raw,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub paragraph: usize,
    pub entry: usize,
    pub side: Side,
    pub matched: String,
    pub count: usize,
}

/// Đoạn của bản dịch = các dòng không rỗng — cùng cách `replace.rs` đánh số đoạn.
pub fn out_paragraphs(out: &str) -> Vec<String> {
    out.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

fn cooccur(lex: &Lexicon, texts: &[&str], entry: usize) -> bool {
    texts.iter().any(|text| COOCCUR.iter().any(|word| text.contains(word)))
        || texts.iter().any(|text| {
            lex.find_zh(text)
                .into_iter()
                .chain(lex.find_vi(text))
                .any(|hit| hit.entry != entry && !lex.entries[hit.entry].weak && !lex.excluded(hit, text))
        })
}

/// Các chỗ khớp của một chương, gộp theo (đoạn, mục). `out` = None → chương chưa dịch, quét raw.
pub fn scan_chapter(lex: &Lexicon, raw: &str, out: Option<&str>) -> Vec<Finding> {
    let raw_paragraphs = paragraphs_of(raw);
    let mut found: BTreeMap<(usize, usize), Finding> = BTreeMap::new();
    let mut push = |paragraph: usize, entry: usize, side: Side, matched: &str| {
        found
            .entry((paragraph, entry))
            .and_modify(|finding| finding.count += 1)
            .or_insert(Finding { paragraph, entry, side, matched: matched.to_string(), count: 1 });
    };
    match out {
        Some(out) => {
            let out_paragraphs = out_paragraphs(out);
            let aligned = out_paragraphs.len() == raw_paragraphs.len();
            for (index, paragraph) in out_paragraphs.iter().enumerate() {
                let raw = if aligned { raw_paragraphs.get(index).map(String::as_str) } else { None };
                for hit in lex.find_vi(paragraph) {
                    if lex.excluded(hit, paragraph) {
                        continue;
                    }
                    if raw.is_some_and(|raw| !lex.mentions_zh(hit.entry, raw)) {
                        continue;
                    }
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph, raw.unwrap_or("")], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Out, &paragraph[hit.start..hit.end]);
                }
            }
        }
        None => {
            for (index, paragraph) in raw_paragraphs.iter().enumerate() {
                for hit in lex.find_zh(paragraph) {
                    if lex.excluded(hit, paragraph) {
                        continue;
                    }
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Raw, &paragraph[hit.start..hit.end]);
                }
            }
        }
    }
    found.into_values().collect()
}
```

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error|panicked|test result"`
Expected: `test result: ok. 12 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/vn_check.rs crates/qt-ai-core/tests/vn_check.rs
git commit -m "feat(qt-ai): luật khớp nội dung nhạy cảm VN — đối chiếu hai bên, từ yếu, loại trừ"
```

---

### Task 3: `vn-check.json` — quét tăng dần, trạng thái, báo cáo

**Files:**
- Modify: `crates/qt-ai-core/src/vn_check.rs`
- Test: `crates/qt-ai-core/tests/vn_check.rs`

**Interfaces:**
- Consumes: `scan_chapter`, `out_paragraphs`, `Lexicon` (Task 1–2).
- Produces: `VN_FILE`, `Status {Pending,Ok,Action}`, `AiVerdict {label,reason,model,at}`, `VnHit {id,chapter,paragraph,entry,group,level,side,matched,count,fingerprint,status,ai}`, `VnFile`, `VnSummary {hits,open,scanned}`, `HitView {hit (flatten), raw: Vec<String>, out: Vec<String>, focus: usize}`, `pub fn load_file(&StoryPaths) -> VnFile`, `pub fn refresh(&StoryPaths, &Lexicon) -> Result<VnSummary>`, `pub fn check_chapter(&StoryPaths, &Lexicon, id: &str) -> Result<usize>`, `pub fn set_status(&StoryPaths, ids: &[String], Status) -> Result<usize>`, `pub fn set_ai(&StoryPaths, &[(String, AiVerdict)]) -> Result<usize>`, `pub fn flag_count(&StoryPaths) -> Option<usize>`, `pub fn report(&StoryPaths) -> Vec<HitView>`.

- [ ] **Step 1: Viết test thất bại** — thêm vào `tests/vn_check.rs`:

```rust
use qt_ai_core::story_fs::story_paths;
use std::fs;

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
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error" | sort | uniq -c`
Expected: lỗi biên dịch, chưa có `refresh`, `VN_FILE`…

- [ ] **Step 3: Thêm phần lưu trữ vào `vn_check.rs`**

Thêm vào khối `use` đầu file:

```rust
use crate::error::{CoreError, Result};
use crate::story::natural_chapter_compare;
use crate::story_fs::{list_raw_chapter_ids, read_text, write_atomic, StoryPaths};
use crate::vn_lexicon::{fnv64, Group, Level};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
```

Thêm vào cuối file:

```rust
pub const VN_FILE: &str = "vn-check.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Pending,
    Ok,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiVerdict {
    pub label: String,
    pub reason: String,
    pub model: String,
    pub at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnHit {
    /// `<chương>:<đoạn>:<mục>`.
    pub id: String,
    pub chapter: String,
    pub paragraph: usize,
    pub entry: String,
    pub group: Group,
    pub level: Level,
    pub side: Side,
    pub matched: String,
    pub count: usize,
    /// Hash đoạn raw + đoạn dịch — đoạn đổi chữ thì trạng thái về pending.
    pub fingerprint: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub ai: Option<AiVerdict>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterMark {
    /// (kích thước, mtime ms) của raw và out.
    pub raw: (u64, u64),
    pub out: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VnFile {
    pub version: u32,
    pub lexicon_hash: String,
    pub chapters: BTreeMap<String, ChapterMark>,
    pub hits: Vec<VnHit>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VnSummary {
    pub hits: usize,
    /// Chưa xét + cần xử lý.
    pub open: usize,
    /// Số chương vừa đọc lại.
    pub scanned: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HitView {
    #[serde(flatten)]
    pub hit: VnHit,
    /// Đoạn trước, đoạn chứa, đoạn sau (có thể ít hơn ở đầu/cuối chương).
    pub raw: Vec<String>,
    pub out: Vec<String>,
    /// Vị trí đoạn chứa trong `raw` / `out`.
    pub focus: usize,
}

fn file_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join(VN_FILE)
}

fn raw_path(paths: &StoryPaths, id: &str) -> PathBuf {
    paths.raw_dir.join(format!("{id}.txt"))
}

fn out_path(paths: &StoryPaths, id: &str) -> PathBuf {
    paths.out_dir.join(format!("{id}.txt"))
}

fn mark(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_millis() as u64;
    Some((meta.len(), mtime))
}

fn chapter_mark(paths: &StoryPaths, id: &str) -> ChapterMark {
    ChapterMark { raw: mark(&raw_path(paths, id)).unwrap_or_default(), out: mark(&out_path(paths, id)) }
}

pub fn load_file(paths: &StoryPaths) -> VnFile {
    read_text(&file_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

fn save_file(paths: &StoryPaths, file: &VnFile) -> Result<()> {
    let text = serde_json::to_string_pretty(file).expect("VnFile luôn serialize được");
    write_atomic(&file_path(paths), &format!("{text}\n"))
}

fn sort_hits(hits: &mut [VnHit]) {
    hits.sort_by(|a, b| {
        natural_chapter_compare(&a.chapter, &b.chapter).then(a.paragraph.cmp(&b.paragraph)).then(a.entry.cmp(&b.entry))
    });
}

pub fn open_count(file: &VnFile) -> usize {
    file.hits.iter().filter(|hit| hit.status != Status::Ok).count()
}

fn chapter_hits(lex: &Lexicon, id: &str, raw: &str, out: Option<&str>) -> Vec<VnHit> {
    let raw_paragraphs = paragraphs_of(raw);
    let out_list = out.map(out_paragraphs).unwrap_or_default();
    scan_chapter(lex, raw, out)
        .into_iter()
        .map(|finding| {
            let entry = &lex.entries[finding.entry];
            let raw_text = raw_paragraphs.get(finding.paragraph).map(String::as_str).unwrap_or("");
            let out_text = out_list.get(finding.paragraph).map(String::as_str).unwrap_or("");
            VnHit {
                id: format!("{id}:{}:{}", finding.paragraph, entry.id),
                chapter: id.to_string(),
                paragraph: finding.paragraph,
                entry: entry.id.clone(),
                group: entry.group,
                level: entry.level,
                side: finding.side,
                matched: finding.matched,
                count: finding.count,
                fingerprint: format!("{:016x}", fnv64(&format!("{raw_text}\u{1}{out_text}"))),
                status: Status::Pending,
                ai: None,
            }
        })
        .collect()
}

fn scan_one(paths: &StoryPaths, lex: &Lexicon, id: &str) -> Vec<VnHit> {
    let Ok(raw) = read_text(&raw_path(paths, id)) else { return vec![] };
    let out = read_text(&out_path(paths, id)).ok();
    chapter_hits(lex, id, &raw, out.as_deref())
}

/// Quét song song (≤ 8 luồng) như `replace.rs`.
fn scan_many(paths: &StoryPaths, lex: &Lexicon, ids: &[&str]) -> Result<Vec<VnHit>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
    let size = ids.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = ids
            .chunks(size)
            .map(|part| scope.spawn(move || part.iter().flat_map(|id| scan_one(paths, lex, id)).collect::<Vec<_>>()))
            .collect();
        let mut all = Vec::new();
        for worker in workers {
            all.extend(worker.join().map_err(|_| CoreError::Internal("luồng quét VN bị lỗi".into()))?);
        }
        Ok(all)
    })
}

/// Hit mới cùng id và cùng fingerprint với hit cũ thì giữ trạng thái + nhãn AI.
fn carry(mut hit: VnHit, previous: &HashMap<String, VnHit>) -> VnHit {
    if let Some(old) = previous.get(&hit.id) {
        if old.fingerprint == hit.fingerprint {
            hit.status = old.status;
            hit.ai = old.ai.clone();
        }
    }
    hit
}

/// Quét tăng dần cả truyện: chỉ đọc chương mới/đổi (theo kích thước + mtime); từ điển đổi thì quét lại toàn bộ.
pub fn refresh(paths: &StoryPaths, lex: &Lexicon) -> Result<VnSummary> {
    let old = load_file(paths);
    let reset = old.lexicon_hash != lex.hash;
    let ids = list_raw_chapter_ids(paths)?;
    let marks: Vec<(String, ChapterMark)> = ids.iter().map(|id| (id.clone(), chapter_mark(paths, id))).collect();
    let todo: Vec<&str> =
        marks.iter().filter(|(id, m)| reset || old.chapters.get(id) != Some(m)).map(|(id, _)| id.as_str()).collect();
    let fresh = scan_many(paths, lex, &todo)?;
    let todo_set: HashSet<&str> = todo.iter().copied().collect();
    let live: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let previous: HashMap<String, VnHit> = old.hits.iter().map(|hit| (hit.id.clone(), hit.clone())).collect();
    let mut hits: Vec<VnHit> = old
        .hits
        .iter()
        .filter(|hit| !reset && live.contains(hit.chapter.as_str()) && !todo_set.contains(hit.chapter.as_str()))
        .cloned()
        .collect();
    hits.extend(fresh.into_iter().map(|hit| carry(hit, &previous)));
    sort_hits(&mut hits);
    let file = VnFile { version: 1, lexicon_hash: lex.hash.clone(), chapters: marks.into_iter().collect(), hits };
    let summary = VnSummary { hits: file.hits.len(), open: open_count(&file), scanned: todo.len() };
    if file != old {
        save_file(paths, &file)?;
    }
    Ok(summary)
}

/// Quét lại đúng một chương (khi chốt chương); trả số chỗ chưa xét + cần xử lý của chương đó.
/// Từ điển đổi → bỏ dấu các chương khác để lần `refresh` sau quét lại chúng.
pub fn check_chapter(paths: &StoryPaths, lex: &Lexicon, id: &str) -> Result<usize> {
    let mut file = load_file(paths);
    if file.lexicon_hash != lex.hash {
        file.chapters.clear();
        file.lexicon_hash = lex.hash.clone();
    }
    let previous: HashMap<String, VnHit> =
        file.hits.iter().filter(|hit| hit.chapter == id).map(|hit| (hit.id.clone(), hit.clone())).collect();
    file.hits.retain(|hit| hit.chapter != id);
    let fresh: Vec<VnHit> = scan_one(paths, lex, id).into_iter().map(|hit| carry(hit, &previous)).collect();
    let open = fresh.iter().filter(|hit| hit.status != Status::Ok).count();
    file.hits.extend(fresh);
    sort_hits(&mut file.hits);
    file.chapters.insert(id.to_string(), chapter_mark(paths, id));
    file.version = 1;
    save_file(paths, &file)?;
    Ok(open)
}

pub fn set_status(paths: &StoryPaths, ids: &[String], status: Status) -> Result<usize> {
    let mut file = load_file(paths);
    let mut changed = 0;
    for hit in file.hits.iter_mut().filter(|hit| ids.contains(&hit.id)) {
        if hit.status != status {
            hit.status = status;
            changed += 1;
        }
    }
    if changed > 0 {
        save_file(paths, &file)?;
    }
    Ok(changed)
}

pub fn set_ai(paths: &StoryPaths, verdicts: &[(String, AiVerdict)]) -> Result<usize> {
    let mut file = load_file(paths);
    let mut changed = 0;
    for (id, verdict) in verdicts {
        if let Some(hit) = file.hits.iter_mut().find(|hit| &hit.id == id) {
            hit.ai = Some(verdict.clone());
            changed += 1;
        }
    }
    if changed > 0 {
        save_file(paths, &file)?;
    }
    Ok(changed)
}

/// Số chỗ chưa xét + cần xử lý; None khi truyện chưa từng quét (không có file).
pub fn flag_count(paths: &StoryPaths) -> Option<usize> {
    file_path(paths).exists().then(|| open_count(&load_file(paths)))
}

/// Mọi chỗ khớp kèm đoạn trước/sau (raw và bản dịch) cho tab Kiểm tra VN.
pub fn report(paths: &StoryPaths) -> Vec<HitView> {
    let file = load_file(paths);
    let mut cache: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();
    file.hits
        .into_iter()
        .map(|hit| {
            let (raw, out) = cache.entry(hit.chapter.clone()).or_insert_with(|| {
                (
                    read_text(&raw_path(paths, &hit.chapter)).map(|t| paragraphs_of(&t)).unwrap_or_default(),
                    read_text(&out_path(paths, &hit.chapter)).map(|t| out_paragraphs(&t)).unwrap_or_default(),
                )
            });
            let lo = hit.paragraph.saturating_sub(1);
            let take = hit.paragraph + 2 - lo;
            let window = |list: &Vec<String>| list.iter().skip(lo).take(take).cloned().collect::<Vec<_>>();
            let (raw, out) = (window(raw), window(out));
            HitView { focus: hit.paragraph - lo, raw, out, hit }
        })
        .collect()
}
```

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error|panicked|test result"`
Expected: `test result: ok. 17 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/vn_check.rs crates/qt-ai-core/tests/vn_check.rs
git commit -m "feat(qt-ai): vn-check.json — quét tăng dần song song, giữ trạng thái xác minh"
```

---

### Task 4: AI sơ loại theo lô

**Files:**
- Modify: `crates/qt-ai-core/src/vn_ai.rs`
- Test: `crates/qt-ai-core/tests/vn_check.rs`

**Interfaces:**
- Consumes: `report`, `set_ai`, `AiVerdict`, `HitView` (Task 3); `TextModel`, `ApiStep`, `Generated` (`crate::api`).
- Produces: `BATCH: usize = 20`, `LABELS`, `SYSTEM_PROMPT`, `VerifyItem {id, matched, raw, out}`, `Verdict {id,label,reason}`, `VerifyOutcome {verified, failed}`, `pub fn items(&[HitView], ids: &[String]) -> Vec<VerifyItem>`, `pub fn payload(&[VerifyItem]) -> String`, `pub fn parse_verdicts(&str) -> Vec<Verdict>`, `pub fn save_verdicts(&StoryPaths, &[Verdict], model: &str) -> Result<usize>`, `pub fn verify_with_model(&StoryPaths, &dyn TextModel, ids: &[String], &AtomicBool) -> Result<VerifyOutcome>`, `pub fn agy_prompt(input: &Path, output: &Path) -> String`.

- [ ] **Step 1: Viết test thất bại** — thêm vào `tests/vn_check.rs`:

```rust
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
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error" | sort | uniq -c`
Expected: lỗi biên dịch, chưa có `parse_verdicts`, `verify_with_model`.

- [ ] **Step 3: Viết `vn_ai.rs`**

```rust
//! AI sơ loại các chỗ khớp nhạy cảm về Việt Nam theo lô — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
//! Đường API dùng `TextModel::complete_json`; đường agy (GUI) ghi `payload` ra file, agent ghi kết quả ra file,
//! cả hai đọc bằng `parse_verdicts`.

use crate::api::{ApiStep, TextModel};
use crate::error::Result;
use crate::story_fs::{now_ms, StoryPaths};
use crate::vn_check::{report, set_ai, AiVerdict, HitView};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub const BATCH: usize = 20;
pub const LABELS: [&str; 4] = ["xucpham", "nhaycam", "trungtinh", "khonglienquan"];

pub const SYSTEM_PROMPT: &str = "Bạn kiểm duyệt truyện dịch Trung → Việt trước khi đăng ở Việt Nam. Mỗi mục trong `items` là một chỗ có từ liên quan Việt Nam (`matched`), kèm đoạn raw tiếng Trung (`raw`) và bản dịch (`out`): đoạn trước, đoạn chứa, đoạn sau (bản dịch có thể rỗng nếu chương chưa dịch). Xếp mỗi mục một nhãn:
- xucpham: miệt thị, chửi bới người Việt / nước Việt, hoặc tô vẽ việc xâm lược, đô hộ Việt Nam.
- nhaycam: chủ quyền biển đảo (Hoàng Sa, Trường Sa, Biển Đông coi là lãnh thổ Trung Quốc, đường chín đoạn), chiến tranh Trung - Việt, nhân vật lịch sử / chính trị Việt Nam, kể theo hướng có thể gây tranh cãi.
- trungtinh: nhắc tới Việt Nam / người Việt bình thường, không miệt thị.
- khonglienquan: chữ trùng nhưng không nói về Việt Nam (nước Việt thời Xuân Thu, Nam Hải trong truyện kỳ ảo, cát vàng...).
Kèm `reason` một câu tiếng Việt. Chỉ trả JSON: {\"verdicts\": [{\"id\": \"...\", \"label\": \"...\", \"reason\": \"...\"}]}.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyItem {
    pub id: String,
    pub matched: String,
    pub raw: Vec<String>,
    pub out: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Verdict {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOutcome {
    pub verified: usize,
    /// Chỗ không nhận được nhãn hợp lệ (lỗi gọi, JSON hỏng, thiếu id).
    pub failed: usize,
}

pub fn items(views: &[HitView], ids: &[String]) -> Vec<VerifyItem> {
    views
        .iter()
        .filter(|view| ids.contains(&view.hit.id))
        .map(|view| VerifyItem { id: view.hit.id.clone(), matched: view.hit.matched.clone(), raw: view.raw.clone(), out: view.out.clone() })
        .collect()
}

pub fn payload(items: &[VerifyItem]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({ "items": items })).expect("VerifyItem luôn serialize được")
}

/// Lấy object JSON ngoài cùng (chịu rào ```json); bỏ mục có nhãn lạ.
pub fn parse_verdicts(text: &str) -> Vec<Verdict> {
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default)]
        verdicts: Vec<Verdict>,
    }
    let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) else { return vec![] };
    if end < start {
        return vec![];
    }
    serde_json::from_str::<Envelope>(&text[start..=end])
        .map(|envelope| envelope.verdicts)
        .unwrap_or_default()
        .into_iter()
        .filter(|verdict| LABELS.contains(&verdict.label.as_str()))
        .collect()
}

pub fn save_verdicts(paths: &StoryPaths, verdicts: &[Verdict], model: &str) -> Result<usize> {
    let at = now_ms();
    let list: Vec<(String, AiVerdict)> = verdicts
        .iter()
        .map(|v| (v.id.clone(), AiVerdict { label: v.label.clone(), reason: v.reason.clone(), model: model.to_string(), at }))
        .collect();
    set_ai(paths, &list)
}

pub fn verify_with_model(paths: &StoryPaths, model: &dyn TextModel, ids: &[String], cancel: &AtomicBool) -> Result<VerifyOutcome> {
    let views = report(paths);
    let all = items(&views, ids);
    let mut outcome = VerifyOutcome::default();
    for batch in all.chunks(BATCH) {
        let verdicts = match model.complete_json(ApiStep::Glossary, SYSTEM_PROMPT, &payload(batch), cancel) {
            Ok(generated) => parse_verdicts(&generated.text),
            Err(_) => vec![],
        };
        let known: Vec<Verdict> = verdicts.into_iter().filter(|v| batch.iter().any(|item| item.id == v.id)).collect();
        outcome.failed += batch.len() - known.len().min(batch.len());
        outcome.verified += save_verdicts(paths, &known, &model.label())?;
    }
    Ok(outcome)
}

/// Lệnh cho agent agy: đọc file đầu vào, ghi kết quả JSON ra file đầu ra.
pub fn agy_prompt(input: &Path, output: &Path) -> String {
    format!(
        "{SYSTEM_PROMPT}\n\nĐọc file {} (JSON, khoá `items`). Ghi kết quả đúng dạng JSON trên vào file {} rồi dừng. Không sửa file nào khác, không hỏi lại.",
        input.display(),
        output.display()
    )
}
```

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test vn_check 2>&1 | grep -E "^error|panicked|test result"`
Expected: `test result: ok. 20 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/vn_ai.rs crates/qt-ai-core/tests/vn_check.rs
git commit -m "feat(qt-ai): AI sơ loại chỗ nhạy cảm VN theo lô 20"
```

---

### Task 5: Cảnh báo khi chốt chương + lệnh `qt-ai vn-check`

**Files:**
- Modify: `crates/qt-ai-core/src/commands/accept.rs` (đoạn ghép `warnings`, khoảng dòng 122)
- Modify: `crates/qt-ai-core/src/bin/qt-ai.rs` (match lệnh + `USAGE`)
- Test: `crates/qt-ai-core/tests/commands.rs`

**Interfaces:**
- Consumes: `vn_check::check_chapter`, `vn_lexicon::Lexicon::load`, `base::BaseStore::from_env`.
- Produces: hằng `pub const VN_WARNING_PREFIX: &str = "[VN]"` trong `vn_check`; cảnh báo chương dạng `"[VN] {n} chỗ nhạy cảm về Việt Nam chưa xác minh — xem tab Kiểm tra VN"`.

- [ ] **Step 1: Viết test thất bại** — thêm vào `crates/qt-ai-core/tests/commands.rs` (dùng sẵn `make_story_dir`, `run_init`, `run_next`, `run_accept`, `work_file`, `WorkKind`):

```rust
#[test]
fn accept_canh_bao_cho_nhay_cam_vn_chua_xac_minh() {
    let dir = make_story_dir(&[("0001", "第1章\n\n坐船转到越南。")]);
    let root = dir.path();
    run_init(root, "qt-ai").unwrap();
    let paths = story_paths(root);
    run_next(root).unwrap();
    fs::write(work_file(&paths, "0001", WorkKind::Draft), "[[1]] Chương 1\n\n[[2]] Ngồi thuyền chuyển qua Việt Nam.").unwrap();
    let accepted = run_accept(root, "0001", true).unwrap();
    let warning = accepted.warnings.iter().find(|w| w.starts_with(qt_ai_core::vn_check::VN_WARNING_PREFIX)).expect("có cảnh báo VN");
    assert!(warning.contains("1 chỗ nhạy cảm về Việt Nam"), "{warning}");
    assert!(root.join(qt_ai_core::vn_check::VN_FILE).exists());
    // Người dùng chốt "Không sao" → quét lại chương không còn cảnh báo.
    let ids: Vec<String> = qt_ai_core::vn_check::load_file(&paths).hits.iter().map(|h| h.id.clone()).collect();
    qt_ai_core::vn_check::set_status(&paths, &ids, qt_ai_core::vn_check::Status::Ok).unwrap();
    let lexicon = qt_ai_core::vn_lexicon::Lexicon::builtin();
    let open = qt_ai_core::vn_check::check_chapter(&paths, &lexicon, "0001").unwrap();
    assert_eq!(open, 0);
    assert_eq!(qt_ai_core::vn_check::chapter_warning(open), None);
}
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core --test commands accept_canh_bao_cho_nhay_cam 2>&1 | grep -E "^error|panicked|test result"`
Expected: lỗi biên dịch (`VN_WARNING_PREFIX` chưa có) hoặc panic "có cảnh báo VN".

- [ ] **Step 3: Cài đặt**

Trong `vn_check.rs` thêm:

```rust
/// Đầu chuỗi cảnh báo chương do kiểm tra VN ghi — GUI nhận ra để dẫn sang tab Kiểm tra VN.
pub const VN_WARNING_PREFIX: &str = "[VN]";

pub fn chapter_warning(open: usize) -> Option<String> {
    (open > 0).then(|| format!("{VN_WARNING_PREFIX} {open} chỗ nhạy cảm về Việt Nam chưa xác minh — xem tab Kiểm tra VN"))
}
```

Trong `commands/accept.rs`, thay dòng

```rust
    let warnings: Vec<String> = check.issues.iter().chain(&check.advisories).cloned().collect();
```

bằng

```rust
    let mut warnings: Vec<String> = check.issues.iter().chain(&check.advisories).cloned().collect();
    // Kiểm tra VN chỉ cảnh báo; lỗi đọc/ghi vn-check.json không được chặn chốt chương.
    let lexicon = crate::vn_lexicon::Lexicon::load(&crate::base::BaseStore::from_env());
    if let Some(warning) = crate::vn_check::check_chapter(&paths, &lexicon, id).ok().and_then(crate::vn_check::chapter_warning) {
        warnings.push(warning);
    }
```

Trong `bin/qt-ai.rs`, thêm nhánh trước `"status" =>`:

```rust
        "vn-check" => {
            let paths = qt_ai_core::story_fs::story_paths(root);
            let lexicon = qt_ai_core::vn_lexicon::Lexicon::load(&qt_ai_core::base::BaseStore::from_env());
            let started = std::time::Instant::now();
            let summary = qt_ai_core::vn_check::refresh(&paths, &lexicon)?;
            println!(
                "Kiểm tra VN: {} chỗ ({} chưa xét/cần xử lý), quét {} chương, {} ms",
                summary.hits,
                summary.open,
                summary.scanned,
                started.elapsed().as_millis()
            );
            for hit in qt_ai_core::vn_check::load_file(&paths).hits.iter().filter(|h| h.status != qt_ai_core::vn_check::Status::Ok) {
                println!("  {} [{:?}] {} ×{}", hit.id, hit.side, hit.matched, hit.count);
            }
            Ok(0)
        }
```

và thêm vào chuỗi `USAGE` một dòng: `  qt-ai vn-check <root>              quét nội dung nhạy cảm về Việt Nam`.

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-core 2>&1 | grep -E "^error|panicked|FAILED|test result" | grep -v "ok. 0 passed"`
Expected: mọi dòng `test result: ok`, không có `FAILED`.

- [ ] **Step 5: Commit**

```bash
git add crates/qt-ai-core/src/vn_check.rs crates/qt-ai-core/src/commands/accept.rs crates/qt-ai-core/src/bin/qt-ai.rs crates/qt-ai-core/tests/commands.rs
git commit -m "feat(qt-ai): cảnh báo [VN] khi chốt chương, lệnh qt-ai vn-check"
```

---

### Task 6: Lệnh Tauri `vn_cmds` + quét khi quét chương mới + số cờ ở tóm tắt truyện

**Files:**
- Create: `apps/qt-ai-gui/src-tauri/src/vn_cmds.rs`
- Modify: `apps/qt-ai-gui/src-tauri/src/lib.rs` (khai báo `mod vn_cmds;`, đăng ký lệnh)
- Modify: `apps/qt-ai-gui/src-tauri/src/library_cmds.rs` (`rescan_story`, `library_rescan`, `import_chapters`)
- Modify: `apps/qt-ai-gui/src-tauri/src/story_cmds.rs` (`RecentSummary`), `summary_cache.rs` (`summarize_uncached`, `Stamp`)
- Modify: `apps/qt-ai-gui/src-tauri/src/replace_cmds.rs` (`refuse_if_running` → `pub(crate)`)

**Interfaces:**
- Consumes: mọi API lõi ở Task 1–4; `replace::{find_in_story, replace_in_story, FindQuery, HitRef, ReplaceOutcome}`; `session_cmds::{resolve_api, session_config}`; `session::run_once`.
- Produces (IPC, camelCase): `vn_refresh(root) -> VnSummary`, `vn_report(root) -> HitView[]`, `vn_set_status(root, ids, status: "pending"|"ok"|"action") -> number`, `vn_ai_verify(root, ids) -> VerifyOutcome`, `vn_builtin_entries() -> Entry[]`, `vn_settings_get() -> VnSettings`, `vn_settings_save(settings) -> VnSettings`, `vn_replace(root, request: {from, to, chapters: string[], paragraphs?: [string, number][]}) -> ReplaceOutcome`; `RecentSummary.vnFlags: number | null`.

- [ ] **Step 1: Viết test thất bại** — tạo `vn_cmds.rs` với phần test trước (module `tests` ở cuối file; hàm `*_inner` chưa có):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn story() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for sub in ["raw", "out"] {
            fs::create_dir_all(dir.path().join(sub)).unwrap();
        }
        fs::write(dir.path().join("raw/0001.txt"), "第1章\n\n坐船转到越南。\n\n又去越南。").unwrap();
        fs::write(dir.path().join("out/0001.txt"), "Chương 1\n\nNgồi thuyền qua Việt Nam.\n\nLại đi Việt Nam.").unwrap();
        fs::write(dir.path().join("raw/0002.txt"), "第2章\n\n越南很热。").unwrap();
        fs::write(dir.path().join("out/0002.txt"), "Chương 2\n\nViệt Nam rất nóng.").unwrap();
        dir
    }

    #[test]
    fn replace_chi_doan_da_chon_roi_quet_lai() {
        let dir = story();
        let lex = Lexicon::builtin();
        refresh_inner(dir.path(), &lex).unwrap();
        let request = VnReplaceRequest {
            from: "Việt Nam".into(),
            to: "Thái Lan".into(),
            chapters: vec!["0001".into()],
            paragraphs: Some(vec![("0001".into(), 1)]),
        };
        let outcome = replace_inner(dir.path(), &request, &lex).unwrap();
        assert_eq!((outcome.chapters, outcome.occurrences), (1, 1));
        let one = fs::read_to_string(dir.path().join("out/0001.txt")).unwrap();
        assert!(one.contains("Ngồi thuyền qua Thái Lan.") && one.contains("Lại đi Việt Nam."), "{one}");
        assert!(fs::read_to_string(dir.path().join("out/0002.txt")).unwrap().contains("Việt Nam"), "chương 2 ngoài phạm vi");
        let left: Vec<String> = qt_ai_core::vn_check::load_file(&story_paths(dir.path())).hits.iter().map(|h| h.id.clone()).collect();
        assert_eq!(left, vec!["0001:2:viet-nam", "0002:1:viet-nam"], "quét lại sau khi thay");
        // Hoàn tác bằng engine Tìm & thay sẵn có.
        qt_ai_core::replace::undo_last_replace(&story_paths(dir.path())).unwrap();
        assert!(fs::read_to_string(dir.path().join("out/0001.txt")).unwrap().contains("Ngồi thuyền qua Việt Nam."));
    }

    #[test]
    fn replace_ca_chuong_khi_khong_chon_doan() {
        let dir = story();
        let lex = Lexicon::builtin();
        let request = VnReplaceRequest { from: "Việt Nam".into(), to: "Thái Lan".into(), chapters: vec!["0001".into(), "0002".into()], paragraphs: None };
        assert_eq!(replace_inner(dir.path(), &request, &lex).unwrap().occurrences, 3);
    }

    #[test]
    fn tom_tat_truyen_co_so_co_vn_sau_khi_quet() {
        let dir = story();
        qt_ai_core::commands::init::run_init(dir.path(), "qt-ai").unwrap();
        assert_eq!(crate::summary_cache::summarize_uncached(dir.path()).vn_flags, None);
        refresh_inner(dir.path(), &Lexicon::builtin()).unwrap();
        assert_eq!(crate::summary_cache::summarize_uncached(dir.path()).vn_flags, Some(3));
    }
}
```

(`ReplaceOutcome` trong `crates/qt-ai-core/src/replace.rs` có hai trường `chapters`, `occurrences`.)

Trong `lib.rs`, test `lenh_nang_phai_la_async_fn_boc_blocking` (khoảng dòng 188): thêm `include_str!("vn_cmds.rs"),` vào mảng `sources` và thêm vào mảng `heavy`:

```rust
            "vn_refresh",
            "vn_report",
            "vn_set_status",
            "vn_ai_verify",
            "vn_builtin_entries",
            "vn_settings_get",
            "vn_settings_save",
            "vn_replace",
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-gui --lib vn_cmds 2>&1 | grep -E "^error" | sort | uniq -c`
Expected: lỗi biên dịch (chưa có `refresh_inner`, `replace_inner`, `VnReplaceRequest`, `vn_flags`).

- [ ] **Step 3: Viết `vn_cmds.rs`** (đặt trên khối test):

```rust
//! Tab "Kiểm tra VN": quét, xác minh (người dùng + AI), thay thế trong bản dịch. Logic ở qt_ai_core::{vn_check, vn_ai}.

use crate::app_config::Engine;
use crate::error::{blocking, CmdResult, CommandError};
use crate::replace_cmds::refuse_if_running;
use crate::session_cmds::{resolve_agy, resolve_api, session_config};
use crate::AppState;
use qt_ai_core::api::HttpModel;
use qt_ai_core::base::BaseStore;
use qt_ai_core::replace::{find_in_story, replace_in_story, FindQuery, HitRef, ReplaceOutcome};
use qt_ai_core::session::run_once;
use qt_ai_core::story_fs::story_paths;
use qt_ai_core::vn_ai::{agy_prompt, items, parse_verdicts, payload, save_verdicts, verify_with_model, VerifyOutcome};
use qt_ai_core::vn_check::{refresh, report, set_status, HitView, Status, VnSummary};
use qt_ai_core::vn_lexicon::{builtin_entries, load_settings, save_settings, Entry, Lexicon, VnSettings};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Manager, Runtime};

fn lexicon() -> Lexicon {
    Lexicon::load(&BaseStore::from_env())
}

pub fn refresh_inner(root: &Path, lex: &Lexicon) -> CmdResult<VnSummary> {
    Ok(refresh(&story_paths(root), lex)?)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnReplaceRequest {
    pub from: String,
    pub to: String,
    /// Chương được phép thay.
    pub chapters: Vec<String>,
    /// Chỉ các đoạn này (chương, đoạn); None = mọi chỗ khớp trong `chapters`.
    #[serde(default)]
    pub paragraphs: Option<Vec<(String, usize)>>,
}

/// Thay nguyên từ, phân biệt hoa thường, giới hạn đúng chương/đoạn bằng danh sách loại trừ của engine Tìm & thay
/// (nên có hoàn tác); xong quét lại.
pub fn replace_inner(root: &Path, request: &VnReplaceRequest, lex: &Lexicon) -> CmdResult<ReplaceOutcome> {
    let paths = story_paths(root);
    let query = FindQuery {
        find: request.from.clone(),
        replace: request.to.clone(),
        case_sensitive: true,
        whole_word: true,
        ..Default::default()
    };
    let found = find_in_story(&paths, &query, usize::MAX)?;
    let wanted: HashSet<&str> = request.chapters.iter().map(String::as_str).collect();
    let excluded_chapters: Vec<String> =
        found.chapters.iter().map(|c| c.chapter.clone()).filter(|id| !wanted.contains(id.as_str())).collect();
    let excluded_hits: Vec<HitRef> = match &request.paragraphs {
        None => vec![],
        Some(keep) => found
            .hits
            .iter()
            .filter(|hit| !keep.iter().any(|(chapter, paragraph)| chapter == &hit.chapter && *paragraph == hit.paragraph))
            .map(|hit| HitRef { chapter: hit.chapter.clone(), paragraph: hit.paragraph, occurrence: hit.occurrence })
            .collect(),
    };
    let outcome = replace_in_story(&paths, &query, &excluded_chapters, &excluded_hits)?;
    refresh(&paths, lex)?;
    Ok(outcome)
}

#[tauri::command]
pub async fn vn_refresh(root: String) -> CmdResult<VnSummary> {
    blocking(move || refresh_inner(Path::new(&root), &lexicon())).await
}

#[tauri::command]
pub async fn vn_report(root: String) -> CmdResult<Vec<HitView>> {
    blocking(move || Ok(report(&story_paths(Path::new(&root))))).await
}

#[tauri::command]
pub async fn vn_set_status(root: String, ids: Vec<String>, status: Status) -> CmdResult<usize> {
    blocking(move || Ok(set_status(&story_paths(Path::new(&root)), &ids, status)?)).await
}

/// Bản cứng của từ điển — Cài đặt liệt kê để bật/tắt từng mục.
#[tauri::command]
pub async fn vn_builtin_entries() -> CmdResult<Vec<Entry>> {
    blocking(move || Ok(builtin_entries())).await
}

#[tauri::command]
pub async fn vn_settings_get() -> CmdResult<VnSettings> {
    blocking(move || Ok(load_settings(&BaseStore::from_env()))).await
}

#[tauri::command]
pub async fn vn_settings_save(settings: VnSettings) -> CmdResult<VnSettings> {
    blocking(move || {
        let base = BaseStore::from_env();
        save_settings(&base, &settings)?;
        Ok(load_settings(&base))
    })
    .await
}

#[tauri::command]
pub async fn vn_replace<R: Runtime>(app: AppHandle<R>, root: String, request: VnReplaceRequest) -> CmdResult<ReplaceOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        replace_inner(Path::new(&root), &request, &lexicon())
    })
    .await
}

/// AI xét các chỗ `ids`: động cơ API dùng API key; agy ghi đầu vào ra work/, agent ghi kết quả ra work/.
#[tauri::command]
pub async fn vn_ai_verify<R: Runtime>(app: AppHandle<R>, root: String, ids: Vec<String>) -> CmdResult<VerifyOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let (engine, api, model) = {
            let config = state.config.lock().unwrap();
            (config.engine, config.api.clone(), config.model.clone())
        };
        let paths = story_paths(Path::new(&root));
        match engine {
            Engine::Api => {
                let model = HttpModel::new(resolve_api(&api)?);
                Ok(verify_with_model(&paths, &model, &ids, &AtomicBool::new(false))?)
            }
            Engine::Agy => {
                let all = items(&report(&paths), &ids);
                std::fs::create_dir_all(&paths.work_dir).map_err(|e| CommandError::new("io", e.to_string()))?;
                let input = paths.work_dir.join("vn-verify-input.json");
                let output = paths.work_dir.join("vn-verify-output.json");
                std::fs::write(&input, payload(&all)).map_err(|e| CommandError::new("io", e.to_string()))?;
                let _ = std::fs::remove_file(&output);
                let agy = resolve_agy(&state)?;
                let config = session_config(Path::new(&root), agy, model.clone(), 1, false);
                run_once(&config, &agy_prompt(&input, &output), &|_| {})?;
                let text = std::fs::read_to_string(&output).unwrap_or_default();
                let verdicts: Vec<_> = parse_verdicts(&text).into_iter().filter(|v| ids.contains(&v.id)).collect();
                let label = format!("agy {}", model.unwrap_or_else(|| "mặc định".into()));
                let verified = save_verdicts(&paths, &verdicts, &label)?;
                let _ = std::fs::remove_file(&input);
                let _ = std::fs::remove_file(&output);
                Ok(VerifyOutcome { verified, failed: all.len().saturating_sub(verified) })
            }
        }
    })
    .await
}
```

Trong `replace_cmds.rs` đổi `fn refuse_if_running` thành `pub(crate) fn refuse_if_running`; trong `session_cmds.rs` đổi `fn resolve_agy` thành `pub(crate) fn resolve_agy` (nhận `&State<'_, AppState>` — `app.state::<AppState>()` trả đúng kiểu này).

Trong `lib.rs`: thêm `mod vn_cmds;` cạnh các `mod` khác; trong `generate_handler!` thêm:

```rust
            vn_cmds::vn_refresh,
            vn_cmds::vn_report,
            vn_cmds::vn_set_status,
            vn_cmds::vn_ai_verify,
            vn_cmds::vn_builtin_entries,
            vn_cmds::vn_settings_get,
            vn_cmds::vn_settings_save,
            vn_cmds::vn_replace,
```

Trong `story_cmds.rs`, `RecentSummary` thêm trường cuối:

```rust
    /// Chỗ nhạy cảm VN chưa xét + cần xử lý; None = truyện chưa từng quét.
    pub vn_flags: Option<usize>,
```

Trong `summary_cache.rs`: `type Stamp = (Option<SystemTime>, Option<SystemTime>, Option<SystemTime>);`, dòng tạo stamp thành
`let stamp: Stamp = (mtime(&paths.state_json), mtime(&paths.story_json), mtime(&paths.root.join(qt_ai_core::vn_check::VN_FILE)));`; trong `summarize_uncached` thêm `vn_flags: qt_ai_core::vn_check::flag_count(&paths),` vào nhánh đọc được và `vn_flags: None` vào nhánh lỗi. Sửa mọi chỗ khác dựng `RecentSummary { … }` (tìm bằng `grep -rn "RecentSummary {" apps/qt-ai-gui/src-tauri/src`) để thêm `vn_flags: None`.

Trong `library_cmds.rs`:
- `rescan_story`: sau `run_init(path, &qt_ai_command())?;` thêm `let _ = qt_ai_core::vn_check::refresh(&story_paths(path), &qt_ai_core::vn_lexicon::Lexicon::load(&qt_ai_core::base::BaseStore::from_env()));`
- `library_rescan`: sau dòng `let report = rescan_library_inner(...)`, thêm:

```rust
        let lexicon = qt_ai_core::vn_lexicon::Lexicon::load(&qt_ai_core::base::BaseStore::from_env());
        for entry in std::fs::read_dir(library).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.join("state.json").is_file() {
                let _ = qt_ai_core::vn_check::refresh(&story_paths(&path), &lexicon);
            }
        }
```

- `import_chapters`: sau khi nhập xong (trước khi dựng snapshot trả về) thêm dòng `refresh` như `rescan_story`.

(Lỗi quét VN không được làm hỏng các lệnh này — luôn `let _ =`.)

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `CARGO_INCREMENTAL=0 cargo test -q -p qt-ai-gui 2>&1 | grep -E "^error|panicked|FAILED|test result" | grep -v "ok. 0 passed"`
Expected: `test result: ok.`, không `FAILED` (kể cả test `lenh_nang_phai_la_async_fn_boc_blocking` — mọi lệnh mới là `async fn` bọc `blocking(move ||`).

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src-tauri/src
git commit -m "feat(qt-ai-gui): lệnh Tauri kiểm tra VN, quét khi quét chương mới, số cờ ở tóm tắt truyện"
```

---

### Task 7: Schema, API frontend và thẻ Cài đặt

**Files:**
- Modify: `apps/qt-ai-gui/src/lib/schema.ts`, `apps/qt-ai-gui/src/lib/types.ts`, `apps/qt-ai-gui/src/lib/api.ts`
- Create: `apps/qt-ai-gui/src/components/vn-settings-editor.tsx`, `apps/qt-ai-gui/src/components/vn-settings-editor.test.tsx`
- Modify: `apps/qt-ai-gui/src/components/pages/settings-page.tsx` (thẻ mới bọc `<VnSettingsEditor />` sau thẻ "Bản mặc định"), `apps/qt-ai-gui/src/components/pages/settings-page.test.tsx` (mock)

**Interfaces:**
- Produces (TS): schema `vnHitViewSchema`, `vnSummarySchema`, `vnSettingsSchema`, `vnVerifyOutcomeSchema`; type `VnHitView`, `VnSettings`, `VnEntry`, `VnReplacement`, `VnStatus`; hằng `VN_GROUP_LABELS`, `VN_LABEL_TEXT`; API `vnRefresh(root)`, `vnReport(root)`, `vnSetStatus(root, ids, status)`, `vnAiVerify(root, ids)`, `vnBuiltinEntries()`, `vnSettingsGet()`, `vnSettingsSave(settings)`, `vnReplace(root, request)`; `recentSummarySchema.vnFlags`.

- [ ] **Step 1: Viết test thất bại** — `vn-settings-editor.test.tsx`:

```tsx
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnSettingsEditor } from "@/components/vn-settings-editor";
import { vnBuiltinEntries, vnSettingsGet, vnSettingsSave } from "@/lib/api";
import type { VnEntry, VnSettings } from "@/lib/types";

vi.mock("@/lib/api", () => ({ vnBuiltinEntries: vi.fn(), vnSettingsGet: vi.fn(), vnSettingsSave: vi.fn() }));

const builtin: VnEntry[] = [
  { id: "viet-nam", group: "ten", level: "canxem", weak: false, zh: ["越南"], vi: ["Việt Nam"], exclude: [] },
  { id: "nam-hai", group: "cq", level: "canxem", weak: true, zh: ["南海"], vi: ["Nam Hải", "Biển Đông"], exclude: [] },
];
const settings: VnSettings = {
  extra: [],
  disabled: [],
  replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }],
};

describe("VnSettingsEditor", () => {
  beforeEach(() => {
    vi.mocked(vnBuiltinEntries).mockReset().mockResolvedValue(builtin);
    vi.mocked(vnSettingsGet).mockReset().mockResolvedValue(settings);
    vi.mocked(vnSettingsSave).mockReset().mockImplementation((next) => Promise.resolve(next));
  });

  it("sửa, thêm dòng bảng thay rồi Lưu", async () => {
    const user = userEvent.setup();
    render(<VnSettingsEditor />);
    const list = await screen.findByRole("list", { name: "Bảng thay" });
    const to = within(within(list).getAllByRole("listitem")[0]).getByLabelText("Cụm thay");
    await user.clear(to);
    await user.type(to, "Xiêm La");
    await user.click(screen.getByRole("button", { name: "Thêm dòng thay" }));
    const second = within(list).getAllByRole("listitem")[1];
    await user.type(within(second).getByLabelText("Cụm gốc"), "Giao Chỉ");
    await user.type(within(second).getByLabelText("Cụm thay"), "Chiêm Thành");
    await user.selectOptions(within(second).getByLabelText("Bối cảnh"), "ancient");
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình VN" }));
    expect(vnSettingsSave).toHaveBeenCalledWith({
      extra: [],
      disabled: [],
      replacements: [
        { from: "Việt Nam", to: "Xiêm La", setting: "modern" },
        { from: "Giao Chỉ", to: "Chiêm Thành", setting: "ancient" },
      ],
    });
  });

  it("tắt mục có sẵn, thêm mục riêng, bỏ dòng trống khi lưu", async () => {
    const user = userEvent.setup();
    render(<VnSettingsEditor />);
    await user.click(await screen.findByRole("checkbox", { name: /nam-hai/ }));
    await user.click(screen.getByRole("button", { name: "Thêm mục riêng" }));
    const extra = within(screen.getByRole("list", { name: "Mục riêng" })).getAllByRole("listitem")[0];
    await user.type(within(extra).getByLabelText("Chữ Hán"), "猴子国, 猴国人");
    await user.type(within(extra).getByLabelText("Tiếng Việt"), "nước khỉ con");
    await user.click(within(extra).getByRole("checkbox", { name: "Miệt thị" }));
    await user.click(screen.getByRole("button", { name: "Thêm dòng thay" }));
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình VN" }));
    expect(vnSettingsSave).toHaveBeenCalledWith({
      disabled: ["nam-hai"],
      extra: [{ id: "rieng-1", group: "ten", level: "mietthi", weak: false, zh: ["猴子国", "猴国人"], vi: ["nước khỉ con"], exclude: [] }],
      replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }],
    });
  });
});
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/vn-settings-editor.test.tsx 2>&1 | tail -5`
Expected: FAIL — không tìm thấy module `@/components/vn-settings-editor`.

- [ ] **Step 3: Schema, type, API, component**

`schema.ts` — thêm (cạnh các schema khác), và thêm `vnFlags` vào `recentSummarySchema`:

```ts
export const VN_GROUPS = ["ten", "cq", "ls", "hd"] as const;
export const VN_STATUSES = ["pending", "ok", "action"] as const;
export const VN_LABELS = ["xucpham", "nhaycam", "trungtinh", "khonglienquan"] as const;
export const vnStatusSchema = z.enum(VN_STATUSES);
export const vnEntrySchema = z.object({
  id: z.string(),
  group: z.enum(VN_GROUPS),
  level: z.enum(["mietthi", "canxem"]),
  weak: z.boolean().default(false),
  zh: z.array(z.string()).default([]),
  vi: z.array(z.string()).default([]),
  exclude: z.array(z.string()).default([]),
});
export const vnReplacementSchema = z.object({
  from: z.string(),
  to: z.string(),
  setting: z.enum(["ancient", "modern", "any"]),
});
export const vnSettingsSchema = z.object({
  extra: z.array(vnEntrySchema).default([]),
  disabled: z.array(z.string()).default([]),
  replacements: z.array(vnReplacementSchema).default([]),
});
export const vnHitViewSchema = z.object({
  id: z.string(),
  chapter: z.string(),
  paragraph: z.number(),
  entry: z.string(),
  group: z.enum(VN_GROUPS),
  level: z.enum(["mietthi", "canxem"]),
  side: z.enum(["raw", "out"]),
  matched: z.string(),
  count: z.number(),
  fingerprint: z.string(),
  status: vnStatusSchema,
  ai: z.object({ label: z.enum(VN_LABELS), reason: z.string(), model: z.string(), at: z.number() }).nullable().optional(),
  raw: z.array(z.string()),
  out: z.array(z.string()),
  focus: z.number(),
});
export const vnSummarySchema = z.object({ hits: z.number(), open: z.number(), scanned: z.number() });
export const vnVerifyOutcomeSchema = z.object({ verified: z.number(), failed: z.number() });
```

```ts
// trong recentSummarySchema:
  /** Chỗ nhạy cảm VN chưa xét + cần xử lý; null = chưa quét. */
  vnFlags: z.number().nullable().optional(),
```

`types.ts` — import các schema trên và thêm:

```ts
export type VnHitView = z.infer<typeof vnHitViewSchema>;
export type VnSettings = z.infer<typeof vnSettingsSchema>;
export type VnEntry = z.infer<typeof vnEntrySchema>;
export type VnReplacement = z.infer<typeof vnReplacementSchema>;
export type VnStatus = z.infer<typeof vnStatusSchema>;
export const VN_GROUP_LABELS = { ten: "Tên gọi VN", cq: "Chủ quyền biển đảo", ls: "Chiến tranh, lịch sử", hd: "Địa danh hiện đại" } as const;
export const VN_LABEL_TEXT = { xucpham: "Xúc phạm", nhaycam: "Nhạy cảm", trungtinh: "Trung tính", khonglienquan: "Không liên quan" } as const;
```

`api.ts` — import các schema + `replaceOutcomeSchema` (đã có) và thêm:

```ts
export const vnRefresh = (root: string) => call("vn_refresh", { root }, (v) => vnSummarySchema.parse(v));
export const vnReport = (root: string) => call("vn_report", { root }, (v) => z.array(vnHitViewSchema).parse(v));
export const vnSetStatus = (root: string, ids: string[], status: VnStatus) =>
  call("vn_set_status", { root, ids, status }, (v) => z.number().parse(v));
export const vnAiVerify = (root: string, ids: string[]) => call("vn_ai_verify", { root, ids }, (v) => vnVerifyOutcomeSchema.parse(v));
export const vnBuiltinEntries = () => call("vn_builtin_entries", undefined, (v) => z.array(vnEntrySchema).parse(v));
export const vnSettingsGet = () => call("vn_settings_get", undefined, (v) => vnSettingsSchema.parse(v));
export const vnSettingsSave = (settings: VnSettings) => call("vn_settings_save", { settings }, (v) => vnSettingsSchema.parse(v));
export interface VnReplaceRequest {
  from: string;
  to: string;
  chapters: string[];
  paragraphs?: [string, number][];
}
export const vnReplace = (root: string, request: VnReplaceRequest) =>
  call("vn_replace", { root, request }, (v) => replaceOutcomeSchema.parse(v));
```

`vn-settings-editor.tsx`:

```tsx
import { Plus, Save, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { vnBuiltinEntries, vnSettingsGet, vnSettingsSave } from "@/lib/api";
import type { VnEntry, VnReplacement, VnSettings } from "@/lib/types";
import { VN_GROUP_LABELS } from "@/lib/types";

const SELECT_CLASS = "h-8 rounded-md border border-input bg-background px-2 text-sm";

/** Mục riêng đang sửa: chữ Hán / tiếng Việt giữ dạng chuỗi "a, b" để gõ dấu phẩy không bị nuốt. */
interface ExtraDraft {
  id: string;
  group: VnEntry["group"];
  mietthi: boolean;
  weak: boolean;
  zh: string;
  vi: string;
  exclude: string[];
}

const splitList = (text: string) =>
  text
    .split(/[,，]/)
    .map((part) => part.trim())
    .filter(Boolean);

const toDraft = (entry: VnEntry): ExtraDraft => ({
  id: entry.id,
  group: entry.group,
  mietthi: entry.level === "mietthi",
  weak: entry.weak,
  zh: entry.zh.join(", "),
  vi: entry.vi.join(", "),
  exclude: entry.exclude,
});

const fromDraft = (draft: ExtraDraft): VnEntry => ({
  id: draft.id,
  group: draft.group,
  level: draft.mietthi ? "mietthi" : "canxem",
  weak: draft.weak,
  zh: splitList(draft.zh),
  vi: splitList(draft.vi),
  exclude: draft.exclude,
});

function nextExtraId(rows: ExtraDraft[]): string {
  let n = rows.length + 1;
  while (rows.some((row) => row.id === `rieng-${n}`)) n += 1;
  return `rieng-${n}`;
}

/** Cấu hình kiểm tra VN (Cài đặt): bật/tắt mục có sẵn, mục riêng, bảng thay. Lưu ở vn-terms.json cạnh bản mặc định. */
export function VnSettingsEditor() {
  const [builtin, setBuiltin] = useState<VnEntry[]>([]);
  const [disabled, setDisabled] = useState<string[]>();
  const [extra, setExtra] = useState<ExtraDraft[]>([]);
  const [replacements, setReplacements] = useState<VnReplacement[]>([]);
  const [busy, setBusy] = useState(false);

  function load(settings: VnSettings) {
    setDisabled(settings.disabled);
    setExtra(settings.extra.map(toDraft));
    setReplacements(settings.replacements);
  }

  useEffect(() => {
    void Promise.all([vnBuiltinEntries(), vnSettingsGet()])
      .then(([entries, settings]) => {
        setBuiltin(entries);
        load(settings);
      })
      .catch(() => toast.error("Không đọc được cấu hình kiểm tra VN"));
  }, []);

  if (!disabled) return <p className="text-sm text-muted-foreground">Đang tải…</p>;

  const updateRow = (index: number, patch: Partial<VnReplacement>) =>
    setReplacements((rows) => rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  const updateExtra = (index: number, patch: Partial<ExtraDraft>) =>
    setExtra((rows) => rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));

  async function save() {
    if (!disabled) return;
    setBusy(true);
    try {
      load(
        await vnSettingsSave({
          extra: extra.map(fromDraft).filter((entry) => entry.zh.length + entry.vi.length > 0),
          disabled,
          replacements: replacements.filter((row) => row.from.trim() && row.to.trim()),
        }),
      );
      toast.success("Đã lưu cấu hình kiểm tra VN");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Mục có sẵn</h3>
        <p className="text-xs text-muted-foreground">Bỏ tick mục hay báo nhầm với truyện của bạn.</p>
        <div className="fine-scrollbar grid max-h-56 gap-1 overflow-y-auto rounded-md border p-2 text-sm sm:grid-cols-2">
          {builtin.map((entry) => (
            <label key={entry.id} className="flex items-start gap-2">
              <input
                type="checkbox"
                className="mt-1"
                checked={!disabled.includes(entry.id)}
                onChange={(e) =>
                  setDisabled((list = []) => (e.target.checked ? list.filter((id) => id !== entry.id) : [...list, entry.id]))
                }
              />
              <span>
                <span className="font-mono text-xs">{entry.id}</span>{" "}
                <span className="text-muted-foreground">
                  {VN_GROUP_LABELS[entry.group]} · {[...entry.vi, ...entry.zh].slice(0, 4).join(", ")}
                </span>
              </span>
            </label>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Mục riêng</h3>
        <p className="text-xs text-muted-foreground">
          Nhiều dạng cách nhau bằng dấu phẩy. Chỗ khớp trong bản dịch chỉ tính khi đoạn raw tương ứng có một dạng chữ Hán.
        </p>
        <ul aria-label="Mục riêng" className="flex flex-col gap-2">
          {extra.map((row, index) => (
            <li key={row.id} className="grid grid-cols-[1fr_1fr_9rem_auto_auto_auto] items-center gap-2 text-sm">
              <Input aria-label="Chữ Hán" value={row.zh} onChange={(e) => updateExtra(index, { zh: e.target.value })} />
              <Input aria-label="Tiếng Việt" value={row.vi} onChange={(e) => updateExtra(index, { vi: e.target.value })} />
              <select
                aria-label="Nhóm"
                className={SELECT_CLASS}
                value={row.group}
                onChange={(e) => updateExtra(index, { group: e.target.value as VnEntry["group"] })}
              >
                {Object.entries(VN_GROUP_LABELS).map(([id, label]) => (
                  <option key={id} value={id}>
                    {label}
                  </option>
                ))}
              </select>
              <label className="flex items-center gap-1">
                <input type="checkbox" checked={row.weak} onChange={(e) => updateExtra(index, { weak: e.target.checked })} />
                Từ yếu
              </label>
              <label className="flex items-center gap-1">
                <input type="checkbox" checked={row.mietthi} onChange={(e) => updateExtra(index, { mietthi: e.target.checked })} />
                Miệt thị
              </label>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                aria-label={`Xoá mục ${row.id}`}
                onClick={() => setExtra((rows) => rows.filter((_, i) => i !== index))}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
        <div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() =>
              setExtra((rows) => [...rows, { id: nextExtraId(rows), group: "ten", mietthi: false, weak: false, zh: "", vi: "", exclude: [] }])
            }
          >
            <Plus /> Thêm mục riêng
          </Button>
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Bảng thay</h3>
        <p className="text-xs text-muted-foreground">
          Gợi ý điền sẵn khi bấm "Thay thế…" ở tab Kiểm tra VN. Dòng "Cổ đại" gợi ý cho truyện cổ đại, "Hiện đại" cho truyện hiện
          đại / hỗn hợp.
        </p>
        <ul aria-label="Bảng thay" className="flex flex-col gap-2">
          {replacements.map((row, index) => (
            <li key={index} className="grid grid-cols-[1fr_1fr_8rem_auto] items-center gap-2">
              <Input aria-label="Cụm gốc" value={row.from} onChange={(e) => updateRow(index, { from: e.target.value })} />
              <Input aria-label="Cụm thay" value={row.to} onChange={(e) => updateRow(index, { to: e.target.value })} />
              <select
                aria-label="Bối cảnh"
                className={SELECT_CLASS}
                value={row.setting}
                onChange={(e) => updateRow(index, { setting: e.target.value as VnReplacement["setting"] })}
              >
                <option value="ancient">Cổ đại</option>
                <option value="modern">Hiện đại</option>
                <option value="any">Mọi truyện</option>
              </select>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                aria-label={`Xoá dòng thay ${index + 1}`}
                onClick={() => setReplacements((rows) => rows.filter((_, i) => i !== index))}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
        <div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() => setReplacements((rows) => [...rows, { from: "", to: "", setting: "any" }])}
          >
            <Plus /> Thêm dòng thay
          </Button>
        </div>
      </div>

      <div className="flex justify-end">
        <Button type="button" size="sm" disabled={busy} onClick={() => void save()}>
          <Save /> Lưu cấu hình VN
        </Button>
      </div>
    </>
  );
}
```

`settings-page.tsx`: import `VnSettingsEditor`; đặt ngay sau thẻ "Bản mặc định" (cùng cột):

```tsx
          <Card
            title="Kiểm tra nội dung nhạy cảm về Việt Nam"
            description="Dùng chung mọi truyện, lưu ở vn-terms.json cạnh bản mặc định. Đổi từ điển thì mỗi truyện quét lại toàn bộ ở lần quét kế tiếp."
          >
            <VnSettingsEditor />
          </Card>
```

Trong `settings-page.test.tsx`, thêm vào factory `vi.mock("@/lib/api", …)` ba khoá: `vnBuiltinEntries: vi.fn(() => Promise.resolve([]))`, `vnSettingsGet: vi.fn(() => Promise.resolve({ extra: [], disabled: [], replacements: [] }))`, `vnSettingsSave: vi.fn()`.

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `cd apps/qt-ai-gui && npx vitest run 2>&1 | grep -E "×|Test Files|Tests" && npm run -s typecheck && npx eslint src/components/vn-settings-editor.tsx src/lib`
Expected: mọi file test qua; typecheck, eslint không lỗi.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/lib apps/qt-ai-gui/src/components/vn-settings-editor.tsx apps/qt-ai-gui/src/components/vn-settings-editor.test.tsx apps/qt-ai-gui/src/components/pages/settings-page.tsx apps/qt-ai-gui/src/components/pages/settings-page.test.tsx
git commit -m "feat(qt-ai-gui): cấu hình kiểm tra VN trong Cài đặt — bật/tắt mục, mục riêng, bảng thay"
```

---

### Task 8: Tab "Kiểm tra VN" — danh sách, lọc, xác minh, AI

**Files:**
- Create: `apps/qt-ai-gui/src/components/vn-check-panel.tsx`, `apps/qt-ai-gui/src/components/vn-check-panel.test.tsx`
- Modify: `apps/qt-ai-gui/src/components/pages/story-page.tsx` (thêm tab `{ id: "vn", label: "Kiểm tra VN" }` và `<Section id="vn" …><VnCheckPanel root={currentRoot} running={running} setting={currentStory.genre.setting} /></Section>`)

**Interfaces:**
- Consumes: `vnRefresh`, `vnReport`, `vnSetStatus`, `vnAiVerify` (Task 7); `useStoryStore` `select(id)` / `setPage("translate")` (như `replace-page.tsx`); `VnReplaceDialog` — task này tạo bản tạm trả `null` để biên dịch được, Task 9 viết thật.
- Produces: `VnCheckPanel({ root, running, setting })`.

- [ ] **Step 1: Viết test thất bại** — `vn-check-panel.test.tsx`:

```tsx
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnCheckPanel } from "@/components/vn-check-panel";
import { vnAiVerify, vnRefresh, vnReport, vnSetStatus } from "@/lib/api";
import type { VnHitView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ vnRefresh: vi.fn(), vnReport: vi.fn(), vnSetStatus: vi.fn(), vnAiVerify: vi.fn(), vnReplace: vi.fn(), vnSettingsGet: vi.fn() }));

const hit = (over: Partial<VnHitView>): VnHitView => ({
  id: "0226:4:khi-viet",
  chapter: "0226",
  paragraph: 4,
  entry: "khi-viet",
  group: "ten",
  level: "mietthi",
  side: "out",
  matched: "khỉ Việt Nam",
  count: 1,
  fingerprint: "x",
  status: "pending",
  ai: null,
  raw: ["前。", "什么东瀛，高丽，越南猴子。", "后。"],
  out: ["Trước.", "Nào Đông Doanh này, Cao Ly này, khỉ Việt Nam này.", "Sau."],
  focus: 1,
  ...over,
});

const giaoChi = (over: Partial<VnHitView> = {}) =>
  hit({ id: "0354:3:giao-chi", chapter: "0354", entry: "giao-chi", level: "canxem", matched: "Giao Chỉ", out: ["", "phía nam đánh chiếm Lữ Tống và Giao Chỉ", ""], ...over });

const renderPanel = () => render(<VnCheckPanel root="D:\\t" running={false} setting="modern" />);

describe("VnCheckPanel", () => {
  beforeEach(() => {
    vi.mocked(vnRefresh).mockReset().mockResolvedValue({ hits: 2, open: 2, scanned: 0 });
    vi.mocked(vnReport).mockReset().mockResolvedValue([hit({}), giaoChi({ status: "ok" })]);
    vi.mocked(vnSetStatus).mockReset().mockResolvedValue(1);
    vi.mocked(vnAiVerify).mockReset().mockResolvedValue({ verified: 1, failed: 0 });
  });

  it("quét khi mở, mặc định ẩn chỗ đã 'Không sao', tô chữ khớp, hiện mức miệt thị", async () => {
    renderPanel();
    expect(vnRefresh).toHaveBeenCalledWith("D:\\t");
    const item = (await screen.findByText("0226")).closest("li")!;
    expect(within(item).getByText("khỉ Việt Nam", { selector: "mark" })).toBeInTheDocument();
    expect(within(item).getByText("Miệt thị")).toBeInTheDocument();
    expect(screen.queryByText("0354")).not.toBeInTheDocument();
    await userEvent.setup().selectOptions(screen.getByLabelText("Trạng thái"), "all");
    expect(screen.getByText("0354")).toBeInTheDocument();
  });

  it("bấm Không sao / Cần xử lý gọi đặt trạng thái rồi tải lại", async () => {
    const user = userEvent.setup();
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("button", { name: "Không sao" }));
    expect(vnSetStatus).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"], "ok");
    await user.click(within(item).getByRole("button", { name: "Cần xử lý" }));
    expect(vnSetStatus).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"], "action");
    expect(vnReport).toHaveBeenCalledTimes(3);
  });

  it("chọn chỗ rồi AI xét; hiện nhãn và lý do AI", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValueOnce([hit({})]).mockResolvedValueOnce([
      hit({ ai: { label: "xucpham", reason: "Gọi người Việt là khỉ.", model: "Fake", at: 1 } }),
    ]);
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    await user.click(screen.getByRole("button", { name: /AI xét 1 chỗ/ }));
    expect(vnAiVerify).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"]);
    expect(await screen.findByText("Gọi người Việt là khỉ.")).toBeInTheDocument();
    expect(screen.getByText("AI: Xúc phạm")).toBeInTheDocument();
  });

  it("lọc theo mức, nhãn AI; chọn tất cả đang lọc", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValue([hit({}), giaoChi({ ai: { label: "trungtinh", reason: "r", model: "m", at: 1 } })]);
    renderPanel();
    await screen.findByText("0226");
    await user.selectOptions(screen.getByLabelText("Mức"), "canxem");
    expect(screen.queryByText("0226")).not.toBeInTheDocument();
    expect(screen.getByText("0354")).toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Mức"), "all");
    await user.selectOptions(screen.getByLabelText("Nhãn AI"), "none");
    expect(screen.getByText("0226")).toBeInTheDocument();
    expect(screen.queryByText("0354")).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Nhãn AI"), "all");
    await user.click(screen.getByRole("checkbox", { name: "Chọn tất cả đang lọc" }));
    expect(screen.getByRole("button", { name: /AI xét 2 chỗ/ })).toBeEnabled();
  });

  it("xếp theo mục từ điển: chỗ cùng mục đứng liền, có đầu nhóm", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValue([hit({}), giaoChi(), hit({ id: "0400:1:khi-viet", chapter: "0400", paragraph: 1 })]);
    renderPanel();
    await screen.findByText("0226");
    await user.selectOptions(screen.getByLabelText("Xếp theo"), "entry");
    const chapters = screen.getAllByTestId("vn-hit-chapter").map((node) => node.textContent);
    expect(chapters).toEqual(["0354", "0226", "0400"]);
    expect(screen.getAllByTestId("vn-entry-header").map((node) => node.textContent)).toEqual(["giao-chi", "khi-viet"]);
  });

  it("Mở chương chuyển sang trang dịch, chọn đúng chương", async () => {
    const user = userEvent.setup();
    const select = vi.fn();
    const setPage = vi.fn();
    useStoryStore.setState({ select, setPage });
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("button", { name: "Mở chương" }));
    expect(select).toHaveBeenCalledWith("0226");
    expect(setPage).toHaveBeenCalledWith("translate");
  });
});
```

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/vn-check-panel.test.tsx 2>&1 | tail -5`
Expected: FAIL — không tìm thấy module.

- [ ] **Step 3: Viết `vn-check-panel.tsx`**

```tsx
import { AlertTriangle, Check, RefreshCw, Sparkles, Wrench } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { VnReplaceDialog } from "@/components/vn-replace-dialog";
import { vnAiVerify, vnRefresh, vnReport, vnSetStatus } from "@/lib/api";
import type { GenreSetting, VnHitView, VnStatus } from "@/lib/types";
import { VN_GROUP_LABELS, VN_LABEL_TEXT } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  running: boolean;
  setting: GenreSetting;
}

const SELECT_CLASS = "h-8 rounded-md border border-input bg-background px-2 text-sm";
type StatusFilter = "open" | VnStatus | "all";

/** Tô chữ khớp trong đoạn (lần đầu tiên). */
function Highlight({ text, matched }: { text: string; matched: string }) {
  const at = matched ? text.indexOf(matched) : -1;
  if (at < 0) return <>{text}</>;
  return (
    <>
      {text.slice(0, at)}
      <mark className="rounded bg-status-warning/30 px-0.5">{matched}</mark>
      {text.slice(at + matched.length)}
    </>
  );
}

/** Tab "Kiểm tra VN": chỗ nhạy cảm về Việt Nam trong raw/bản dịch, xác minh, AI sơ loại, thay thế. */
export function VnCheckPanel({ root, running, setting }: Props) {
  const select = useStoryStore((s) => s.select);
  const setPage = useStoryStore((s) => s.setPage);
  const [hits, setHits] = useState<VnHitView[]>();
  const [status, setStatus] = useState<StatusFilter>("open");
  const [group, setGroup] = useState("all");
  const [level, setLevel] = useState("all");
  const [aiLabel, setAiLabel] = useState("all");
  const [order, setOrder] = useState<"chapter" | "entry">("chapter");
  const [selected, setSelected] = useState<string[]>([]);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [replaceOpen, setReplaceOpen] = useState(false);

  const reload = useCallback(async () => setHits(await vnReport(root)), [root]);

  useEffect(() => {
    let alive = true;
    void (async () => {
      try {
        await vnRefresh(root);
        if (alive) await reload();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : "Không quét được");
      }
    })();
    return () => {
      alive = false;
    };
  }, [root, reload]);

  const shown = useMemo(() => {
    const list = (hits ?? []).filter(
      (hit) =>
        (status === "all" || (status === "open" ? hit.status !== "ok" : hit.status === status)) &&
        (group === "all" || hit.group === group) &&
        (level === "all" || hit.level === level) &&
        (aiLabel === "all" || (aiLabel === "none" ? !hit.ai : hit.ai?.label === aiLabel)),
    );
    // sort ổn định: trong cùng mục vẫn giữ thứ tự chương.
    return order === "entry" ? [...list].sort((a, b) => a.entry.localeCompare(b.entry)) : list;
  }, [hits, status, group, level, aiLabel, order]);

  const allShownSelected = shown.length > 0 && shown.every((hit) => selected.includes(hit.id));

  async function mark(ids: string[], next: VnStatus) {
    await vnSetStatus(root, ids, next);
    await reload();
  }

  async function verify() {
    setBusy(true);
    try {
      const outcome = await vnAiVerify(root, selected);
      toast.success(`AI đã xét ${outcome.verified} chỗ${outcome.failed ? `, ${outcome.failed} chỗ lỗi — thử lại` : ""}`);
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI xét thất bại");
    } finally {
      setBusy(false);
    }
  }

  if (!hits) return <p className="text-sm text-muted-foreground">Đang quét…</p>;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <select aria-label="Trạng thái" className={SELECT_CLASS} value={status} onChange={(e) => setStatus(e.target.value as StatusFilter)}>
          <option value="open">Chưa xét + cần xử lý</option>
          <option value="pending">Chưa xét</option>
          <option value="action">Cần xử lý</option>
          <option value="ok">Không sao</option>
          <option value="all">Tất cả</option>
        </select>
        <select aria-label="Nhóm" className={SELECT_CLASS} value={group} onChange={(e) => setGroup(e.target.value)}>
          <option value="all">Mọi nhóm</option>
          {Object.entries(VN_GROUP_LABELS).map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
        <select aria-label="Mức" className={SELECT_CLASS} value={level} onChange={(e) => setLevel(e.target.value)}>
          <option value="all">Mọi mức</option>
          <option value="mietthi">Miệt thị</option>
          <option value="canxem">Cần xem</option>
        </select>
        <select aria-label="Nhãn AI" className={SELECT_CLASS} value={aiLabel} onChange={(e) => setAiLabel(e.target.value)}>
          <option value="all">Mọi nhãn AI</option>
          <option value="none">Chưa xét AI</option>
          {Object.entries(VN_LABEL_TEXT).map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
        <select aria-label="Xếp theo" className={SELECT_CLASS} value={order} onChange={(e) => setOrder(e.target.value as "chapter" | "entry")}>
          <option value="chapter">Theo chương</option>
          <option value="entry">Theo mục từ điển</option>
        </select>
        <span className="text-xs text-muted-foreground">
          {shown.length} / {hits.length} chỗ
        </span>
        <div className="flex-1" />
        <Button type="button" size="sm" variant="ghost" onClick={() => void vnRefresh(root).then(reload)}>
          <RefreshCw /> Quét lại
        </Button>
        <Button type="button" size="sm" variant="secondary" disabled={busy || selected.length === 0} onClick={() => void verify()}>
          <Sparkles /> AI xét {selected.length} chỗ
        </Button>
        <Button type="button" size="sm" disabled={running || selected.length === 0} onClick={() => setReplaceOpen(true)}>
          <Wrench /> Thay thế…
        </Button>
      </div>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          aria-label="Chọn tất cả đang lọc"
          checked={allShownSelected}
          onChange={(e) => {
            const ids = shown.map((hit) => hit.id);
            setSelected((list) => (e.target.checked ? [...new Set([...list, ...ids])] : list.filter((id) => !ids.includes(id))));
          }}
        />
        Chọn tất cả đang lọc
      </label>
      {shown.length === 0 && <p className="py-6 text-center text-sm text-muted-foreground">Không có chỗ nào.</p>}
      <ul className="flex flex-col gap-2">
        {shown.map((hit, index) => {
          const open = expanded.includes(hit.id);
          const range = (list: string[]) => (open ? list : list.slice(hit.focus, hit.focus + 1));
          const header = order === "entry" && (index === 0 || shown[index - 1].entry !== hit.entry);
          return (
            <li key={hit.id} className="rounded-lg border bg-card p-3">
              {header && (
                <p data-testid="vn-entry-header" className="mb-2 font-mono text-xs font-semibold text-muted-foreground">
                  {hit.entry}
                </p>
              )}
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  aria-label={`Chọn ${hit.id}`}
                  checked={selected.includes(hit.id)}
                  onChange={(e) => setSelected((s) => (e.target.checked ? [...s, hit.id] : s.filter((id) => id !== hit.id)))}
                />
                <span data-testid="vn-hit-chapter" className="font-mono">
                  {hit.chapter}
                </span>
                <span className="text-xs text-muted-foreground">
                  đoạn {hit.paragraph} · {VN_GROUP_LABELS[hit.group]} · {hit.side === "raw" ? "raw (chưa dịch)" : "bản dịch"}
                  {hit.count > 1 ? ` · ×${hit.count}` : ""}
                </span>
                {hit.level === "mietthi" && (
                  <span className="flex items-center gap-1 rounded-full bg-destructive/15 px-2 text-xs text-destructive">
                    <AlertTriangle className="size-3" /> Miệt thị
                  </span>
                )}
                {hit.ai && (
                  <span className={cn("rounded-full px-2 text-xs", hit.ai.label === "xucpham" ? "bg-destructive/15 text-destructive" : "bg-muted")}>
                    AI: {VN_LABEL_TEXT[hit.ai.label]}
                  </span>
                )}
                <div className="flex-1" />
                <Button
                  type="button"
                  size="xs"
                  variant="link"
                  onClick={() => {
                    select(hit.chapter);
                    setPage("translate");
                  }}
                >
                  Mở chương
                </Button>
                <Button type="button" size="xs" variant={hit.status === "ok" ? "secondary" : "ghost"} onClick={() => void mark([hit.id], "ok")}>
                  <Check /> Không sao
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant={hit.status === "action" ? "destructive" : "ghost"}
                  onClick={() => void mark([hit.id], "action")}
                >
                  Cần xử lý
                </Button>
              </div>
              {hit.ai && <p className="mt-1 text-xs text-muted-foreground">{hit.ai.reason}</p>}
              <div className="mt-2 grid gap-2 text-sm md:grid-cols-2">
                <div className="flex flex-col gap-1 font-serif">
                  {range(hit.raw).map((text, i) => (
                    <p key={i}>
                      <Highlight text={text} matched={hit.side === "raw" ? hit.matched : ""} />
                    </p>
                  ))}
                </div>
                <div className="flex flex-col gap-1">
                  {range(hit.out).map((text, i) => (
                    <p key={i}>
                      <Highlight text={text} matched={hit.side === "out" ? hit.matched : ""} />
                    </p>
                  ))}
                </div>
              </div>
              <Button
                type="button"
                size="xs"
                variant="link"
                onClick={() => setExpanded((list) => (open ? list.filter((id) => id !== hit.id) : [...list, hit.id]))}
              >
                {open ? "Thu gọn" : "Xem đoạn trước/sau"}
              </Button>
            </li>
          );
        })}
      </ul>
      <VnReplaceDialog
        root={root}
        setting={setting}
        hits={hits.filter((hit) => selected.includes(hit.id))}
        allHits={hits}
        open={replaceOpen}
        onOpenChange={setReplaceOpen}
        onDone={() => {
          setSelected([]);
          void reload();
        }}
      />
    </div>
  );
}
```

Ghi chú cho người làm: `VnReplaceDialog` có ở Task 9. Để Task 8 biên dịch độc lập, tạo trước `vn-replace-dialog.tsx` tối thiểu:

```tsx
import type { GenreSetting, VnHitView } from "@/lib/types";

export function VnReplaceDialog(_props: {
  root: string;
  setting: GenreSetting;
  hits: VnHitView[];
  allHits: VnHitView[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDone: () => void;
}) {
  return null;
}
```

`story-page.tsx`: thêm `{ id: "vn", label: "Kiểm tra VN" }` vào `TABS` sau `"titles"`; import `VnCheckPanel`; thêm Section:

```tsx
              <Section id="vn" active={active} title="Kiểm tra nội dung nhạy cảm về Việt Nam">
                <VnCheckPanel root={currentRoot} running={running} setting={currentStory.genre.setting} />
              </Section>
```

và thêm vào mock `@/lib/api` của `story-page.test.tsx` các khoá `vnRefresh: vi.fn(() => Promise.resolve({ hits: 0, open: 0, scanned: 0 })), vnReport: vi.fn(() => Promise.resolve([])), vnSetStatus: vi.fn(), vnAiVerify: vi.fn(), vnReplace: vi.fn(), vnSettingsGet: vi.fn()`.

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `cd apps/qt-ai-gui && npx vitest run 2>&1 | grep -E "×|Test Files|Tests" && npm run -s typecheck && npx eslint src/components/vn-check-panel.tsx src/components/vn-replace-dialog.tsx src/components/pages/story-page.tsx`
Expected: mọi test qua, không lỗi typecheck/eslint.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/components/vn-check-panel.tsx apps/qt-ai-gui/src/components/vn-check-panel.test.tsx apps/qt-ai-gui/src/components/vn-replace-dialog.tsx apps/qt-ai-gui/src/components/pages/story-page.tsx apps/qt-ai-gui/src/components/pages/story-page.test.tsx
git commit -m "feat(qt-ai-gui): tab Kiểm tra VN — lọc, ngữ cảnh, Không sao / Cần xử lý, AI xét"
```

---

### Task 9: Hộp thoại thay thế + huy hiệu ở màn chọn truyện

**Files:**
- Modify: `apps/qt-ai-gui/src/components/vn-replace-dialog.tsx` (thay bản tối thiểu)
- Create: `apps/qt-ai-gui/src/components/vn-replace-dialog.test.tsx`
- Modify: `apps/qt-ai-gui/src/components/story-picker.tsx` (huy hiệu ⚠)
- Test: `apps/qt-ai-gui/src/components/story-picker.test.tsx`

**Interfaces:**
- Consumes: `vnReplace`, `vnSettingsGet` (Task 7); `VnHitView`; `resolveChapterRef(rows, text)` (`lib/chapters.ts`, trả chỉ số trong `rows` hoặc -1); `useStoryStore((s) => s.snapshot?.chapters)`.
- Produces: `VnReplaceDialog({ root, setting, hits, allHits, open, onOpenChange, onDone })`.

- [ ] **Step 1: Viết test thất bại** — `vn-replace-dialog.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnReplaceDialog } from "@/components/vn-replace-dialog";
import { vnReplace, vnSettingsGet } from "@/lib/api";
import type { StorySnapshot, VnHitView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ vnReplace: vi.fn(), vnSettingsGet: vi.fn() }));

const base = { group: "ten", level: "canxem", side: "out", count: 1, fingerprint: "x", status: "pending", ai: null, raw: [], out: [], focus: 0, entry: "viet-nam", matched: "Việt Nam" } as const;
const a: VnHitView = { ...base, id: "0001:1:viet-nam", chapter: "0001", paragraph: 1 };
const b: VnHitView = { ...base, id: "0002:3:viet-nam", chapter: "0002", paragraph: 3 };
const c: VnHitView = { ...base, id: "0009:2:viet-nam", chapter: "0009", paragraph: 2 };

function renderDialog(hits: VnHitView[]) {
  const onDone = vi.fn();
  render(<VnReplaceDialog root="D:\\t" setting="modern" hits={hits} allHits={[a, b, c]} open onOpenChange={vi.fn()} onDone={onDone} />);
  return onDone;
}

describe("VnReplaceDialog", () => {
  beforeEach(() => {
    vi.mocked(vnSettingsGet).mockReset().mockResolvedValue({ extra: [], disabled: [], replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }] });
    vi.mocked(vnReplace).mockReset().mockResolvedValue({ chapters: 2, occurrences: 2 });
  });

  it("điền sẵn cụm gốc + cụm thay theo bảng; phạm vi 'chỗ đang chọn' gửi đúng đoạn", async () => {
    const user = userEvent.setup();
    const onDone = renderDialog([a, b]);
    expect(await screen.findByDisplayValue("Thái Lan")).toBeInTheDocument();
    expect(screen.getByLabelText("Cụm gốc")).toHaveValue("Việt Nam");
    expect(screen.getByText(/2 chỗ ở 2 chương/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", {
      from: "Việt Nam",
      to: "Thái Lan",
      chapters: ["0001", "0002"],
      paragraphs: [["0001", 1], ["0002", 3]],
    });
    expect(onDone).toHaveBeenCalled();
  });

  it("phạm vi 'mọi chương có chỗ khớp' không gửi danh sách đoạn", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Mọi chương có chỗ khớp/ }));
    expect(screen.getByText(/3 chỗ ở 3 chương/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", { from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0002", "0009"] });
  });

  it("phạm vi 'chọn chương' dùng các chương được tick", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    await user.click(screen.getByRole("checkbox", { name: "0009" }));
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", { from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0009"] });
  });

  it("phạm vi 'chọn chương': tick theo khoảng (số thứ tự hoặc mã chương)", async () => {
    const user = userEvent.setup();
    useStoryStore.setState({ snapshot: { chapters: ["0001", "0002", "0005", "0009"].map((id) => ({ id })) } as unknown as StorySnapshot });
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    await user.type(screen.getByLabelText("Từ chương"), "2");
    await user.type(screen.getByLabelText("Tới chương"), "0009");
    await user.click(screen.getByRole("button", { name: "Tick khoảng" }));
    expect(screen.getByRole("checkbox", { name: "0001" })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name: "0002" })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "0009" })).toBeChecked();
  });
});
```

Thêm vào `story-picker.test.tsx` một test (theo cách các test sẵn có dựng `summaries`): truyện có `vnFlags: 3` thì có phần tử `getByTitle("3 chỗ nhạy cảm về Việt Nam chưa xác minh")`; `vnFlags: 0` hoặc `null` thì không có.

- [ ] **Step 2: Chạy test, xác nhận thất bại**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/vn-replace-dialog.test.tsx src/components/story-picker.test.tsx 2>&1 | grep -E "×|Tests"`
Expected: các test mới FAIL.

- [ ] **Step 3: Viết hộp thoại + huy hiệu**

`vn-replace-dialog.tsx`:

```tsx
import { Wrench } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { vnReplace, vnSettingsGet } from "@/lib/api";
import { resolveChapterRef } from "@/lib/chapters";
import type { GenreSetting, VnHitView, VnReplacement } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  setting: GenreSetting;
  /** Chỗ đang chọn ở tab Kiểm tra VN. */
  hits: VnHitView[];
  /** Mọi chỗ khớp của truyện (để tính phạm vi theo chương). */
  allHits: VnHitView[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDone: () => void;
}

type Scope = "selected" | "chapters" | "all";

function suggestion(replacements: VnReplacement[], from: string, setting: GenreSetting): string {
  const wanted = setting === "ancient" ? "ancient" : "modern";
  return (
    replacements.find((r) => r.from === from && (r.setting === wanted || r.setting === "any"))?.to ??
    replacements.find((r) => r.from === from)?.to ??
    ""
  );
}

/** Thay cụm nhạy cảm trong bản dịch: chỗ đang chọn, các chương tick, hoặc mọi chương có chỗ khớp. Có hoàn tác ở trang Tìm & thay. */
export function VnReplaceDialog({ root, setting, hits, allHits, open, onOpenChange, onDone }: Props) {
  const translated = hits.filter((hit) => hit.side === "out");
  const initialFrom = translated[0]?.matched ?? "";
  const [from, setFrom] = useState(initialFrom);
  const [to, setTo] = useState("");
  const [scope, setScope] = useState<Scope>("selected");
  const [ticked, setTicked] = useState<string[]>([]);
  const [rangeFrom, setRangeFrom] = useState("");
  const [rangeTo, setRangeTo] = useState("");
  const [busy, setBusy] = useState(false);
  const rows = useStoryStore((s) => s.snapshot?.chapters);

  useEffect(() => {
    if (!open) return;
    setFrom(initialFrom);
    setScope("selected");
    setTicked([...new Set(translated.map((hit) => hit.chapter))]);
    void vnSettingsGet().then((settings) => setTo(suggestion(settings.replacements, initialFrom, setting)));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ nạp lại khi mở hộp thoại
  }, [open]);

  const sameTerm = useMemo(() => allHits.filter((hit) => hit.side === "out" && hit.matched === from), [allHits, from]);
  const chaptersWithTerm = useMemo(() => [...new Set(sameTerm.map((hit) => hit.chapter))], [sameTerm]);
  const target =
    scope === "selected"
      ? translated.filter((hit) => hit.matched === from)
      : scope === "chapters"
        ? sameTerm.filter((hit) => ticked.includes(hit.chapter))
        : sameTerm;
  const targetChapters = [...new Set(target.map((hit) => hit.chapter))];

  /** Tick các chương có chỗ khớp nằm trong khoảng; ô trống = từ đầu / tới cuối. Nhận số thứ tự (#12) hoặc mã chương. */
  function tickRange() {
    const list = rows ?? [];
    const position = new Map(list.map((row, index) => [row.id, index]));
    const lo = rangeFrom.trim() ? resolveChapterRef(list, rangeFrom) : 0;
    const hi = rangeTo.trim() ? resolveChapterRef(list, rangeTo) : list.length - 1;
    if (lo < 0 || hi < 0 || lo > hi) {
      toast.error("Khoảng chương không hợp lệ");
      return;
    }
    setTicked(chaptersWithTerm.filter((id) => {
      const at = position.get(id);
      return at !== undefined && at >= lo && at <= hi;
    }));
  }

  async function run() {
    setBusy(true);
    try {
      const request =
        scope === "selected"
          ? { from, to, chapters: targetChapters, paragraphs: target.map((hit) => [hit.chapter, hit.paragraph] as [string, number]) }
          : { from, to, chapters: targetChapters };
      const outcome = await vnReplace(root, request);
      toast.success(`Đã thay ${outcome.occurrences} chỗ ở ${outcome.chapters} chương — hoàn tác ở trang Tìm & thay`);
      onOpenChange(false);
      onDone();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không thay được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Thay cụm nhạy cảm</DialogTitle>
          <DialogDescription>
            Chỉ thay trong bản dịch (out/), nguyên từ, phân biệt hoa thường. Hoàn tác lần thay gần nhất ở trang Tìm &amp; thay.
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="vn-from">Cụm gốc</Label>
            <Input id="vn-from" value={from} onChange={(e) => setFrom(e.target.value)} />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="vn-to">Cụm thay</Label>
            <Input id="vn-to" value={to} onChange={(e) => setTo(e.target.value)} />
          </div>
        </div>
        <fieldset className="flex flex-col gap-1 text-sm">
          <label className="flex items-center gap-2">
            <input type="radio" name="vn-scope" checked={scope === "selected"} onChange={() => setScope("selected")} />
            Chỉ các chỗ đang chọn
          </label>
          <label className="flex items-center gap-2">
            <input type="radio" name="vn-scope" checked={scope === "chapters"} onChange={() => setScope("chapters")} />
            Chọn chương
          </label>
          {scope === "chapters" && (
            <div className="ml-6 flex items-center gap-2">
              <Input aria-label="Từ chương" placeholder="Từ (#1 hoặc mã)" className="h-8 w-32" value={rangeFrom} onChange={(e) => setRangeFrom(e.target.value)} />
              <Input aria-label="Tới chương" placeholder="Tới" className="h-8 w-32" value={rangeTo} onChange={(e) => setRangeTo(e.target.value)} />
              <Button type="button" size="sm" variant="outline" onClick={tickRange}>
                Tick khoảng
              </Button>
            </div>
          )}
          {scope === "chapters" && (
            <div className="ml-6 flex max-h-32 flex-wrap gap-2 overflow-auto">
              {chaptersWithTerm.map((chapter) => (
                <label key={chapter} className="flex items-center gap-1 font-mono text-xs">
                  <input
                    type="checkbox"
                    aria-label={chapter}
                    checked={ticked.includes(chapter)}
                    onChange={(e) => setTicked((list) => (e.target.checked ? [...list, chapter] : list.filter((c) => c !== chapter)))}
                  />
                  {chapter}
                </label>
              ))}
            </div>
          )}
          <label className="flex items-center gap-2">
            <input type="radio" name="vn-scope" checked={scope === "all"} onChange={() => setScope("all")} />
            Mọi chương có chỗ khớp
          </label>
        </fieldset>
        <p className="rounded-md bg-muted p-3 text-sm">
          Sẽ thay {target.length} chỗ ở {targetChapters.length} chương.
        </p>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button disabled={busy || !from.trim() || !to.trim() || target.length === 0} onClick={() => void run()}>
            <Wrench /> Thay
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

`story-picker.tsx`: trong `p` chứa tên truyện (sau `<span className="truncate">{item.name}</span>`), thêm:

```tsx
                {(item.vnFlags ?? 0) > 0 && (
                  <span
                    title={`${item.vnFlags} chỗ nhạy cảm về Việt Nam chưa xác minh`}
                    className="shrink-0 rounded-full bg-destructive/15 px-2 py-0.5 text-[11px] font-medium text-destructive"
                  >
                    ⚠ VN {item.vnFlags}
                  </span>
                )}
```

- [ ] **Step 4: Chạy test, xác nhận qua**

Run: `cd apps/qt-ai-gui && npx vitest run 2>&1 | grep -E "×|Test Files|Tests" && npm run -s typecheck && npm run -s lint`
Expected: mọi test qua; typecheck, lint sạch.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src/components/vn-replace-dialog.tsx apps/qt-ai-gui/src/components/vn-replace-dialog.test.tsx apps/qt-ai-gui/src/components/story-picker.tsx apps/qt-ai-gui/src/components/story-picker.test.tsx
git commit -m "feat(qt-ai-gui): thay cụm nhạy cảm VN theo chỗ / chương / tất cả; huy hiệu ở màn chọn truyện"
```

---

### Task 10: Kiểm chứng thật trên thư viện

**Files:** không sửa code (chỉ đọc kết quả). Nếu phát hiện báo nhầm mới, thêm mẫu loại trừ vào `builtin_entries()` kèm test ở `tests/vn_check.rs` rồi commit riêng.

- [ ] **Step 1: Build CLI**

Run: `CARGO_INCREMENTAL=0 cargo build -q -p qt-ai-core --bin qt-ai`
Expected: build xong, không lỗi.

- [ ] **Step 2: Quét cả thư viện trên bản sao (không đụng thư viện thật — lệnh ghi `vn-check.json`)**

```bash
SP=/tmp/claude-1000/-home-hungnp-projects-quick-translator-engine/5a862baf-85e9-46ab-b6b3-00c27b3f3098/scratchpad; rm -rf $SP/vnlib && mkdir -p $SP/vnlib
cd /home/hungnp/projects/bookshelf/books/common
for s in */; do [ -d "$s/raw" ] && mkdir -p "$SP/vnlib/$s" && cp -r "$s/raw" "$s/out" "$s/state.json" "$SP/vnlib/$s" 2>/dev/null; done
time (for s in $SP/vnlib/*/; do /home/hungnp/projects/quick-translator-engine/target/debug/qt-ai vn-check "$s" | head -1; done)
```

Expected: mỗi truyện một dòng "Kiểm tra VN: … ms"; tổng thời gian cả thư viện ≤ 10 giây (bản debug có thể chậm hơn, đo lại bằng `--release` nếu vượt).

- [ ] **Step 3: Chạy lại lần hai, xác nhận quét tăng dần**

Run: lặp lại vòng `for` ở Step 2.
Expected: mọi dòng "quét 0 chương", mỗi truyện ≤ 100 ms.

- [ ] **Step 4: Đối chiếu các chỗ thật đã biết**

```bash
/home/hungnp/projects/quick-translator-engine/target/debug/qt-ai vn-check $SP/vnlib/truong-sinh-bat-dau-tu-thien-long/
/home/hungnp/projects/quick-translator-engine/target/debug/qt-ai vn-check $SP/vnlib/huyen-giam-tien-toc/
/home/hungnp/projects/quick-translator-engine/target/debug/qt-ai vn-check $SP/vnlib/thau-huong-cao-thu/
/home/hungnp/projects/quick-translator-engine/target/debug/qt-ai vn-check $SP/vnlib/xuyen-qua-chu-thien-ta-chon-duong-huong-ma-nu/
```

Expected: Thiên Long có `0226…:khi-viet` và `0354…:giao-chi`; Huyền giám có `giao-chi` (交趾人) và `an-nam`; Thâu hương có `viet-nam` (thủ đô Hà Nội của Việt Nam); Xuyên qua chư thiên có `giao-chi` (đất cũ người Hán như Giao Chỉ). Không có "Hoàng Sa" của Thâu hương (Hoàng Sam), không có "Nam Hải" / "Biển Đông" của arc One Piece ở Thiên Long.

- [ ] **Step 5: Thử AI xét trong app (thủ công)**

Mở app dev (`cd apps/qt-ai-gui && npm run tauri dev`), mở Trường sinh bắt đầu từ Thiên Long → tab Kiểm tra VN → chọn hai chỗ trên → "AI xét". Expected: `khi-viet` nhãn "Xúc phạm", `giao-chi` nhãn "Nhạy cảm" hoặc "Xúc phạm", có lý do một câu. Ghi lại kết quả vào mô tả commit/PR.
