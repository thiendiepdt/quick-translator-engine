# Dictionaries — Format, load order, priority, cache

Nguồn: `LoadDictionaries`, `load*Dictionary`, `DictionaryConfigurationHelper` (decompile).

## 1. Cấu hình đường dẫn — `Dictionaries.config`

File text `key=path` (một dòng mỗi từ điển). Đường dẫn tương đối thư mục app:

```
Names=Names.txt
NamesPhu=Names2\123.txt
VietPhrase=VietPhrase\VietPhrase.txt
ChinesePhienAmWords=Resources\ChinesePhienAmWords.txt
DanhTu=Resources\DanhTu.txt
HoNguoi=Resources\HoNguoi.txt
HauTu=Resources\HauTu.txt
LacViet=Resources\LacViet.txt
IgnoredChinesePhrases=IgnoredChinesePhrases.txt
LuatNhan=LuatNhan.txt
Pronouns=Resources\Pronouns.txt
```

> Rust: đọc config này, cho phép override thư mục gốc qua `--data-dir`. Dùng `\` trên Windows;
> khi port sang Linux phải chuẩn hoá separator.

## 2. Format chung của mỗi dòng

- Encoding: **UTF-8, có BOM** (`detectEncodingFromByteOrderMarks: true`). Dòng đầu có `﻿`.
- Mỗi dòng: `key=value`. **Split bằng ký tự `=` đầu tiên**, `array.Length == 2` mới nhận
  → nếu `value` chứa `=`, split thành >2 phần và **dòng bị bỏ** (trừ LuatNhan cũng vậy).
  (Ghi chú tương thích: nên tôn trọng đúng hành vi này để "y hệt".)
- Dòng trùng key: **chỉ dòng ĐẦU tiên được giữ** (`!ContainsKey` guard). Dòng sau bị bỏ.
- LuatNhan: bỏ qua dòng bắt đầu bằng `#` (comment).

## 3. Các từ điển & cấu trúc nội bộ

| File | Biến engine | Value | Ghi chú |
|---|---|---|---|
| Names.txt | `onlyNameChinhDictionary`, gộp vào `onlyNameDictionary` | `Tên/nghĩa2` | Tên riêng "chính" |
| Names2/123.txt | `onlyNamePhuDictionary`, ghi đè vào `onlyNameDictionary` | | Tên "phụ", **ghi đè** name chính nếu trùng key (dùng `[key]=` không guard) |
| VietPhrase.txt | `onlyVietPhraseDictionary` | `nghĩa1/nghĩa2/...` | Từ điển nghĩa chính |
| ChinesePhienAmWords.txt | `hanVietDictionary` | 1 âm Hán Việt | 1 ký tự Hán → âm; định nghĩa `isChinese` |
| LacViet.txt | `lacVietDictionary` | định nghĩa nhiều dòng | Tra cứu chi tiết ([meanings-lacviet.md](meanings-lacviet.md)) |
| LuatNhan.txt | `luatNhanDictionary` | mẫu → khuôn dịch | Luật văn phạm ([luat-nhan.md](luat-nhan.md)) |
| Pronouns.txt | `pronounDictionary` | đại từ | Dùng dựng `dictionaryN` cho `{n}` |
| DanhTu.txt | `danhTuDictionary` | danh từ | |
| HoNguoi.txt | `hoNguoiDictionary` | họ người | Dựng `hoHauTuCache` |
| HauTu.txt | `hauTuDictionary` | hậu tố | Dựng `hoHauTuCache` |
| IgnoredChinesePhrases.txt | `ignoredChinesePhraseList` | (danh sách) | Cụm bị xoá trước khi dịch (mode analyzer) |

## 4. <a name="merge"></a>Thứ tự nạp & dựng từ điển tổng hợp

Trong `LoadDictionaries` (song song hoá bằng `Task.WhenAll`, nhưng thứ tự **logic** là):

```
1. loadOnlyNameDictionary():
     - Đọc Names.txt   → onlyNameChinhDictionary[key]=value ; onlyNameDictionary[key]=value
                         onlyNameOneMeaningDictionary[key] = value.Split('/','|')[0]
     - Đọc Names2      → onlyNamePhuDictionary[key]=value ; onlyNameDictionary[key]=value (GHI ĐÈ)
                         onlyNameOneMeaningDictionary[key] = value.Split('/','|')[0] (ghi đè)
2. loadOnlyVietPhraseDictionary()  → onlyVietPhraseDictionary  (từ VietPhrase.txt)
3. loadVietPhraseDictionary():                       # dựng vietPhraseDictionary
     - copy toàn bộ onlyNameDictionary        (ưu tiên: thêm trước)
     - copy onlyVietPhraseDictionary nếu key CHƯA có (guard !ContainsKey)
     → HỆ QUẢ: key trùng → Name thắng VietPhrase
4. vPDictToVPOneMeaningDict():                        # dựng vietPhraseOneMeaningDictionary
     - với mỗi (key,value) trong vietPhraseDictionary:
         value chứa '/' hoặc '|'  →  lấy phần đầu (Split('/','|')[0])
         ngược lại                →  giữ nguyên
5. UpdateDictionaryN():        dictionaryN = pronounDictionary ∪ onlyNameOneMeaningDictionary (guard)
6. UpdateLuatNhanListsAndCaches():   tách luatNhanNList / luatNhanSList + compile regex cache
7. BuildHoHauTuCache():        hoHauTuCache = { ho+hauTu : mọi cặp }
```

### Điểm mấu chốt "y hệt"
- **Name ưu tiên hơn VietPhrase** với cùng key (bước 3).
- **Names2 ghi đè Names** với cùng key (bước 1, dùng indexer không guard).
- `vietPhraseOneMeaningDictionary` = lấy nghĩa đầu tiên; **tách bằng cả `/` và `|`**.
- Nhưng `onlyNameOneMeaningDictionary` tách bằng `/` **và** `|` (char set `"/|"`).
  → phân biệt: Names dùng separator `[/|]`; OneMeaning của VietPhrase cũng `Split('/','|')`.

## 5. Value nhiều nghĩa

- Trong mode **VietPhrase**: giữ nguyên `nghĩa1/nghĩa2` — engine không tự chọn, xuất đúng value.
  (UI có thể hiện popup chọn; CLI mặc định xuất cả chuỗi value như từ điển.)
- Trong mode **VietPhraseOneMeaning**: chỉ `nghĩa1` (phần trước dấu `/` hoặc `|` đầu tiên).

> Kiểm chứng bằng golden test: đưa cụm có value dạng `a/b/c` qua cả 2 mode, so sánh.

## 6. Cache phụ (dựng sau khi load)

| Cache | Nội dung | Dùng cho |
|---|---|---|
| `vietPhraseDictionary` | Name ∪ VietPhrase | mode VietPhrase |
| `vietPhraseOneMeaningDictionary` | như trên, 1 nghĩa | mode OneMeaning |
| `dictionaryN` | Pronouns ∪ Name(1 nghĩa) | thay `{n}` trong LuatNhan |
| `luatNhanNList` / `luatNhanSList` | tách luật theo `{n}` / `{s}` | matching |
| `luatNhanNCache` / `luatNhanSCache` | `Regex` đã compile | matching nhanh |
| `hoHauTuCache` | tập `họ+hậu_tố` | luật `{h}{t}` dựng tên |

## 7. Reimplement Rust — gợi ý cấu trúc

- Dùng `HashMap<String, String>` (hoặc `FxHashMap` cho tốc độ) cho từng từ điển.
- VietPhrase 763k entry / 28MB: cân nhắc load 1 lần, giữ trong `Arc<Dictionaries>` để server
  chia sẻ giữa request. Thời gian load chấp nhận được nếu đọc tuần tự + parse `split('=')`.
- Giữ **đúng thứ tự merge** ở mục 4 — đây là nơi dễ lệch kết quả nhất.
- BOM: strip `﻿` ở ký tự đầu file.
