# Hán Việt — Phiên âm từng chữ

Nguồn: `ChineseToHanViet(string, out mapping)`, `ChineseToHanViet(char)`, `ToNarrow`, `isChinese`.

## 1. Phiên âm một ký tự — `ChineseToHanViet(char)`

```
ChineseToHanViet(c):
    if c == ' ': return ""
    if hanVietDictionary.TryGetValue(c.ToString(), out v): return v   # tra ChinesePhienAmWords.txt
    return ToNarrow(c.ToString())                                     # không có → full-width→half-width
```

- `hanVietDictionary` nạp từ `ChinesePhienAmWords.txt` (`chữ=âm`, ví dụ `一=nhất`).
- Không tìm thấy → `ToNarrow`: chuyển ký tự **full-width** (`！`..`～`, U+FF01..U+FF5E) về
  **half-width** ASCII (`!`..`~`), còn lại giữ nguyên. (Số/dấu câu Trung dạng full-width → ASCII.)

```
ToNarrow(str): với mỗi ký tự c:
    if '！' <= c <= '～':  c - 0xFF01 + 0x21   # dịch về ASCII tương ứng
    else:                 giữ nguyên
```

## 2. `isChinese` — định nghĩa "ký tự Hán" của QT

```
isChinese(c) = hanVietDictionary.ContainsKey(c.ToString())
```

**Quan trọng**: một ký tự được coi là "Hán" **chỉ khi** nó có trong từ điển Hán Việt.
Không dùng Unicode range. Khi reimplement Rust, `is_chinese` phải tra `han_viet` map.

## 3. Dịch cả chuỗi Hán Việt — `ChineseToHanViet(string, out mapping[])`

Dùng cho mode **HanViet** (view). Duyệt từng ký tự, chèn khoảng trắng giữa hai chữ Hán liên tiếp:

```
ChineseToHanViet(chinese, out mapping):
    result = StringBuilder(); last = ""
    for i = 0 .. len-2:            # tới ký tự áp chót
        c = chinese[i]; next = chinese[i+1]
        if isChinese(c):
            appendTranslatedWord(result, ChineseToHanViet(c), ref last, ...)   # nối + viết hoa đầu câu
            if isChinese(next):
                result.Append(" "); last += " "        # hai chữ Hán liền nhau → chèn space
            mapping.Add(range của âm vừa thêm)
        else:
            result.Append(c); last += c                # ký tự thường giữ nguyên
            mapping.Add(range(., 1))
    # xử lý ký tự cuối cùng (length-1) tương tự
    return result.ToString()
```

Điểm khác với VietPhrase:
- **Không** tra cụm dài; thuần từng ký tự.
- Space chỉ chèn giữa **hai ký tự Hán liên tiếp** (không sau ký tự thường).
- Vẫn dùng `appendTranslatedWord` → **viết hoa chữ cái đầu câu** (sau `.`, `\n`, `"`...),
  nên output HanViet cũng có auto-capitalize giống VietPhrase.

### Contract ranges của bản Rust

`TranslationResult` của Rust luôn trả hai mảng song song `source_ranges` và
`target_ranges`, kể cả mode HanViet. Đây là contract mở rộng để UI highlight hai chiều,
không phải bản sao của `chineseHanVietMappingArray` trong QT2025 (mảng gốc chỉ chứa range
đích và entry cuối được tính sau khi append nên có thể lệch khỏi đoạn text thực tế).

Offset và length vẫn tính theo UTF-16. Tuy nhiên Rust duyệt theo Unicode scalar: ký tự
ngoài BMP như emoji tạo một range dài 2, thay vì hai range surrogate dài 1 như vòng lặp
`char` của .NET.

## 4. Ngoài ra — `TranslateChineseToHanViet` / `ChineseToHanVietForAnalyzer`

Hai biến thể "đơn giản" (dùng cho Analyzer, **không** auto-capitalize):

```
TranslateChineseToHanViet(s): join(" ", mỗi ký tự → âm HV hoặc chính ký tự)   # luôn có space giữa MỌI ký tự
ChineseToHanVietForAnalyzer(s): tương tự, append " " sau mỗi ký tự rồi TrimEnd
```

Hai hàm này **khác** `ChineseToHanViet(string, out)` ở chỗ: chèn space giữa *mọi* token
(kể cả ký tự thường) và không viết hoa. MVP CLI mode "hanviet" nên dùng
`ChineseToHanViet(string, out)` (bản có mapping, giống UI). Ghi chú để không nhầm.

## 5. Reimplement Rust — checklist

- [x] `han_viet: HashMap<char, String>` từ `ChinesePhienAmWords.txt`.
- [x] `to_narrow(&str)`: chỉ dịch U+FF01..U+FF5E → ASCII.
- [x] `is_chinese(c) = han_viet.contains_key(&c)`.
- [x] `char_to_han_viet(' ') == ""`; miss → `to_narrow`.
- [x] Chuỗi: space giữa 2 chữ Hán liên tiếp; auto-capitalize qua `append_translated_word`.
- [x] Regression test mode HanViet và non-BMP ranges.
