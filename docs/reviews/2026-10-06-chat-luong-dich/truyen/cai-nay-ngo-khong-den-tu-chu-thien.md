# Review: Cái này Ngộ Không đến từ chư thiên

- **Truyện:** slug `cai-nay-ngo-khong-den-tu-chu-thien`, setting `mixed`, kể ngôi 3, đã dịch 83 chương.
- **Thể loại thật:** đồng nhân Tây Du x Dragon Ball. Arc mở đầu ở núi Ngũ Hành, sau đó gần 80 chương ở thế giới Dragon Ball; chương 83 là lời tác giả.
- **Chương đã đọc:** 0001, 0041, 0081, 0082 (+0083). Mọi file out/ có cùng mtime nên hai chương "mới nhất" được chọn theo `updatedAt` trong state.json.
- **Đánh số dòng:** `dong` = chỉ số dòng không trống, dòng 0 là tiêu đề.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| xung_ho | 6.5 | Ngộ Không giữ "lão Tôn/ta–ngươi" rất chắc. Yamcha và Krillin gọi Ngộ Không lúc "ngươi", lúc "cậu", lúc "huynh", có câu còn để nguyên "cậu/huynh". Addressing trong glossary đã lỗi thời. |
| giong_van | 8 | Giọng hào sảng đúng chất Tây Du, cảnh Dragon Ball đủ hiện đại. Còn vài từ convert ("nghẹn khuất", "khí diễm"). |
| de_hieu | 8.5 | Dịch sát, đủ đoạn, không sót ý; "đem…" còn ở 22 file. |
| kho_khan | 7.5 | Phần lớn thán từ dịch tốt. Lệch: 呔 thành "Thối!", 嘿嘿 thành "Hì hì", 矮冬瓜 thành "Quả dưa lùn". |
| thuat_ngu | 7 | Glossary được áp nhất quán. Hệ tên lai Hán Việt/tên Anh khiến một nhân vật tách đôi; glossary có mục trùng và lẫn tên người ủng hộ. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống).** Yamcha gọi Ngộ Không là "ngươi" 5 lần/4 file nhưng "cậu" 4 lần/4 file.
   - ch61: `你认识那老头吗？` → "Ngộ Không, **ngươi** quen biết lão già đó sao?"
   - Ở ch40 và ch79 cùng cặp này lại dùng "cậu".
   - Nguyên nhân: glossary ghi `雅木茶→悟空: ta–ngươi` và không có mốc.
2. **XUNG_HO_VAI_VE (vừa, hệ thống).** ch80: `悟空/大师兄！…等你再回来` → "Ngộ Không / đại sư huynh! … Đợi **cậu/huynh** quay lại…". Dấu gạch chéo của gốc bị chép thẳng vào đại từ.
   - Đề xuất: tách thành hai lời gọi rồi dùng một đại từ chung.
3. **CAST_MAU_THUAN (vừa, hệ thống).** Glossary ghi `孙悟空→龟仙人: lão Tôn–ngươi`, nhưng bản dịch từ ch73 đã là "Sư phụ… hỏi **người**". Ở đây bản dịch đúng, glossary sai.
   - Addressing không có mốc, nên model khi làm theo glossary, khi tự sửa.
4. **CALQUE "nghẹn khuất" (vừa, 6 lần/5 file).** ch82: `憋屈！无比的憋屈！` → "Nghẹn khuất! Nghẹn khuất khôn cùng!"
   - Đề xuất: "Ấm ức! Ấm ức không để đâu cho hết!"
5. **CALQUE "đem" (vừa, 26 lần/22 file).** ch5: "Ngươi cứ yên tâm **đem** cái bí mật… kể hết cho ta nghe thế sao?"
   - Đề xuất: "Ngươi yên tâm kể hết cho ta nghe cái bí mật…"
6. **GLOSSARY_PHIEN_AM (vừa, 4 lần/3 file).** 呔 → "Thối!", kể cả tiêu đề "Chương 3 Thối! Yêu quái!". Lỗi do style và signature_phrases ép.
   - Trong tiếng Việt "Thối" là hôi thối. Đề xuất "Hây!" hoặc "Này!".
7. **DINH_DANG (vừa, 30 file).** Chương 11–40 dùng ngoặc thẳng `"…"`, các chương còn lại dùng `“…”`.
8. **MEME_NGHIA_DEN (thấp, 6 lần/3 file).** 矮冬瓜 → "Quả dưa lùn"; 冬瓜 là bí đao, nghĩa thật là chửi "đồ lùn".
   - Đề xuất: "Đồ lùn tịt".
9. **THUAT_NGU_TROI (thấp).** 猫仙人 → "Miêu Tiên Nhân" nhưng 卡林塔 → "tháp Karin", nên người đọc không nối được hai cái tên. Quy Tiên Nhân, Vũ Thiên lão sư cũng lệch hệ với Krillin, Yamcha.
10. **GLOSSARY_TRUNG (thấp).** Có nhiều mục trùng: 四星球/四星珠, 六字真言贴/帖, 普尔/普洱. Bốn tên người ủng hộ trong lời tác giả (极光, 南宫, 帝雨, 与珏) bị đưa vào glossary.names.
11. **KHAC – giọng cười (thấp).** 嘿嘿 của Ngộ Không: "Hì hì"/"Hì" 24 lần, "Hắc hắc"/"Hắc" 14 lần.
    - ch10: "“Hì hì.” Ngộ Không vẫn cợt nhả" nghe mềm và con gái, không hợp Đại Thánh.
12. **DAI_TU_MO_HO (thấp, lẻ).** ch70: `师父他老人家` → "**sư phụ người** lại có quá khứ huy hoàng", đọc thành "sư phụ của ngài".
    - Đề xuất: "sư phụ ta".
13. **TIEU_DE (thấp, 10 file).** Chương 1–10 viết "Chương 1 Ngộ Không: …", thiếu ":" sau số chương; các chương sau đều có.
14. **DINH_DANG (thấp, 169 lần/54 file).**
    - `——！` thành "— !" có dấu cách trước "!".
    - Thán từ phiên âm cứng: "Ách a — !" (呃啊), "Ôm!" (嗡, ch82).

**Đối chiếu các lỗi đã biết:**
- 嗯 → "Ân": 0 lần; 嗯 được dịch là "Ừm" 29 lần.
- "bản thân hắn": 7 lần/7 file, thưa.
- 你 chung chung → "ngươi": không thấy.
- "Theo sự": 1 lần.
- "phiến", "tiện nghi": mỗi từ 2 lần, đều dùng đúng nghĩa tiếng Việt.
- "người phụ nữ": 4 lần, phần lớn hợp ngữ cảnh.
- 呵 → "Hừ": 2/14 lần.
- Không thấy lỗi khoá sai bối cảnh.

## Gợi ý

- **Addressing có mốc.** Lưu dạng `A→B: X (từ ch N)`, cập nhật khi quan hệ đổi, dọn lại sau mỗi arc.
- **Cấm dấu "/" trong lời thoại.** Thêm luật: gốc có "A/B" gộp lời hai người thì tách thành hai lời gọi.
- **Sửa luật 呔.** Đổi 呔 sang "Hây!"/"Này!" trong toneRules và signature_phrases.
- **Bổ sung danh sách calque cấm:** 憋屈, 把→"đem", 一片→"một mảnh", 气焰→"khí diễm".
- **Chốt một hệ tên cho đồng nhân anime** ngay lúc setup, thay vì để `names: mixed`.
- **Auto-glossary bỏ qua chương lời tác giả** (上架感言…) và gộp các biến thể khác nhau một chữ.
- **Hậu xử lý trong app:** chuẩn hoá ngoặc kép, dấu "— !", và định dạng tiêu đề "Chương N: …".
- **Bảng thán từ/tiếng cười cho nhân vật chính** trong style, để giọng cười không trôi giữa các chương.

## Làm tốt

- Đủ đoạn, sát nghĩa, văn mượt.
- Giọng Đại Thánh đạt: "lão Tôn", "lão già Như Lai", "cái hệ thống của nợ nhà ngươi".
- Hai bối cảnh tách rõ giọng; Bulma nói "tôi–cậu", "lão già dê xồm".
- Đã tránh được hầu hết lỗi cũ: không có "Ân", ít "bản thân hắn".
- Thuật ngữ hệ thống, tên chiêu và chú Lục Tự Chân Ngôn dịch nhất quán, đúng kiểu.
