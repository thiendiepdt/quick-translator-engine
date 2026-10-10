# Review: Huyền Giám: Thân hóa ngàn vạn, ngược dòng lúc thành đạo

- **Slug:** huyen-giam-than-hoa-ngan-van
- **Setting:** story.json không có genre.
- **Thể loại thật:** đồng nhân tiên hiệp Huyền Giám Tiên Tộc, xuyên việt cộng hệ thống phân hồn. Truyện dày thuật ngữ đạo thống, có niên biểu 【…ngươi…】 và lời tác giả viết giọng hiện đại.
- **Ngôi kể:** ngôi 3.
- **Số chương:** 80 chương đã dịch. Đọc trọn chương 1, 41, 79 và 80 (79–80 là hai chương dịch mới nhất). Lỗi nghi có tính hệ thống đều đã grep trên cả 80 chương.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Niên biểu dùng "ngươi", hội thoại dùng "đạo hữu" đều đúng. Nhưng 祂 dịch "Ngài" suốt ch47–78 thì sang ch79–80 thành "hắn", lẫn với "hắn" của người phàm. |
| Giọng văn | 7.5 | Hán Việt trang nhã, đúng style.voice. Vẫn còn văn convert. |
| Dễ hiểu | 7.0 | Không sót đoạn. Vài câu nội tâm nhiều thuật ngữ bị đọc sai cấu trúc. |
| Khó khăn | 7.0 | Thành ngữ dịch tốt. Thán từ bị ép khuôn, meme fandom dịch nghĩa đen. |
| Thuật ngữ | 6.5 | Glossary 630 mục được áp khá chặt, nhưng có phiên âm sai, mục tự mâu thuẫn và vài thuật ngữ cốt lõi bị trôi. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI – cao, hệ thống.** 祂 được dịch "Ngài" khoảng 130 lần trong 18 file (ch47–78). Đến ch79–80 có 34 chữ 祂 nhưng không còn chữ "Ngài" nào, toàn bộ thành "hắn".
   - ch79 dòng 5: 却被祂死死栓住 → "lại bị hắn ghìm chặt lại"
   - Đề xuất: "lại bị Ngài ghìm chặt".
   - Quy ước này chưa từng ghi vào story.json, nên đúng ở hai chương phản ánh prompt hiện tại thì nó rơi mất.

2. **DAI_TU_MO_HO – cao, hệ quả của lỗi 1.**
   - ch80 dòng 127, gốc: 祂像提鱼一般抓着忘川…取走了他…把他往渔篓里一甩，不再管他
   - Hiện tại: "Hắn xách Vong Xuyên như xách một con cá… mà hắn đã rèn luyện… rồi quẳng hắn vào trong giỏ cá, không đoái hoài tới hắn nữa."
   - Bốn chữ "hắn" chỉ hai người. Đề xuất: "Ngài xách Vong Xuyên… quẳng hắn vào giỏ cá, chẳng đoái hoài nữa."

3. **SAI_NGHIA – cao.**
   - ch79 dòng 17, gốc: 一个背叛了元府、只能龟缩在望月湖的金丹
   - Hiện tại: "Một kẻ phản bội Nguyên Phủ, chỉ có thể rụt cổ co cụm ở hồ Vọng Nguyệt như tên Kim Đan kia"
   - Kẻ phản bội và tên Kim Đan là cùng một người, bản dịch lại thêm phép so sánh. Đề xuất: "Một tên Kim Đan phản bội Nguyên Phủ, chỉ biết rụt cổ trốn ở hồ Vọng Nguyệt…"

4. **SAI_NGHIA – vừa.**
   - ch79 dòng 39, gốc: 如东方游亡余
   - Hiện tại: "táng mạng ở Dư Vị", tức biến cụm này thành địa danh.
   - Nghĩa đúng là chết khi cầu dư vị.

5. **SAI_NGHIA – vừa, nằm trong glossary.signature_phrases.**
   - ch1 dòng 61: 山无陵 → "Núi không mòn", ngược với nghĩa gốc là núi bị san phẳng.
   - Đề xuất: "Núi không còn gò đỉnh".

6. **GLOSSARY_PHIEN_AM – vừa, hệ thống.** Ba chữ bị phiên âm sai:
   - 邺 → "Diệp" (5 lần, 2 file), đúng là Nghiệp: Nghiệp Cối, Bạch Nghiệp Đô Tiên Đạo.
   - 鹧 → "Chiết" (An Chiết Ngôn, 8 lần), đúng là Chá.
   - 瓒 → "Tán", đúng là Toản.
   - Riêng câu ch46 dòng 51 còn dịch 江北 thành "phía bắc sông", trong khi glossary đã có mục "Giang Bắc".

7. **THUAT_NGU_TROI – vừa.** ch41 dòng 7: 泓浵 (pháp kiếm chính của nhân vật chính) → "Hoành Đồng". Glossary ghi "Hồng Đồng", và các chương khác dùng "Hồng Đồng" 15 lần.

8. **GLOSSARY_TRUNG – thấp.** Glossary đọc cùng một chữ thành hai âm:
   - 窞: "Nhập Khảm Đảm" ở một mục, "Hành Đạm tính" ở mục khác.
   - 锃: "Đinh Uy Trừng" ở một mục, "Thiên Tu Tĩnh Kim Kinh" ở mục khác.

9. **GLOSSARY_DANH_XUNG – vừa.** 钓鱼佬 là biệt danh fandom của Tiêu Sơ Đình.
   - ch46 dịch "lão câu cá".
   - ch62 dòng 91 thành tên riêng "Điếu Ngư Lão" vì glossary khoá như vậy.
   - Chuyện tương tự ở ch1 dòng 35: 人人有牢坐 dịch nghĩa đen thành "ai nấy đều có ngục để ngồi".

10. **AN_THAN_TU – vừa, hệ thống.** 4 lần trong 4 file.
    - Không chỉ 嗯 mà cả 唔 (ch2: "Ân, món này ngon đây") và 额 cũng bị dịch thành "Ân".
    - Nặng nhất là ch68 dòng 9, trong lời tác giả viết giọng hiện đại: 额，能不能爆更 → "Ân, có bạo chương được hay không".
    - Đề xuất: "ờm, có bùng chương được không".

11. **HU_HA – vừa, hệ thống.** 呵 → "Hừ" 6 lần trong 5 file.
    - Ví dụ ch79 dòng 63: "Hừ, Thanh Huyền bọn họ…"
    - Cùng chương, 哼 cũng ra "Hừ", nên sắc thái cười khẩy và bực tức bị gộp làm một.

12. **CALQUE – vừa, hệ thống.** "đem" xuất hiện 66 lần trong 39 file.
    - ch41 dòng 75: "đem cả tính mạng giao cho hắn sai sử".
    - Đề xuất: "sẵn sàng giao cả mạng cho hắn sai khiến".

13. **CALQUE – thấp.** ch41 dòng 97: "Hài tử ngươi đúng là vận khí tốt thật". Cũng thấy "tắc tắc lấy làm kỳ lạ", "một vinh cùng vinh, một tổn cùng tổn".

14. **DAI_TU_MO_HO – thấp.** ch41 dòng 71: câu trước "hắn" là Ninh Tùy, câu này "Hắn chẳng hề để tâm xua tay" lại là Huyền Lư.

15. **THUAT_NGU_TROI – thấp.** Viết hoa không thống nhất:
    - "long thuộc" (65 lần) và "Long thuộc" (12 lần)
    - "thái hư" và "Thái Hư"
    - "thăng dương" và "Thăng Dương"

## Gợi ý

- **Ghi quy ước 祂 vào story.json.** 祂 → "Ngài", 祂们 → "các Ngài/chư vị". App nên cảnh báo khi chương có 祂 mà bản dịch không có chữ "Ngài" nào.
- **Bổ sung story.json.** Thêm genre.setting, và thêm vào glossary các thuật ngữ cốt lõi còn thiếu: 龙属, 妖属, 释修, 亡余/求余, 收位不余.
- **Luật thán từ.**
  - "Ân" chỉ dùng cho 嗯 trong thoại cổ trang.
  - 唔/额 và mọi thán từ trong lời tác giả dịch "ừm/ờ".
  - 呵 dịch "ha/cười nhạt"; chỉ 哼 mới là "Hừ".
- **Lời tác giả (PS, cảm nghĩ lên kệ) dùng ngữ vực hiện đại riêng.** Lóng và meme dịch theo nghĩa, không đưa vào glossary làm tên riêng.
- **Auto-glossary cần kiểm phiên âm.** Một chữ chỉ được có một âm trong toàn glossary; soát lại nghĩa của signature_phrases.
- **checkRules.** Thêm ngưỡng cho "đem", "Hài tử", "tắc tắc". Nhắc model nêu tên khi chủ thể "hắn" đổi giữa hai câu liền nhau.

## Làm tốt

- Không sót chữ Hán trong 80 chương; giữ đúng format 【】 và 《》.
- Niên biểu dùng "ngươi" nhất quán. 你 chỉ người chung chung trong lời kể được dịch "người ta".
- Glossary lớn nhưng tên người, công pháp, quả vị gần như được áp đúng.
- Thành ngữ dịch tự nhiên: "hạ bùa mê thuốc lú", "ngồi trên núi xem hổ đấu", "mang ngọc mắc tội".
- Văn tả cảnh mạt thế có nhịp, giàu hình ảnh.
