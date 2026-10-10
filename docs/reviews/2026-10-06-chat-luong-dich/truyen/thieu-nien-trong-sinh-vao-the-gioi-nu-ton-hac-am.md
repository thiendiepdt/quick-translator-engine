# Review: Thiếu Niên Trọng Sinh Vào Thế Giới Nữ Tôn Hắc Ám

- Slug: `thieu-nien-trong-sinh-vao-the-gioi-nu-ton-hac-am`. setting = `ancient`. Thể loại thực tế: huyền huyễn hắc ám **phương Tây** (quý tộc phong kiến, huyết tộc, tà thần Hy Lạp/Bắc Âu), nữ tôn, xuyên không. Truyện kể ngôi 3.
- Đã dịch 507 chương. Đọc trọn các chương 0001, 0255 (ở giữa), 0072 (dịch lại gần nhất) và 0508 (mới nhất). Lỗi nghi có tính hệ thống đều đã grep toàn bộ out/ để đếm.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.5 | Kẻ dưới vẫn xưng "ta" với chủ; Lore lúc là "hắn", lúc là "cậu" tuỳ cụm chương |
| Giọng văn | 6.5 | Giữ được chất u tối, nhưng Hán Việt dày ("nữ nhân", "thâm uyên", "huyết sắc quốc độ"), tiếng "Ân" lạc vào bối cảnh phương Tây |
| Dễ hiểu | 7 | Phần lớn đọc trôi, còn calque "Cùng với việc…/Theo…", "hai kiện thánh vật" |
| Khó khăn | 7 | Lóng và thành ngữ chuyển tốt; 呵 thành "Hừ" làm lệch sắc thái |
| Thuật ngữ | 5 | Glossary trùng và mâu thuẫn, phiên âm sai tên công quốc, biệt danh chính dịch hai kiểu |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_VAI_VE (cao, hệ thống: 64 lần/40 file)**. Ở 0002, dòng 53: `少爷，我应该跟您说过…` → "Thiếu gia, **ta** nhớ là đã từng nói với ngài…". Nên sửa thành "Thiếu gia, **tôi** nhớ đã từng dặn ngài…". Nữ hầu, nô lệ ("Thưa đại nhân, ta tên là Số 13", 0072/131) và kẻ cầu xin đều xưng "ta" với bề trên, như thể cặp nào cũng mặc định "ta–ngươi".
2. **THUAT_NGU_TROI (cao, hệ thống)**. 夜叉小姐 (raw 767 lần) được dịch thành "Dạ Xoa tiểu thư" 418 lần/77 file, lẫn với "tiểu thư Dạ Xoa" 246 lần/93 file (ví dụ 0072/337 so với 0255/57). Glossary không có mục cho biệt danh này.
3. **GLOSSARY_PHIEN_AM (cao, hệ thống: 127 lần/75 file)**. 棘罪 được phiên thành "Cực Tội", trong khi 棘 đọc là "**cức**" (kinh cức). Ví dụ 0001/49: "Công quốc Cực Tội". Nên đổi thành "Cức Tội", hoặc dịch nghĩa cho khớp với "Cây Gai".
4. **XUNG_HO_TROI (cao, hệ thống)**. Đại từ chỉ Lore trong lời kể: 78 chương dùng "cậu" (0046–0087, 0257–0265, 0337–0360…), các chương còn lại dùng "hắn". 5 chương trộn cả hai trong cùng một chương: 0063, 0072, 0258, 0261, 0466. Ở 0072, nửa đầu chương dùng "hắn", đến dòng 251 đổi thành "cậu cười khổ". cast.json không ghi mốc nào cho việc này.
5. **GLOSSARY_TRUNG (vừa, hệ thống)**:
   - 尤特克拉希尔 được dịch "Yggdrasil" 24 lần nhưng "Utkrasil" 3 lần (0258/139: "Thánh Sơn Utkrasil").
   - Ba tên 情与欲之神, 爱与欲之神 và 情欲之神 (gần như chắc là cùng một vị thần) thành ba bản dịch: "Thần Tình Cảm Và Dục Vọng", "Thần Tình Yêu và Dục Vọng", "Thần Tình Dục".
   - 休伯莉娅 và 休伯利娅 được dịch thành Huberia và Hyperia.
6. **AN_THAN_TU (vừa, hệ thống: thán từ "Ân" 36 lần/28 file)**. Ở 0069/63, `“嗯”了一声` → "khẽ đáp một tiếng “Ân”". Nên dùng "ừm". Lỗi lặp lại do setting `ancient` áp luật cổ trang Trung Hoa lên truyện phương Tây.
7. **CALQUE "nữ nhân" (vừa, 327 lần/156 file)**. Ở 0072/181: "Nữ nhân đứng trên bàn dài…" nên đổi thành "Ả đứng trên bàn dài…".
8. **GLOSSARY_PHIEN_AM (vừa, 19 lần/4 file)**. 鸩羽 được phiên thành "Trầm Vũ" (0508/41), đúng phải là "**Trậm** Vũ" (鸩 = trậm). Lỗi rơi đúng vào arc mới nhất.
9. **SAI_NGHIA (vừa, lỗi lẻ)**. Ở 0108/85, 亚斯兰古国 được dịch "cổ quốc **Islan**", đúng phải là **Aslan**. Lỗi này lẫn quốc gia cổ với họ của nhân vật chính, do glossary ghi sai.
10. **DAI_TU_MO_HO (vừa, lỗi lẻ)**. Ở 0001/67, lời kể gọi Eliza là "thọ thần của **ngài**", rồi ngay dòng 69 lại gọi "**mụ** ham sắc như mạng".
11. **HU_HA (vừa, "Hừ" 36 lần/34 file)**. Ở 0006/145, `“呵，她会喜欢的。”…发出一声轻笑` → "“Hừ, người sẽ thích thôi.” … bật ra một tiếng cười khẽ". 呵 là cười khẩy, không phải hừ giận. Thêm nữa, 她 (chỉ đại công) bị dịch thành "người" viết thường, nên câu đọc như "người ta sẽ thích".
12. **CALQUE 随着 (vừa, 'Cùng với việc' 26 lần/23 file, 'Theo việc/sự' 13 lần)**. Ở 0072/225: "Theo bản năng như trẻ sơ sinh của thiếu niên bú mớm dòng máu…" làm câu gãy. Nên viết: "Thiếu niên theo bản năng mút lấy dòng máu… như một đứa trẻ sơ sinh".
13. **THUAT_NGU_TROI (thấp, 18 lần/9 file)**. Ở 0255/113, 深渊 được dịch "thâm uyên", lệch với glossary "Vực Sâu" (504 lần).
14. **SAI_NGHIA (thấp, lỗi lẻ)**. Ở 0072/123, 梳理 được dịch "chải chuốt… suy nghĩ", đúng phải là "sắp xếp/gỡ rối".
15. **TIEU_DE (thấp, hệ thống)**. 89/366 tiêu đề viết hoa mọi chữ ("Chân Tướng", "Nấm Mồ Đế Quốc"), số còn lại viết hoa kiểu câu.

## Gợi ý

- Thêm setting `western_fantasy` (hoặc cờ tắt luật "嗯 → Ân" và luật Hán Việt dày) để truyện quý tộc phương Tây không bị khoá vào cổ trang.
- Đặt luật mặc định: kẻ dưới nói với bề trên xưng "tôi". Ghi cặp 安妮→洛尔 = "tôi–thiếu gia/ngài" vào addressing.
- Ghim đại từ người kể cho nhân vật chính (Lore = "hắn") trong cast.json. Cho lệnh `check` cảnh báo khi một chương dùng "cậu/hắn" lệch với mục đã ghim.
- Cho auto-glossary bắt cả danh xưng lặp lại nhiều lần (夜叉小姐), không chỉ tên riêng. Trước khi thêm mục mới, so khớp chuỗi con với mục đã có (尤特克拉希尔 ⊂ …圣山) để không sinh bản dịch thứ hai. Kiểm tra phiên âm Hán Việt và kiểm tra chéo tên đã có (亚斯兰 ≠ 伊斯蓝).
- Sửa tay glossary: 棘罪 → Cức Tội (hoặc dịch nghĩa), 鸩羽 → Trậm Vũ, 亚斯兰古国 → cổ quốc Aslan, xoá mục "Utkrasil", gộp các tên thần trùng nhau.
- Bổ sung style.avoid: "nữ nhân", "Cùng với việc/Theo việc", "thâm uyên", "huyết sắc". Ghi rõ 呵 = cười khẩy, không dịch là "Hừ".
- Chuẩn hoá cách viết hoa tiêu đề chương ở bước hậu xử lý.

## Làm tốt

- Không sót chữ Hán nào trong 507 chương; giữ đủ số đoạn.
- Tên phương Tây được chuyển đúng dạng Latinh (Lore, Anne, Eliza, Persephone…).
- Giữ được không khí hắc ám và nhịp nội tâm, nhất là đoạn mộng ở 0072 và cảnh Bình Nguyên Xám ở 0255.
- Lóng và thành ngữ chuyển tự nhiên: "Xong đời rồi" (完犊子), "bỏ đá xuống giếng" (落井下石).
- Khi Anne lật mặt và đổi từ 您 sang 你, bản dịch đổi theo từ "ngài" sang "cậu", giữ được sắc thái.
