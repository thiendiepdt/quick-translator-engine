# Review: Trọng Sinh Thành Cẩu: Ta Đến Canh Giữ Nhà Tắm Nữ

- Setting: modern. Thể loại thực tế: học đường đô thị hài, mỗi chương phá một vụ nhỏ, văn gốc toàn câu cụt. Kể ngôi 3.
- Đã dịch 12 chương. Đã đọc trọn: ch1, ch4, ch7, ch11, ch12 (ch11–12 là bản dịch mới nhất).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.0 | Giữa người với người thì chuẩn. Cách gọi Đại Hoàng trôi từ "chị–em" sang "mày/tớ" vì không có mốc người→chó |
| Giọng văn | 7.5 | Hiện đại, trẻ. Lọt một ít Hán Việt (đả thảo kinh xà, Khô Lâu, Đao Ba, chó sinh). Hay độn chữ |
| Dễ hiểu | 8.0 | Trơn, đúng nghĩa, đúng số. Còn vài calque (Toàn trường, Bạn học X, Đã là…) |
| Khó khăn | 7.0 | Lóng tốt. Mất chơi chữ 恩狗, catchphrase dịch hai kiểu, nhịp câu cụt bị kéo dài |
| Thuật ngữ | 7.0 | Tên nhất quán, không sót chữ Hán. Từ khoá lặp (外卖, 狗生巅峰, 守护犬, 打草惊蛇) chưa chốt |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống)**: ch12:403 `你说……内鬼会是谁？` → "Mày xem... kẻ nội gián có thể là ai được nhỉ?". Đề xuất: "Em nghĩ xem…". Lâm Tuyết Nhi gọi chó là "em", xưng "chị" suốt ch2–7, đến ch12 lại "mày"/"Tớ". Tô Tiểu Manh cũng từ "em" (ch6–8) chuyển sang "mày" (ch12:215).
2. **CAST_MAU_THUAN (vừa, hệ thống)**: `cast.json` có `"addressing": {}`. story.json có 12 cặp nhưng không cặp nào có 大黄. cast.json cũng thiếu 大黄 và 张浩. Đây là gốc của lỗi 1.
3. **THUAT_NGU_TROI 外卖 (vừa, hệ thống)**: "đồ ăn giao tận nơi" xuất hiện 8 lần, "đồ ăn giao tới" 7 lần. Ví dụ ch7:171 "túi đồ ăn giao tới?" → nên là "túi đồ ăn ship?". Cả tiêu đề ch6 và ch8 cũng cứng.
4. **THUAT_NGU_TROI 狗生巅峰 (vừa, hệ thống)**: dịch thành "Đỉnh cao chó sinh." (ch2, ch12) và "đỉnh cao của kiếp làm chó" (ch5, ch8). Đề xuất chốt "Đỉnh cao đời chó." trong signature_phrases.
5. **MEME_NGHIA_DEN (vừa)**: ch12:217 `救命恩狗` → "chú chó ân nhân cứu mạng". Chơi chữ 恩人→恩狗 bị mất, đề xuất "ân nhân… à nhầm, ân 'cẩu' cứu mạng". Ở ch5 cũng mất tương tự.
6. **CALQUE (vừa)**: ch7:61 `既然贼专偷女生外卖` → "Đã là kẻ chuyên trộm đồ ăn của nữ sinh, chúng ta…". Câu đọc thành "chúng ta là kẻ trộm". Đề xuất: "Bọn trộm đã chuyên nhắm đồ ăn của nữ sinh thì mình giăng bẫy luôn."
7. **DINH_DANG (vừa, hệ thống)**: ch1–10 dùng "......" 361 lần, còn ch11–12 dùng "...". Nên chuẩn hoá về một kiểu "...".
8. **THEM_Y (vừa, hệ thống)**: bản dịch độn chữ vào câu cụt. `结果令人震惊。` thành "khiến tất cả mọi người đều rùng mình chấn động". `等待。` thành "Kiên nhẫn chờ đợi.". `欣慰` thành "trong lòng cảm thấy an lòng" (còn lặp chữ "lòng").
9. **CALQUE (vừa)**: ch12:21 `全场寂静` → "Toàn trường im phăng phắc", trong khi cảnh chỉ có công an và bốn cô gái. Đề xuất: "Cả phòng lặng ngắt."
10. **CALQUE (thấp, 4 lần/4 file)**: dịch 同学 thành "Bạn học Lâm Tuyết Nhi". Đề xuất "Lâm Tuyết Nhi này". Câu công an nói nên dùng "em/sinh viên Trần Tư Tư".
11. **GLOSSARY_HAN_VIET_NANG (thấp)**: "bang Khô Lâu", "Đao Ba" cho băng nhóm hiện đại. Đề xuất "băng Đầu Lâu", "Thẹo".
12. **THUAT_NGU_TROI (thấp)**: 打草惊蛇 có hai cách dịch, "đả thảo kinh xà" (ch3, ch4) và "đánh rắn động cỏ" (ch11). Đề xuất chốt "rút dây động rừng".
13. **BOI_CANH_SAI (thấp)**: glossary signature_phrases ghi "để ngươi trải nghiệm cho đã". Bản dịch ch1 đã tự dùng "cậu" và lệch glossary. Cần sửa glossary.
14. **DINH_DANG (thấp)**: ở ch11, tag thoại viết thường sau dấu ” 11 lần, ví dụ "” cô cất giọng khe khẽ hỏi".
15. **XUNG_HO_NGUOI_KE (thấp)**: trong độc thoại nội tâm ch1, Đại Hoàng xưng lẫn "mình/ông đây/ta". Ở ch11:321–323 lại trộn "ông đây" với "tao".

## Gợi ý

- Auto-cast/glossary phải sinh mốc xưng hô cho cặp người→nhân vật phi nhân (thú cưng), và gộp addressing của story.json vào cast.json.
- Auto-glossary nên bắt cả các cụm thường lặp ≥2 chương: 外卖, 狗生巅峰, 校园守护犬 (đang có hai kiểu "chó hộ mệnh" và "chó bảo hộ khuôn viên trường"), 打草惊蛇.
- Thêm vào prompt modern các luật sau: biệt danh giang hồ dịch nghĩa thuần Việt; không dùng "Bạn học X"; 全场 không dịch thành "Toàn trường"; với văn câu cụt thì giữ câu ngắn, không độn.
- Hậu xử lý: chuẩn hoá dấu ba chấm và viết hoa tag thoại. Nên dịch lại hoặc sửa ch1–10.
- Cảnh báo khi glossary của truyện setting modern có chứa "ngươi/ta".

## Làm tốt

- Tên nhất quán, không sót chữ Hán, không có "Ân". "bản thân" và "đem" hiếm.
- Lóng tự nhiên: "Vãi", "Vãi chưởng", "đỉnh chóp", "ông đây", "nhảy aptomat", "Kho văn mẫu đốn tim".
- Xưng hô giữa người với người đúng vai vế: tớ–cậu, em–chị, "cháu" với bảo vệ.
- Số liệu đúng, không sót ý. Tiêu đề viết hoa kiểu câu. Đại từ hắn/gã rõ ràng.
