# QT-CLI — Kiến trúc & Thiết kế hệ thống

> Tài liệu thiết kế đã được duyệt (brainstorm). Đây là bản đồ gốc cho toàn bộ project.
> Ngày: 2026-07-21.

## 1. Mục tiêu

Tái tạo **Quick Translator (QT2025)** — công cụ dịch Trung → Việt theo từ điển longest-match —
dưới dạng:

1. **CLI** (`qt`): đọc `stdin`, dịch và xuất `stdout` (dịch file/batch là bước mở rộng).
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
├── reference/decompiled/        # C# decompile của TranslatorEngine
└── QT2025/                      # Từ điển + config gốc dùng để chạy/smoke test
```

### Ranh giới module (isolation)

- **`qt-core`** là thư viện thuần: `Engine::from_dicts(dictionaries) -> Engine`, rồi
  `engine.translate(text, mode, options) -> String` hoặc
  `engine.translate_with_ranges(...) -> TranslationResult`. Không phụ thuộc HTTP/CLI.
  Toàn bộ độ chính xác "y hệt" nằm ở đây; source-level regression test đã có, còn bộ golden
  output lấy trực tiếp từ app QT thật chưa có.
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

## 6. API surface hiện tại

**CLI** (`qt`):
```
qt translate --mode vietphrase < input.txt > output.txt
echo "他很厉害" | qt translate --mode hanviet
qt translate --mode vietphrase-one --wrap < input.txt
```

**HTTP** (`qt-server`):
```
POST /translate        { "text": "...", "mode": "vietphrase", "wrap": false, "pretty": false }
                       -> { "translated": "..." }
                       Thêm `"ranges": true` để nhận `sourceRanges` + `targetRanges`.
POST /translate/batch  { "texts": ["..."], "mode": "vietphrase" }
                       -> { "translated": ["..."] }
GET  /modes
GET  /health
```
Server nạp thư mục `QT_DATA_DIR` một lần khi khởi động (mặc định `data`; khi chạy repo dùng
`QT2025`). Mỗi range có dạng `{ "start": n, "length": n }`; offset/length dùng đơn vị
UTF-16 như QT2025/.NET và JavaScript. Với batch, hai field range là ma trận song song với
`texts`. `/meanings` vẫn thuộc giai đoạn sau.

## 7. Chiến lược verify "y hệt"

- **Nguồn chân lý**: code decompile từ `TranslatorEngine.dll`
  (`reference/decompiled/TranslatorEngine.decompiled.cs`) và config/từ điển trong `QT2025/`.
- **Golden test (chưa có)**: sẽ chứa cặp `input.zh → expected.txt` lấy từ app QT thật;
  `qt-core` phải khớp **từng ký tự**.
- Ưu tiên verify theo thứ tự: HanViet (đơn giản nhất) → VietPhraseOneMeaning → VietPhrase
  → LuatNhan/số (phức tạp nhất).

## 8. Lộ trình (mỗi bước là 1 spec → plan riêng)

1. **Docs** — đặc tả engine + format dữ liệu. ✅
2. `qt-core` MVP: dict loader + HanViet + VietPhrase + VietPhraseOneMeaning. ✅
3. `qt-cli`: stdin/stdout + `--mode`/`--data-dir`/`--wrap`. ✅
4. `qt-api`: HTTP server. ✅
5. Chuyển số Hán + source↔target ranges. ✅
6. LuatNhan tổng quát + StandardizeInput đầy đủ. ✅
7. Nghĩa/LacViet.
8. (sau) Tauri GUI, quản lý từ điển (thêm/sửa Names, VietPhrase).

## 9. Ngoài phạm vi (YAGNI giai đoạn đầu)

- Charset detection (Mozilla `nsDetector` trong DLL) — input giả định UTF-8.
- History/log từ điển (`*.History`, DataSet) — chỉ cần khi làm tính năng edit dict.
- UI docking, hotkey, font/màu — thuộc GUI, làm sau.
