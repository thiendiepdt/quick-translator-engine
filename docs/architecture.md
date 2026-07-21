# QT-CLI — Kiến trúc & Thiết kế hệ thống

> Tài liệu thiết kế đã được duyệt (brainstorm). Đây là bản đồ gốc cho toàn bộ project.
> Ngày: 2026-07-21.

## 1. Mục tiêu

Tái tạo **Quick Translator (QT2025)** — công cụ dịch Trung → Việt theo từ điển longest-match —
dưới dạng:

1. **CLI** (`qt`): đọc `stdin`, dịch, xuất `stdout`; dịch file; batch.
2. **API** (`qt-server`): HTTP REST server, nạp từ điển 1 lần rồi phục vụ nhiều request,
   để tích hợp vào các tool/server dịch/convert truyện khác.
3. Nền tảng cho **Tauri desktop GUI** về sau (dùng chung engine).

Nguyên tắc tối thượng: **kết quả dịch phải "y hệt" QT2025**. Engine được reimplement bằng
Rust nhưng bám sát từng bước thuật toán trong `TranslatorEngine.dll` (đã decompile — xem
[engine/](engine/)).

## 2. Nguồn gốc — QT2025 là gì

QT2025 là app **.NET WinForms** (Windows):

| Thành phần | Vai trò |
|---|---|
| `QuickTranslator.exe` | UI (WinForms + WeifenLuo docking) |
| `TranslatorEngine.dll` | **Engine dịch** — nguồn chân lý thuật toán |
| `JiebaNet.Segmenter.dll` | Segmentation (dùng cho phần Analyzer, không phải core dịch) |
| `HtmlAgilityPack.dll` | Bóc text từ HTML |
| Bộ từ điển (`data/`) | Names, VietPhrase, LacViet, ChinesePhienAmWords, LuatNhan, ... |

Engine đã được decompile ra C# và phân tích đầy đủ. Tài liệu đặc tả nằm trong [docs/engine/](engine/).

## 3. Kiến trúc repo — Cargo workspace

```
qt-cli/
├── docs/                        # ← DELIVERABLE #1 (tài liệu ánh xạ, viết trước)
│   ├── architecture.md          #   (file này)
│   └── engine/                  #   đặc tả thuật toán "y hệt"
│       ├── overview.md          #     pipeline tổng quát + thứ tự ưu tiên
│       ├── translation-algorithm.md  # main loop TranslateAll
│       ├── dictionaries.md      #     mọi dict: format, load order, priority
│       ├── han-viet.md          #     phiên âm Hán Việt + fallback
│       ├── luat-nhan.md         #     luật văn phạm {n}/{s}/{h}{t}
│       ├── number-conversion.md #     số Hán → Việt
│       └── meanings-lacviet.md  #     tra cứu nghĩa chi tiết
├── crates/
│   ├── qt-core/                 # Engine thuần Rust (no I/O bên ngoài dict loader)
│   ├── qt-cli/                  # binary `qt`
│   └── qt-api/                  # binary `qt-server` (axum)
├── data/                        # Bộ từ điển (copy từ QT2025), có thể override qua config
├── tests/golden/                # Cặp Trung→output QT thật, so từng ký tự
└── QT2025/                      # Source gốc để tham chiếu (không build)
```

### Ranh giới module (isolation)

- **`qt-core`** là thư viện thuần: `Engine::new(dictionaries) -> Engine`, rồi
  `engine.translate(text, mode, options) -> TranslationResult`. Không phụ thuộc HTTP/CLI.
  Toàn bộ độ chính xác "y hệt" nằm ở đây và được phủ bởi golden test.
- **`qt-cli`** và **`qt-api`** là vỏ mỏng: parse input → gọi `qt-core` → format output.
  Không chứa logic dịch.

## 4. Engine pipeline (ánh xạ DLL → module Rust)

| Module Rust (dự kiến) | Ánh xạ trong `TranslatorEngine.dll` | Nhiệm vụ |
|---|---|---|
| `dict` | `LoadDictionaries`, `load*Dictionary`, `vPDictToVPOneMeaningDict` | Nạp & merge từ điển, dựng cache |
| `translate` | `TranslateAll`, `ProcessTranslation`, `ProcessHanViet` | Main loop longest-match |
| `priority` | `isLongestPhraseInSentence`, `containsName` | Quyết định chọn cụm |
| `han_viet` | `ChineseToHanViet`, `ToNarrow` | Phiên âm từng chữ + fallback |
| `luat_nhan` | `containsLuatNhan`, `matchLuatNhan*`, `ChineseToLuatNhanOneMeaning`, `TransLuatNhan` | Luật văn phạm |
| `number` | `ConvertChineseNumberToLong`, `FindLongestNumber`, `NumberModifier`, ... | Số Hán |
| `meanings` | `ChineseToMeanings` | Tra cứu nghĩa (LacViet) |
| `text` | `appendTranslatedWord`, `WrapTranslation`, `nextCharIsChinese` | Ghép chuỗi, viết hoa, wrap |

Chi tiết từng phần: [docs/engine/](engine/).

## 5. Các mode output

QT xuất nhiều "view". MVP tập trung 3 mode mày dùng nhất:

| Mode | Hàm gốc | Mô tả |
|---|---|---|
| **VietPhrase** | `ChineseToVietPhrase` | Dịch nghĩa, mỗi cụm giữ đủ nghĩa (`nghĩa1/nghĩa2`) |
| **VietPhraseOneMeaning** | `ChineseToVietPhraseOneMeaning` | Như trên nhưng mỗi cụm chỉ lấy nghĩa đầu |
| **HanViet** | `ChineseToHanViet` | Phiên âm Hán Việt từng chữ |
| *(sau)* Nghĩa/LacViet | `ChineseToMeanings` | Tra cứu chi tiết, không dịch liền mạch |

Tham số dịch (khớp API gốc): `wrapType` (0=thường, 1=bọc `[...]`),
`translationAlgorithm` (0/1/2 — ảnh hưởng luật "cụm dài nhất"), `prioritizedName` (bool),
`scanRange` (độ dài quét tối đa). Xem [translation-algorithm.md](engine/translation-algorithm.md).

## 6. API surface (dự kiến)

**CLI** (`qt`):
```
qt translate --mode vietphrase < input.txt > output.txt
echo "他很厉害" | qt translate --mode hanviet
qt translate --mode vietphrase-one --wrap file.txt
```

**HTTP** (`qt-server`):
```
POST /translate   { "text": "...", "mode": "vietphrase", "options": {...} }
                  -> { "translated": "...", "ranges": [...] }
POST /meanings    { "text": "..." }        # tra cứu (ChineseToMeanings)
GET  /health
```
Server nạp `data/` một lần khi khởi động (VietPhrase ~28MB, ~763k entry).

## 7. Chiến lược verify "y hệt"

- **Nguồn chân lý**: code decompile từ `TranslatorEngine.dll` (`scratchpad/decompiled/`).
- **Golden test**: `tests/golden/` chứa cặp `input.zh → expected.txt` (output QT thật).
  `qt-core` chạy qua phải khớp **từng ký tự**.
- Ưu tiên verify theo thứ tự: HanViet (đơn giản nhất) → VietPhraseOneMeaning → VietPhrase
  → LuatNhan/số (phức tạp nhất).

## 8. Lộ trình (mỗi bước là 1 spec → plan riêng)

1. **Docs** (bước hiện tại) — đặc tả engine + format dữ liệu. ✅ ưu tiên
2. `qt-core` MVP: dict loader + HanViet + VietPhrase + VietPhraseOneMeaning.
3. `qt-cli`: stdin/stdout, file, `--mode`.
4. `qt-api`: HTTP server.
5. LuatNhan + số Hán (tăng độ khớp) → Nghĩa/LacViet.
6. (sau) Tauri GUI, quản lý từ điển (thêm/sửa Names, VietPhrase).

## 9. Ngoài phạm vi (YAGNI giai đoạn đầu)

- Charset detection (Mozilla `nsDetector` trong DLL) — input giả định UTF-8.
- History/log từ điển (`*.History`, DataSet) — chỉ cần khi làm tính năng edit dict.
- UI docking, hotkey, font/màu — thuộc GUI, làm sau.
