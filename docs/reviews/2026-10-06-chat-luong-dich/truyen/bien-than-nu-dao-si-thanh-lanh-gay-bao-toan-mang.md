# Review: Biến Thân, Nữ Đạo Sĩ Thanh Lãnh Gây Bão Toàn Mạng

- **Slug:** bien-than-nu-dao-si-thanh-lanh-gay-bao-toan-mang
- **Setting:** mixed. Thực tế đây là đô thị hiện đại, xuyên không đổi giới, có hệ thống, sống điền viên ở đạo quan, có livestream, show thực tế và bình luận mạng.
- **Ngôi kể:** ngôi 3.
- **Số chương đã dịch:** 182.
- **Chương đã đọc trọn:** 0001, 0091, 0181, 0182 (hai chương cuối là hai chương có mtime mới nhất).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6 | Thoại giữa sư tỷ, đệ và muội chuẩn. Nhưng lời kể lại gọi Thanh Nguyệt là "muội/muội ấy" khoảng 200 lần trong 39 file. Cặp Trương Chí Quốc và Lưu Khánh trôi giữa ông/cậu và hắn/ông. |
| Giọng văn | 7.5 | Mượt, trẻ trung, đúng chất điền viên hiện đại. Còn calque "vi diệu" và "Dương chủ biên". |
| Dễ hiểu | 8 | Không thấy sai nghĩa, sót ý hay sai số trong 4 chương. |
| Khó khăn | 6.5 | Phần lớn lóng xử lý tốt. Riêng 楼上 thành "Lầu trên" (~47 lần), 桀桀桀 thành "Kiệt kiệt kiệt", và chơi chữ 青/清 bị mất. |
| Thuật ngữ | 6 | "Đạo quan" và "đạo quán" đổi theo từng đợt dịch (131 và 166 lần). 12 tiêu đề viết hoa mọi chữ. Glossary xưng hô cũ mâu thuẫn với cast.json. |

## Lỗi (nặng → nhẹ)

1. **Người kể dùng "muội/đệ" làm đại từ** (cao, hệ thống)
   - Đếm được: ~200 lần/39 file, cộng "đệ ấy" 17 lần/9 file. Vẫn còn ở ch.160–179.
   - Ví dụ ch.91 dòng 5: 她重新抓了一把 → "**Muội** bốc lại một nắm…". Nên là "**Cô bé** bốc lại…".
   - Ví dụ ch.88: "Chu Hiểu Lâm mỉm cười đưa cho muội ấy…".
   - Nguyên nhân: model đang áp toneRules ("gọi … đệ, muội") vào cả lời kể.

2. **道观 dịch lẫn "đạo quan" và "đạo quán"** (cao, hệ thống)
   - Đếm được: 131 lần/56 file so với 166 lần/65 file, đổi theo từng cụm 20 chương.
   - Ví dụ ch.182 dòng 43: 云隐道观 → "đạo quán Vân Ẩn", trong khi glossary ghi 云隐观 = "đạo quan Vân Ẩn".
   - Nguyên nhân: glossary không chốt danh từ chung 道观.

3. **楼上 dịch nghĩa đen** (cao, hệ thống)
   - Đếm được: 42 lần "Lầu trên" và ~5 lần "Tầng trên".
   - Ví dụ ch.182 dòng 69: "Người tầng trên ơi…". Nên là "Bạn ở trên ơi…".
   - Ví dụ ch.14: "Tầng trên không chừng là M đấy nhỉ".

4. **Xưng hô Trương Chí Quốc ↔ Lưu Khánh trôi** (vừa, hệ thống)
   - Ch.49 và 73 gọi nhau "ông". Ch.182 dòng 31 lại là "Lão Lưu này, cái đầu của **cậu**…".
   - cast.json không có mục xưng hô cho cặp này.

5. **Đại từ kể lệch vai vế** (vừa)
   - Ch.181 dòng 81: Lưu Khánh là "Hắn nhận lấy điện thoại", trong khi Trương là "ông" ngay trong cùng cảnh.
   - Ch.49 thì ngược lại, Trương là "hắn".
   - "Hắn" mang sắc thái miệt thị với một cộng sự thân thiện.

6. **Thán từ tượng thanh** (vừa)
   - Ch.181 dòng 129: 桀桀桀 → "Kiệt kiệt kiệt". Nên là "Khè khè khè".
   - 嘿嘿 → "hắc hắc" 5 lần/4 file (ch.91 dòng 213).

7. **微妙 → "vi diệu"** (vừa, hệ thống, 8 lần/8 file)
   - Ch.91 dòng 87: "vẻ mặt có chút vi diệu". Nên là "vẻ mặt hơi khó tả".

8. **Tiêu đề Title Case** (vừa, 12 file, ch.114–123)
   - Ví dụ: "Chương 116: Gây Bão Toàn Mạng".

9. **Mất chơi chữ 青/清** (vừa, lẻ)
   - Ch.1 dòng 117: "Lục Thanh Viễn — không, bây giờ nên gọi cô là Lục Thanh Viễn rồi". Câu thành vô nghĩa.
   - Nên thêm nghĩa "xanh"/"trong" để người đọc thấy được hai chữ khác nhau.

10. **Họ đặt trước chức danh** (thấp)
    - "Dương chủ biên" 5 lần (ch.181 dòng 31), "Trương đạo" 11 lần/3 file.
    - Trong khi đa số chỗ đã dịch "đạo diễn Trương" (58 lần).

11. **直接 thành "trực tiếp"** (thấp)
    - Ch.182 dòng 63: "trực tiếp chấn động luôn". Nên là "sốc tận óc luôn".

12. **热度 lẫn "độ nóng" và "độ hot"** (thấp)
    - "Độ nóng" 5 file. Ch.181 có cả hai kiểu, ở dòng 115 và dòng 141.

13. **Glossary và cast mâu thuẫn** (thấp)
    - story.json ghi 李四→陆清远 = anh–em.
    - cast.json có mốc từ ch.60 đổi thành tôi–ngài.

14. **Danh xưng 老X nằm trong mục names** (thấp)
    - Ch.98 dòng 113: "Tôi là **Lão** Chu". Nên viết thường "lão Chu".

15. **Dấu "......" sáu chấm** (thấp)
    - 339 lần/108 file, lẫn với "...".

## Gợi ý

- **Sửa toneRules:** đệ/muội/tỷ chỉ dùng trong thoại; lời kể dùng "cô bé/con bé" và "cậu bé/thằng bé". Thêm vào prompt chung luật cấm từ xưng hô thân tộc làm đại từ trong lời kể ngôi ba.
- **checkRule:** grep "muội ấy", "đệ ấy" và "muội" trần ngoài ngoặc kép, cùng "Lầu trên/Tầng trên", rồi cảnh báo.
- **Glossary auto chốt cả danh từ chung lặp nhiều:** 道观 → đạo quan, 热度 → độ hot. App báo khi một từ Hán có 2 cách dịch với tỷ lệ ngang nhau.
- **Bảng lóng mạng cố định:**
  - 楼上 → bạn ở trên
  - 直接 dùng làm trạng từ → bỏ hoặc "luôn"
  - 桀桀/嘿嘿 → khè khè/hì hì
  - 微妙 → khó tả/tế nhị, cấm "vi diệu"
- **cast.json tự sinh mục xưng hô và đại từ kể** cho các cặp nhân vật phụ hay xuất hiện cùng nhau (张志国–刘庆).
- **Hậu kiểm:**
  - Đưa tiêu đề Title Case về viết hoa chữ đầu câu (trừ tên riêng và tên chiêu thức).
  - Đổi "......" thành "...".
  - Loại danh xưng 老X khỏi mục names.
  - Đồng bộ glossary.addressing với các mốc trong cast.json.
- **Chơi chữ tên nhân vật:** thêm ghi chú trong story.json chỉ cách xử lý, thay vì gộp 陆青远 và 陆清远 thành một mục.

## Làm tốt

- Thoại giữa ba tỷ đệ ấm và nhất quán.
- Lời kể đổi đại từ "hắn" sang "cô" đúng thời điểm sau khi đổi giới.
- Xưng hô với dân làng thuần Việt, tự nhiên: "chú Sầm", "con bé Lục", "ông Sầm".
- Nhiều lóng mạng sống động: "bẻ cò", "ké chút tiên khí", "sợ thiên hạ chưa đủ loạn".
- Không sót ý, không sai số, không sót chữ Hán; giữ đúng dòng.
