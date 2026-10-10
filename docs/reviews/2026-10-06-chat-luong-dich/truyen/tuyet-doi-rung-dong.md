# Review: Tuyệt Đối Rung Động (tuyet-doi-rung-dong)

**Thông tin chung**
- Setting: `modern`. Thể loại thật là ngôn tình thanh mai trúc mã oan gia, đi từ tuổi thơ qua cấp ba, đại học, rồi kết hôn; có thêm ngoại truyện về cặp phụ.
- Ngôi kể: ngôi 3.
- Đã dịch 87 chương. Chương đã đọc: 0001, 0044, 0086, 0087 (hai chương cuối dịch muộn nhất theo state.json; mtime trong out/ trùng nhau hết).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.5 | Cặp chính bị khoá "tôi–cậu" từ 6 tuổi đến lúc cưới. "ta" lọt vào độc thoại. Mẹ nói với con dùng "tao–mày". |
| Giọng văn | 7 | Trôi chảy, trẻ trung. Còn Hán Việt kiểu convert (học trưởng/học muội, đại lão, tiêu sái). Nam chính bị gọi "hắn" cả trong cảnh ngọt. |
| Dễ hiểu | 7.5 | Ít sót ý. Còn vài chỗ sai nghĩa (随你, 总不能, 意外) và vài câu calque. |
| Độ khó | 7 | Giữ được nhiều miếng hài. Mất cú chốt "随你呗"; 呵 bị dịch thành "Hừ". |
| Thuật ngữ | 6.5 | 卫皎皎 trôi sang "Vệ Kiểu Kiểu" ở 5 chương. Mục addressing trong glossary nặng Hán Việt và không có mốc. |

## Lỗi (xếp từ nặng đến nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống).** Cặp chính "tôi–cậu" suốt 87 chương, kể cả sau khi cưới.
   - ch86 dòng 11: 我今晚要吃龙虾，你去给我做 → "Tôi tối nay muốn ăn tôm hùm, cậu đi làm cho tôi." Đề xuất: "Tối nay em muốn ăn tôm hùm, anh đi làm cho em đi."
   - ch77 dòng 185 còn có "Vợ ơi, tôi sai rồi."
   - Nguyên nhân: glossary chỉ có một mục, không có mốc; truyện không có cast.json.
2. **SAI_NGHIA (cao, lẻ).** ch87 dòng 13: 随你呗 → "Tùy bố thôi ạ." Đề xuất: "Giống bố đấy ạ." Đây là cú chốt hài (bố cũng sợ mẹ), dịch sai thì mất luôn miếng hài.
3. **THUAT_NGU_TROI (cao, hệ thống).** 卫皎皎 thành "Vệ Kiểu Kiểu" 21 lần/5 file (0066, 0074, 0075, 0077, 0079), trong khi glossary đã chốt "Giảo Giảo" (đúng ở 123 lần/16 file). Ví dụ ch77 dòng 245.
4. **XUNG_HO_NGUOI_KE (vừa, hệ thống).** Luật "ta" của ngôi một lọt vào truyện kể ngôi ba hiện đại.
   - ch43 dòng 349: 我要，失约了 → "Ta phải, lỡ hẹn rồi." Nhưng ch44 dòng 183 dịch cùng câu này là "Tôi phải lỡ hẹn rồi".
   - ch2 dòng 129 trộn "ta–cậu".
   - Lời tác giả lúc xưng "ta" (12 file), lúc "mình" (21 file), lúc "tôi" (4 file).
5. **HU_HA (vừa, 5 lần/5 file).** 呵 (cười khẽ) bị dịch thành "Hừ". ch28 dòng 229: "“Hừ......” Hắn bật cười khẽ", tức là tiếng khinh lại đi kèm câu "cười khẽ", tự mâu thuẫn.
6. **GLOSSARY_HAN_VIET_NANG (vừa, hệ thống).**
   - Số lần: học trưởng 15, học tỷ 10, học muội 7, đại lão 9, Tiết muội muội 21.
   - Glossary tự sinh ghi "林昂→萧苒: tôi–học muội", trái với style.avoid và lệch cả với bản dịch thực tế (đã dùng "em").
   - ch60 dòng 83: "Học muội, không cần căng thẳng thế đâu." Đề xuất: "Em đừng căng thẳng thế."
7. **TIEU_DE (vừa, hệ thống).**
   - Raw không có tên chương, nên đoạn đầu bị dịch làm tiêu đề: 69/87 tiêu đề dài hơn 80 ký tự, dài nhất 297.
   - 7 file dùng "Chương N." thay cho "Chương N:".
   - Ở ch1, đề từ dính liền vào đoạn mở đầu.
8. **XUNG_HO_VAI_VE (vừa).** ch36 dòng 181: bà Thư nói với con "Tao cung phụng cho mày…". Mục glossary "tao–mày" quá chợ búa so với một người mẹ thành đạt, khắt khe. Đề xuất giữ "mẹ–con" với giọng lạnh.
9. **SAI_NGHIA (vừa).** ch44 dòng 125: 总不能 → "tổng thể đâu thể". Đề xuất: "chẳng lẽ lại…".
10. **CALQUE (thấp, hệ thống).**
    - ch44 dòng 177: "đúng là bản thân Giang Trì Chu không sai".
    - ch86 dòng 153: "Nhìn thấy hắn cái liếc mắt đầu tiên".
    - Ngoài ra: "đem" 43 lần/30 file, "chiếm tiện nghi" 2 lần.
11. **SAI_NGHIA (thấp).** ch86 dòng 509: 意外 → "tai nạn bất ngờ" (đúng ra là "vô tình chạm phải"). Cũng chương này, dòng 463: 吃得起 → "hắn ăn được".
12. **KHAC (thấp, hệ thống).**
    - "hắn" xuất hiện 1877 lần/85 file, kể cả trong cảnh tình cảm (ch86 dòng 511 "Hắn dịu dàng chu đáo đến thế").
    - Mẹ của Giang (Tống Nhàn) được gọi "cô", trong khi Ôn Như Ý cùng vai lại là "bà ta" (ch44).
13. **XUNG_HO_TROI (thấp).**
    - Giang nói chuyện xưng "tôi" nhưng tin nhắn lại xưng "tớ" (ch44 dòng 67).
    - Nhan Hy nói với bạn lúc "tớ", lúc "tôi" (ch87 dòng 281).
14. **MEME_NGHIA_DEN (thấp).** "phát cẩu lương" (ch50 dòng 329), "chiếc áo bông nhỏ tri kỷ", "thụ sủng nhược kinh".
15. **DINH_DANG (thấp).** Ngoặc kép thẳng `"` ở 10 file 0063–0072; tin nhắn bị bọc hai lớp “【…】”.

## Gợi ý sửa prompt và app

- **Addressing có mốc.** Glossary cần ghi được dạng "tôi–cậu → anh–em từ ch~X". Khi raw xuất hiện 老婆/老公/结婚 mà addressing vẫn "tôi–cậu", app nên cảnh báo.
- **Check glossary sau khi dịch.** Raw có khoá mà bản dịch thiếu đích chuẩn, lại có biến thể lạ, thì báo lỗi (ví dụ Kiểu/Giảo).
- **Giới hạn luật "ta".** Chỉ áp khi truyện kể ngôi 1. Độc thoại trong truyện ngôi 3 theo đại từ thoại của nhân vật. Lời tác giả chốt một kiểu ("mình").
- **Bộ sinh glossary tự động** phải tuân style.avoid: bỏ học trưởng/học muội/muội muội/đại lão khi setting là modern. Không xếp biệt danh vào nhóm addressing.
- **Bảng thán từ trong prompt.** 呵 dịch là "ha/hì/khẽ cười"; "Hừ" chỉ dành cho 哼.
- **Tiền xử lý raw.**
  - Tách "第N章" khỏi đoạn đầu, chốt một định dạng tiêu đề.
  - Phát hiện mojibake để cảnh báo.
- **Đại từ ngôi ba cho romance hiện đại.** Cho phép chọn "anh/cậu" cho nam chính thay vì "hắn".

## Làm tốt

- Không sót chữ Hán; không có "ngươi" hay "Ân".
- Raw bị lỗi mã (ch44, ch86), bản dịch vẫn đoán đúng ý.
- Bản địa hoá tốt: Taobao, WeChat, lớp chín, lớp mười hai, ban xã hội/tự nhiên.
- Các câu chữ ký dịch nhất quán.
- Cặp phụ Lâm Ngang–Tiêu Nhiễm chuyển "tôi/anh–em" tinh tế theo độ xa gần.
- Thoại trẻ trung, nhiều thành ngữ Việt hay.
