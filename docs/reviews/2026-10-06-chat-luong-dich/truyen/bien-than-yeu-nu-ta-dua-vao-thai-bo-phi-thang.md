# Review: Biến thân yêu nữ, ta dựa vào thải bổ phi thăng

- Setting trong story.json: `ancient`. Thể loại thực tế: tu tiên hắc ám hài hước, có 18+; tác giả hay chèn chi tiết hiện đại và có một arc ảo cảnh hiện đại (0192).
- Ngôi kể: ngôi 3. Đã dịch 308 chương, chưa có cast.json.
- Chương đã đọc trọn: 0001 (chương đầu), 0155 (chương giữa, ch.154), 0307 và 0308 (hai chương dịch mới nhất, ch.303–304).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.5 | Huynh–muội, tỷ–đệ nói trước đám đông thì đúng. Nhưng xưng hô trôi giữa các chương: ch.0155 bỗng thành "nàng–ngươi", Tiểu Liên lúc "ngươi" lúc "em". Model cũng mặc định "ta–ngươi", kể cả lão tăng nói với người sắp chết. |
| Giọng văn | 7.5 | Đọc mượt, giọng tu tiên đúng, chửi thề và cảnh 18+ dịch thẳng. Arc hiện đại đổi sang anh/em đúng. Thỉnh thoảng lọt từ hiện đại ("người phụ nữ", "súng liên thanh"). |
| Dễ hiểu | 7.5 | Đa số câu rõ. Còn calque lặp nhiều lần và vài chỗ đại từ làm sai nghĩa. |
| Khó khăn | 6.5 | Nhịp hài và cảnh giả nai giữ được. Chơi chữ bị mất (枕月/枕约, 直抒胸臆), 呵呵 thành "Hì hì", 嗯 thành "Ân". |
| Thuật ngữ | 6 | Glossary phình tới 710 mục. Tên tác giả gõ nhầm không được gộp nên một nhân vật bị tách thành 2–3 tên. Có phiên âm sai và tên chiêu lai Hán Việt với thuần Việt. |

## Lỗi (xếp từ nặng đến nhẹ)

1. **XUNG_HO_TROI (cao, lỗi hệ thống)**. Ch.0155 dòng 121.
   - Gốc: 提醒你了，还怎么看到你那惊慌失措的表情？
   - Hiện tại: "Nhắc nhở nàng rồi thì làm sao thấy được vẻ mặt kinh hoàng hoảng hốt của nàng nữa?"
   - Đề xuất: "Nhắc muội rồi thì sao thấy được bộ dạng hoảng hốt của muội nữa?"
   - Vì sao: ch.0148–0151 và 0156 đều dùng huynh/muội, riêng 0155 đổi thành nàng/ngươi (cả với Minh Chính). Không có cast.json ghi mốc nên mỗi chương model tự chọn.

2. **THUAT_NGU_TROI (cao, lỗi hệ thống)**. Tác giả gõ nhầm 公秩序/公冶序/公输序, glossary nhận thành ba tên: Công Trật Tự 63 lần, Công Dã Tự 4, Công Thâu Tự 5. Ngay trong ch.0283, dòng 107 viết "Công Trật Tự", dòng 137 đã thành "Công Thâu Tự". Cùng kiểu lỗi này:
   - Tạ Vãn Ngâm 78 lần / Tạ Uyển Ngâm 4 lần.
   - Nguyệt Ẩn lão quỷ / Nguyệt Ảnh lão quỷ, mỗi tên 19 lần.

3. **DAI_TU_MO_HO (cao)**. Ch.0155 dòng 291.
   - Gốc: 与她母亲当年的体质表现，颇为不同
   - Hiện tại: "biểu hiện thể chất này ngược lại rất khác với mẫu thân nó năm xưa"
   - Vì sao: chữ "nó" đứng ngay sau "thằng nhóc Minh Chính" nên người đọc hiểu là mẹ của Minh Chính, trong khi 她 là Triều Vũ.
   - Đề xuất: "…khác với thể chất của mẹ con bé năm xưa."

4. **MEME_NGHIA_DEN (vừa)**. Ch.0307 dòng 109.
   - Gốc: 枕月？枕约！…都叫你约到枕头边上了
   - Hiện tại: "Người ta đã bảo ngươi hẹn tới bên gối rồi…"
   - Vì sao: người đọc không biết Chẩm = gối nên mất trò chơi chữ, và câu sau dịch lệch nghĩa.
   - Đề xuất: "Chẩm Nguyệt là 'gối trăng' à? Đọc là 'gối hẹn' thì đúng hơn! Đã hẹn được người ta lên tận gối rồi…"

5. **XUNG_HO_VAI_VE (vừa, lỗi hệ thống)**. Ch.0001 dòng 87.
   - Hiện tại: lão tăng nói "Lúc ngươi hôn mê, đã cho ngươi uống…", dù vừa gọi cô là "nữ thí chủ".
   - Đề xuất: "Lúc thí chủ hôn mê, lão nạp đã…"
   - Vì sao: toneRules đã dặn sư tăng xưng "thí chủ/lão nạp". Trong cả truyện, "ngươi" xuất hiện 3565 lần/300 file, còn "nữ thí chủ" chỉ 8 lần. Nữ chính cũng gọi người tình là "ngươi" ngay giữa cảnh âu yếm.

6. **XUNG_HO_TROI (vừa)**. Tiểu thư nói với Tiểu Liên:
   - Ở 0155: "Ngươi bảo ông ta…".
   - Ở 0159 dòng 219: "Chẳng phải bảo em đi giết…".
   - Tiểu Liên tự xưng "nô tỳ" (0147), sang 0159 lại xưng "em".

7. **XUNG_HO_TROI (thấp)**. Ch.0001: dòng 119 nàng gọi tiểu hòa thượng là "người", đến dòng 177 (cảnh quyến rũ) lại đổi thành "Ngươi có thể ôm ta…". Nên dùng "Tiểu sư phụ ôm ta một chút được không?"

8. **GLOSSARY_PHIEN_AM (vừa)**.
   - 裹脚蛊 dịch thành "Quỏa Cước Cổ" (0163:37), từ không có trong tiếng Việt; đúng là "Khỏa".
   - 缸 dịch thành "Cương" (Phong Cương Đạo, Thi Cẩu Cương); đúng là "Cang".
   - 蚣 dịch thành "Cung"; đúng là "Công".
   - 天田 bị đảo thành "Điền Thiên".

9. **CALQUE (vừa, lỗi hệ thống)**.
   - "một tầng" 100 lần/81 file, ví dụ 0307:13 "nổi lên một tầng ghét bỏ". Nên viết "lộ rõ vẻ chê bai".
   - "không tính là" 31 lần, "một cỗ" 31 lần.
   - "đem" 108 lần, ví dụ 0155:111 "đem mặt mũi Nam Cung gia vứt đi đâu".
   - "chiếm tiện nghi" 8 lần.

10. **AN_THAN_TU (vừa, lỗi hệ thống)**. 嗯 thành "Ân" 181 lần/135 file, ví dụ 0155:135 `"Ân" một tiếng`. Đọc khô, nhất là trong cảnh phòng the. Hai chương mới nhất lại dịch 嗯 thành "A?", nên cả truyện không thống nhất.

11. **MEME_NGHIA_DEN (thấp)**. 0308:13: 直抒胸臆 dịch thành "nói thẳng cõi lòng". Câu này chơi chữ 胸 (ngực) nối với câu chê "ngực khủng" ngay trước đó, dịch phẳng thì mất miếng hài.

12. **HU_HA (thấp)**. 0155:107: 呵呵 (cười khẩy) dịch thành "Hì hì", nghe nhí nhảnh. Nên dùng "Hừ hừ".

13. **BOI_CANH_SAI (thấp)**. "như súng liên thanh bắn liên hồi" (0155:255); "người phụ nữ" xuất hiện 48 lần/26 file trong bối cảnh tu tiên.

14. **SAI_NGHIA (thấp)**. 0308:69: 竟是这个 dịch thành "Lại là thứ này sao?". Nên là "Hóa ra là thứ này?"

15. **TIEU_DE / DINH_DANG (thấp)**.
   - 23 tiêu đề thiếu dấu ":" và viết hoa cả cụm, ví dụ "Chương 18 Tình Báo Lâm Gia".
   - Ngoặc kép lẫn hai kiểu "" và “” (55 file chỉ dùng “”, 9 file trộn cả hai).

## Gợi ý

- Sinh cast.json với mốc xưng hô cho các cặp chính: Triều Vũ với Minh Chính, Triều Vũ với Phương Dật (nói công khai và lúc riêng tư), tiểu thư với Tiểu Liên.
- Auto-glossary: nếu tên mới chỉ lệch 1 chữ so với mục có sẵn và cùng ngữ cảnh thì đề nghị gộp. Cho phép ánh xạ lỗi gõ của tác giả về tên chuẩn.
- Kiểm tra phiên âm glossary bằng bảng Hán Việt, cảnh báo âm lạ hoặc tên bị đảo thứ tự.
- Bỏ luật bắt 嗯 dịch thành "Ân", cho dùng ừ/ừm theo ngữ cảnh.
- Thêm checkRules đếm calque ("một tầng", "không tính là", "chiếm tiện nghi", "đem…") và từ hiện đại khi setting là ancient.
- Prompt nhắc: gặp chơi chữ hoặc tên có nghĩa thì dịch nghĩa ngay trong câu để giữ miếng hài.
- Chuẩn hóa sau khi dịch: tiêu đề "Chương N: …" viết hoa chữ đầu câu, dùng một kiểu ngoặc kép.

## Làm tốt

- Đủ đoạn ở cả 308 chương, không sót chữ Hán.
- Arc ảo cảnh hiện đại tự chuyển sang anh/em/mẹ.
- Thoại giả nai của nữ chính và giọng chua ngoa của Nam Cung Tình rất sống.
- Lời thô tục và cảnh 18+ dịch thẳng, không nói giảm.
- 便宜爹 dịch thành "người cha hời", tránh được calque "tiện nghi".
