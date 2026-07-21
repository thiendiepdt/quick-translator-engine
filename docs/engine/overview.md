# Engine Overview — Pipeline & thứ tự ưu tiên

Nguồn: decompile `TranslatorEngine.dll` (class `TranslatorEngine`, `TransLuatNhan`).
Mọi số dòng dưới đây tham chiếu file decompile để đối chiếu khi implement.

## 1. Ba lối vào công khai

```
ChineseToVietPhrase(chinese, wrapType, translationAlgorithm, prioritizedName, scanRange)
    -> TranslateAll(..., dictionary = vietPhraseDictionary,           ...)
ChineseToVietPhraseOneMeaning(...)
    -> TranslateAll(..., dictionary = vietPhraseOneMeaningDictionary,  ...)
ChineseToHanViet(chinese, out mapping[])          # độc lập, không qua TranslateAll
ChineseToMeanings(chinese, out phraseLength)       # tra cứu, không dịch liền mạch
```

Cả 3 mode dịch (VietPhrase, VietPhraseOneMeaning) đi qua **cùng một hàm `TranslateAll`**,
chỉ khác **từ điển** truyền vào. HanViet đi đường riêng.

## 2. Tham số (khớp chữ ký gốc)

| Tham số | Kiểu | Ý nghĩa |
|---|---|---|
| `wrapType` | int | `0` = xuất thường; `1` = bọc mỗi cụm dịch trong `[...]` |
| `translationAlgorithm` | int | `0`/`1`/`2` — điều khiển luật "cụm dài nhất trong câu" (xem [translation-algorithm.md](translation-algorithm.md#longest-phrase)) |
| `prioritizedName` | bool | Ưu tiên tên riêng: bỏ qua cụm VietPhrase nếu nó "nuốt" một Name phía sau |
| `scanRange` | int | Độ dài (số ký tự) tối đa để quét cụm từ mỗi vị trí |

> **Default**: các giá trị này do UI QuickTranslator lưu trong `QuickTranslatorMain.config`
> (binary). Khi implement, expose chúng làm option, default hợp lý:
> `wrapType=0`, `translationAlgorithm=1`, `prioritizedName=true`, `scanRange=5`.
> (Cần chốt lại bằng golden test khi có mẫu output thật.)

## 3. Thứ tự ưu tiên khi dịch một vị trí

Tại mỗi vị trí `num2` trong văn bản, `TranslateAll` thử theo thứ tự:

```
1. Số đã pre-scan tại vị trí này? (PreScanForNumbers)  → ghi nhớ, ưu tiên độ dài số
2. Quét cụm dài nhất num6 = scanRange → 1:
     a. Cụm ∈ dictionary  AND  qua kiểm tra "longest phrase"  AND  qua kiểm tra name-priority
            → ProcessTranslation (dịch cụm)  → nhảy qua num6 ký tự
     b. Cụm ∉ dictionary, len>2, chưa bị chặn → thử LuatNhan (HandleNhanBy)
            → nếu khớp: dịch theo luật, nhảy
3. Nếu vị trí này là số (đã pre-scan) và chưa dịch → áp luật "{s}" cho số
4. Vẫn chưa dịch → ProcessHanViet: phiên âm Hán Việt 1 ký tự, tiến 1 ký tự
```

Bản chất: **cụm dài nhất trong từ điển thắng**; nếu không có cụm nào, thử luật văn phạm;
cuối cùng fallback về Hán Việt từng chữ. Chi tiết từng nhánh:
[translation-algorithm.md](translation-algorithm.md).

## 4. Ưu tiên giữa các từ điển (khi merge)

`vietPhraseDictionary` được dựng bằng cách gộp (hàm `loadVietPhraseDictionary`):

```
1. onlyNameDictionary   (Names.txt + Names2)   ← thêm trước → ƯU TIÊN CAO NHẤT
2. onlyVietPhraseDictionary (VietPhrase.txt)   ← chỉ thêm nếu key chưa tồn tại
```

Nghĩa là **với cùng một key, Names thắng VietPhrase**. Chi tiết & mọi từ điển:
[dictionaries.md](dictionaries.md).

## 5. Sơ đồ khối

```
input (chinese, UTF-8)
      │
      ├─ PreScanForNumbers  ─────────────► map vị trí→số
      ├─ NumberModifier (đổi 余/多 + 百千万亿)
      ▼
  ┌─ while num2 <= end ──────────────────────────────────┐
  │   thử cụm dài→ngắn trong `dictionary`:                │
  │     • khớp + longest + name-ok → ProcessTranslation   │
  │     • không khớp + len>2       → HandleNhanBy (LuậtNhân)│
  │   nếu là số                    → áp luật {s}          │
  │   còn lại                      → ProcessHanViet (1 ký tự)│
  └──────────────────────────────────────────────────────┘
      ▼
  TranslationResult { TranslatedText, ChinesePhraseRanges[], VietPhraseRanges[] }
```

`ChinesePhraseRanges` / `VietPhraseRanges` là ánh xạ vị trí cụm nguồn ↔ cụm dịch,
dùng cho highlight trong UI. Với CLI/API chỉ cần `TranslatedText`, nhưng nên giữ ranges
để tương thích Tauri sau này.
