# Review: Trường Sinh Là Mèo (truong-sinh-la-meo)

- **Thể loại:** setting `ancient`. Thực tế là tiên hiệp điền viên hài: mèo xuyên không trường sinh, theo chân nhiều đời nhà họ Triệu; về sau có tông môn, kiếm tu và nhiều vương triều. Truyện kể ngôi 3.
- **Phạm vi:** 268 chương có cả raw lẫn out. Đọc trọn 4 chương: 0001, 0134, 0267, 0268. Các lỗi nghi là hệ thống đều đã grep trên toàn bộ out/ để đếm.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Hứa Du lúc "hắn" lúc "nó". Hai chị em gọi cùng một người mẹ là "mẹ" và "nương". Triệu Tĩnh Văn khi "hắn", khi "chàng", khi "lão". Nhiều câu có hai chữ "hắn" chỉ hai người khác nhau. |
| Giọng văn | 7.5 | Văn mượt, ấm, giọng "Hứa gia" tự đắc giữ được. Hán Việt vừa phải. |
| Dễ hiểu | 7.5 | Ít văn convert. Riêng chương 75 bị bịa thêm đoạn, ngoài ra có vài calque lẻ. |
| Khó khăn | 6.0 | Meme cốt lõi bị dịch nghĩa đen (大实力, 成都). 嗯/唔 bị ép thành "Ân". |
| Thuật ngữ | 6.0 | Tên nhân vật nhất quán, nhưng glossary tự sinh bị bẩn và địa danh đảo trật tự lung tung. |

## Lỗi (nặng → nhẹ)

1. **THEM_Y (cao, lẻ). Ch75 dòng 43.** Raw chỉ có 411 ký tự lời tác giả, kết thúc ở `【猛喵比心~~】ღ(´･ᴗ･`)`. Bản dịch bịa thêm khoảng 4 đoạn: "Lảm nhảm thêm một chút cho đỡ vắng vẻ… xúc xích với pate cho mèo cũng tăng giá phi mã rồi các bác ạ… phiếu bé ngoan, nguyệt phiếu…". Tỉ lệ độ dài của chương này là 6,4, trong khi trung vị là 3,75.
   - Đề xuất: cắt toàn bộ phần thêm.
   - Vì sao: model tự viết ra nội dung, có lẽ để bù cho chương quá ngắn.
2. **MEME_NGHIA_DEN (cao, hệ thống: 20 lần/20 file raw).** Ch43: `【蓝大实力来了？】` → "【Lam Đại Thực Lực tới rồi?】".
   - Đề xuất: "Cô nàng bốc lửa họ Lam tới rồi à?"
   - Vì sao: 大实力 là lóng chỉ cô gái vòng một lớn. Lỗi còn bị glossary lưu thành tên riêng (蓝大实力, 程大实力) nên lặp mãi.
3. **MEME_NGHIA_DEN (vừa).** Ch1 dòng 215: `没去过成都，也不向往。` → "Chưa từng đến Thành Đô, cũng chẳng hề hướng tới."
   - Đề xuất: "Hắn thẳng như cột đình, chẳng có hứng thú với 'mấy chuyện đó'."
   - Vì sao: "Thành Đô" là meme chỉ đồng tính nam. Độc giả Việt đọc sẽ thấy vô nghĩa. 成都 còn bị lưu vào glossary places.
4. **XUNG_HO_TROI (cao, hệ thống).** Trong thoại, "mẹ" xuất hiện 140 lần, "nương" 105 lần; 15 chương dùng cả hai. Ví dụ ch124: dòng 103 Triệu Đạm Nguyệt nói "Mẹ thấy chưa", đến dòng 113 em trai gọi cùng người ấy là "Nương, nương xem tỷ con kìa".
   - Vì sao: glossary addressing được sinh theo từng cặp nhân vật, và cách này trái với toneRules ("cha"/"mẹ").
5. **XUNG_HO_TROI (vừa, hệ thống).** Hứa Du có khoảng 125 cụm "hắn" và khoảng 50 cụm "nó"; 23 chương trộn cả hai. Ví dụ ch244: raw `如果他不愿意去` → "nếu nó không muốn đi".
   - Vì sao: story.json chưa chốt đại từ cho nhân vật chính.
6. **XUNG_HO_TROI (vừa).** Triệu Tĩnh Văn trong ch110–169 được gọi là hắn 9 lần, ông 1, chàng 1, lão 4. Ví dụ ch134 dòng 121: "làm nhiều việc cho chàng", trong khi lúc này ông đã là nhị thúc của những đứa cháu trưởng thành.
   - Đề xuất: dùng "ông".
7. **DAI_TU_MO_HO (vừa, hệ thống).** Ch268 dòng 171: "khẩu khí thật lớn, hắn còn chưa nếm qua linh khí của kiếm tu đâu". Chữ "hắn" ở đây là Trương Quyết Trần, nhưng cả chương cũng gọi Tần Quán Hải là "hắn", nên dễ hiểu ngược.
   - Đề xuất: đổi sang "y", hoặc lặp lại tên.
   - Ch1 dòng 205 có lỗi tương tự: "hắn" chỉ Triệu Khánh Phong, xen giữa những chữ "hắn" chỉ Hứa Du.
8. **XUNG_HO_VAI_VE (vừa).** Ch134 dòng 129: ngục tốt tự xưng "thảo dân" với điển sử, trong khi raw không có tự xưng nào.
   - Đề xuất: "tiểu nhân".
9. **NGUOI_CHUNG (vừa, hệ thống).** Có 233 dòng lời kể chứa "ngươi". Ch267 dòng 57: `人的死活你都不管` → "Đến sống chết của người ta ngươi còn chẳng màng…". Lỗi tương tự ở ch17, 102, 131, 195.
   - Đề xuất: dịch thành "người ta", "ai", hoặc lược bỏ.
10. **AN_THAN_TU (vừa, hệ thống: "Ân" 22 lần/21 file, "Ừ" chỉ 3).** Ch32: `唔……似乎又是个男孩。` → "Ân...... hình như lại là một bé trai." Câu này là nội tâm của con mèo mang đầu óc hiện đại.
    - Đề xuất: "Ừm…"
11. **BAN_THAN (thấp, hệ thống: "bản thân hắn/nó/…" 51 lần/42 file).** Ch169: "bản thân hắn vậy mà lại bị quất vào mông?"
    - Đề xuất: "Hứa gia mà lại bị quất vào mông?"
12. **THUAT_NGU_TROI (thấp, hệ thống).** "Hoài Thủy huyện" 56 lần, "huyện Hoài Thủy" 29 lần; "Bắc Dương quận" 24, "quận Bắc Dương" 12. 附庸 lúc dịch "phụ dung" (12), lúc "phụ thuộc" (22).
    - Vì sao: glossary trộn hai trật tự từ cho địa danh.
13. **GLOSSARY_DANH_XUNG (thấp).** Glossary đang lưu 老许 = "Lão Hứa" (còn bị đưa vào cast.json), 甄子丹, 成都 và các mục 大实力 như tên riêng.
14. **CALQUE (thấp, lẻ).** Ch134 có ba chỗ:
    - "đường chết một dải" (死路一条)
    - "khó năng khả quý" (难能可贵)
    - "đặc biệt tới hướng cô nương cáo biệt" (特来向你告别)

    Ngoài ra "kim sắc lôi long" xuất hiện ở 12 file.
15. **TIEU_DE (thấp).** Tiêu đề ch76 là "Con đường lên tiên (Cầu đặt mua đầu!!!)", tức lời xin vé của tác giả lẫn vào tiêu đề. Ch75 là chương lời tác giả nhưng vẫn được dịch như một chương truyện.

## Gợi ý

- Kiểm tra tỉ lệ độ dài out/raw theo từng truyện và gắn cờ chương vượt khoảng 1,4 lần trung vị. Prompt cần cấm tuyệt đối việc viết thêm nội dung, kể cả khi chương ngắn.
- Nhận diện lời tác giả (上架感言, 求首订, 月票) để bỏ qua hoặc đánh dấu. Cắt đuôi "(求…)" trong tiêu đề chương.
- Glossary tự sinh không được lưu meme, lóng, cách gọi 老X/小X hay tên người thật. Dọn 5 mục hiện có.
- Thêm luật vào prompt: lóng mạng hoặc meme thì diễn ý tương đương tiếng Việt, không dịch từng chữ.
- Chốt quy ước địa danh ("huyện X, quận X, trấn X, vương triều Đại X") và chuẩn hoá lại glossary.
- Addressing cần một mục chung cho cả gia tộc (gọi mẹ là "mẹ"), đồng bộ với toneRules.
- Chốt đại từ cho Hứa Du ("hắn"). cast.json nên có mốc tuổi hoặc đại từ cho nhân vật sống qua nhiều thế hệ.
- Bỏ luật bắt buộc 嗯 → "Ân", cho phép "Ừ/Ừm/Vâng".
- 你 chỉ người chung chung trong lời kể thì không dịch là "ngươi". 自己 trong lời kể nửa trực tiếp thì dịch theo giọng nhân vật.

## Làm tốt

- Giọng 【Hứa gia】 hài và tự đắc, giữ được chất của truyện.
- Văn kể mượt, cảm xúc ở các cảnh tử biệt truyền tải tốt.
- Tên nhân vật, cảnh giới, tông môn nhất quán suốt 268 chương. Không sót chữ Hán. Tiêu đề chương viết hoa đúng kiểu câu.
- Xử lý khéo lỗi đánh máy trong raw (璇玑镜, 太青山).
- Thoại phân vai rõ: sơn phỉ, thương nhân, tiểu nhị, quan lại đều có giọng riêng.
