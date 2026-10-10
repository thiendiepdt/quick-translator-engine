# Review: Trọng Sinh Thành Mèo: Thân Phận Mèo Lão Đại Của Ta Bị Lộ Ra Ánh Sáng Toàn Mạng

- **Setting:** `modern`. Thực tế truyện là hài đô thị (show livestream thú cưng, đạn mạc); từ khoảng ch.100 chuyển sang arc giang hồ mèo chó chuột, từ ch.200 trở đi có thêm tu tiên (Ngự Thú Tông).
- **Ngôi kể:** ngôi 3, nên luật "người kể xưng ta" không áp dụng.
- **Đã dịch:** 228 chương.
- **Chương đã đọc trọn:** 0001, 0115, 0227, 0228. Ngoài ra đã grep toàn bộ out/ (raw và out khớp số đoạn 100%).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Các cặp chính ổn định. Nhưng đạn mạc cư dân mạng xưng "ta" ở 396 dòng/85 file, còn mèo gọi cô chủ là "người phụ nữ kia". |
| Giọng văn | 6.5 | Văn mượt, đúng chất hài. Vẫn rải nhiều cụm Hán hoặc convert: "đạn mạc", "phòng phát sóng trực tiếp", "Chuột Vương", "thuận lý thành chương". |
| Dễ hiểu | 7.5 | Ít sót ý. Còn calque "đem…/theo việc…/bản thân", vài chỗ lặp từ, một chỗ sai sắc thái nghĩa. |
| Khó khăn | 7.0 | Nhiều meme dịch tốt. Vài meme bị dịch nghĩa đen, và câu cửa miệng "女人，你…" mất chất trêu. |
| Thuật ngữ | 5.5 | 乐哥 trôi giữa "anh Lạc" và "Lạc ca". 直播间/弹幕/鼠王 mỗi từ dịch hai kiểu. Glossary có phiên âm sai và cặp xưng hô tự mâu thuẫn. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_VAI_VE (cao, hệ thống, 396 dòng/85 file).** Đạn mạc dịch 我 → "ta".
   - Ví dụ ch.0011 dòng 71: `我现在投胎成为猫，还有机会吗？` → "Bây giờ ta đi đầu thai làm mèo thì còn cơ hội không?"
   - Đề xuất: "Giờ tui đầu thai làm mèo thì còn kịp không ta?"
   - Vì sao: cư dân mạng hiện đại mà xưng "ta" nghe như cổ trang. Nghi luật "ta" của người kể bị áp nhầm cho mọi lời nói ngôi 1.
2. **THUAT_NGU_TROI (cao, 303 lần/32 file).** 乐哥 bị dịch "Lạc ca" thay vì "anh Lạc" như glossary đã ghim (anh Lạc xuất hiện 939 lần/151 file).
   - "Lạc ca" xuất hiện theo cụm: 0021–0027, 0057–0066, 0160–0165, và trọn đợt mới nhất 0219–0228.
   - Ví dụ ch.0228 dòng 19: "…Lạc ca đã gặp phải tai nạn xe cộ" → nên là "anh Lạc đã gặp tai nạn xe".
3. **THUAT_NGU_TROI (vừa, hệ thống).** Hai thuật ngữ đinh của truyện chưa được chốt.
   - 直播间: "phòng phát sóng trực tiếp" 177 lần/85 file, "phòng livestream" 80 lần/33 file.
   - 弹幕: "đạn mạc" 141 dòng, "bình luận" 30 dòng.
   - Ví dụ ch.0115 dòng 3: nên viết "cả buổi livestream ai nấy đều nín thở".
4. **CALQUE (vừa, 72/94 dòng có 女人).** 女人 → "người phụ nữ kia/này".
   - Ví dụ ch.0163 dòng 89: "Này người phụ nữ kia, cô chắc chắn hộp đồ hộp này không bị hỏng đấy chứ?" (câu còn lặp "hộp đồ hộp"). Đề xuất: "Này cô kia, chắc là hộp này chưa hỏng đấy chứ?"
   - Ví dụ ch.0001 dòng 41: "Người phụ nữ này, cô muốn làm bản miêu ngạt chết đấy à!" Đề xuất: "Cô gái à, cô định làm bản miêu chết ngạt luôn hả!"
   - Cách dịch này trái toneRules, vốn bảo gọi cô chủ là "cô ta".
5. **MEME_NGHIA_DEN (vừa, lẻ).** Ch.0115 dòng 153: `你大腿上还缺挂件吗` → "đùi cô còn thiếu món đồ trang trí nào nữa không?"
   - Đề xuất: "chị còn thiếu người ôm đùi không ạ?"
   - Bản hiện tại vô nghĩa và dễ bị hiểu lệch sang ý dung tục.
6. **GLOSSARY_PHIEN_AM (vừa, 22 lần/5 file).** 花很多 → "Hoa Hứa Đa".
   - 很 đọc Hán Việt là "ngận", không phải "hứa".
   - Đây là tên chơi chữ (tiêu nhiều tiền), nên dịch nghĩa sẽ hay hơn.
7. **CAST_MAU_THUAN (vừa).** Glossary addressing tự mâu thuẫn.
   - 吴宇→吴洪 ghi "anh–em", nhưng 吴洪→吴宇 lại ghi "tôi–anh".
   - 加菲→雄霸 ghi "tôi–anh", trong khi 辉腾→雄霸 ghi "em–anh".
   - toneRules cho phép "nàng/nữ nhân này", trái với chính style.avoid.
   - cast.json.addressing chỉ chứa danh xưng, không có cặp xưng hô hay mốc chương.
8. **GLOSSARY_HAN_VIET_NANG (thấp, 62 lần/7 file).** 鼠王 → "Chuột Vương".
   - Ch.0103 còn dịch "Vua chuột"; auto-glossary chốt dạng Hán từ ch.0104, gây trôi.
9. **CALQUE (thấp, hệ thống).** "đem" xuất hiện 51 lần/43 file; "theo việc/theo sự" 7 lần/7 file.
   - Ví dụ ch.0228 dòng 151: "Quả nhiên theo việc toàn bộ chuột…" → nên là "Quả nhiên, chuột… vừa đồng loạt giơ chân lên".
10. **DINH_DANG (thấp).** 434/2269 dòng đạn mạc trong 25 file bị bọc ngoặc kép, trong khi raw không có.
    - Ví dụ ch.0001 dòng 103.
11. **TIEU_DE (thấp).** Tiêu đề dịch từng chữ.
    - Ch.0115: "Tay nhỏ cần kiệm quán xuyến việc nhà" (小能手 bị dịch thành "tay nhỏ"). Đề xuất: "Bé mèo đảm đang tằn tiện".
    - Ch.0012: "Có ta một miếng thịt ăn, thì có mày một ngụm canh uống".
    - 3 tiêu đề viết hoa mọi chữ.
12. **BAN_THAN (thấp).** 160/727 dòng có 自己 dịch thành "bản thân".
    - Ví dụ ch.0228 dòng 99: "nghe thấy bản thân chỉ nhận được số phiếu này" → nên là "nghe mình chỉ được ngần ấy phiếu".
13. **SAI_NGHIA (thấp, lẻ).** Ch.0227 dòng 85: 交待 → "lời giải thích tốt nhất cho chính bản thân mình".
    - Đề xuất: "thế mới là không phụ chính mình".
14. **HU_HA (thấp, 5/23 dòng có 呵).** Ch.0131 dòng 43: 呵呵 → "hừ hừ"; nên là "hì hì".

## Gợi ý

- **Luật xưng "ta":** prompt chỉ áp cho người kể ngôi 1. Ghi rõ: đạn mạc và cư dân mạng hiện đại xưng tôi/mình/tui. Truyện kể ngôi 3 thì không đưa luật này vào prompt.
- **Hậu kiểm mục pinned:** thêm checkRule tự động. Raw có 乐哥 mà out có "Lạc ca" thì báo lỗi, áp dụng cho mọi mục pinned.
- **Chốt sẵn thuật ngữ livestream:** 直播间 → "buổi livestream", 弹幕 → "bình luận", 鼠王 → "Vua Chuột". Khi auto-glossary thêm mục mới, đối chiếu với các bản dịch đã có trong out/ để không sinh dạng thứ hai.
- **Sửa toneRules:** bỏ "nàng/nữ nhân này". Thêm ví dụ "女人，你… → Này cô kia/Cô gái à…", cấm "người phụ nữ này/kia".
- **Cặp xưng hô trong auto-glossary:** kiểm tra đối xứng A→B với B→A. Chuyển các cặp xưng hô sang cast.json, kèm mốc chương.
- **Phiên âm tên:** kiểm bằng bảng tra Hán Việt. Cho dịch nghĩa với tên chơi chữ, ví dụ 爱喝水的杯子 → "Cái Cốc Thích Uống Nước" thay cho "Ái Hát Thủy Đích Bôi Tử".
- **Định dạng đạn mạc:** giữ đúng dạng `: nội dung` như raw, không thêm ngoặc kép.
- **Ghi arc theo mốc chương trong story.json:** hiện chỉ có một giá trị `modern`, trong khi từ khoảng ch.200 có tu tiên.

## Làm tốt

- Không sót chữ Hán. Số đoạn khớp 100% với raw.
- Xưng hô cặp chính ổn định: Vũ Tình–Tiểu Lạc "chị–em"; đàn em gọi Lâm Lạc "anh"; Lâm Lạc với đàn em "tao–mày". Ngô Dụng tự xưng "Dụng", đúng giọng quân sư.
- Meme dịch tốt: 壕无人性, 穿小鞋, 懵逼, "chẳng hiểu ất giáp gì". Giữ được chơi chữ 笑的合不拢腿 → "cười đến không khép được chân".
- 嗯 dịch theo ngữ cảnh ("Ừm/Dạ/vâng"), không có "Ân". Không dùng "ngươi" cho người chung chung.
- Arc giang hồ và tu tiên chuyển giọng hợp lý, không tràn sang phần show đô thị.
