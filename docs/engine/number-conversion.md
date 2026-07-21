# Number Conversion — Số Hán → Việt

Nguồn: class `TransLuatNhan` (decompile ~dòng 465–953) + `NumberModifier`, `PreScanForNumbers`.

## 1. Bảng ký tự

```
chineseNumberMap:  零/〇=0  一=1  二/两=2  三=3  四=4  五=5  六=6  七=7  八=8  九=9
chineseUnitMapNone: 十=10  百=100  千=1000  万=10000  亿=100000000
NumberChars: tập ký tự có thể mở đầu một số (chữ số Hán + Latin)
```
`NumberToVietnameseText` dùng đơn vị Việt: **vạn** (10⁴), **ức** (10⁸). Số < 10⁴ format `N0` (en-US, có dấu phẩy nghìn).

## 2. <a name="prescan"></a>`PreScanForNumbers`

Quét toàn chuỗi, tại mỗi vị trí có ký tự số gọi `FindLongestNumber`; nếu ra số và **không**
trùng key trong `onlyVietPhraseDictionary` thì ghi `NumberInfo{StartIndex, Length, Text, Type}`
rồi nhảy qua `matchedLength`. Kết quả map `vị trí → NumberInfo` dùng trong `TranslateAll`.

## 3. <a name="numbermodifier"></a>`NumberModifier`

Chuẩn hoá "hơn/dư" trước khi dịch: nếu gặp `余` hoặc `多` mà **ngay sau** là `百/千/万/亿`
thì **hoán đổi** hai ký tự (đưa đơn vị lên trước):
```
"...余万..." → "...万余..."      ("vạn dư/hơn vạn")
```
Áp cho cả input `TranslateAll` và `ChineseToMeanings`.

## 4. `FindLongestNumber` — nhận diện & phân loại số

Regex: `(?:(?:\d+(?:[.,]\d+)?|[零一二三四五六七八九十百千万亿两〇]+)[ \t]*)+`
(match từ `startIndex`, phải khớp đúng tại đó). Phân loại `NumberType`:

| Type | Điều kiện |
|---|---|
| `PureLatin` | chỉ chữ số Latin, không `.`/`,`, không bắt đầu `0` |
| `LatinDecimal` | Latin có `.`/`,` hoặc bắt đầu `0` |
| `PureChinese` | chỉ chữ số Hán |
| `Mixed` | có cả Latin và Hán |
| `NotANumber` | không khớp |

- Nếu match chỉ dài 1 và là `百/千/万/亿` (đơn vị đứng một mình) → trả `null` (không coi là số).
- `Mixed`: nếu có chữ số Hán "giá trị" (`零..九/两`) → chuyển Latin→Hán rồi bỏ space; ngược lại trả chuỗi Latin trim.

## 5. `ConvertChineseNumberToLong` — số Hán → long

Thứ tự xử lý (đệ quy):
```
1. long.TryParse → nếu là Latin thuần, trả luôn
2. MixedNumberRegex ^(\d+)\s*([万亿])\s*(\d)$   → a×unit + b×(unit/10)
3. Có '亿': tách trước/sau '亿'; trước×10⁸ + sau (nếu 1 ký tự chữ số → ×10⁷, else đệ quy)
4. Có '万': tương tự với 10⁴ (sau 1 ký tự → ×10³)
5. Chuỗi ≥3 ký tự toàn chữ số Hán, không có đơn vị → ghép chữ số thành số Latin (đọc rời)
6. else → ConvertUnderTenThousand
```

`ConvertUnderTenThousand`:
```
"十"        → 10
bắt đầu "十" → 10 + convert(phần sau)
else: duyệt trái→phải; gặp chữ số nhớ vào num2; gặp đơn vị cộng (num2 hoặc 1)×đơn vị
```

## 6. Dải số (ranges) & thập phân

`ChineseToLuatNhanOneMeaning` (luật `{s}`) gọi các hàm dải theo thứ tự:

| Hàm | Mẫu | Ví dụ → kết quả |
|---|---|---|
| `TryConvertVietnameseRangeNumber` | `ComplexRange` `^[十百千]+[1-9][1-9][万亿]$`; `SimpleRangeWithUnit` `^[..][..][十百千][万亿]$`; `SimpleRange` `^[..][..][十百千万亿]$` | `三四万` → "3-4 vạn"; `八九亿`→"8-9 ức" |
| `TryConvertPostfixedRangeNumber` | 2 chữ số cuối, phần đầu là số ×10 tròn chục | `二十三四` → "23-24" |
| `ConvertChineseDecimalToString` | chứa `点`; hoặc chuỗi chữ số Hán rời | `三点五`→"3.5"; `一二三`→"123" |

Quy tắc chọn trong luật `{s}` (count==1), xem
[luat-nhan.md §6](luat-nhan.md#6-chinesetoluatnhanonemeaning--sinh-chuỗi-dịch-từ-luật).

## 7. `NumberToVietnameseText`

```
0 → "0"
|n| < 10000 → n.ToString("N0")            # có dấu phẩy nghìn, vd 1,234
else: tách ức (/10⁸, đệ quy + " ức") , vạn (/10⁴ + " vạn") , phần dư (N0)
       nối bằng space
```
Ví dụ `123456789` → "1 ức 2,345 vạn 6,789".

## 8. Luật đặc biệt

- `TranslateSLuongRule(x两)`: bỏ `两` cuối → `NumberToVietnameseText(convert(phần trước)) + " lượng"`.
- Luật `百分[之]?{s}`: `{s}` = `ConvertChineseDecimalToString(valueN)` → phần trăm.
- Hậu xử lý ngày: khuôn kết quả `^ngày <1..9>` → đổi "ngày" thành "mùng".

## 9. Reimplement Rust — lưu ý

- Cẩn thận `NumberToVietnameseText` dùng format `N0` (dấu phẩy `,` ngăn nghìn kiểu en-US).
  Rust: tự format thêm dấu phẩy, không dùng locale mặc định.
- Đệ quy `ConvertChineseNumberToLong` giữ đúng thứ tự nhánh 亿 → 万 → ghép-rời → under-10k.
- Overflow: dùng `i64` như `long` gốc.
- Golden test: bộ số phong phú (đơn vị lồng, dải, thập phân, hỗn hợp Latin-Hán, 余/多).
