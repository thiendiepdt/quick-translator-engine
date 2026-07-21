# Translation Algorithm — `TranslateAll` (main loop)

Đặc tả chính xác vòng lặp dịch. Nguồn: `TranslatorEngine.TranslateAll` (decompile ~dòng 1462).

## 1. Chữ ký

```
TranslateAll(chinese, wrapType, translationAlgorithm, prioritizedName, scanRange,
             dictionary, ref lastTranslatedWord) -> TranslationResult
```
`dictionary` = `vietPhraseDictionary` (mode VietPhrase) hoặc `vietPhraseOneMeaningDictionary`
(mode OneMeaning). Xem [dictionaries.md](dictionaries.md).

## 2. Chuẩn bị trước vòng lặp

```
list  = []   # ChinesePhraseRanges (vị trí cụm nguồn)
list2 = []   # VietPhraseRanges    (vị trí cụm dịch trong output)
result = StringBuilder()
lastTranslatedWord = ""
num  = chinese.Length - 1          # index cuối
num2 = 0                            # con trỏ hiện tại
num3 = num4 = num5 = -1            # biên trạng thái cho LuatNhan (xem luat-nhan.md)

numbers = PreScanForNumbers(chinese).ToDictionary(n => n.StartIndex)   # map vị trí→NumberInfo
chinese = NumberModifier(chinese)  # đổi "余/多 + 百/千/万/亿" → "百/千/万/亿 + 余/多"
```

- `PreScanForNumbers`: quét trước mọi cụm số, lưu `{StartIndex, Length, Text, Type}`.
  Bỏ qua số nếu bản thân nó là một key trong `onlyVietPhraseDictionary`. Xem
  [number-conversion.md](number-conversion.md#prescan).
- `NumberModifier`: chuẩn hoá "hơn vạn/nghìn" (数 `万余` → `余万`...). Xem
  [number-conversion.md](number-conversion.md#numbermodifier).

## 3. Vòng lặp chính

```
while num2 <= num:
    flag  = false        # đã dịch được ở vòng này chưa
    flag2 = true         # cho phép thử LuatNhan
    numbers.TryGetValue(num2, out value)   # value = NumberInfo tại vị trí này (hoặc null)

    # (A) Quét cụm dài → ngắn
    for num6 = scanRange down to 1:
        if num2 + num6 <= len(chinese)  AND  (value == null OR num6 >= value.Length):
            text = chinese.Substring(num2, num6)

            if dictionary.TryGetValue(text, out value2):        # (A1) khớp từ điển
                isLongest = isLongestPhraseInSentence(chinese, num2, num6, dictionary, translationAlgorithm)
                nameOk    = (!prioritizedName) OR (!containsName(chinese, num2, num6))
                algoOk    = (translationAlgorithm != 0 AND translationAlgorithm != 2)
                            OR isLongest
                            OR (prioritizedName AND onlyNameDictionary.ContainsKey(text))
                if nameOk AND algoOk:
                    ProcessTranslation(chinese, text, value2, num2, num6, wrapType, ...)
                    break                                        # đã dịch cụm, thoát for

            elif !text.Contains('\n') AND !text.Contains('\t')   # (A2) thử LuatNhan
                 AND flag2 AND num6 > 2 AND num3 < num2 + num6 - 1:
                HandleNhanBy(chinese, ref num2, num6, ref num3, ref num4, ref num5,
                             ref flag2, ..., ref flag, dictionary, result, ...)
                if flag: break

    if flag: continue     # đã dịch, quay lại while với num2 đã tiến

    # (B) Vị trí này là số nhưng chưa dịch → áp luật "{s}"
    if !flag AND value != null:
        t = ChineseToLuatNhanOneMeaning(value.Text, "{s}", "")
        if !IsNullOrEmpty(t):
            ProcessTranslation(chinese, value.Text, t, num2, value.Length, wrapType, ...)

    # (C) Vẫn chưa dịch → fallback Hán Việt 1 ký tự
    if !flag:
        ProcessHanViet(chinese, wrapType, ref num2, ..., result, ...)
```

Kết thúc, trả:
```
TranslationResult {
    TranslatedText     = result.ToString(),
    ChinesePhraseRanges = list.ToArray(),
    VietPhraseRanges    = list2.ToArray()
}
```

> **Lưu ý implement**: điều kiện `num6 >= value.Length` ở (A) đảm bảo: nếu vị trí này bắt đầu
> một cụm số dài `value.Length`, ta không cho cụm từ điển ngắn hơn số "cắt" ngang cụm số.

## 4. <a name="longest-phrase"></a>`isLongestPhraseInSentence`

Quyết định cụm hiện tại có phải "cụm dài nhất khả dĩ" hay không — chống việc ghép sai khi
một cụm ngắn nằm lồng trong ngữ cảnh của cụm dài hơn bắt đầu ở vị trí *sau*.

```
isLongestPhraseInSentence(chinese, startIndex, phraseLength, dictionary, translationAlgorithm):
    if phraseLength < 2: return true
    num = (translationAlgorithm == 0) ? phraseLength
                                      : max(phraseLength, 3)
    end = startIndex + phraseLength - 1
    for i = startIndex + 1 .. end:                 # mọi vị trí BÊN TRONG cụm (trừ ký tự đầu)
        for num3 = 20 down to num+1:               # cụm dài hơn `num`
            if len(chinese) >= i + num3 AND dictionary.ContainsKey(chinese.Substring(i, num3)):
                return false                        # tồn tại cụm dài hơn chồng lấn → cụm này KHÔNG dài nhất
    return true
```

- `translationAlgorithm == 0`: nghiêm ngặt — bất kỳ cụm dài hơn chồng lấn nào cũng loại cụm hiện tại.
- `translationAlgorithm == 1` hoặc `2`: nới lỏng — chỉ loại nếu có cụm chồng lấn **dài hơn max(phraseLength,3)**.
- Trong (A), điều kiện `algoOk` khiến: với algo `1`, cụm luôn được nhận (không cần `isLongest`);
  với algo `0`/`2`, cụm chỉ được nhận nếu `isLongest` true (hoặc là Name khi `prioritizedName`).

## 5. <a name="contains-name"></a>`containsName` (name-priority)

Khi `prioritizedName = true`, một cụm VietPhrase bị **từ chối** nếu nó che một tên riêng
bắt đầu ở vị trí bên trong cụm:

```
containsName(chinese, startIndex, phraseLength):
    if phraseLength < 2: return false
    if onlyNameDictionary.ContainsKey(chinese.Substring(startIndex, phraseLength)):
        return false                      # bản thân cụm LÀ một Name → không coi là "che" name
    end = startIndex + phraseLength - 1
    for i = startIndex + 1 .. end:        # mọi vị trí bên trong (trừ đầu)
        for num3 = 20 down to 2:
            if len(chinese) >= i + num3 AND onlyNameDictionary.ContainsKey(chinese.Substring(i, num3)):
                return true               # có Name lồng bên trong → từ chối cụm
    return false
```

Kết hợp trong (A): nếu `prioritizedName` và `containsName` true → cụm bị bỏ qua, vòng for
tiếp tục với `num6` nhỏ hơn (hoặc rơi xuống ProcessHanViet), để tên riêng phía sau được dịch đúng.

## 6. `ProcessTranslation` — ghi một cụm đã dịch

```
ProcessTranslation(chinese, subString, translation, startIndex, length, wrapType, ...):
    list.Add(CharRange(startIndex, length))                 # range nguồn
    text = WrapTranslation(translation, wrapType)           # bọc [...] nếu wrapType==1
    appendTranslatedWord(result, text, ref lastTranslatedWord)   # nối + auto viết hoa/space
    list2.Add(CharRange(result.Length - text.Length, text.Length))  # range đích
    if nextCharIsChinese(chinese, startIndex + length - 1):  # còn chữ Hán ngay sau
        result.Append(" "); lastTranslatedWord += " "
    flag = true
    num2 += length                                          # tiến con trỏ qua cả cụm
```

## 7. `ProcessHanViet` — fallback 1 ký tự

```
ProcessHanViet(chinese, wrapType, ref num2, ...):
    if isChinese(chinese[num2]):
        t = WrapTranslation(ChineseToHanViet(chinese[num2]), wrapType)
        appendTranslatedWord(result, t, ref lastTranslatedWord)
        if nextCharIsChinese(chinese, num2): result.Append(" "); lastTranslatedWord += " "
    elif (chinese[num2] == '"' || '\'')  AND  lastTranslatedWord không kết bằng khoảng trắng/.?!\t
         AND ký tự kế tiếp không phải ' ' hoặc ',':
        result.Append(" ").Append(chinese[num2])            # mở ngoặc kép: chèn space trước
    else:
        result.Append(chinese[num2])                        # ký tự thường (dấu câu, latin...) giữ nguyên
    num2++                                                   # LUÔN chỉ tiến 1 ký tự
```

Xem chi tiết phiên âm: [han-viet.md](han-viet.md).

## 8. `appendTranslatedWord` — quy tắc nối chuỗi & viết hoa

Đây là chi tiết dễ sai nhất khi tái tạo "y hệt". Nguồn: decompile ~dòng 2624.

```
appendTranslatedWord(result, translatedText, ref lastTranslatedWord):
    if lastTranslatedWord kết thúc bằng một trong:
           "\n"  "\t"  ". "  "\""  "'"  "? "  "! "  ".\" "  "?\" "  "!\" "  ": "
        lastTranslatedWord = toUpperCase(translatedText)     # đầu câu → VIẾT HOA chữ cái đầu
    elif lastTranslatedWord kết thúc bằng " " hoặc "(":
        lastTranslatedWord = translatedText                  # đã có space/mở ngoặc → nối thẳng
    else:
        lastTranslatedWord = " " + translatedText            # mặc định chèn 1 space trước

    # Nếu từ mới bắt đầu bằng dấu , . ? ! và output đang kết bằng space → bỏ space thừa
    if (translatedText rỗng OR bắt đầu bằng , . ? !) AND result kết thúc bằng ' ':
        xoá ký tự space cuối của result
    result.Append(lastTranslatedWord)
```

`toUpperCase`: viết hoa ký tự đầu; nếu chuỗi bắt đầu bằng `[` (do wrap) thì viết hoa ký tự
sau `[`.

## 9. `WrapTranslation` & `nextCharIsChinese`

```
WrapTranslation(t, wrapType) = (wrapType == 0) ? t : "[" + t + "]"

nextCharIsChinese(chinese, endIdx) =
    (chinese.Length - 1 > endIdx) AND isChinese(chinese[endIdx + 1])
```

`isChinese(c)` = `hanVietDictionary.ContainsKey(c)` — tức "c có trong từ điển Hán Việt".
Đây là định nghĩa "ký tự Hán" của QT: **một ký tự là Hán khi và chỉ khi nó có phiên âm Hán Việt**.
Hệ quả: khi tái tạo, `isChinese` phải dựa trên `ChinesePhienAmWords.txt`, không dùng Unicode block.
