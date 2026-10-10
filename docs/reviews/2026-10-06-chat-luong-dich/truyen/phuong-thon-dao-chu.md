# Review: Phương Thốn Đạo Chủ (phuong-thon-dao-chu)

Bối cảnh: tu tiên/huyền huyễn cổ đại (setting `ancient`), kể ở ngôi 3. Đã dịch 393 file. Chương đã đọc trọn: 0001, 0197 (ch191), 0391 (ch381), 0393 (ch384). Chương 0392 chỉ đọc được bản dịch vì raw đã thay đổi (xem lỗi 1).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Xưng hô giữa người với người ổn (Lý Thuận–lão Phùng, Khổng Chiêu gọi "đệ", lão tổ tông là "người"); đại từ chỉ thần 祂 trôi giữa Ngài/Người/hắn |
| Giọng văn | 7.5 | Đúng chất tu tiên trầm ổn, Hán Việt vừa phải; vài thành ngữ còn dịch chữ |
| Dễ hiểu | 7.5 | Đủ đoạn, ít sót ý; còn calque "đem…", "Theo sự…", vài chỗ sai nghĩa |
| Khó khăn | 7 | Giữ được cảm xúc và nhịp đánh nhau; thán từ đơn điệu, có câu vụng |
| Thuật ngữ | 5.5 | Glossary tự sinh có mục tự mâu thuẫn và mục là từ thường; thuật ngữ cốt lõi dịch 2–3 kiểu |

## Lỗi (nặng → nhẹ)

1. **KHAC, cao, hệ thống. raw/out lệch nhau.** raw/ được tải lại sáng 06/10 (07:39). out/0392 là "Chương 382: Vị trí các chòm tinh tú" trong khi raw/0392 là `第383章 道合陷囹圄`. Ch382 không còn trong raw, ch383 chưa được dịch mà state vẫn ghi "done". out/0364–0378 mang tiêu đề của chương kế (vd out/0378 "Cực cảnh gặp Ngoại Thần", raw là 千身齐狩猎). Ở 0370/0371, đơn xin nghỉ bị hoán đổi.
2. **GLOSSARY_TRUNG, cao.** 0360:121 `将沉默吸干` → "hút cạn Trầm Mặc". 沉默 ở đây là lỗi gõ của tên 陈默 (Trần Mặc). Auto-glossary tạo mục tên riêng "沉默 => Trầm Mặc", vừa tách một nhân vật làm hai, vừa đưa từ thường "im lặng" (127 lần trong raw) vào glossary tên riêng. Đề xuất: "Trần Mặc".
3. **XUNG_HO_TROI, cao, hệ thống.** 祂 xuất hiện 309 lần trong raw. Bản dịch dùng "Ngài" 161 lần/55 file, "Người" 14 lần ở 0380, còn 0393:85 thì `祂赐福万灵` → "Hắn ban phúc cho vạn linh". Cần chốt một đại từ.
4. **GLOSSARY_TRUNG, vừa.** Glossary tự mâu thuẫn: 十二长生 => "Mười Hai Trường Sinh" nhưng 十二长生法 => "Thập Nhị Trường Sinh Pháp". Kết quả là 76 lần "Thập Nhị" so với 9 lần "Mười Hai" (vd 0092:143). Tương tự, 与天借寿 có hai bản: "Dữ Thiên Tá Thọ" và "Vay thọ cùng trời".
5. **GLOSSARY_DANH_XUNG, vừa.** Từ thường 天地 bị đưa vào items => "Thiên Địa" nên bị viết hoa giữa câu 220 lần/91 file, trong khi "trời đất" xuất hiện 511 lần. Vd 0393:13 "hợp cùng Thiên Địa" → nên là "hợp cùng trời đất". Các mục cùng loại: 故人, 同归, 造化, 天象.
6. **THUAT_NGU_TROI, vừa.** 玄黄气 (371 lần) không có mục glossary riêng, nên có ba cách viết: "Huyền Hoàng Khí" 308 lần, "khí Huyền Hoàng" 36, "Huyền Hoàng khí" 20. Vd 0391:29.
7. **THUAT_NGU_TROI, vừa.** 0197:3 viết "võ đạo thế giới" theo trật tự Hán (25 lần/6 file) trong khi glossary là "thế giới Võ Đạo". 武道世界 xuất hiện từ ch159 nhưng đến ch263 mới được chốt. 重置 cũng trôi giữa "thiết lập lại" và "tái lập".
8. **CALQUE, vừa, hệ thống.** 0152:125 `随着…散去` → "Theo sự tan biến của dòng chữ cuối cùng này…" ("Theo sự/Theo việc" 54 lần). 0166:125 "Mà đem lực chú ý tập trung vào…" ("đem" 134 lần/105 file).
9. **SAI_NGHIA, vừa.** 0197:115 `老祖宗的口风` → "cách ăn nói của lão tổ tông". Nên là "khẩu khí" (ý tứ, lập trường).
10. **SAI_NGHIA, thấp.** 0391:61 `与虎谋皮` → "mưu sự với hổ" làm mất vế "lột da". Nên là "bàn với hổ chuyện lột da hổ".
11. **CALQUE, thấp.** Thành ngữ dịch chữ: 0197:35 "họa lây nơi ao cá" (池鱼之殃 → "vạ lây"); "vì nguyên cớ…" (的缘故); ch1 "thành trì đầy vết thương" (满目疮痍).
12. **KHAC, thấp, hệ thống.** "mà thôi" xuất hiện 92 lần/78 file dù bị cấm trong style.avoid, vd 0197:23. Luật này không được kiểm.
13. **CALQUE, thấp.** 0001:35 "Hắn dứt lời chưa dứt" là câu tự mâu thuẫn. Đề xuất: "Lời hắn còn chưa dứt".
14. **VAN_KHO, thấp, hệ thống.** 嗯？ được dịch thành "A?" 65/73 lần cho mọi sắc thái (vd 0038:105). Nên linh hoạt: Hửm?/Hả?/Ừ. Lỗi "Ân" gần như không có (5 lần).
15. **DINH_DANG, thấp.** Ngoặc thẳng dùng ở 238 file, ngoặc cong ở 157 file, 9 file trộn cả hai (0001:157). Dấu lửng "......" và "..." dùng lẫn lộn.

## Gợi ý

- App: lưu hash raw lúc accept; khi raw bị tải lại mà hash đổi thì đánh dấu chương "stale", kể cả tiêu đề.
- Auto-glossary: thêm danh sách stopword cho từ thường (天地, 沉默, 故人…). Tên đồng âm khác một chữ (沉默/陈默, 吕锋/吕峰) thì gộp thành alias.
- Lint glossary: báo mục cha/con mâu thuẫn; với cụm có tần suất cao mà chưa có mục (玄黄气) thì tự đề xuất khi out đang có từ 2 cách dịch trở lên.
- Thêm luật đại từ cho 祂 vào story.json.
- checkRules bằng regex cho "Theo sự|Theo việc", "đem", "mà thôi".
- Thêm bước hậu xử lý chuẩn hoá ngoặc kép và dấu lửng.
- Prompt: thêm ví dụ cách dịch thán từ 嗯/呵.

## Làm tốt

- 391/393 file khớp số đoạn, không sót chữ Hán.
- Giọng tu tiên trầm ổn; Lý Thuận điềm tĩnh, lão Phùng dân dã.
- Cảnh gia đình ở ch345 tự nhiên ("Ba ơi, sao ba lại khóc thế ạ?").
- Xưng hô sư môn đúng vai vế.
- Không dùng từ cấm (cư nhiên, não hải…).
