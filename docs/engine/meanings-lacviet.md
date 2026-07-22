# Meanings / Lạc Việt — Tra cứu nghĩa chi tiết

Nguồn: `ChineseToMeanings(chinese, out phraseLength)` (decompile ~dòng 1724).

Đây là chức năng **tra cứu** (không dịch liền mạch) — dùng khi người đọc bấm vào một cụm để
xem mọi nghĩa khả dĩ. Trả về một khối text nhiều mục, phân cách bằng `-----------------`.

## 1. Chữ ký & giới hạn

```
ChineseToMeanings(chinese, out phraseLength) -> string
```
- Chỉ xét tối đa **20 ký tự đầu** (`text2 = chinese[..20]`).
- `phraseLength` = độ dài cụm đầu tiên tra được (dùng để con trỏ nhảy trong UI).
- Áp `NumberModifier` lên bản sao trước khi xử lý luật/số.

## 2. Thứ tự tra (ghép dồn vào output)

```
1. LuatNhan khớp tại vị trí 0?  → "<cụm> (Luật Nhân) <dịch>"           (đặt flag, set phraseLength)
2. (nếu chưa có luật) Số tại vị trí 0? → "<số> (Luật S) <dịch>"
3. for num = min(20,len) downto 1:  cụm = text2[..num]
     - onlyNamePhuDictionary   → "<cụm> (Names phụ) <value, '/'→'; '>"
     - onlyNameChinhDictionary → "<cụm> (Name chính) <value>"
     - onlyVietPhraseDictionary→ "<cụm> (VietPhrase) <value>"
       (mỗi cái set phraseLength nếu chưa set)
4. for num = min(20,len) downto 1:  cụm = text2[..num]
     - lacVietDictionary       → "<cụm> (Lạc Việt)\n<value>"           (phraseLength=1 nếu chưa set)
5. Nếu vẫn phraseLength==0:  "<ký tự đầu>\n----\nNot Found"
```

Nghĩa là output có thể chứa **nhiều mục** cho nhiều độ dài cụm khác nhau (dài→ngắn), gồm cả
Names phụ, Name chính, VietPhrase, và định nghĩa Lạc Việt.

## 3. Format `LacViet.txt`

Value là định nghĩa nhiều dòng, dùng ký tự đặc biệt:
```
凉=✚[liáng] \n\t1. lạnh, mát, nguội \n\t2. mỏng, bạc, ít, hóng gió \n\t3. họ Lương
阿=✚[ā] Hán Việt: A\n\t1. anh; chú; em ...\n✚ [ē] Hán Việt: A\n\t...
```
- `✚` mở một mục nghĩa (một cách đọc / pinyin).
- `\n`, `\t` là **chuỗi literal 2 ký tự** trong file (không phải newline thật) — engine giữ
  nguyên value, UI tự render. Khi xuất CLI/API nên giữ nguyên hoặc cung cấp option unescape.

## 4. Trạng thái và lưu ý triển khai

- Chức năng này **chưa được triển khai** trong `qt-core`, CLI hoặc HTTP API.
- Value VietPhrase/Names: thay `/` → `; ` khi hiển thị (bước 3), nhưng LacViet giữ nguyên.
- Giữ đúng thứ tự mục và separator `\n-----------------\n` để "y hệt".
- API dự kiến có thể expose `POST /meanings`; schema response cần được chốt khi triển khai,
  không nên suy ra từ endpoint dịch hiện tại.
