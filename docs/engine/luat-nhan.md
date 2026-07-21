# Luật Nhân — Luật văn phạm dạng mẫu

Nguồn: `containsLuatNhan`, `matchLuatNhanWithN`, `matchLuatNhanS`, `ChineseToLuatNhanOneMeaning`,
`HandleNhanBy`, `UpdateLuatNhanListsAndCaches`, `GetNormalizedLength`, `FindHoHauTuPhrase`.

"Luật Nhân" là các mẫu (pattern) dịch cụm mà từ điển thường không bắt được: ngày giờ, số +
đơn vị, "X của Y", tên riêng ghép họ+hậu tố... Định nghĩa trong `LuatNhan.txt`.

## 1. Format `LuatNhan.txt`

`mẫu=khuôn` (bỏ dòng `#`). Ba loại placeholder:

| Placeholder | Ý nghĩa | Ví dụ dòng |
|---|---|---|
| `{s}` | một **số** (Hán/Latin) | `在接下来的{s}年里=trong vòng {s} năm sau đó` |
| `{n}` | một **danh từ/đại từ/tên** (tra `dictionaryN`) | `超出{n}的掌控=vượt khỏi tầm kiểm soát của {n}` |
| `{1} {2} {3}` | tham chiếu nhóm số thứ i (khi có nhiều `{s}`) | `{s}年{s}月{s}号=ngày {3} tháng {2} năm {1}` |
| `{h}{t}` | (ảo, không có trong file) họ + hậu tố | dựng động từ `hoHauTuCache` |

Trong mẫu còn có cú pháp regex-lite: `(A|B|C)` (lựa chọn), `[x]?` (tuỳ chọn), ví dụ
`(上午|下午|中午|傍晚|早上|晚上)?{s}(点钟|小时|点|时){s}分[钟]?`.

## 2. Phân loại & compile (`UpdateLuatNhanListsAndCaches`)

Sau khi load, tách:
```
luatNhanNList = luật chứa {n} và KHÔNG chứa {s}
luatNhanSList = luật chứa {s} và KHÔNG chứa {n}
```
(Luật chứa cả `{n}` và `{s}` không rơi vào list nào ⇒ bị bỏ qua trong matching hiện tại.)

### Compile regex cho `{s}` (`luatNhanSCache`)
Với mỗi key (trừ `"{s}"` trần):
```
t = key.Replace("(", "(?:").Replace("{s}", " {s} ").Trim()
t = Regex.Replace(t, "\\s+", "\\s*")                  # khoảng trắng linh hoạt
pattern = t.Replace("{s}", "((?:\\d+\\s*[万亿])|(?:\\d+)|(?:[零一二三四五六七八九十百千万亿两〇]+))")
# đặc biệt:
if key == "{s}两":   pattern += "(?!(?:[...数字...]){1,2})"    # negative lookahead
if key == "百分[之]?{s}": pattern = t.Replace("{s}", "([零一二三四五六七八九十百千万亿两〇点\\d]+)")
luatNhanSCache[key] = Regex(pattern, Compiled)
```

### Compile regex cho `{n}` (`luatNhanNCache`)
```
t = key.Replace("(", "(?:")
nếu {n} nằm GIỮA (có hậu tố sau nó):
    prefix = phần trước {n}; suffix = phần sau {n}
    pattern = prefix + "((?:(?!" + Regex.Escape(suffix) + ")[^,\\. ?]){1,10}?)" + suffix
ngược lại (kết thúc bằng {n}):
    pattern = t.Replace("{n}", "([^,\\. ?]{1,10})")
luatNhanNCache[key] = Regex(pattern, Compiled)
```
`{n}` bắt tối đa 10 ký tự, không vượt dấu câu/space, ưu tiên ngắn nhất (`?`).

## 3. `GetNormalizedLength` — độ dài "chuẩn hoá" để xếp ưu tiên

Khi có nhiều luật `{n}` khớp, chọn luật có **mẫu dài nhất** theo độ dài đã bỏ regex:
```
GetNormalizedLength(key):
    thay mỗi (A|B) bằng chuỗi 'c' dài = min độ dài các nhánh
    thay mỗi [x]   bằng 1 ký tự 'c'
    xoá mọi '?'
    trả .Length
```
`matchLuatNhanWithN` duyệt `luatNhanNList` theo `OrderByDescending(GetNormalizedLength)`.

## 4. `containsLuatNhan` — tìm luật khớp sớm nhất

Trả về **index bắt đầu** của luật khớp (càng nhỏ = càng sớm trong chuỗi), `-1` nếu không có.
So sánh 3 nguồn, lấy match có `index` nhỏ nhất:

```
containsLuatNhan(chinese, out luatNhan, out matchedLength, out valueN):
    num = MaxValue
    # (a) luật {n}
    idxN = matchLuatNhanWithN(chinese, ...)     # >=0 nếu khớp
    if 0 <= idxN < num: num=idxN; luatNhan=...; matchedLength=...; valueN=...
    # (b) luật {h}{t}: quét i = 0..min(len,num), tìm FindHoHauTuPhrase
    for i in 0..min(len, num):
        FindHoHauTuPhrase(chinese, i, out bestHTLength)
        if bestHTLength > 0:
            num=i; luatNhan="{h}{t}"; matchedLength=bestHTLength; valueN=""; break
    # (c) luật {s}: chỉ khi chuỗi có ký tự số và matchIndex < num
    if num>0 AND chinese chứa NumberChars AND matchLuatNhanS(...) AND matchIndex < num:
        num=matchIndex; luatNhan=...; matchedLength=...; valueN=...
    return (num == MaxValue) ? -1 : num
```

### `matchLuatNhanWithN` (chi tiết resolve `{n}`)
Với mỗi luật (theo độ dài giảm dần), match regex; với group `{n}` bắt được `value2`:
- Key kết thúc `{n}`: thử prefix dài→ngắn của `value2` tra `dictionaryN` → lấy nghĩa, cắt `matchedLength`.
- Key bắt đầu `{n}`: thử suffix của `value2` (bỏ dần ký tự đầu) tra `dictionaryN`.
- Ngược lại: tra thẳng `value2` trong `dictionaryN`.
`dictionaryN` = Pronouns ∪ Names(1 nghĩa). Nếu `{n}` không phải danh từ đã biết ⇒ luật trượt.

### `matchLuatNhanS`
Duyệt `luatNhanSList`, match regex đã compile; nhận nếu group 1 không rỗng và `match.Value`
không phải key trong `onlyVietPhraseDictionary` (tránh đè cụm từ điển thật).

### `FindHoHauTuPhrase` (luật `{h}{t}` — dựng tên riêng)
```
for num = 6 downto 2:
    text = chinese.Substring(startIndex, num)
    if hoHauTuCache.Contains(text)  AND  !vietPhraseDictionary.ContainsKey(text):
        # đảm bảo không có cụm VietPhrase dài hơn (num+1..20) phủ lên
        nếu KHÔNG có cụm dài hơn trong vietPhraseDictionary:
            bestHTLength = num; return text
return null   (bestHTLength = 0)
```
Ý nghĩa: chuỗi `họ+hậu_tố` (vd `李哥`) mà không phải cụm từ điển → nhận diện là tên,
dịch qua Hán Việt (khuôn `{h}{t}` xử lý ở `ChineseToLuatNhanOneMeaning`? — thực tế
`{h}{t}` chỉ đánh dấu vùng; việc dịch tên rơi về Hán Việt/Names, xem lưu ý §7).

## 5. `HandleNhanBy` — áp luật trong main loop

Được gọi ở nhánh (A2) khi cụm hiện tại **không** có trong từ điển và `num6 > 2`.
Dùng các biến trạng thái `num3/num4/num5` để tránh áp lặp cùng vùng:

```
HandleNhanBy(chinese, ref num2, num6, ref num3, ref num4, ref num5, ref flag2, ...):
    if num2 < num4:                        # đang trong vùng đã xét
        if num4 < num2+num6 AND num6 <= num5-num4: num6 = num4-num2+1
        return
    num7 = containsLuatNhan(chinese.Substring(num2, num6), out luatNhan, out matchedLength, out valueN)
    num4 = num2 + num7
    num5 = num4 + matchedLength
    if num7 == 0:                          # luật khớp NGAY tại num2
        if luật KHÔNG chứa {n}:
            # nếu có Name lồng trong [1..matchedLength) → HỦY (ưu tiên Name)
            nếu tồn tại onlyNameDictionary chứa substring bên trong: return
        text = chinese.Substring(num2, matchedLength)
        t = ChineseToLuatNhanOneMeaning(text, luatNhan, valueN)
        if !IsNullOrEmpty(t):
            ProcessTranslation(chinese, text, t.Trim(), num2, matchedLength, wrapType, ..., ref flag, ...)
    elif num7 <= 0? (num7 < 0):            # không khớp trong cửa sổ này
        num3 = num2 + num6 - 1             # đánh dấu đã quét tới đây (chặn thử lại)
        flag2 = false
        # nới cửa sổ tới 100+ ký tự Hán liên tiếp để kiểm tra xa hơn
        j = 100; while num2+j < len AND isChinese(chinese[num2+j-1]): j++
        if num2+j <= len AND containsLuatNhan(substring(num2,j)) < 0:
            num3 = num2 + j - 1
```

> Khi `num7 > 0` (luật khớp nhưng *lệch* sau `num2`): không dịch ngay, chỉ để vòng lặp
> tiến tới đúng vị trí luật bằng cơ chế `num4/num5` ở lần lặp sau.

## 6. `ChineseToLuatNhanOneMeaning` — sinh chuỗi dịch từ luật

Nguồn ~dòng 3078. Tóm tắt các nhánh:

```
if luatNhan chứa {n}:
    if valueN rỗng: return null
    return luatNhanNList[luatNhan].Replace("{n}", valueN.Trim())

if luatNhan == "{s}两":           return TranslateSLuongRule(chinese)         # "... lượng"
if luatNhan == "百分[之]?{s}":    return khuôn.Replace("{s}", ConvertChineseDecimalToString(valueN))

if luatNhan chứa {s}:
    if luatNhan chứa 余/多:  xử lý "hơn/dư" (bỏ ký tự 余|多, convert số, thay {s})
    count = số lần {s} trong luatNhan
    dựng regex từ luatNhan (thay '(' → '(?:', {s} → nhóm số), match `chinese`
    if count == 1:
        text5 = group1
        newValue = theo thứ tự:
            - có '.'/','  hoặc bắt đầu '0' (len>1)         → giữ nguyên text5
            - TryConvertVietnameseRangeNumber(text5)        → dải "a-b vạn/ức/ngàn"
            - TryConvertPostfixedRangeNumber(text5)         → dải "x-y"
            - len==2, cả 2 là chữ số Hán                    → "d1-d2"
            - còn lại: ConvertChineseNumberToLong → nếu khuôn có "năm|chương" thì để số thô,
                       ngược lại NumberToVietnameseText
        result = khuôn.Replace("{s}", newValue)
    else (nhiều {s}):
        với i=1..count: group[i] → số (thô nếu "năm|chương", else NumberToVietnameseText)
        thay {1},{2},... trong khuôn
    # hậu xử lý: "ngày <1..9>" → "mùng <..>"
    if result khớp ^ngày (\d) với 0<d<10:  thay "ngày"→"mùng"
    return result
```

Chi tiết chuyển số: [number-conversion.md](number-conversion.md).

## 7. Lưu ý reimplement

- Regex .NET và Rust `regex` crate khác nhau ở vài điểm (lookahead `(?!...)`: crate `regex`
  **không** hỗ trợ). Luật `{s}两` dùng negative lookahead ⇒ cần crate `fancy-regex` hoặc
  xử lý thủ công. **Ghi rõ** khi implement để không bỏ sót.
- Thứ tự ưu tiên `{n}` > `{h}{t}` > `{s}` theo vị trí index nhỏ nhất — phải giữ đúng.
- `{h}{t}` chỉ đánh dấu vùng tên; kết quả dịch tên đến từ Hán Việt/Names, không phải khuôn.
- Đây là phần **phức tạp & rủi ro nhất**. Đưa vào giai đoạn 5 của lộ trình, có golden test riêng
  cho từng loại luật (ngày, giờ, %, dải số, tên họ+hậu tố).
